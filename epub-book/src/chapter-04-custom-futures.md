# Chapter 4: Custom Future Implementation

## Introduction

While async/await syntax handles most use cases, understanding how to implement the Future trait manually is crucial for:

- Building custom async primitives
- Understanding how async/await works under the hood
- Creating specialized async components
- Debugging complex async issues

In this chapter, we'll explore the custom future implementation from our `custom_delay.rs` example and learn how to build futures from scratch.

## When to Implement Custom Futures

### Good Use Cases

✅ **Custom async primitives**
- Timers and delays
- Custom channels
- Async locks and synchronization primitives

✅ **Integration with external systems**
- Wrapping callback-based APIs
- Interfacing with C libraries
- Custom I/O operations

✅ **Performance-critical code**
- Zero-allocation futures
- Specialized state machines
- Custom executors

### When NOT to Implement Custom Futures

❌ **Regular application logic** - use async/await instead
❌ **Simple transformations** - use combinators
❌ **Standard patterns** - use existing crates

## Our Custom Delay Future Example

Let's examine the custom delay future from `src/examples/custom_delay.rs`:

**File reference**: `src/examples/custom_delay.rs`

```bash
# Run the custom delay example
cargo run --bin custom_delay

# Test the implementation
cargo test --bin custom_delay
```

### The DelayFuture Structure

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

/// Shared state between the future and the timer thread
struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

/// A future that completes after a specified duration
pub struct DelayFuture {
    shared_state: Arc<Mutex<SharedState>>,
    duration: Duration,
    started: bool,
}
```

### Key Design Patterns

1. **Shared State Pattern**: Using `Arc<Mutex<SharedState>>` for thread-safe communication
2. **Lazy Execution**: Timer only starts on first poll
3. **Waker Management**: Storing and using wakers for efficient scheduling
4. **Background Work**: Using threads for blocking operations

### Implementation Details

```rust
impl DelayFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        Self {
            shared_state,
            duration,
            started: false,
        }
    }

    fn start_timer(&mut self) {
        if self.started {
            return;
        }
        
        self.started = true;
        let shared_state = Arc::clone(&self.shared_state);
        let duration = self.duration;

        // Spawn a background thread to handle the timing
        thread::spawn(move || {
            thread::sleep(duration);
            
            let mut state = shared_state.lock().unwrap();
            state.completed = true;
            
            // Wake the future if a waker was registered
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        });
    }
}
```

### The Future Implementation

```rust
impl Future for DelayFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed {
            Poll::Ready(format!("Delay of {:?} completed!", self.duration))
        } else {
            // Start the timer on first poll (lazy execution)
            drop(shared_state); // Release lock before starting timer
            self.start_timer();
            let mut shared_state = self.shared_state.lock().unwrap();

            // Store the waker for later notification
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
```

## Key Concepts Demonstrated

### 1. Lazy Execution

The timer doesn't start until the future is first polled:

```rust
// Creating a future does NO work
let delay = DelayFuture::new(Duration::from_secs(1));

// Work only starts when polled/awaited
delay.await; // Timer starts here
```

### 2. Waker Management

Proper waker handling is crucial for efficient scheduling:

```rust
// Store waker when returning Pending
shared_state.waker = Some(cx.waker().clone());
Poll::Pending

// Wake the future when ready
if let Some(waker) = state.waker.take() {
    waker.wake(); // Notify executor to re-poll
}
```

### 3. Thread Safety

Using `Arc<Mutex<T>>` for safe shared state:

```rust
// Shared between future and background thread
let shared_state = Arc::new(Mutex::new(SharedState {
    completed: false,
    waker: None,
}));
```

### 4. Background Work

Offloading blocking operations to background threads:

```rust
// Don't block the executor thread
thread::spawn(move || {
    thread::sleep(duration); // Blocking operation
    // ... notify completion
});
```

## Advanced Patterns

### State Machine Implementation

Many custom futures are best implemented as state machines:

```rust
enum DelayState {
    NotStarted,
    Running { start_time: Instant },
    Completed,
}

impl Future for StateMachineDelay {
    type Output = Duration;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match &mut self.state {
                DelayState::NotStarted => {
                    self.state = DelayState::Running {
                        start_time: Instant::now(),
                    };
                    // Continue to next state
                }
                DelayState::Running { start_time } => {
                    let elapsed = start_time.elapsed();
                    if elapsed >= self.duration {
                        let total_time = elapsed;
                        self.state = DelayState::Completed;
                        return Poll::Ready(total_time);
                    } else {
                        // Set up waker and return Pending
                        self.setup_waker(cx.waker().clone());
                        return Poll::Pending;
                    }
                }
                DelayState::Completed => {
                    panic!("Future polled after completion");
                }
            }
        }
    }
}
```

## Best Practices

### 1. Always Implement Lazy Execution

```rust
// ❌ Wrong - starts work in constructor
impl DelayFuture {
    pub fn new(duration: Duration) -> Self {
        thread::spawn(/* timer logic */); // DON'T DO THIS
        Self { /* ... */ }
    }
}

// ✅ Correct - starts work on first poll
impl Future for DelayFuture {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            self.start_work(); // Start here
            self.started = true;
        }
        // ... rest of poll logic
    }
}
```

### 2. Proper Waker Management

```rust
// Always store waker when returning Pending
if self.is_ready() {
    Poll::Ready(self.get_result())
} else {
    self.waker = Some(cx.waker().clone()); // Store for later
    Poll::Pending
}
```

### 3. Cancellation Safety

```rust
impl Drop for DelayFuture {
    fn drop(&mut self) {
        // Clean up resources when future is dropped
        self.cancel_background_work();
    }
}
```

### 4. Never Block in Poll

```rust
// ❌ Wrong - blocks the executor
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    std::thread::sleep(Duration::from_secs(1)); // DON'T DO THIS
    Poll::Ready(())
}

// ✅ Correct - use background threads for blocking work
fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if !self.started {
        self.start_background_work(cx.waker().clone());
        self.started = true;
    }
    
    if self.is_complete() {
        Poll::Ready(self.get_result())
    } else {
        Poll::Pending
    }
}
```

## Testing Custom Futures

Our example includes comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_delay_future_timing() {
        let start = Instant::now();
        let result = DelayFuture::new(Duration::from_millis(100)).await;
        let elapsed = start.elapsed();

        assert!(result.contains("completed"));
        assert!(elapsed >= Duration::from_millis(95));
        assert!(elapsed <= Duration::from_millis(150));
    }

    #[tokio::test]
    async fn test_multiple_delays_concurrent() {
        let start = Instant::now();
        
        let (r1, r2, r3) = tokio::join!(
            DelayFuture::new(Duration::from_millis(50)),
            DelayFuture::new(Duration::from_millis(75)),
            DelayFuture::new(Duration::from_millis(100))
        );
        
        let elapsed = start.elapsed();
        
        // Should complete in time of longest delay
        assert!(elapsed >= Duration::from_millis(95));
        assert!(elapsed <= Duration::from_millis(150));
    }
}
```

## Common Patterns Summary

### Immediate Completion
```rust
fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    Poll::Ready(self.value)
}
```

### One-time Async Operation
```rust
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
```

### State Machine
```rust
fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    loop {
        match self.state {
            State::A => { /* transition to B */ }
            State::B => { /* transition to C or return Pending */ }
            State::C => return Poll::Ready(result),
        }
    }
}
```

## What's Next?

Now that you understand how to implement custom futures, let's explore how these concepts apply to building complex state machines and understanding the Pin mechanism in more detail.

In [Chapter 5: State Machines and Polling](./chapter-05-state-machines.md), we'll dive deeper into:
- Advanced state machine patterns
- Efficient polling strategies
- Managing complex async workflows
- Real-world state machine examples

---

## Exercises

1. **Implement a Counter Future**: Create a future that counts from 0 to N, yielding control between each count
2. **Build a Timeout Future**: Implement a future that fails if another future doesn't complete within a time limit
3. **Create a Retry Future**: Build a future that retries another future up to N times on failure

## Key Takeaways

- Custom futures are state machines at heart
- Always implement lazy execution
- Proper waker management is crucial
- Make futures cancellation-safe
- Never block in the poll method
- Use background threads for blocking operations
- Test thoroughly with timing assertions 