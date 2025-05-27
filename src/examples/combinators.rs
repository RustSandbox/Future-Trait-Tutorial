//! # Future Combinators Tutorial
//!
//! This example demonstrates how to use various future combinators to compose
//! complex asynchronous workflows. It covers:
//!
//! 1. Basic combinators: map, and_then, or_else
//! 2. Concurrent combinators: join!, try_join!, select!
//! 3. Collection combinators: join_all, try_join_all
//! 4. Dynamic collections: FuturesUnordered, FuturesOrdered
//! 5. Custom combinator implementations
//! 6. Real-world composition patterns

use futures::{
    future::{join_all, try_join_all, FutureExt, TryFutureExt},
    stream::{FuturesUnordered, StreamExt},
    Future,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// # Function: simulate_api_call
///
/// Simulates an API call that might succeed or fail.
/// This function demonstrates how to create futures that return Results,
/// which is essential for error handling in async code.
///
/// ## Arguments:
/// - `name`: The name of the API endpoint being called
/// - `delay`: How long the API call takes to complete
/// - `should_succeed`: Whether this call should succeed or fail
///
/// ## Returns:
/// - `Result<String, String>`: Success message or error message
///
/// ## Example:
/// ```rust
/// let result = simulate_api_call("users", Duration::from_millis(100), true).await;
/// match result {
///     Ok(data) => println!("API call succeeded: {}", data),
///     Err(error) => println!("API call failed: {}", error),
/// }
/// ```
async fn simulate_api_call(
    name: &str,
    delay: Duration,
    should_succeed: bool,
) -> Result<String, String> {
    println!("🌐 Starting API call to '{}'", name);

    // Simulate network delay
    sleep(delay).await;

    if should_succeed {
        let response = format!("Data from '{}' API (took {:?})", name, delay);
        println!("✅ API call to '{}' succeeded", name);
        Ok(response)
    } else {
        let error = format!("Failed to call '{}' API", name);
        println!("❌ API call to '{}' failed", name);
        Err(error)
    }
}

/// # Function: simulate_database_query
///
/// Simulates a database query operation.
/// This demonstrates another type of async operation that we can compose
/// with other futures using combinators.
///
/// ## Arguments:
/// - `table`: The database table being queried
/// - `delay`: Query execution time
///
/// ## Returns:
/// - A string containing the query result
async fn simulate_database_query(table: &str, delay: Duration) -> String {
    println!("🗄️  Executing database query on table '{}'", table);
    sleep(delay).await;
    let result = format!("Query result from table '{}' (took {:?})", table, delay);
    println!("✅ Database query on '{}' completed", table);
    result
}

/// # Function: demonstrate_map_combinator
///
/// Demonstrates the `map` combinator, which transforms the output of a future.
/// This is similar to the map function on iterators, but for async operations.
///
/// ## Key Learning Points:
/// - `map` transforms the success value of a future
/// - The transformation function is applied only if the future succeeds
/// - `map` preserves the error type for Result futures
/// - Chaining multiple maps creates a transformation pipeline
async fn demonstrate_map_combinator() {
    println!("\n=== Map Combinator ===");

    // Example 1: Simple map transformation
    println!("1. Basic map transformation:");
    let result = simulate_database_query("users", Duration::from_millis(100))
        .map(|data| {
            println!("   Transforming data...");
            format!("TRANSFORMED: {}", data.to_uppercase())
        })
        .await;

    println!("   Final result: {}", result);

    // Example 2: Chaining multiple maps
    println!("\n2. Chaining multiple maps:");
    let result = simulate_api_call("products", Duration::from_millis(80), true)
        .map_ok(|data| {
            println!("   First transformation: adding prefix");
            format!("PREFIX_{}", data)
        })
        .map_ok(|data| {
            println!("   Second transformation: adding suffix");
            format!("{}_SUFFIX", data)
        })
        .await;

    match result {
        Ok(data) => println!("   Chained result: {}", data),
        Err(error) => println!("   Error: {}", error),
    }

    // Example 3: Map with error handling
    println!("\n3. Map with potential error:");
    let result = simulate_api_call("invalid", Duration::from_millis(50), false)
        .map_ok(|data| {
            println!("   This transformation won't run due to error");
            format!("TRANSFORMED: {}", data)
        })
        .await;

    match result {
        Ok(data) => println!("   Unexpected success: {}", data),
        Err(error) => println!("   Expected error: {}", error),
    }
}

/// # Function: demonstrate_and_then_combinator
///
/// Demonstrates the `and_then` combinator, which chains futures sequentially.
/// This is used when the output of one future is needed as input for the next.
///
/// ## Key Learning Points:
/// - `and_then` chains futures where the second depends on the first
/// - The second future is only created if the first succeeds
/// - Errors short-circuit the chain
/// - This enables sequential async workflows
async fn demonstrate_and_then_combinator() {
    println!("\n=== And Then Combinator ===");

    // Example 1: Sequential API calls where second depends on first
    println!("1. Sequential dependent operations:");
    let start = Instant::now();

    let result = simulate_api_call("auth", Duration::from_millis(100), true)
        .and_then(|auth_token| async move {
            println!("   Using auth token: {}", auth_token);
            // Second API call that depends on the first
            simulate_api_call("user_data", Duration::from_millis(150), true).await
        })
        .and_then(|user_data| async move {
            println!("   Processing user data: {}", user_data);
            // Third operation that depends on the second
            simulate_database_query("user_preferences", Duration::from_millis(80)).await;
            Ok(format!("Complete user profile based on: {}", user_data))
        })
        .await;

    let elapsed = start.elapsed();
    match result {
        Ok(profile) => {
            println!("   Final profile: {}", profile);
            println!("   Total time: {:?} (sequential)", elapsed);
        }
        Err(error) => println!("   Chain failed: {}", error),
    }

    // Example 2: Error handling in chains
    println!("\n2. Error handling in and_then chains:");
    let result = simulate_api_call("auth", Duration::from_millis(50), true)
        .and_then(|_auth_token| async move {
            // This call will fail
            simulate_api_call("protected_data", Duration::from_millis(100), false).await
        })
        .and_then(|_data| async move {
            println!("   This won't execute due to previous error");
            Ok("This won't be reached".to_string())
        })
        .await;

    match result {
        Ok(data) => println!("   Unexpected success: {}", data),
        Err(error) => println!("   Expected chain failure: {}", error),
    }
}

/// # Function: demonstrate_join_combinators
///
/// Demonstrates various join combinators for concurrent execution.
/// These are essential for running multiple independent operations in parallel.
///
/// ## Key Learning Points:
/// - `join!` runs futures concurrently and waits for all to complete
/// - `try_join!` is like join! but short-circuits on first error
/// - Concurrent execution can significantly improve performance
/// - All futures must complete for join to return
async fn demonstrate_join_combinators() {
    println!("\n=== Join Combinators ===");

    // Example 1: Basic concurrent execution with join!
    println!("1. Concurrent execution with join!:");
    let start = Instant::now();

    let (api_result, db_result, cache_result) = tokio::join!(
        simulate_api_call("orders", Duration::from_millis(150), true),
        simulate_database_query("inventory", Duration::from_millis(200)),
        simulate_database_query("cache", Duration::from_millis(100))
    );

    let elapsed = start.elapsed();
    println!("   API result: {:?}", api_result);
    println!("   DB result: {}", db_result);
    println!("   Cache result: {}", cache_result);
    println!(
        "   Total time: {:?} (concurrent, ~200ms not 450ms)",
        elapsed
    );

    // Example 2: try_join! with error handling
    println!("\n2. try_join! with error handling:");
    let start = Instant::now();

    let result = tokio::try_join!(
        simulate_api_call("service1", Duration::from_millis(100), true),
        simulate_api_call("service2", Duration::from_millis(150), true),
        simulate_api_call("service3", Duration::from_millis(120), true)
    );

    let elapsed = start.elapsed();
    match result {
        Ok((s1, s2, s3)) => {
            println!("   All services succeeded:");
            println!("     Service 1: {}", s1);
            println!("     Service 2: {}", s2);
            println!("     Service 3: {}", s3);
            println!("   Total time: {:?}", elapsed);
        }
        Err(error) => println!("   One service failed: {}", error),
    }

    // Example 3: try_join! with failure (short-circuiting)
    println!("\n3. try_join! with failure (demonstrates short-circuiting):");
    let start = Instant::now();

    let result = tokio::try_join!(
        simulate_api_call("fast_service", Duration::from_millis(50), true),
        simulate_api_call("failing_service", Duration::from_millis(100), false),
        simulate_api_call("slow_service", Duration::from_millis(300), true)
    );

    let elapsed = start.elapsed();
    match result {
        Ok(_) => println!("   Unexpected success"),
        Err(error) => {
            println!("   Expected failure: {}", error);
            println!(
                "   Time: {:?} (note: may complete before slow_service)",
                elapsed
            );
        }
    }
}

/// # Function: demonstrate_select_combinator
///
/// Demonstrates the `select!` macro for racing futures.
/// This is useful for timeouts, fallbacks, and "first wins" scenarios.
///
/// ## Key Learning Points:
/// - `select!` returns as soon as any future completes
/// - Unfinished futures are cancelled (dropped)
/// - Useful for implementing timeouts and fallback strategies
/// - Can handle both success and error cases
async fn demonstrate_select_combinator() {
    println!("\n=== Select Combinator ===");

    // Example 1: Racing multiple API calls (fastest wins)
    println!("1. Racing API calls (fastest wins):");
    let start = Instant::now();

    tokio::select! {
        result = simulate_api_call("fast_api", Duration::from_millis(100), true) => {
            println!("   Fast API won: {:?}", result);
        }
        result = simulate_api_call("slow_api", Duration::from_millis(300), true) => {
            println!("   Slow API won: {:?}", result);
        }
        result = simulate_api_call("medium_api", Duration::from_millis(200), true) => {
            println!("   Medium API won: {:?}", result);
        }
    }

    let elapsed = start.elapsed();
    println!("   Race completed in: {:?}", elapsed);

    // Example 2: Timeout implementation
    println!("\n2. Timeout implementation:");
    let start = Instant::now();

    tokio::select! {
        result = simulate_api_call("slow_operation", Duration::from_millis(300), true) => {
            println!("   Operation completed: {:?}", result);
        }
        _ = sleep(Duration::from_millis(150)) => {
            println!("   Operation timed out after 150ms");
        }
    }

    let elapsed = start.elapsed();
    println!("   Timeout example completed in: {:?}", elapsed);

    // Example 3: Fallback strategy
    println!("\n3. Fallback strategy:");
    let start = Instant::now();

    tokio::select! {
        result = simulate_api_call("primary_service", Duration::from_millis(200), false) => {
            match result {
                Ok(data) => println!("   Primary service succeeded: {}", data),
                Err(error) => println!("   Primary service failed: {}", error),
            }
        }
        result = simulate_api_call("backup_service", Duration::from_millis(250), true) => {
            match result {
                Ok(data) => println!("   Backup service succeeded: {}", data),
                Err(error) => println!("   Backup service failed: {}", error),
            }
        }
    }

    let elapsed = start.elapsed();
    println!("   Fallback completed in: {:?}", elapsed);
}

/// # Function: demonstrate_collection_combinators
///
/// Demonstrates combinators that work with collections of futures.
/// These are essential for handling dynamic numbers of async operations.
///
/// ## Key Learning Points:
/// - `join_all` waits for all futures in a collection to complete
/// - `try_join_all` is like join_all but fails fast on first error
/// - Collections can be built dynamically at runtime
/// - Results maintain the same order as input futures
async fn demonstrate_collection_combinators() {
    println!("\n=== Collection Combinators ===");

    // Example 1: join_all with dynamic collection
    println!("1. join_all with dynamic collection:");
    let start = Instant::now();

    // Build a collection of futures dynamically
    let mut futures = Vec::new();
    for i in 1..=5 {
        let delay = Duration::from_millis(50 + i * 20);
        futures.push(simulate_database_query(&format!("table_{}", i), delay));
    }

    // Wait for all futures to complete
    let results = join_all(futures).await;
    let elapsed = start.elapsed();

    println!("   All queries completed:");
    for (i, result) in results.iter().enumerate() {
        println!("     Query {}: {}", i + 1, result);
    }
    println!("   Total time: {:?}", elapsed);

    // Example 2: try_join_all with potential failures
    println!("\n2. try_join_all with potential failures:");
    let start = Instant::now();

    let api_futures = vec![
        Box::pin(simulate_api_call("api1", Duration::from_millis(80), true)),
        Box::pin(simulate_api_call("api2", Duration::from_millis(120), true)),
        Box::pin(simulate_api_call("api3", Duration::from_millis(100), true)),
        Box::pin(simulate_api_call("api4", Duration::from_millis(90), true)),
    ];

    let result = try_join_all(api_futures).await;
    let elapsed = start.elapsed();

    match result {
        Ok(responses) => {
            println!("   All API calls succeeded:");
            for (i, response) in responses.iter().enumerate() {
                println!("     API {}: {}", i + 1, response);
            }
        }
        Err(error) => println!("   One API call failed: {}", error),
    }
    println!("   Total time: {:?}", elapsed);

    // Example 3: try_join_all with failure
    println!("\n3. try_join_all with failure (fail-fast behavior):");
    let start = Instant::now();

    let mixed_futures = vec![
        Box::pin(simulate_api_call(
            "good_api1",
            Duration::from_millis(100),
            true,
        )),
        Box::pin(simulate_api_call(
            "bad_api",
            Duration::from_millis(150),
            false,
        )), // This will fail
        Box::pin(simulate_api_call(
            "good_api2",
            Duration::from_millis(300),
            true,
        )), // Won't complete
    ];

    let result = try_join_all(mixed_futures).await;
    let elapsed = start.elapsed();

    match result {
        Ok(_) => println!("   Unexpected success"),
        Err(error) => {
            println!("   Expected failure: {}", error);
            println!("   Failed fast in: {:?} (before slow operation)", elapsed);
        }
    }
}

/// # Function: demonstrate_futures_unordered
///
/// Demonstrates FuturesUnordered for processing results as they complete.
/// This is useful when you want to handle results immediately rather than
/// waiting for all futures to complete.
///
/// ## Key Learning Points:
/// - FuturesUnordered processes futures as they complete
/// - Results come back in completion order, not submission order
/// - Useful for streaming results and early processing
/// - Can handle dynamic addition of new futures
async fn demonstrate_futures_unordered() {
    println!("\n=== FuturesUnordered ===");

    // Example 1: Processing results as they complete
    println!("1. Processing results as they complete:");
    let start = Instant::now();

    let mut unordered = FuturesUnordered::new();

    // Add futures with different completion times
    unordered.push(simulate_api_call("slow", Duration::from_millis(200), true));
    unordered.push(simulate_api_call("fast", Duration::from_millis(50), true));
    unordered.push(simulate_api_call(
        "medium",
        Duration::from_millis(100),
        true,
    ));

    println!("   Processing results as they arrive:");
    let mut count = 0;
    while let Some(result) = unordered.next().await {
        count += 1;
        let elapsed = start.elapsed();
        match result {
            Ok(data) => println!("     Result {}: {} (at {:?})", count, data, elapsed),
            Err(error) => println!("     Error {}: {} (at {:?})", count, error, elapsed),
        }
    }

    let total_elapsed = start.elapsed();
    println!("   All results processed in: {:?}", total_elapsed);

    // Example 2: Dynamic addition of futures
    println!("\n2. Dynamic addition of futures:");
    let start = Instant::now();

    let mut unordered = FuturesUnordered::new();

    // Start with some initial futures
    unordered.push(simulate_database_query(
        "initial1",
        Duration::from_millis(100),
    ));
    unordered.push(simulate_database_query(
        "initial2",
        Duration::from_millis(150),
    ));

    let mut processed = 0;
    let mut added_dynamic = false;

    while let Some(result) = unordered.next().await {
        processed += 1;
        let elapsed = start.elapsed();
        println!("     Processed: {} (at {:?})", result, elapsed);

        // Dynamically add more futures after processing the first result
        if processed == 1 && !added_dynamic {
            println!("   Adding dynamic futures...");
            unordered.push(simulate_database_query(
                "dynamic1",
                Duration::from_millis(80),
            ));
            unordered.push(simulate_database_query(
                "dynamic2",
                Duration::from_millis(120),
            ));
            added_dynamic = true;
        }
    }

    let total_elapsed = start.elapsed();
    println!("   Dynamic processing completed in: {:?}", total_elapsed);
}

/// # Function: demonstrate_custom_combinator
///
/// Demonstrates how to create custom combinators by implementing them
/// as functions that take and return futures.
///
/// ## Key Learning Points:
/// - Custom combinators encapsulate common async patterns
/// - They can be reused across different parts of an application
/// - Combinators compose well with existing async/await code
/// - They help create domain-specific async abstractions
async fn demonstrate_custom_combinator() {
    println!("\n=== Custom Combinators ===");

    /// # Function: with_retry
    ///
    /// A custom combinator that retries a future operation up to a specified
    /// number of times if it fails. This demonstrates how to create reusable
    /// async patterns.
    ///
    /// ## Arguments:
    /// - `future_fn`: A function that creates the future to retry
    /// - `max_retries`: Maximum number of retry attempts
    ///
    /// ## Returns:
    /// - The result of the future, or the last error if all retries fail
    async fn with_retry<F, Fut, T, E>(mut future_fn: F, max_retries: usize) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempts = 0;

        loop {
            attempts += 1;
            println!("     Attempt {} of {}", attempts, max_retries + 1);

            match future_fn().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempts > max_retries {
                        println!("     All retries exhausted");
                        return Err(error);
                    }
                    println!("     Attempt failed: {}, retrying...", error);
                    sleep(Duration::from_millis(100)).await; // Brief delay between retries
                }
            }
        }
    }

    /// # Function: with_timeout
    ///
    /// A custom combinator that adds a timeout to any future.
    /// This shows how to compose existing combinators into new ones.
    ///
    /// ## Arguments:
    /// - `future`: The future to add a timeout to
    /// - `timeout`: The timeout duration
    ///
    /// ## Returns:
    /// - The future's result or a timeout error
    async fn with_timeout<F, T>(future: F, timeout: Duration) -> Result<T, &'static str>
    where
        F: Future<Output = T>,
    {
        tokio::select! {
            result = future => Ok(result),
            _ = sleep(timeout) => Err("Operation timed out"),
        }
    }

    // Example 1: Using the retry combinator
    println!("1. Custom retry combinator:");
    let start = Instant::now();

    let result = with_retry(
        || simulate_api_call("unreliable_service", Duration::from_millis(50), false),
        3, // Retry up to 3 times
    )
    .await;

    let elapsed = start.elapsed();
    match result {
        Ok(data) => println!("   Retry succeeded: {}", data),
        Err(error) => println!("   Retry failed after all attempts: {}", error),
    }
    println!("   Total time: {:?}", elapsed);

    // Example 2: Using the timeout combinator
    println!("\n2. Custom timeout combinator:");
    let start = Instant::now();

    let result = with_timeout(
        simulate_api_call("slow_service", Duration::from_millis(200), true),
        Duration::from_millis(100), // 100ms timeout
    )
    .await;

    let elapsed = start.elapsed();
    match result {
        Ok(api_result) => match api_result {
            Ok(data) => println!("   Operation completed: {}", data),
            Err(error) => println!("   Operation failed: {}", error),
        },
        Err(timeout_error) => println!("   {}", timeout_error),
    }
    println!("   Total time: {:?}", elapsed);

    // Example 3: Combining custom combinators
    println!("\n3. Combining custom combinators:");
    let start = Instant::now();

    let result = with_timeout(
        with_retry(
            || simulate_api_call("flaky_service", Duration::from_millis(80), true),
            2,
        ),
        Duration::from_millis(500), // Overall timeout
    )
    .await;

    let elapsed = start.elapsed();
    match result {
        Ok(retry_result) => match retry_result {
            Ok(data) => println!("   Combined operation succeeded: {}", data),
            Err(error) => println!("   Retry failed: {}", error),
        },
        Err(timeout_error) => println!("   {}", timeout_error),
    }
    println!("   Total time: {:?}", elapsed);
}

/// # Function: main
///
/// The main function demonstrates all the combinator patterns in a
/// progressive manner, from simple transformations to complex compositions.
///
/// ## Learning Progression:
/// 1. Basic transformations with map
/// 2. Sequential composition with and_then
/// 3. Concurrent execution with join variants
/// 4. Racing and timeouts with select
/// 5. Collection processing
/// 6. Stream-like processing with FuturesUnordered
/// 7. Custom combinator creation
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Future Combinators Tutorial");
    println!("==============================");
    println!("This example demonstrates various ways to compose futures using combinators.");

    // Basic transformation combinators
    demonstrate_map_combinator().await;

    // Sequential composition
    demonstrate_and_then_combinator().await;

    // Concurrent execution
    demonstrate_join_combinators().await;

    // Racing and selection
    demonstrate_select_combinator().await;

    // Collection processing
    demonstrate_collection_combinators().await;

    // Stream-like processing
    demonstrate_futures_unordered().await;

    // Custom combinators
    demonstrate_custom_combinator().await;

    println!("\n✅ Combinators Tutorial completed!");
    println!("Key takeaways:");
    println!("  - map: Transform future outputs");
    println!("  - and_then: Chain dependent operations sequentially");
    println!("  - join!: Run independent operations concurrently");
    println!("  - try_join!: Concurrent with fail-fast error handling");
    println!("  - select!: Race futures, first one wins");
    println!("  - join_all/try_join_all: Handle collections of futures");
    println!("  - FuturesUnordered: Process results as they complete");
    println!("  - Custom combinators: Create reusable async patterns");

    println!("\nNext: Try 'cargo run --bin error_handling' to learn about error handling patterns");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    /// Test map combinator functionality
    #[tokio::test]
    async fn test_map_combinator() {
        let result = simulate_database_query("test", Duration::from_millis(10))
            .map(|data| format!("MAPPED: {}", data))
            .await;

        assert!(result.contains("MAPPED:"));
        assert!(result.contains("test"));
    }

    /// Test and_then combinator with success case
    #[tokio::test]
    async fn test_and_then_success() {
        let result = simulate_api_call("test", Duration::from_millis(10), true)
            .and_then(|_| async { Ok("second_result".to_string()) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "second_result");
    }

    /// Test and_then combinator with error case
    #[tokio::test]
    async fn test_and_then_error() {
        let result = simulate_api_call("test", Duration::from_millis(10), false)
            .and_then(|_| async { Ok("should_not_reach".to_string()) })
            .await;

        assert!(result.is_err());
    }

    /// Test concurrent execution timing
    #[tokio::test]
    async fn test_concurrent_timing() {
        let start = Instant::now();

        let (_r1, _r2, _r3) = tokio::join!(
            simulate_api_call("test1", Duration::from_millis(50), true),
            simulate_api_call("test2", Duration::from_millis(60), true),
            simulate_api_call("test3", Duration::from_millis(40), true)
        );

        let elapsed = start.elapsed();

        // Should complete in roughly the time of the longest operation (60ms)
        // not the sum of all operations (150ms)
        assert!(elapsed >= Duration::from_millis(55));
        assert!(elapsed <= Duration::from_millis(100));
    }

    /// Test try_join! fail-fast behavior
    #[tokio::test]
    async fn test_try_join_fail_fast() {
        let start = Instant::now();

        let result = tokio::try_join!(
            simulate_api_call("fast_success", Duration::from_millis(30), true),
            simulate_api_call("fast_failure", Duration::from_millis(50), false),
            simulate_api_call("slow_success", Duration::from_millis(200), true)
        );

        let elapsed = start.elapsed();

        // Should fail and complete quickly, not wait for the slow operation
        assert!(result.is_err());
        assert!(elapsed >= Duration::from_millis(45));
        assert!(elapsed <= Duration::from_millis(100)); // Much less than 200ms
    }

    /// Test select! racing behavior
    #[tokio::test]
    async fn test_select_racing() {
        let start = Instant::now();
        let mut fast_won = false;

        tokio::select! {
            _ = simulate_api_call("fast", Duration::from_millis(30), true) => {
                fast_won = true;
            }
            _ = simulate_api_call("slow", Duration::from_millis(100), true) => {
                fast_won = false;
            }
        }

        let elapsed = start.elapsed();

        // Fast operation should win
        assert!(fast_won);
        assert!(elapsed >= Duration::from_millis(25));
        assert!(elapsed <= Duration::from_millis(60));
    }

    /// Test collection combinators
    #[tokio::test]
    async fn test_join_all() {
        let futures = vec![
            simulate_database_query("table1", Duration::from_millis(20)),
            simulate_database_query("table2", Duration::from_millis(30)),
            simulate_database_query("table3", Duration::from_millis(25)),
        ];

        let results = join_all(futures).await;

        assert_eq!(results.len(), 3);
        assert!(results[0].contains("table1"));
        assert!(results[1].contains("table2"));
        assert!(results[2].contains("table3"));
    }
}
