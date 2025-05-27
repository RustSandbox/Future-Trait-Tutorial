# Appendix A: Code Examples

This appendix provides a comprehensive reference to all the code examples in this book, organized by complexity and use case.

## Running the Examples

All examples are located in the `src/examples/` directory and can be run using Cargo:

```bash
# Basic async/await concepts
cargo run --bin basic_future

# Custom Future implementation
cargo run --bin custom_delay

# Error handling patterns
cargo run --bin error_handling

# Autonomous agent state machine
cargo run --bin autonomous_agent
```

## Testing the Examples

Each example includes comprehensive tests:

```bash
# Test all examples
cargo test

# Test specific examples
cargo test --bin basic_future
cargo test --bin custom_delay
cargo test --bin error_handling
cargo test --bin autonomous_agent
```

## Example Overview

### 1. Basic Future (`basic_future.rs`)

**Complexity**: Beginner
**Concepts**: async/await fundamentals, future laziness, concurrent execution

```rust
// Key concepts demonstrated:
async fn demonstrate_laziness() {
    // Creating futures does NO work
    let future1 = simulate_work("Task 1", Duration::from_millis(100));
    let future2 = simulate_work("Task 2", Duration::from_millis(100));
    
    // Work only starts when awaited
    let (r1, r2) = tokio::join!(future1, future2);
}
```

**What you'll learn**:
- How async functions work
- Future laziness principle
- Sequential vs concurrent execution
- Basic error handling with `Result<T, E>`

### 2. Custom Delay (`custom_delay.rs`)

**Complexity**: Intermediate
**Concepts**: Future trait implementation, state machines, waker management

```rust
// Key implementation pattern:
impl Future for DelayFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_complete() {
            Poll::Ready(self.get_result())
        } else {
            self.store_waker(cx.waker().clone());
            Poll::Pending
        }
    }
}
```

**What you'll learn**:
- Manual Future trait implementation
- `Poll::Ready` vs `Poll::Pending`
- Waker usage for efficient scheduling
- Thread-safe shared state with `Arc<Mutex<T>>`

### 3. Error Handling (`error_handling.rs`)

**Complexity**: Intermediate to Advanced
**Concepts**: Async error patterns, timeouts, retries, circuit breakers

```rust
// Key patterns demonstrated:
async fn retry_with_backoff<F, T, E>(
    operation: F,
    max_attempts: usize,
    base_delay: Duration,
) -> Result<T, E> {
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_attempts => return Err(e),
            Err(_) => {
                let delay = base_delay * 2_u32.pow(attempt - 1);
                tokio::time::sleep(delay).await;
            }
        }
    }
    unreachable!()
}
```

**What you'll learn**:
- Custom error types with `thiserror`
- Timeout handling with `tokio::time::timeout`
- Retry patterns with exponential backoff
- Circuit breaker implementation
- Concurrent error handling strategies

### 4. Autonomous Agent (`autonomous_agent.rs`)

**Complexity**: Advanced
**Concepts**: Complex state machines, external API integration, channel communication

```rust
// Key state machine pattern:
impl Future for AutonomousAgent {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match &mut self.state {
                AgentState::Initializing => { /* start planning */ }
                AgentState::Planning { receiver } => { /* poll for response */ }
                AgentState::Acting { response } => { /* apply action */ }
                AgentState::Completed { final_progress } => {
                    return Poll::Ready(*final_progress);
                }
                AgentState::Failed { error } => { /* handle failure */ }
            }
        }
    }
}
```

**What you'll learn**:
- Complex async state machine implementation
- Channel-based communication with `oneshot`
- Background task coordination
- Integration with external APIs
- Advanced error handling and recovery

## Code Patterns Reference

### Future Implementation Patterns

#### Immediate Completion
```rust
impl Future for ImmediateFuture {
    type Output = T;
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.value)
    }
}
```

#### One-time Async Operation
```rust
impl Future for OneShotFuture {
    type Output = T;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.start_operation(cx.waker().clone());
            self.started = true;
        }
        
        if self.is_complete() {
            Poll::Ready(self.get_result())
        } else {
            Poll::Pending
        }
    }
}
```

#### State Machine
```rust
impl Future for StateMachineFuture {
    type Output = T;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.state {
                State::A => { /* transition to B */ }
                State::B => { /* transition to C or return Pending */ }
                State::C => return Poll::Ready(result),
            }
        }
    }
}
```

### Error Handling Patterns

#### Basic Error Propagation
```rust
async fn chain_operations() -> Result<String, MyError> {
    let data = fetch_data().await?;
    let processed = process_data(data).await?;
    let result = save_result(processed).await?;
    Ok(result)
}
```

#### Timeout with Custom Error
```rust
async fn with_timeout<F, T>(future: F, duration: Duration) -> Result<T, TimeoutError>
where
    F: Future<Output = Result<T, MyError>>,
{
    tokio::select! {
        result = future => result.map_err(TimeoutError::Inner),
        _ = tokio::time::sleep(duration) => Err(TimeoutError::Timeout),
    }
}
```

#### Concurrent Error Collection
```rust
async fn collect_results() -> Vec<Result<T, E>> {
    let futures = vec![operation1(), operation2(), operation3()];
    futures::future::join_all(futures).await
}
```

### Concurrency Patterns

#### Join All (Wait for All)
```rust
let (r1, r2, r3) = tokio::join!(
    operation1(),
    operation2(),
    operation3()
);
```

#### Try Join (Fail Fast)
```rust
let (r1, r2, r3) = tokio::try_join!(
    operation1(),
    operation2(),
    operation3()
)?;
```

#### Select First (Race)
```rust
tokio::select! {
    result = operation1() => handle_first(result),
    result = operation2() => handle_second(result),
    _ = tokio::time::sleep(timeout) => handle_timeout(),
}
```

## Testing Patterns

### Basic Async Test
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = my_async_function().await;
    assert_eq!(result, expected_value);
}
```

### Timing Test
```rust
#[tokio::test]
async fn test_timing() {
    let start = Instant::now();
    let result = timed_operation().await;
    let elapsed = start.elapsed();
    
    assert!(elapsed >= Duration::from_millis(95));
    assert!(elapsed <= Duration::from_millis(150));
}
```

### Error Test
```rust
#[tokio::test]
async fn test_error_handling() {
    let result = failing_operation().await;
    assert!(result.is_err());
    
    match result {
        Err(MyError::Specific) => { /* expected */ }
        _ => panic!("Unexpected error type"),
    }
}
```

### Concurrent Test
```rust
#[tokio::test]
async fn test_concurrent_operations() {
    let start = Instant::now();
    
    let (r1, r2, r3) = tokio::join!(
        operation1(),
        operation2(),
        operation3()
    );
    
    let elapsed = start.elapsed();
    
    // Should complete in time of longest operation, not sum
    assert!(elapsed < Duration::from_millis(200));
}
```

## Performance Considerations

### Memory Usage
- **Futures are small**: Typically a few hundred bytes
- **Stack vs Heap**: Futures live on the heap when boxed
- **State machines**: Compiler generates efficient state machines

### CPU Usage
- **Zero-cost abstractions**: No runtime overhead
- **Cooperative scheduling**: No preemptive context switching
- **Efficient polling**: Only poll when woken

### I/O Efficiency
- **Non-blocking**: Never block the executor thread
- **Batching**: Group operations when possible
- **Connection pooling**: Reuse connections

## Common Pitfalls and Solutions

### Pitfall: Forgetting to await
```rust
// ❌ Wrong
async fn wrong() {
    some_async_function(); // Future created but never executed
}

// ✅ Correct
async fn correct() {
    some_async_function().await; // Future executed
}
```

### Pitfall: Blocking in async context
```rust
// ❌ Wrong
async fn wrong() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks executor
}

// ✅ Correct
async fn correct() {
    tokio::time::sleep(Duration::from_secs(1)).await; // Yields control
}
```

### Pitfall: Sequential when concurrent is possible
```rust
// ❌ Inefficient
async fn sequential() {
    let a = fetch_a().await;
    let b = fetch_b().await; // Waits for a
    let c = fetch_c().await; // Waits for b
}

// ✅ Efficient
async fn concurrent() {
    let (a, b, c) = tokio::join!(
        fetch_a(),
        fetch_b(),
        fetch_c()
    );
}
```

## Next Steps

After working through these examples, you should:

1. **Understand the fundamentals** of async programming in Rust
2. **Be able to implement** custom futures when needed
3. **Handle errors effectively** in async contexts
4. **Build complex async systems** using state machines
5. **Apply best practices** for production code

For more advanced topics, see:
- [Appendix B: Testing Strategies](./appendix-b-testing.md)
- [Appendix C: Common Pitfalls](./appendix-c-pitfalls.md)
- [Appendix D: Resources](./appendix-d-resources.md) 