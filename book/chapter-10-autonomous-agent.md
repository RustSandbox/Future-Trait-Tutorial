# Chapter 10: Autonomous Agent Example

## Introduction

This chapter explores our most advanced example: an autonomous agent implemented as a complex async state machine. This demonstrates how all the concepts we've learned come together in a real-world scenario.

**File reference**: `src/examples/autonomous_agent.rs`

```bash
# Run the autonomous agent example
cargo run --bin autonomous_agent

# Test the implementation
cargo test --bin autonomous_agent
```

## The Autonomous Agent Architecture

Our autonomous agent demonstrates:

- **Complex state machine implementation**
- **Integration with external APIs** (simulated LLM)
- **Channel-based communication patterns**
- **Background task coordination**
- **Error handling in stateful async operations**
- **Cancellation safety and timeout handling**

### Agent State Machine

```rust
#[derive(Debug)]
enum AgentState {
    /// Agent is starting up, no goal set yet
    Initializing,
    /// Agent is making a decision via API call
    Planning {
        /// Channel to receive the API response
        receiver: oneshot::Receiver<Result<AgentResponse, String>>,
    },
    /// Agent has received a response and is processing it
    Acting {
        /// The response received from the API
        response: AgentResponse,
    },
    /// Agent has reached its goal
    Completed {
        /// Final progress value
        final_progress: u32,
    },
    /// Agent encountered an unrecoverable error
    Failed {
        /// Error message describing the failure
        error: String,
    },
}
```

### Agent Response Structure

```rust
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct AgentResponse {
    /// The action value to add to progress
    pub action: u32,
    /// The goal to work towards (only used on first response)
    pub goal: u32,
}
```

## The AutonomousAgent Future

```rust
pub struct AutonomousAgent {
    /// Current state of the agent
    state: AgentState,
    /// Current progress towards the goal
    progress: u32,
    /// The goal to achieve
    goal: u32,
    /// Mock LLM client for decision making
    llm_client: Arc<MockLlmClient>,
}
```

### Future Implementation

The agent implements the `Future` trait as a sophisticated state machine:

```rust
impl Future for AutonomousAgent {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match &mut self.state {
                AgentState::Initializing => {
                    // Transition to planning state
                    self.start_planning(cx.waker().clone());
                }
                
                AgentState::Planning { receiver } => {
                    // Poll for API response
                    match Pin::new(receiver).poll(cx) {
                        Poll::Ready(Ok(Ok(response))) => {
                            self.process_response(response);
                        }
                        Poll::Ready(Ok(Err(error))) => {
                            self.handle_error(error);
                        }
                        Poll::Ready(Err(_)) => {
                            self.handle_channel_error();
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                
                AgentState::Acting { response } => {
                    // Process the response and update state
                    self.apply_action(response.clone());
                }
                
                AgentState::Completed { final_progress } => {
                    return Poll::Ready(*final_progress);
                }
                
                AgentState::Failed { error } => {
                    panic!("Agent failed: {}", error);
                }
            }
        }
    }
}
```

## Key Implementation Details

### 1. Lazy Execution

The agent doesn't start any work until first polled:

```rust
impl AutonomousAgent {
    pub fn new(llm_client: MockLlmClient) -> Self {
        Self {
            state: AgentState::Initializing, // No work started yet
            progress: 0,
            goal: 0,
            llm_client: Arc::new(llm_client),
        }
    }
}
```

### 2. Channel-Based Communication

Uses `oneshot` channels for async communication with the LLM:

```rust
fn start_planning(&mut self, waker: Waker) {
    let (tx, rx) = oneshot::channel();
    let llm_client = Arc::clone(&self.llm_client);
    let context = self.progress.to_string();

    // Spawn background task for LLM call
    tokio::spawn(async move {
        let result = llm_client.get_decision(&context).await;
        let _ = tx.send(result);
        waker.wake(); // Wake the future when response is ready
    });

    self.state = AgentState::Planning { receiver: rx };
}
```

### 3. Mock LLM Client

Simulates external API calls with realistic behavior:

```rust
pub struct MockLlmClient {
    response_delay: Duration,
    failure_rate: f64,
}

impl MockLlmClient {
    pub async fn get_decision(&self, context: &str) -> Result<AgentResponse, String> {
        // Simulate network delay
        tokio::time::sleep(self.response_delay).await;
        
        // Simulate potential failures
        if rand::random::<f64>() < self.failure_rate {
            return Err("LLM service temporarily unavailable".to_string());
        }
        
        // Parse current progress from context
        let current_progress: u32 = context.parse().unwrap_or(0);
        
        let response = if current_progress == 0 {
            // First call: set both action and goal
            AgentResponse {
                action: 150,
                goal: 1000,
            }
        } else {
            // Subsequent calls: provide action based on current progress
            let action = 50 + (current_progress % 100);
            AgentResponse {
                action,
                goal: 0, // Goal is ignored after first call
            }
        };
        
        Ok(response)
    }
}
```

## Advanced Patterns Demonstrated

### 1. State Transitions

The agent carefully manages state transitions:

```rust
fn process_response(&mut self, response: AgentResponse) {
    if self.goal == 0 {
        // First response sets the goal
        self.goal = response.goal;
        println!("🎯 Goal set: {}", self.goal);
    }
    
    self.state = AgentState::Acting { response };
}

fn apply_action(&mut self, response: AgentResponse) {
    self.progress += response.action;
    println!("📈 Progress: {} / {}", self.progress, self.goal);
    
    if self.progress >= self.goal {
        self.state = AgentState::Completed {
            final_progress: self.progress,
        };
    } else {
        // Continue planning for next action
        self.state = AgentState::Initializing;
    }
}
```

### 2. Error Handling

Robust error handling throughout the state machine:

```rust
fn handle_error(&mut self, error: String) {
    println!("⚠️ LLM error: {}", error);
    
    // Could implement retry logic here
    if self.should_retry(&error) {
        self.state = AgentState::Initializing; // Retry
    } else {
        self.state = AgentState::Failed { error };
    }
}

fn handle_channel_error(&mut self) {
    let error = "Communication channel closed unexpectedly".to_string();
    println!("💥 Channel error: {}", error);
    self.state = AgentState::Failed { error };
}
```

### 3. Cancellation Safety

The agent is safe to drop at any time:

```rust
impl Drop for AutonomousAgent {
    fn drop(&mut self) {
        println!("🛑 Agent dropped, cleaning up resources");
        // Any cleanup logic would go here
    }
}
```

## Usage Examples

### Basic Agent

```rust
async fn basic_agent_example() {
    println!("🤖 Starting basic autonomous agent");
    
    let llm_client = MockLlmClient::new(
        Duration::from_millis(100), // Response delay
        0.1, // 10% failure rate
    );
    
    let agent = AutonomousAgent::new(llm_client);
    
    match agent.await {
        final_progress => {
            println!("✅ Agent completed with progress: {}", final_progress);
        }
    }
}
```

### Agent with Timeout

```rust
async fn agent_with_timeout() {
    let llm_client = MockLlmClient::new(
        Duration::from_millis(50),
        0.0, // No failures
    );
    
    let agent = AutonomousAgent::new(llm_client);
    
    match tokio::time::timeout(Duration::from_secs(10), agent).await {
        Ok(final_progress) => {
            println!("✅ Agent completed: {}", final_progress);
        }
        Err(_) => {
            println!("⏰ Agent timed out");
        }
    }
}
```

### Concurrent Agents

```rust
async fn concurrent_agents() {
    println!("🤖 Running multiple agents concurrently");
    
    let agents = (0..3).map(|i| {
        let llm_client = MockLlmClient::new(
            Duration::from_millis(50 + i * 25),
            0.1,
        );
        AutonomousAgent::new(llm_client)
    });
    
    let results = futures::future::join_all(agents).await;
    
    for (i, result) in results.into_iter().enumerate() {
        println!("Agent {}: completed with {}", i, result);
    }
}
```

## Testing the Agent

Comprehensive tests ensure the agent works correctly:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_basic_completion() {
        let llm_client = MockLlmClient::new(Duration::from_millis(10), 0.0);
        let agent = AutonomousAgent::new(llm_client);
        
        let result = agent.await;
        assert!(result > 0);
    }

    #[tokio::test]
    async fn test_agent_with_failures() {
        let llm_client = MockLlmClient::new(Duration::from_millis(10), 0.5);
        let agent = AutonomousAgent::new(llm_client);
        
        // Should eventually complete despite failures
        let result = tokio::time::timeout(Duration::from_secs(5), agent).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_agents() {
        let agents: Vec<_> = (0..3).map(|_| {
            let llm_client = MockLlmClient::new(Duration::from_millis(10), 0.0);
            AutonomousAgent::new(llm_client)
        }).collect();
        
        let results = futures::future::join_all(agents).await;
        
        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result > 0);
        }
    }
}
```

## Key Insights

### 1. State Machine Design

The agent demonstrates how to build complex state machines:

- **Clear state definitions** with associated data
- **Explicit state transitions** with validation
- **Error states** for robust error handling
- **Completion detection** with final results

### 2. Async Integration

Shows how to integrate with external systems:

- **Channel-based communication** for async operations
- **Background task spawning** for concurrent work
- **Waker management** for efficient scheduling
- **Timeout handling** for reliability

### 3. Real-World Patterns

Demonstrates production-ready patterns:

- **Lazy execution** - no work until needed
- **Cancellation safety** - safe to drop anytime
- **Error recovery** - graceful handling of failures
- **Resource cleanup** - proper resource management

## What's Next?

This autonomous agent example brings together all the concepts we've learned throughout this book. It shows how the Future trait can be used to build sophisticated, real-world async systems.

In the final chapters, we'll explore performance optimization and best practices for production async Rust code.

---

## Key Takeaways

- Complex state machines can be elegantly implemented as Futures
- Channel-based communication enables clean async integration
- Proper error handling is crucial for robust async systems
- Background tasks and waker management enable efficient scheduling
- Real-world async systems require careful attention to cancellation safety
- Testing async state machines requires comprehensive scenario coverage 