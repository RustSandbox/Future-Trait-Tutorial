//! # Basic Future Example
//!
//! This example demonstrates the fundamental concepts of async/await syntax
//! and how futures work in Rust. It covers:
//!
//! 1. Basic async function definition and usage
//! 2. Understanding future laziness
//! 3. Simple async operations with timing
//! 4. The difference between sync and async execution

use std::time::{Duration, Instant};
use tokio::time::sleep;

/// # Function: simple_async_operation
///
/// This is a basic async function that simulates some work by sleeping
/// for a specified duration. This demonstrates:
/// - How to define an async function using the `async` keyword
/// - How async functions return a Future that must be awaited
/// - How to use tokio::time::sleep for non-blocking delays
///
/// ## Arguments:
/// - `duration`: The amount of time to sleep, simulating work
/// - `message`: A message to print when the operation completes
///
/// ## Returns:
/// - A String containing the completion message
///
/// ## Example:
/// ```rust
/// let result = simple_async_operation(Duration::from_millis(100), "Task 1").await;
/// println!("{}", result); // Prints: "Task 1 completed after 100ms"
/// ```
async fn simple_async_operation(duration: Duration, message: &str) -> String {
    println!("Starting: {}", message);

    // Record the start time to measure actual duration
    let start = Instant::now();

    // This is a non-blocking sleep - it yields control back to the executor
    // The executor can run other tasks while this one is sleeping
    sleep(duration).await;

    let elapsed = start.elapsed();
    let result = format!("{} completed after {:?}", message, elapsed);
    println!("{}", result);

    result
}

/// # Function: demonstrate_future_laziness
///
/// This function demonstrates that futures in Rust are lazy - they don't
/// start executing until they are polled (awaited). This is a crucial
/// concept for understanding Rust's async model.
///
/// ## Key Learning Points:
/// - Creating a future doesn't start the work
/// - Only when you .await does the work begin
/// - Multiple futures can be created and stored before execution
async fn demonstrate_future_laziness() {
    println!("\n=== Demonstrating Future Laziness ===");

    // Step 1: Create futures but don't await them yet
    // Notice that "Starting: ..." messages don't appear yet
    println!("Creating futures (no work starts yet)...");

    let future1 = simple_async_operation(Duration::from_millis(100), "Lazy Task 1");
    let future2 = simple_async_operation(Duration::from_millis(150), "Lazy Task 2");

    println!("Futures created, but no work has started!");

    // Step 2: Now we await the futures, which starts their execution
    println!("Now awaiting the futures...");

    let start_time = Instant::now();

    // These run sequentially - total time will be ~250ms
    let result1 = future1.await;
    let result2 = future2.await;

    let total_time = start_time.elapsed();

    println!("Sequential execution completed in {:?}", total_time);
    println!("Result 1: {}", result1);
    println!("Result 2: {}", result2);
}

/// # Function: demonstrate_concurrent_execution
///
/// This function shows how to run multiple futures concurrently using
/// tokio::join!. This is one of the key benefits of async programming.
///
/// ## Key Learning Points:
/// - tokio::join! runs futures concurrently
/// - Total time is the maximum of individual times, not the sum
/// - All futures must complete for join! to return
async fn demonstrate_concurrent_execution() {
    println!("\n=== Demonstrating Concurrent Execution ===");

    let start_time = Instant::now();

    // tokio::join! runs all futures concurrently
    // Total time will be ~200ms (the longest task), not 450ms (sum of all)
    let (result1, result2, result3) = tokio::join!(
        simple_async_operation(Duration::from_millis(100), "Concurrent Task 1"),
        simple_async_operation(Duration::from_millis(200), "Concurrent Task 2"),
        simple_async_operation(Duration::from_millis(150), "Concurrent Task 3")
    );

    let total_time = start_time.elapsed();

    println!("Concurrent execution completed in {:?}", total_time);
    println!("Results:");
    println!("  - {}", result1);
    println!("  - {}", result2);
    println!("  - {}", result3);
}

/// # Function: demonstrate_async_vs_sync
///
/// This function compares synchronous and asynchronous execution patterns
/// to highlight the performance benefits of async programming.
///
/// ## Key Learning Points:
/// - Synchronous code blocks the thread
/// - Asynchronous code allows concurrent execution
/// - The performance difference can be significant for I/O-bound tasks
fn demonstrate_async_vs_sync() {
    println!("\n=== Comparing Sync vs Async Execution ===");

    // Simulate synchronous execution
    println!("Synchronous execution:");
    let sync_start = Instant::now();

    // In real sync code, these would be blocking I/O operations
    std::thread::sleep(Duration::from_millis(100)); // Simulating sync task 1
    std::thread::sleep(Duration::from_millis(150)); // Simulating sync task 2
    std::thread::sleep(Duration::from_millis(200)); // Simulating sync task 3

    let sync_duration = sync_start.elapsed();
    println!("Sync total time: {:?}", sync_duration);
}

/// # Function: demonstrate_error_in_async
///
/// This function shows how errors work in async functions and how they
/// can be handled using standard Rust error handling patterns.
///
/// ## Key Learning Points:
/// - Async functions can return Result<T, E>
/// - The ? operator works in async functions
/// - Errors propagate naturally through the async call stack
async fn demonstrate_error_in_async() -> Result<String, &'static str> {
    println!("\n=== Error Handling in Async Functions ===");

    // Simulate an operation that might fail
    let success = true; // Change to false to see error handling

    if success {
        let result = simple_async_operation(Duration::from_millis(50), "Error Demo Task").await;
        Ok(result)
    } else {
        Err("Simulated async error")
    }
}

/// # Function: main
///
/// The main function demonstrates all the concepts covered in this example.
/// It uses the #[tokio::main] attribute to set up an async runtime.
///
/// ## Key Learning Points:
/// - #[tokio::main] sets up the async runtime
/// - main() can be async when using this attribute
/// - All async operations must be awaited in main
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Basic Future Tutorial - Understanding Async/Await in Rust");
    println!("===========================================================");

    // Example 1: Demonstrate that futures are lazy
    demonstrate_future_laziness().await;

    // Example 2: Show concurrent execution with join!
    demonstrate_concurrent_execution().await;

    // Example 3: Compare with synchronous execution
    demonstrate_async_vs_sync();

    // Example 4: Error handling in async functions
    match demonstrate_error_in_async().await {
        Ok(result) => println!("\nAsync operation succeeded: {}", result),
        Err(error) => println!("\nAsync operation failed: {}", error),
    }

    // Example 5: Simple async closure (advanced concept)
    println!("\n=== Async Closures ===");

    let async_closure =
        || async { simple_async_operation(Duration::from_millis(75), "Closure Task").await };

    let closure_result = async_closure().await;
    println!("Closure result: {}", closure_result);

    println!("\n✅ Basic Future Tutorial completed successfully!");
    println!("Next: Try running 'cargo run --bin custom_delay' to learn about custom Future implementations");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    /// Test that demonstrates how to test async functions
    #[tokio::test]
    async fn test_simple_async_operation() {
        let start = Instant::now();
        let result = simple_async_operation(Duration::from_millis(50), "Test Task").await;
        let elapsed = start.elapsed();

        // Verify the result contains our message
        assert!(result.contains("Test Task"));
        assert!(result.contains("completed"));

        // Verify timing (with some tolerance for test environment)
        assert!(elapsed >= Duration::from_millis(45));
        assert!(elapsed <= Duration::from_millis(100));
    }

    /// Test concurrent execution timing
    #[tokio::test]
    async fn test_concurrent_execution_timing() {
        let start = Instant::now();

        // Run three tasks concurrently
        let (_r1, _r2, _r3) = tokio::join!(
            simple_async_operation(Duration::from_millis(50), "Test 1"),
            simple_async_operation(Duration::from_millis(75), "Test 2"),
            simple_async_operation(Duration::from_millis(100), "Test 3")
        );

        let elapsed = start.elapsed();

        // Should complete in roughly the time of the longest task (100ms)
        // not the sum of all tasks (225ms)
        assert!(elapsed >= Duration::from_millis(95));
        assert!(elapsed <= Duration::from_millis(150));
    }

    /// Test error handling in async functions
    #[tokio::test]
    async fn test_async_error_handling() {
        let result = demonstrate_error_in_async().await;

        // Should succeed with our current implementation
        assert!(result.is_ok());

        let success_message = result.unwrap();
        assert!(success_message.contains("Error Demo Task"));
    }
}
