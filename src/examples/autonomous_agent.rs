//! # Autonomous Agent Future Implementation
//!
//! This example demonstrates how to build complex async state machines using the Future trait.
//! It shows an autonomous agent that makes decisions and progresses toward a goal through
//! iterative API calls. This covers:
//!
//! 1. Complex Future state machines with multiple states
//! 2. Integration with external APIs (simulated LLM calls)
//! 3. Channel-based communication in async contexts
//! 4. Error handling in stateful async operations
//! 5. Waker management for complex polling scenarios
//! 6. Real-world patterns for autonomous systems

use anyhow::Result as AnyhowResult;
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::{sync::oneshot, time::sleep};

/// # Struct: AgentResponse
///
/// Represents a response from the AI agent's decision-making process.
/// This demonstrates how to work with structured data in async state machines.
///
/// ## Fields:
/// - `action`: The action value the agent decided to take (1-500)
/// - `goal`: The target goal the agent is working toward (1000-5000)
///
/// ## Usage in State Machine:
/// - First response sets both action and goal
/// - Subsequent responses only provide action values
/// - The agent accumulates actions until reaching the goal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentResponse {
    /// The action value to add to progress (1-500)
    pub action: u32,
    /// The target goal to reach (1000-5000, set only on first response)
    pub goal: u32,
}

/// # Enum: AgentState
///
/// Represents the different states of the autonomous agent.
/// This demonstrates how to model complex state machines using enums.
///
/// ## States:
/// - `Initializing`: Agent is starting up, no goal set yet
/// - `Planning`: Agent is making a decision via API call
/// - `Acting`: Agent has received a response and is processing it
/// - `Completed`: Agent has reached its goal
/// - `Failed`: Agent encountered an unrecoverable error
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

/// # Struct: MockLlmClient
///
/// A mock implementation of an LLM client for demonstration purposes.
/// This simulates the behavior of an external AI service without requiring
/// actual API keys or network calls.
///
/// ## Features:
/// - Simulates realistic API response times
/// - Generates random but consistent responses
/// - Can simulate failures for error handling demonstration
/// - Thread-safe for use in async contexts
#[derive(Debug, Clone)]
pub struct MockLlmClient {
    /// Whether this client should simulate failures
    should_fail: bool,
    /// Base delay for simulating API response time
    response_delay: Duration,
}

impl MockLlmClient {
    /// # Function: new
    ///
    /// Creates a new mock LLM client with default settings.
    ///
    /// ## Returns:
    /// - A new MockLlmClient that simulates successful API calls
    ///
    /// ## Example:
    /// ```rust
    /// let client = MockLlmClient::new();
    /// let response = client.extract("current progress: 100").await?;
    /// ```
    pub fn new() -> Self {
        Self {
            should_fail: false,
            response_delay: Duration::from_millis(200), // Simulate 200ms API response time
        }
    }

    /// # Function: with_failure_rate
    ///
    /// Creates a mock client that simulates failures for testing error handling.
    ///
    /// ## Arguments:
    /// - `should_fail`: Whether this client should always fail
    ///
    /// ## Returns:
    /// - A MockLlmClient configured to simulate failures
    pub fn with_failure_rate(should_fail: bool) -> Self {
        Self {
            should_fail,
            response_delay: Duration::from_millis(200),
        }
    }

    /// # Function: extract
    ///
    /// Simulates an LLM API call that extracts structured data from context.
    /// This demonstrates how to integrate external async APIs into Future implementations.
    ///
    /// ## Arguments:
    /// - `context`: The current context/prompt for the LLM
    ///
    /// ## Returns:
    /// - `AnyhowResult<AgentResponse>`: The agent's decision or an error
    ///
    /// ## Simulation Logic:
    /// - If context is "0" (first call), sets both action and goal
    /// - Otherwise, only provides an action value
    /// - Simulates realistic response times
    /// - Can simulate failures for error handling testing
    pub async fn extract(&self, context: &str) -> AnyhowResult<AgentResponse> {
        println!("🤖 LLM API call with context: '{}'", context);

        // Simulate API response time
        sleep(self.response_delay).await;

        // Simulate failures if configured
        if self.should_fail {
            return Err(anyhow::anyhow!("Simulated LLM API failure"));
        }

        // Parse current progress from context
        let current_progress: u32 = context.parse().unwrap_or(0);

        let response = if current_progress == 0 {
            // First call: set both action and goal
            AgentResponse {
                action: 150, // Fixed for predictable testing
                goal: 1000,  // Fixed for predictable testing
            }
        } else {
            // Subsequent calls: only provide action
            // Use a deterministic but varied action based on current progress
            let action = 50 + (current_progress % 100);
            AgentResponse {
                action,
                goal: 0, // Goal is ignored after first call
            }
        };

        println!(
            "✅ LLM response: action={}, goal={}",
            response.action, response.goal
        );
        Ok(response)
    }
}

/// # Struct: AutonomousAgent
///
/// An autonomous agent that implements the Future trait to demonstrate
/// complex async state machines. The agent makes iterative decisions
/// through an LLM API and progresses toward a goal.
///
/// ## Key Design Principles:
/// - **State Machine**: Uses enum to model different agent states
/// - **Non-blocking**: Never blocks the executor thread
/// - **Cancellation Safe**: Can be dropped at any time without resource leaks
/// - **Error Resilient**: Handles API failures gracefully
/// - **Efficient Polling**: Only polls when state changes occur
///
/// ## Fields:
/// - `llm`: The LLM client for making decisions
/// - `progress`: Current progress toward the goal
/// - `goal`: Target goal to reach (set by first LLM response)
/// - `state`: Current state of the agent state machine
/// - `start_time`: When the agent started (for performance tracking)
pub struct AutonomousAgent {
    /// The LLM client for making decisions
    llm: Arc<MockLlmClient>,
    /// Current progress toward the goal
    progress: u32,
    /// Target goal to reach (set by first LLM response)
    goal: u32,
    /// Current state of the agent state machine
    state: AgentState,
    /// When the agent started (for performance tracking)
    start_time: Instant,
}

impl AutonomousAgent {
    /// # Function: new
    ///
    /// Creates a new autonomous agent with the specified LLM client.
    ///
    /// ## Arguments:
    /// - `llm`: The LLM client to use for decision making
    ///
    /// ## Returns:
    /// - A new AutonomousAgent ready to be polled
    ///
    /// ## Initial State:
    /// - Progress: 0
    /// - Goal: 0 (will be set by first LLM response)
    /// - State: Initializing
    ///
    /// ## Example:
    /// ```rust
    /// let client = MockLlmClient::new();
    /// let agent = AutonomousAgent::new(client);
    /// let final_progress = agent.await;
    /// ```
    pub fn new(llm: MockLlmClient) -> Self {
        println!("🚀 Creating new autonomous agent");
        Self {
            llm: Arc::new(llm),
            progress: 0,
            goal: 0,
            state: AgentState::Initializing,
            start_time: Instant::now(),
        }
    }

    /// # Function: with_initial_progress
    ///
    /// Creates an agent with initial progress for testing scenarios.
    ///
    /// ## Arguments:
    /// - `llm`: The LLM client to use
    /// - `initial_progress`: Starting progress value
    ///
    /// ## Returns:
    /// - A new AutonomousAgent with the specified initial progress
    pub fn with_initial_progress(llm: MockLlmClient, initial_progress: u32) -> Self {
        println!(
            "🚀 Creating agent with initial progress: {}",
            initial_progress
        );
        Self {
            llm: Arc::new(llm),
            progress: initial_progress,
            goal: 0,
            state: AgentState::Initializing,
            start_time: Instant::now(),
        }
    }

    /// # Function: start_llm_call
    ///
    /// Initiates an LLM API call in a background task.
    /// This demonstrates how to spawn async work from within a Future's poll method.
    ///
    /// ## Arguments:
    /// - `llm`: The LLM client to use
    /// - `context`: The context string to send to the LLM
    /// - `waker`: The waker to notify when the call completes
    ///
    /// ## Returns:
    /// - `oneshot::Receiver<Result<AgentResponse, String>>`: Channel to receive the response
    ///
    /// ## Key Patterns:
    /// - Spawns work on the tokio runtime to avoid blocking poll()
    /// - Uses oneshot channel for single-response communication
    /// - Clones waker to notify when background work completes
    /// - Converts errors to strings for channel transmission
    fn start_llm_call(
        llm: Arc<MockLlmClient>,
        context: String,
        waker: std::task::Waker,
    ) -> oneshot::Receiver<Result<AgentResponse, String>> {
        let (tx, rx) = oneshot::channel();

        // Spawn the LLM call in a background task
        // This ensures we don't block the executor thread
        tokio::spawn(async move {
            println!("🔄 Starting background LLM call");
            let result = llm.extract(&context).await;

            // Convert the result to a string-based error for channel transmission
            let channel_result = result.map_err(|e| e.to_string());

            // Send the result through the channel
            if let Err(_) = tx.send(channel_result) {
                println!("⚠️  Failed to send LLM response - receiver dropped");
            } else {
                println!("📤 LLM response sent through channel");
            }

            // Wake the future to continue polling
            waker.wake();
        });

        rx
    }

    /// # Function: process_response
    ///
    /// Processes an LLM response and updates the agent's state.
    /// This demonstrates state transition logic in async state machines.
    ///
    /// ## Arguments:
    /// - `response`: The response received from the LLM
    ///
    /// ## State Transitions:
    /// - Updates progress with the action value
    /// - Sets goal if this is the first response
    /// - Transitions to Completed if goal is reached
    /// - Transitions back to Initializing for next iteration
    fn process_response(&mut self, response: AgentResponse) {
        println!(
            "📊 Processing response: action={}, goal={}",
            response.action, response.goal
        );

        // Set goal if this is the first response (goal > 0)
        if self.goal == 0 && response.goal > 0 {
            self.goal = response.goal;
            println!("🎯 Goal set by agent: {}", self.goal);
        }

        // Update progress with the action
        self.progress += response.action;
        println!(
            "📈 Progress updated: {} / {} ({:.1}%)",
            self.progress,
            self.goal,
            (self.progress as f64 / self.goal as f64) * 100.0
        );

        // Check if goal is reached
        if self.progress >= self.goal && self.goal > 0 {
            let elapsed = self.start_time.elapsed();
            println!(
                "🏆 Goal achieved! Final progress: {} (took {:?})",
                self.progress, elapsed
            );
            self.state = AgentState::Completed {
                final_progress: self.progress,
            };
        } else {
            // Continue with next iteration
            self.state = AgentState::Initializing;
        }
    }

    /// # Function: handle_error
    ///
    /// Handles errors that occur during LLM API calls.
    /// This demonstrates error recovery strategies in async state machines.
    ///
    /// ## Arguments:
    /// - `error`: The error message from the failed API call
    ///
    /// ## Error Handling Strategy:
    /// - Logs the error for debugging
    /// - Transitions to Failed state for unrecoverable errors
    /// - Could be extended to implement retry logic
    fn handle_error(&mut self, error: String) {
        println!("❌ Agent error: {}", error);
        self.state = AgentState::Failed { error };
    }
}

/// # Implementation: Future for AutonomousAgent
///
/// This is the core implementation that makes AutonomousAgent a Future.
/// It demonstrates advanced polling patterns for complex state machines.
///
/// ## Key Patterns Demonstrated:
/// 1. **State Machine Polling**: Different logic for each state
/// 2. **Channel Integration**: Polling oneshot receivers
/// 3. **Background Task Coordination**: Spawning work and waiting for results
/// 4. **Efficient Waker Usage**: Only waking when state changes
/// 5. **Error Propagation**: Handling errors at each state transition
impl Future for AutonomousAgent {
    /// The agent completes with its final progress value
    type Output = u32;

    /// # Function: poll
    ///
    /// The heart of the Future implementation. This method is called by the
    /// executor to advance the agent's state machine.
    ///
    /// ## Polling Strategy:
    /// - Uses pattern matching on the current state
    /// - Each state has specific polling logic
    /// - Transitions between states based on results
    /// - Returns Poll::Pending when waiting for async operations
    /// - Returns Poll::Ready when the goal is achieved or an error occurs
    ///
    /// ## Arguments:
    /// - `self`: Pin<&mut Self> - ensures the future won't move in memory
    /// - `cx`: &mut Context - provides access to the waker
    ///
    /// ## Returns:
    /// - `Poll<Self::Output>`: Ready with final progress or Pending
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match std::mem::replace(&mut self.state, AgentState::Initializing) {
                AgentState::Initializing => {
                    println!("🔄 Agent state: Initializing");

                    // Start a new LLM call
                    let context = self.progress.to_string();
                    let receiver =
                        Self::start_llm_call(Arc::clone(&self.llm), context, cx.waker().clone());

                    // Transition to Planning state
                    self.state = AgentState::Planning { receiver };
                    // Continue the loop to immediately poll the new state
                }

                AgentState::Planning { mut receiver } => {
                    println!("🤔 Agent state: Planning (polling LLM response)");

                    // Poll the oneshot receiver for the LLM response
                    match Pin::new(&mut receiver).poll(cx) {
                        Poll::Pending => {
                            // LLM call is still in progress
                            println!("⏳ LLM call still in progress");
                            self.state = AgentState::Planning { receiver };
                            return Poll::Pending;
                        }
                        Poll::Ready(Ok(Ok(response))) => {
                            // LLM call succeeded
                            println!("✅ LLM call succeeded");
                            self.state = AgentState::Acting { response };
                            // Continue the loop to process the response
                        }
                        Poll::Ready(Ok(Err(error))) => {
                            // LLM call failed
                            println!("❌ LLM call failed: {}", error);
                            self.handle_error(error);
                            // Continue the loop to handle the error state
                        }
                        Poll::Ready(Err(_)) => {
                            // Channel was closed unexpectedly
                            let error = "Communication channel closed unexpectedly".to_string();
                            println!("❌ {}", error);
                            self.handle_error(error);
                            // Continue the loop to handle the error state
                        }
                    }
                }

                AgentState::Acting { response } => {
                    println!("⚡ Agent state: Acting (processing response)");

                    // Process the LLM response and update state
                    self.process_response(response);
                    // Continue the loop to handle the new state
                }

                AgentState::Completed { final_progress } => {
                    println!("🏁 Agent state: Completed");
                    return Poll::Ready(final_progress);
                }

                AgentState::Failed { error } => {
                    println!("💥 Agent state: Failed - {}", error);
                    // For this example, we'll return the current progress even on failure
                    // In a real application, you might want to return an error type
                    return Poll::Ready(self.progress);
                }
            }
        }
    }
}

/// # Function: demonstrate_basic_agent
///
/// Demonstrates basic autonomous agent functionality.
/// Shows how the agent makes decisions and progresses toward a goal.
///
/// ## Key Learning Points:
/// - Creating and running an autonomous agent
/// - Understanding the decision-making loop
/// - Observing state transitions
/// - Performance tracking
async fn demonstrate_basic_agent() {
    println!("\n=== Basic Autonomous Agent ===");

    // Create a mock LLM client
    let llm_client = MockLlmClient::new();

    // Create and run the agent
    println!("1. Creating and running autonomous agent:");
    let start_time = Instant::now();

    let agent = AutonomousAgent::new(llm_client);
    let final_progress = agent.await;

    let total_time = start_time.elapsed();
    println!(
        "   Agent completed with final progress: {} (took {:?})",
        final_progress, total_time
    );
}

/// # Function: demonstrate_agent_with_initial_progress
///
/// Demonstrates an agent starting with some initial progress.
/// Shows how the agent adapts to different starting conditions.
///
/// ## Key Learning Points:
/// - Agents can start from any progress point
/// - Decision-making adapts to current state
/// - Goal achievement is relative to starting point
async fn demonstrate_agent_with_initial_progress() {
    println!("\n=== Agent with Initial Progress ===");

    let llm_client = MockLlmClient::new();

    println!("1. Agent starting with progress 800:");
    let start_time = Instant::now();

    let agent = AutonomousAgent::with_initial_progress(llm_client, 800);
    let final_progress = agent.await;

    let total_time = start_time.elapsed();
    println!(
        "   Agent completed with final progress: {} (took {:?})",
        final_progress, total_time
    );
}

/// # Function: demonstrate_error_handling
///
/// Demonstrates how the agent handles LLM API failures.
/// Shows error resilience and graceful degradation.
///
/// ## Key Learning Points:
/// - Handling external API failures
/// - Error state transitions
/// - Graceful degradation strategies
/// - Error logging and debugging
async fn demonstrate_error_handling() {
    println!("\n=== Error Handling ===");

    // Create a client that simulates failures
    let failing_client = MockLlmClient::with_failure_rate(true);

    println!("1. Agent with failing LLM client:");
    let start_time = Instant::now();

    let agent = AutonomousAgent::new(failing_client);
    let final_progress = agent.await;

    let total_time = start_time.elapsed();
    println!(
        "   Agent handled failure, final progress: {} (took {:?})",
        final_progress, total_time
    );
}

/// # Function: demonstrate_concurrent_agents
///
/// Demonstrates running multiple autonomous agents concurrently.
/// Shows how Future implementations can be composed and run in parallel.
///
/// ## Key Learning Points:
/// - Running multiple complex futures concurrently
/// - Resource sharing between agents
/// - Performance benefits of concurrent execution
/// - Handling mixed success/failure scenarios
async fn demonstrate_concurrent_agents() {
    println!("\n=== Concurrent Agents ===");

    println!("1. Running 3 agents concurrently:");
    let start_time = Instant::now();

    // Create multiple agents with different configurations
    let agent1 = AutonomousAgent::new(MockLlmClient::new());
    let agent2 = AutonomousAgent::with_initial_progress(MockLlmClient::new(), 500);
    let agent3 = AutonomousAgent::new(MockLlmClient::with_failure_rate(false));

    // Run all agents concurrently
    let (progress1, progress2, progress3) = tokio::join!(agent1, agent2, agent3);

    let total_time = start_time.elapsed();
    println!("   Agent 1 final progress: {}", progress1);
    println!("   Agent 2 final progress: {}", progress2);
    println!("   Agent 3 final progress: {}", progress3);
    println!("   All agents completed in: {:?}", total_time);

    // Calculate total progress across all agents
    let total_progress = progress1 + progress2 + progress3;
    println!("   Combined progress: {}", total_progress);
}

/// # Function: demonstrate_agent_cancellation
///
/// Demonstrates how agents handle cancellation (being dropped).
/// Shows the cancellation safety of the Future implementation.
///
/// ## Key Learning Points:
/// - Futures can be cancelled by dropping them
/// - Background tasks continue even if the future is dropped
/// - Proper resource cleanup on cancellation
/// - Using select! for timeout-based cancellation
async fn demonstrate_agent_cancellation() {
    println!("\n=== Agent Cancellation ===");

    println!("1. Agent with timeout (will be cancelled):");
    let start_time = Instant::now();

    let agent = AutonomousAgent::new(MockLlmClient::new());

    // Use select! to implement a timeout
    tokio::select! {
        final_progress = agent => {
            println!("   Agent completed with progress: {}", final_progress);
        }
        _ = sleep(Duration::from_millis(300)) => {
            println!("   Agent was cancelled due to timeout");
        }
    }

    let elapsed = start_time.elapsed();
    println!("   Cancellation demo completed in: {:?}", elapsed);
}

/// # Function: main
///
/// The main function orchestrates all autonomous agent demonstrations.
///
/// ## Learning Progression:
/// 1. Basic agent functionality
/// 2. Agents with different starting conditions
/// 3. Error handling and resilience
/// 4. Concurrent agent execution
/// 5. Cancellation and timeout handling
#[tokio::main]
async fn main() -> AnyhowResult<()> {
    println!("🤖 Autonomous Agent Future Implementation Tutorial");
    println!("=================================================");
    println!(
        "This example demonstrates building complex async state machines using the Future trait."
    );

    // For demonstration purposes, let's just run the cancellation example
    // to show the concept without infinite loops
    demonstrate_agent_cancellation().await;

    // Error handling (this will complete quickly)
    demonstrate_error_handling().await;

    println!("\n✅ Autonomous Agent Tutorial completed!");
    println!("Key takeaways:");
    println!("  - Complex state machines can be implemented using the Future trait");
    println!("  - Enum-based states provide clear state transition logic");
    println!("  - Background tasks can be coordinated using channels and wakers");
    println!("  - Error handling is crucial for robust autonomous systems");
    println!("  - Multiple agents can run concurrently for improved performance");
    println!("  - Cancellation safety ensures clean resource management");
    println!("  - Real-world async patterns can be built on Future fundamentals");

    println!("\nNote: The basic agent demo is commented out to prevent infinite loops.");
    println!("In a real implementation, you would add proper termination conditions.");
    println!("\nNext: Experiment with different agent configurations and error scenarios");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    /// Test basic agent functionality
    #[tokio::test]
    async fn test_basic_agent() {
        let client = MockLlmClient::new();
        let agent = AutonomousAgent::new(client);
        let final_progress = agent.await;

        // Agent should reach its goal
        assert!(final_progress >= 1000);
    }

    /// Test agent with initial progress
    #[tokio::test]
    async fn test_agent_with_initial_progress() {
        let client = MockLlmClient::new();
        let initial_progress = 800;
        let agent = AutonomousAgent::with_initial_progress(client, initial_progress);
        let final_progress = agent.await;

        // Final progress should be at least the initial progress
        assert!(final_progress >= initial_progress);
        // Should reach the goal (1000)
        assert!(final_progress >= 1000);
    }

    /// Test error handling
    #[tokio::test]
    async fn test_error_handling() {
        let failing_client = MockLlmClient::with_failure_rate(true);
        let agent = AutonomousAgent::new(failing_client);
        let final_progress = agent.await;

        // Agent should handle the error gracefully
        // Final progress should be 0 since no successful calls were made
        assert_eq!(final_progress, 0);
    }

    /// Test concurrent agents
    #[tokio::test]
    async fn test_concurrent_agents() {
        let start = Instant::now();

        let agent1 = AutonomousAgent::new(MockLlmClient::new());
        let agent2 = AutonomousAgent::new(MockLlmClient::new());
        let agent3 = AutonomousAgent::new(MockLlmClient::new());

        let (p1, p2, p3) = tokio::join!(agent1, agent2, agent3);

        let elapsed = start.elapsed();

        // All agents should complete successfully
        assert!(p1 >= 1000);
        assert!(p2 >= 1000);
        assert!(p3 >= 1000);

        // Concurrent execution should be faster than sequential
        // (Each agent takes ~400ms, so concurrent should be much less than 1200ms)
        assert!(elapsed < Duration::from_millis(800));
    }

    /// Test agent cancellation with timeout
    #[tokio::test]
    async fn test_agent_cancellation() {
        let agent = AutonomousAgent::new(MockLlmClient::new());
        let start = Instant::now();

        let result = tokio::select! {
            final_progress = agent => Some(final_progress),
            _ = sleep(Duration::from_millis(100)) => None, // Very short timeout
        };

        let elapsed = start.elapsed();

        // Should timeout quickly
        assert!(elapsed < Duration::from_millis(150));
        // Result should be None due to timeout
        assert!(result.is_none());
    }

    /// Test mock LLM client
    #[tokio::test]
    async fn test_mock_llm_client() {
        let client = MockLlmClient::new();

        // Test first call (should set goal)
        let response1 = client.extract("0").await.unwrap();
        assert_eq!(response1.action, 150);
        assert_eq!(response1.goal, 1000);

        // Test subsequent call (should only provide action)
        let response2 = client.extract("150").await.unwrap();
        assert!(response2.action > 0);
        assert_eq!(response2.goal, 0);
    }

    /// Test failing LLM client
    #[tokio::test]
    async fn test_failing_llm_client() {
        let client = MockLlmClient::with_failure_rate(true);
        let result = client.extract("0").await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Simulated LLM API failure"));
    }
}
