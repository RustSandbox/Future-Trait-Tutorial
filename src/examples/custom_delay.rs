//! # Custom Delay Future Implementation
//!
//! This example demonstrates how to implement the Future trait manually.
//! It covers the core concepts of async programming in Rust:
//!
//! 1. Implementing the Future trait from scratch
//! 2. Understanding Poll::Ready vs Poll::Pending
//! 3. Working with Waker for efficient scheduling
//! 4. Managing shared state between threads
//! 5. Proper resource cleanup and cancellation safety

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

/// # Struct: SharedState
///
/// This struct represents the shared state between the Future and the
/// background thread that performs the actual timing work. It demonstrates
/// how to safely share mutable state across thread boundaries.
///
/// ## Fields:
/// - `completed`: Boolean flag indicating if the delay has finished
/// - `waker`: Optional Waker that the background thread uses to notify
///   the executor when the delay completes
///
/// ## Thread Safety:
/// - Wrapped in Arc<Mutex<_>> for safe sharing between threads
/// - The Mutex ensures only one thread can modify the state at a time
/// - Arc provides shared ownership across multiple threads
#[derive(Debug)]
struct SharedState {
    /// Indicates whether the delay operation has completed
    /// When true, the Future should return Poll::Ready
    completed: bool,

    /// The waker for the task that is waiting on this delay
    /// The background thread uses this to wake up the executor
    /// when the delay completes
    waker: Option<Waker>,
}

/// # Struct: DelayFuture
///
/// A custom Future implementation that completes after a specified duration.
/// This demonstrates the fundamental pattern for implementing custom futures:
///
/// 1. Store shared state that can be accessed by both the Future and external code
/// 2. Implement the Future trait with proper poll() logic
/// 3. Handle waker registration for efficient scheduling
///
/// ## Key Design Principles:
/// - **Lazy Execution**: Work only starts when the future is first polled
/// - **Cancellation Safe**: Can be dropped at any time without resource leaks
/// - **Efficient Scheduling**: Uses Waker to avoid busy-waiting
///
/// ## Fields:
/// - `shared_state`: Arc<Mutex<SharedState>> for thread-safe state sharing
/// - `duration`: The delay duration (stored for debugging/inspection)
pub struct DelayFuture {
    /// Shared state between the Future and the background timer thread
    shared_state: Arc<Mutex<SharedState>>,

    /// The duration this future will delay for
    /// Stored primarily for debugging and inspection purposes
    duration: Duration,

    /// Flag to track if we've started the background work
    /// This ensures we only spawn the timer thread once
    started: bool,
}

impl DelayFuture {
    /// # Function: new
    ///
    /// Creates a new DelayFuture that will complete after the specified duration.
    ///
    /// ## Important Design Decision:
    /// This constructor does NOT start the timer immediately. Following the
    /// principle of "lazy futures," the actual work (spawning the timer thread)
    /// only begins when the future is first polled.
    ///
    /// ## Arguments:
    /// - `duration`: How long to delay before completing
    ///
    /// ## Returns:
    /// - A new DelayFuture instance ready to be polled
    ///
    /// ## Example:
    /// ```rust
    /// let delay = DelayFuture::new(Duration::from_millis(100));
    /// // No work has started yet - the future is lazy!
    /// let result = delay.await; // Now the work begins
    /// ```
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        DelayFuture {
            shared_state,
            duration,
            started: false,
        }
    }

    /// # Function: start_timer
    ///
    /// Starts the background timer thread. This is called from poll()
    /// on the first poll to implement lazy execution.
    ///
    /// ## Key Implementation Details:
    /// - Spawns a new thread to perform the blocking sleep
    /// - The thread sleeps for the specified duration
    /// - When the sleep completes, it updates the shared state
    /// - If a waker was registered, it calls wake() to notify the executor
    ///
    /// ## Thread Safety:
    /// - Uses Arc::clone to share ownership of the state with the thread
    /// - The spawned thread takes ownership of its Arc clone
    /// - Mutex ensures safe concurrent access to the shared state
    fn start_timer(&mut self) {
        if self.started {
            return; // Already started, don't spawn multiple threads
        }

        self.started = true;
        let thread_shared_state = Arc::clone(&self.shared_state);
        let duration = self.duration;

        // Spawn a background thread to perform the blocking sleep
        // This keeps the async executor thread free to handle other tasks
        thread::spawn(move || {
            // Perform the blocking sleep operation
            // This is okay because we're in a dedicated thread
            thread::sleep(duration);

            // Update the shared state to indicate completion
            let mut state = thread_shared_state.lock().unwrap();
            state.completed = true;

            // If a waker was registered, wake up the task
            // This notifies the executor that this future is ready to be polled again
            if let Some(waker) = state.waker.take() {
                // Calling wake() schedules the task for re-polling
                waker.wake();
            }

            // The thread ends here, automatically cleaning up resources
        });
    }
}

/// # Implementation: Future for DelayFuture
///
/// This is the core implementation of the Future trait for our custom DelayFuture.
/// It demonstrates the fundamental polling pattern that all futures must implement.
///
/// ## Key Concepts Demonstrated:
/// 1. **State Checking**: First check if work is already complete
/// 2. **Lazy Initialization**: Start work only on first poll
/// 3. **Waker Registration**: Store the waker for later notification
/// 4. **Proper Return Values**: Return Ready when done, Pending when waiting
impl Future for DelayFuture {
    /// The type of value this Future produces when it completes
    /// In this case, we return a simple message string
    type Output = String;

    /// # Function: poll
    ///
    /// This is the heart of the Future trait. The executor calls this method
    /// to advance the future's progress. Our implementation follows the
    /// standard pattern for custom futures:
    ///
    /// ## Poll Implementation Pattern:
    /// 1. Check if the work is already complete → return Poll::Ready
    /// 2. If not complete, ensure background work has started
    /// 3. Register the current task's waker for later notification
    /// 4. Return Poll::Pending to indicate more work is needed
    ///
    /// ## Arguments:
    /// - `self`: Pin<&mut Self> - ensures the future won't move in memory
    /// - `cx`: &mut Context - provides access to the current task's waker
    ///
    /// ## Returns:
    /// - Poll::Ready(String) when the delay has completed
    /// - Poll::Pending when still waiting for the delay to finish
    ///
    /// ## Memory Safety:
    /// The Pin<&mut Self> parameter ensures that once this future is polled,
    /// it won't be moved in memory. This is crucial for futures that might
    /// contain self-references in their generated state machines.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Step 1: Acquire the lock on our shared state
        // This ensures thread-safe access to the completion flag and waker
        let mut shared_state = self.shared_state.lock().unwrap();

        // Step 2: Check if the delay has already completed
        if shared_state.completed {
            // The background thread has finished the delay
            // Return the final result and complete the future
            return Poll::Ready(format!(
                "Delay of {:?} completed successfully!",
                self.duration
            ));
        }

        // Step 3: If not completed, ensure the background timer has started
        // This implements the "lazy execution" principle - work only starts
        // when the future is actually polled by an executor
        drop(shared_state); // Release the lock before starting timer
        self.start_timer();
        let mut shared_state = self.shared_state.lock().unwrap(); // Re-acquire lock

        // Step 4: Register the current task's waker
        // This is crucial for efficient scheduling - it tells the background
        // thread how to notify the executor when the delay completes

        // Optimization: Use clone_from if a waker is already stored
        // This is more efficient than always cloning a new waker
        if let Some(existing_waker) = &mut shared_state.waker {
            // Update the existing waker to the current one
            // This handles cases where the future might be polled from different tasks
            existing_waker.clone_from(cx.waker());
        } else {
            // First time polling - store the waker
            shared_state.waker = Some(cx.waker().clone());
        }

        // Step 5: Return Pending to indicate the future is not ready yet
        // The executor will stop polling this future until wake() is called
        Poll::Pending
    }
}

/// # Function: demonstrate_custom_future_usage
///
/// This function shows various ways to use our custom DelayFuture,
/// demonstrating that it behaves just like any other future.
///
/// ## Key Learning Points:
/// - Custom futures integrate seamlessly with async/await
/// - They can be used with combinators like join!
/// - Multiple instances can run concurrently
/// - They follow the same cancellation semantics as built-in futures
async fn demonstrate_custom_future_usage() {
    println!("\n=== Using Custom DelayFuture ===");

    // Example 1: Basic usage with .await
    println!("1. Basic usage:");
    let start = Instant::now();
    let result = DelayFuture::new(Duration::from_millis(100)).await;
    let elapsed = start.elapsed();
    println!("   Result: {}", result);
    println!("   Actual time: {:?}", elapsed);

    // Example 2: Concurrent execution with multiple custom futures
    println!("\n2. Concurrent execution:");
    let start = Instant::now();

    let (result1, result2, result3) = tokio::join!(
        DelayFuture::new(Duration::from_millis(150)),
        DelayFuture::new(Duration::from_millis(100)),
        DelayFuture::new(Duration::from_millis(200))
    );

    let elapsed = start.elapsed();
    println!("   Results:");
    println!("     - {}", result1);
    println!("     - {}", result2);
    println!("     - {}", result3);
    println!("   Total time: {:?} (should be ~200ms, not 450ms)", elapsed);

    // Example 3: Mixing custom futures with built-in ones
    println!("\n3. Mixing with built-in futures:");
    let start = Instant::now();

    let (custom_result, builtin_result) = tokio::join!(
        DelayFuture::new(Duration::from_millis(75)),
        tokio::time::sleep(Duration::from_millis(75))
    );

    let elapsed = start.elapsed();
    println!("   Custom future result: {}", custom_result);
    println!("   Built-in future completed");
    println!("   Total time: {:?}", elapsed);
}

/// # Function: demonstrate_future_cancellation
///
/// This function demonstrates how futures can be cancelled (dropped)
/// and shows that our custom future handles cancellation gracefully.
///
/// ## Key Learning Points:
/// - Futures can be cancelled by dropping them
/// - Background threads continue running even if the future is dropped
/// - Proper cancellation handling requires careful design
/// - select! can be used to implement timeouts and cancellation
async fn demonstrate_future_cancellation() {
    println!("\n=== Future Cancellation ===");

    // Example 1: Timeout using select!
    println!("1. Timeout example:");
    let start = Instant::now();

    tokio::select! {
        result = DelayFuture::new(Duration::from_millis(200)) => {
            println!("   Delay completed: {}", result);
        }
        _ = tokio::time::sleep(Duration::from_millis(100)) => {
            println!("   Timeout occurred - future was cancelled");
        }
    }

    let elapsed = start.elapsed();
    println!("   Time elapsed: {:?}", elapsed);

    // Example 2: Explicit dropping
    println!("\n2. Explicit dropping:");
    let delay_future = DelayFuture::new(Duration::from_millis(500));
    println!("   Created future for 500ms delay");

    // Start polling the future but don't await it completely
    let mut pinned_future = Box::pin(delay_future);
    let waker = futures::task::noop_waker();
    let mut context = Context::from_waker(&waker);

    // Poll once to start the background work
    match pinned_future.as_mut().poll(&mut context) {
        Poll::Ready(result) => println!("   Unexpectedly ready: {}", result),
        Poll::Pending => println!("   Future is pending (as expected)"),
    }

    // Now drop the future
    drop(pinned_future);
    println!("   Future dropped - background thread may still be running");

    // Wait a bit to show the background thread continues
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("   Note: Background thread continues even after future is dropped");
}

/// # Function: demonstrate_poll_mechanics
///
/// This function provides a low-level demonstration of how polling works,
/// showing the actual Poll::Ready and Poll::Pending states.
///
/// ## Educational Value:
/// - Shows the raw polling interface that async/await abstracts away
/// - Demonstrates how executors interact with futures
/// - Illustrates the state transitions in future execution
async fn demonstrate_poll_mechanics() {
    println!("\n=== Low-Level Poll Mechanics ===");

    let mut delay_future = Box::pin(DelayFuture::new(Duration::from_millis(150)));
    let waker = futures::task::noop_waker();
    let mut context = Context::from_waker(&waker);

    println!("Manually polling the future:");

    let start = Instant::now();
    let mut poll_count = 0;

    loop {
        poll_count += 1;
        let elapsed = start.elapsed();

        match delay_future.as_mut().poll(&mut context) {
            Poll::Ready(result) => {
                println!("   Poll #{}: Ready after {:?}", poll_count, elapsed);
                println!("   Result: {}", result);
                break;
            }
            Poll::Pending => {
                println!("   Poll #{}: Pending after {:?}", poll_count, elapsed);
                // In a real executor, we would wait for wake() to be called
                // For demonstration, we'll just sleep a bit and poll again
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }

    println!("   Total polls: {}", poll_count);
}

/// # Function: main
///
/// The main function orchestrates all the demonstrations, showing
/// progressively more advanced concepts of custom Future implementation.
///
/// ## Learning Progression:
/// 1. Basic usage of custom futures
/// 2. Cancellation and timeout handling
/// 3. Low-level polling mechanics
/// 4. Integration with the broader async ecosystem
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Custom Future Implementation Tutorial");
    println!("=======================================");
    println!("This example demonstrates how to implement the Future trait manually.");

    // Demonstrate basic usage of our custom future
    demonstrate_custom_future_usage().await;

    // Show how cancellation works
    demonstrate_future_cancellation().await;

    // Demonstrate low-level polling mechanics
    demonstrate_poll_mechanics().await;

    println!("\n✅ Custom Future Tutorial completed!");
    println!("Key takeaways:");
    println!("  - Futures are lazy - work starts only when polled");
    println!("  - poll() returns Ready when complete, Pending when waiting");
    println!("  - Waker enables efficient scheduling without busy-waiting");
    println!("  - Custom futures integrate seamlessly with async/await");
    println!("  - Proper state management is crucial for thread safety");

    println!("\nNext: Try 'cargo run --bin combinators' to learn about future combinators");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tokio_test;

    /// Test basic functionality of our custom DelayFuture
    #[tokio::test]
    async fn test_delay_future_basic() {
        let start = Instant::now();
        let result = DelayFuture::new(Duration::from_millis(50)).await;
        let elapsed = start.elapsed();

        // Verify the result message
        assert!(result.contains("Delay of"));
        assert!(result.contains("completed successfully"));

        // Verify timing (with tolerance for test environment)
        assert!(elapsed >= Duration::from_millis(45));
        assert!(elapsed <= Duration::from_millis(100));
    }

    /// Test concurrent execution of multiple DelayFutures
    #[tokio::test]
    async fn test_delay_future_concurrent() {
        let start = Instant::now();

        let (r1, r2, r3) = tokio::join!(
            DelayFuture::new(Duration::from_millis(30)),
            DelayFuture::new(Duration::from_millis(50)),
            DelayFuture::new(Duration::from_millis(40))
        );

        let elapsed = start.elapsed();

        // All should complete successfully
        assert!(r1.contains("completed successfully"));
        assert!(r2.contains("completed successfully"));
        assert!(r3.contains("completed successfully"));

        // Should complete in roughly the time of the longest delay (50ms)
        assert!(elapsed >= Duration::from_millis(45));
        assert!(elapsed <= Duration::from_millis(80));
    }

    /// Test that DelayFuture can be cancelled with select!
    #[tokio::test]
    async fn test_delay_future_cancellation() {
        let start = Instant::now();
        let mut completed = false;

        tokio::select! {
            result = DelayFuture::new(Duration::from_millis(200)) => {
                completed = true;
                assert!(result.contains("completed successfully"));
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                // Timeout occurred - this is expected
            }
        }

        let elapsed = start.elapsed();

        // Should timeout, not complete
        assert!(!completed);
        assert!(elapsed >= Duration::from_millis(45));
        assert!(elapsed <= Duration::from_millis(80));
    }

    /// Test manual polling of DelayFuture
    #[tokio::test]
    async fn test_delay_future_manual_poll() {
        let mut delay_future = Box::pin(DelayFuture::new(Duration::from_millis(100)));
        let waker = futures::task::noop_waker();
        let mut context = Context::from_waker(&waker);

        // First poll should return Pending
        match delay_future.as_mut().poll(&mut context) {
            Poll::Ready(_) => panic!("Should not be ready immediately"),
            Poll::Pending => {} // Expected
        }

        // Wait for completion
        let result = delay_future.await;
        assert!(result.contains("completed successfully"));
    }
}
