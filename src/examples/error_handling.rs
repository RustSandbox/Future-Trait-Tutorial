//! # Error Handling in Async Rust Tutorial
//!
//! This example demonstrates comprehensive error handling patterns in async Rust.
//! It covers:
//!
//! 1. Basic Result<T, E> patterns with async functions
//! 2. Error propagation with the ? operator
//! 3. Custom error types with thiserror
//! 4. Timeout and cancellation error handling
//! 5. Error recovery and fallback strategies
//! 6. Collecting and handling multiple errors
//! 7. Best practices for async error handling

use anyhow::{Context, Result as AnyhowResult};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::time::{sleep, timeout};

/// # Enum: ApiError
///
/// A custom error type that demonstrates how to create domain-specific
/// errors for async operations. Using thiserror makes error creation
/// and handling much more ergonomic.
///
/// ## Error Variants:
/// - `NetworkError`: Represents network-related failures
/// - `AuthenticationError`: Authentication/authorization failures
/// - `RateLimitError`: API rate limiting errors
/// - `ValidationError`: Input validation errors
/// - `TimeoutError`: Operation timeout errors
/// - `ServiceUnavailable`: Service is temporarily unavailable
///
/// ## Key Features:
/// - Implements Display and Error traits automatically via thiserror
/// - Each variant can carry additional context data
/// - Can be easily converted to/from other error types
#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Authentication failed: {reason}")]
    AuthenticationError { reason: String },

    #[error("Rate limit exceeded. Retry after {retry_after} seconds")]
    RateLimitError { retry_after: u64 },

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Operation timed out after {duration:?}")]
    TimeoutError { duration: Duration },

    #[error("Service temporarily unavailable")]
    ServiceUnavailable,
}

/// # Enum: DatabaseError
///
/// Another custom error type for database operations, demonstrating
/// how different subsystems can have their own error types.
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {details}")]
    ConnectionFailed { details: String },

    #[error("Query failed: {query} - {error}")]
    QueryFailed { query: String, error: String },

    #[error("Transaction rolled back: {reason}")]
    TransactionFailed { reason: String },

    #[error("Database is locked")]
    DatabaseLocked,
}

/// # Function: simulate_api_request
///
/// Simulates an API request that can fail in various ways.
/// This demonstrates how to create async functions that return
/// custom error types and handle different failure scenarios.
///
/// ## Arguments:
/// - `endpoint`: The API endpoint being called
/// - `should_succeed`: Whether the request should succeed
/// - `error_type`: What type of error to simulate if it fails
///
/// ## Returns:
/// - `Result<String, ApiError>`: Success data or specific error type
///
/// ## Example:
/// ```rust
/// match simulate_api_request("users", false, "network").await {
///     Ok(data) => println!("Success: {}", data),
///     Err(ApiError::NetworkError { message }) => {
///         println!("Network failed: {}", message);
///     }
///     Err(e) => println!("Other error: {}", e),
/// }
/// ```
async fn simulate_api_request(
    endpoint: &str,
    should_succeed: bool,
    error_type: &str,
) -> Result<String, ApiError> {
    println!("🌐 Making API request to '{}'", endpoint);

    // Simulate network delay
    sleep(Duration::from_millis(100)).await;

    if should_succeed {
        let response = format!("API response from '{}' endpoint", endpoint);
        println!("✅ API request to '{}' succeeded", endpoint);
        Ok(response)
    } else {
        let error = match error_type {
            "network" => ApiError::NetworkError {
                message: format!("Failed to connect to {}", endpoint),
            },
            "auth" => ApiError::AuthenticationError {
                reason: "Invalid API key".to_string(),
            },
            "rate_limit" => ApiError::RateLimitError { retry_after: 60 },
            "validation" => ApiError::ValidationError {
                field: "user_id".to_string(),
                message: "Must be a positive integer".to_string(),
            },
            "timeout" => ApiError::TimeoutError {
                duration: Duration::from_millis(5000),
            },
            _ => ApiError::ServiceUnavailable,
        };

        println!("❌ API request to '{}' failed: {}", endpoint, error);
        Err(error)
    }
}

/// # Function: simulate_database_operation
///
/// Simulates a database operation that can fail in database-specific ways.
/// This demonstrates how different subsystems can have their own error types.
///
/// ## Arguments:
/// - `operation`: The database operation being performed
/// - `should_succeed`: Whether the operation should succeed
/// - `error_type`: What type of database error to simulate
///
/// ## Returns:
/// - `Result<String, DatabaseError>`: Success data or database error
async fn simulate_database_operation(
    operation: &str,
    should_succeed: bool,
    error_type: &str,
) -> Result<String, DatabaseError> {
    println!("🗄️  Executing database operation: '{}'", operation);

    // Simulate database processing time
    sleep(Duration::from_millis(80)).await;

    if should_succeed {
        let result = format!("Database operation '{}' completed successfully", operation);
        println!("✅ Database operation '{}' succeeded", operation);
        Ok(result)
    } else {
        let error = match error_type {
            "connection" => DatabaseError::ConnectionFailed {
                details: "Connection pool exhausted".to_string(),
            },
            "query" => DatabaseError::QueryFailed {
                query: operation.to_string(),
                error: "Syntax error in SQL".to_string(),
            },
            "transaction" => DatabaseError::TransactionFailed {
                reason: "Deadlock detected".to_string(),
            },
            _ => DatabaseError::DatabaseLocked,
        };

        println!("❌ Database operation '{}' failed: {}", operation, error);
        Err(error)
    }
}

/// # Function: demonstrate_basic_error_handling
///
/// Demonstrates basic error handling patterns with async functions.
/// Shows how to use match statements and if-let patterns with Result types.
///
/// ## Key Learning Points:
/// - Pattern matching on Result types in async contexts
/// - Handling specific error variants
/// - Converting between error types
/// - Using context to add additional error information
async fn demonstrate_basic_error_handling() {
    println!("\n=== Basic Error Handling ===");

    // Example 1: Successful operation
    println!("1. Successful operation:");
    match simulate_api_request("users", true, "").await {
        Ok(data) => println!("   Success: {}", data),
        Err(error) => println!("   Unexpected error: {}", error),
    }

    // Example 2: Handling specific error types
    println!("\n2. Handling specific error types:");
    match simulate_api_request("protected", false, "auth").await {
        Ok(data) => println!("   Unexpected success: {}", data),
        Err(ApiError::AuthenticationError { reason }) => {
            println!("   Authentication failed: {}", reason);
            println!("   → Redirecting to login page");
        }
        Err(ApiError::RateLimitError { retry_after }) => {
            println!("   Rate limited. Retry after {} seconds", retry_after);
            println!("   → Implementing exponential backoff");
        }
        Err(error) => {
            println!("   Other error: {}", error);
            println!("   → Using generic error handling");
        }
    }

    // Example 3: Using if-let for specific error handling
    println!("\n3. Using if-let for specific errors:");
    let result = simulate_api_request("data", false, "rate_limit").await;
    if let Err(ApiError::RateLimitError { retry_after }) = result {
        println!(
            "   Rate limited! Waiting {} seconds before retry",
            retry_after
        );
        // In real code, you might implement retry logic here
    }

    // Example 4: Converting errors with context
    println!("\n4. Adding context to errors:");
    let result: AnyhowResult<String> = simulate_api_request("analytics", false, "network")
        .await
        .context("Failed to fetch analytics data for dashboard");

    match result {
        Ok(data) => println!("   Analytics data: {}", data),
        Err(error) => {
            println!("   Error with context: {}", error);
            // Print the error chain
            let mut source = error.source();
            while let Some(err) = source {
                println!("   Caused by: {}", err);
                source = err.source();
            }
        }
    }
}

/// # Function: demonstrate_error_propagation
///
/// Demonstrates how to use the ? operator for error propagation in async functions.
/// Shows how errors bubble up through the call stack and how to handle them.
///
/// ## Key Learning Points:
/// - Using ? operator with async functions
/// - Error propagation through multiple async calls
/// - Converting between different error types
/// - Early return on errors
async fn demonstrate_error_propagation() {
    println!("\n=== Error Propagation with ? Operator ===");

    /// # Function: fetch_user_profile
    ///
    /// A complex operation that makes multiple async calls and uses
    /// the ? operator to propagate errors. This demonstrates how
    /// errors can bubble up through multiple layers of async calls.
    ///
    /// ## Error Propagation Flow:
    /// 1. Authenticate user (can fail with auth error)
    /// 2. Fetch user data (can fail with network error)
    /// 3. Fetch user preferences (can fail with database error)
    /// 4. Combine all data into profile
    ///
    /// ## Returns:
    /// - `AnyhowResult<String>`: Complete user profile or error
    async fn fetch_user_profile(user_id: u32) -> AnyhowResult<String> {
        println!("   Fetching complete profile for user {}", user_id);

        // Step 1: Authenticate (using ? to propagate errors)
        let _auth_token = simulate_api_request("auth", true, "")
            .await
            .context("Authentication step failed")?;

        // Step 2: Fetch user data (this will succeed)
        let user_data = simulate_api_request("user_data", true, "")
            .await
            .context("Failed to fetch user data")?;

        // Step 3: Fetch user preferences (this will fail)
        let _preferences = simulate_database_operation("SELECT preferences", false, "connection")
            .await
            .context("Failed to fetch user preferences")?;

        // If we get here, everything succeeded
        Ok(format!("Complete profile: {} with preferences", user_data))
    }

    /// # Function: fetch_user_profile_with_fallback
    ///
    /// Similar to fetch_user_profile but with fallback handling.
    /// Demonstrates how to recover from certain types of errors.
    async fn fetch_user_profile_with_fallback(user_id: u32) -> AnyhowResult<String> {
        println!("   Fetching profile with fallback for user {}", user_id);

        // Step 1: Authenticate
        let _auth_token = simulate_api_request("auth", true, "")
            .await
            .context("Authentication step failed")?;

        // Step 2: Fetch user data
        let user_data = simulate_api_request("user_data", true, "")
            .await
            .context("Failed to fetch user data")?;

        // Step 3: Try to fetch preferences, but provide fallback
        let preferences =
            match simulate_database_operation("SELECT preferences", false, "connection").await {
                Ok(prefs) => prefs,
                Err(DatabaseError::ConnectionFailed { .. }) => {
                    println!("   Database connection failed, using default preferences");
                    "Default preferences".to_string()
                }
                Err(e) => {
                    // For other database errors, we still want to fail
                    return Err(anyhow::Error::new(e).context("Critical database error"));
                }
            };

        Ok(format!("Profile: {} with {}", user_data, preferences))
    }

    // Example 1: Error propagation causing failure
    println!("1. Error propagation (will fail):");
    match fetch_user_profile(123).await {
        Ok(profile) => println!("   Profile: {}", profile),
        Err(error) => {
            println!("   Failed to fetch profile: {}", error);
            // Show the full error chain
            for (i, cause) in error.chain().enumerate() {
                if i == 0 {
                    println!("   Root cause: {}", cause);
                } else {
                    println!("   Caused by: {}", cause);
                }
            }
        }
    }

    // Example 2: Error recovery with fallback
    println!("\n2. Error recovery with fallback:");
    match fetch_user_profile_with_fallback(456).await {
        Ok(profile) => println!("   Profile with fallback: {}", profile),
        Err(error) => println!("   Even fallback failed: {}", error),
    }
}

/// # Function: demonstrate_timeout_handling
///
/// Demonstrates how to handle timeouts in async operations.
/// Shows different timeout strategies and error handling approaches.
///
/// ## Key Learning Points:
/// - Using tokio::time::timeout for operation timeouts
/// - Handling timeout errors specifically
/// - Implementing retry logic with timeouts
/// - Combining timeouts with other error types
async fn demonstrate_timeout_handling() {
    println!("\n=== Timeout Handling ===");

    // Example 1: Basic timeout handling
    println!("1. Basic timeout (will timeout):");
    let start = Instant::now();

    let result = timeout(
        Duration::from_millis(50),                       // Short timeout
        simulate_api_request("slow_endpoint", true, ""), // Takes 100ms
    )
    .await;

    let elapsed = start.elapsed();
    match result {
        Ok(Ok(data)) => println!("   Success: {}", data),
        Ok(Err(api_error)) => println!("   API error: {}", api_error),
        Err(_timeout_error) => {
            println!("   Operation timed out after {:?}", elapsed);
            println!("   → Could implement retry logic here");
        }
    }

    // Example 2: Timeout with successful completion
    println!("\n2. Timeout with successful completion:");
    let start = Instant::now();

    let result = timeout(
        Duration::from_millis(200),                      // Longer timeout
        simulate_api_request("fast_endpoint", true, ""), // Takes 100ms
    )
    .await;

    let elapsed = start.elapsed();
    match result {
        Ok(Ok(data)) => {
            println!("   Success within timeout: {}", data);
            println!("   Completed in {:?}", elapsed);
        }
        Ok(Err(api_error)) => println!("   API error: {}", api_error),
        Err(_timeout_error) => println!("   Unexpected timeout"),
    }

    // Example 3: Implementing retry with timeout
    println!("\n3. Retry logic with timeout:");

    /// # Function: retry_with_timeout
    ///
    /// Implements a retry mechanism with timeout for each attempt.
    /// This is a common pattern for resilient async operations.
    ///
    /// ## Arguments:
    /// - `max_retries`: Maximum number of retry attempts
    /// - `timeout_duration`: Timeout for each individual attempt
    /// - `retry_delay`: Delay between retry attempts
    ///
    /// ## Returns:
    /// - `AnyhowResult<String>`: Success or final error after all retries
    async fn retry_with_timeout(
        max_retries: usize,
        timeout_duration: Duration,
        retry_delay: Duration,
    ) -> AnyhowResult<String> {
        for attempt in 1..=max_retries {
            println!("     Attempt {} of {}", attempt, max_retries);

            let result = timeout(
                timeout_duration,
                simulate_api_request("unreliable_service", attempt == max_retries, "network"),
            )
            .await;

            match result {
                Ok(Ok(data)) => {
                    println!("     Success on attempt {}", attempt);
                    return Ok(data);
                }
                Ok(Err(api_error)) => {
                    println!("     API error on attempt {}: {}", attempt, api_error);
                    if attempt == max_retries {
                        return Err(anyhow::Error::new(api_error).context("All retries failed"));
                    }
                }
                Err(_timeout_error) => {
                    println!("     Timeout on attempt {}", attempt);
                    if attempt == max_retries {
                        return Err(anyhow::anyhow!("All retries timed out"));
                    }
                }
            }

            if attempt < max_retries {
                println!("     Waiting {:?} before retry", retry_delay);
                sleep(retry_delay).await;
            }
        }

        unreachable!()
    }

    let start = Instant::now();
    match retry_with_timeout(3, Duration::from_millis(150), Duration::from_millis(100)).await {
        Ok(data) => println!("   Retry succeeded: {}", data),
        Err(error) => println!("   All retries failed: {}", error),
    }
    let elapsed = start.elapsed();
    println!("   Total retry time: {:?}", elapsed);
}

/// # Function: demonstrate_concurrent_error_handling
///
/// Demonstrates how to handle errors when running multiple async operations
/// concurrently. Shows different strategies for dealing with partial failures.
///
/// ## Key Learning Points:
/// - Using try_join! for fail-fast error handling
/// - Collecting results from multiple operations
/// - Handling partial successes and failures
/// - Error aggregation strategies
async fn demonstrate_concurrent_error_handling() {
    println!("\n=== Concurrent Error Handling ===");

    // Example 1: try_join! with fail-fast behavior
    println!("1. try_join! with fail-fast (will fail):");
    let start = Instant::now();

    let result = tokio::try_join!(
        simulate_api_request("service1", true, ""),
        simulate_api_request("service2", false, "network"), // This will fail
        simulate_api_request("service3", true, ""),         // Won't complete due to fail-fast
    );

    let elapsed = start.elapsed();
    match result {
        Ok((s1, s2, s3)) => {
            println!("   All succeeded: {}, {}, {}", s1, s2, s3);
        }
        Err(error) => {
            println!("   Failed fast due to: {}", error);
            println!("   Time: {:?} (stopped early)", elapsed);
        }
    }

    // Example 2: Collecting all results (successes and failures)
    println!("\n2. Collecting all results (partial success):");
    let start = Instant::now();

    let (result1, result2, result3) = tokio::join!(
        simulate_api_request("service1", true, ""),
        simulate_api_request("service2", false, "auth"),
        simulate_api_request("service3", true, ""),
    );

    let elapsed = start.elapsed();

    let mut successes = Vec::new();
    let mut failures = Vec::new();

    match result1 {
        Ok(data) => successes.push(("service1", data)),
        Err(error) => failures.push(("service1", error)),
    }

    match result2 {
        Ok(data) => successes.push(("service2", data)),
        Err(error) => failures.push(("service2", error)),
    }

    match result3 {
        Ok(data) => successes.push(("service3", data)),
        Err(error) => failures.push(("service3", error)),
    }

    println!("   Completed in {:?}", elapsed);
    println!("   Successes: {}", successes.len());
    for (service, data) in successes {
        println!("     {}: {}", service, data);
    }

    println!("   Failures: {}", failures.len());
    for (service, error) in failures {
        println!("     {}: {}", service, error);
    }

    // Example 3: Using FuturesUnordered for streaming results
    println!("\n3. Streaming results with error handling:");
    use futures::stream::{FuturesUnordered, StreamExt};

    let mut futures: FuturesUnordered<
        std::pin::Pin<
            Box<dyn std::future::Future<Output = (&str, Result<String, ApiError>)> + Send>,
        >,
    > = FuturesUnordered::new();
    futures.push(Box::pin(async {
        ("api1", simulate_api_request("api1", true, "").await)
    }));
    futures.push(Box::pin(async {
        (
            "api2",
            simulate_api_request("api2", false, "rate_limit").await,
        )
    }));
    futures.push(Box::pin(async {
        ("api3", simulate_api_request("api3", true, "").await)
    }));
    futures.push(Box::pin(async {
        (
            "api4",
            simulate_api_request("api4", false, "validation").await,
        )
    }));

    let mut completed = 0;
    let mut success_count = 0;
    let mut error_count = 0;

    while let Some((service, result)) = futures.next().await {
        completed += 1;
        match result {
            Ok(data) => {
                success_count += 1;
                println!("   ✅ {}: {}", service, data);
            }
            Err(error) => {
                error_count += 1;
                println!("   ❌ {}: {}", service, error);
            }
        }
    }

    println!(
        "   Summary: {} completed, {} succeeded, {} failed",
        completed, success_count, error_count
    );
}

/// # Function: demonstrate_error_recovery_strategies
///
/// Demonstrates various strategies for recovering from errors in async code.
/// Shows how to implement fallbacks, circuit breakers, and graceful degradation.
///
/// ## Key Learning Points:
/// - Implementing fallback mechanisms
/// - Circuit breaker pattern for failing services
/// - Graceful degradation strategies
/// - Error recovery with alternative data sources
async fn demonstrate_error_recovery_strategies() {
    println!("\n=== Error Recovery Strategies ===");

    // Example 1: Simple fallback chain
    println!("1. Fallback chain:");

    /// # Function: fetch_data_with_fallback
    ///
    /// Attempts to fetch data from multiple sources, falling back
    /// to alternatives if the primary source fails.
    async fn fetch_data_with_fallback() -> AnyhowResult<String> {
        // Try primary source
        match simulate_api_request("primary_api", false, "network").await {
            Ok(data) => {
                println!("   Primary source succeeded");
                return Ok(format!("Primary: {}", data));
            }
            Err(error) => {
                println!("   Primary source failed: {}", error);
            }
        }

        // Try secondary source
        match simulate_api_request("secondary_api", false, "auth").await {
            Ok(data) => {
                println!("   Secondary source succeeded");
                return Ok(format!("Secondary: {}", data));
            }
            Err(error) => {
                println!("   Secondary source failed: {}", error);
            }
        }

        // Try cache as last resort
        match simulate_database_operation("SELECT FROM cache", true, "").await {
            Ok(data) => {
                println!("   Cache fallback succeeded");
                Ok(format!("Cached: {}", data))
            }
            Err(error) => Err(anyhow::Error::new(error).context("All data sources failed")),
        }
    }

    match fetch_data_with_fallback().await {
        Ok(data) => println!("   Final result: {}", data),
        Err(error) => println!("   Complete failure: {}", error),
    }

    // Example 2: Partial success with degraded functionality
    println!("\n2. Graceful degradation:");

    /// # Function: build_dashboard_data
    ///
    /// Builds dashboard data from multiple sources, providing
    /// graceful degradation when some sources fail.
    async fn build_dashboard_data() -> String {
        let mut dashboard = String::from("Dashboard Data:\n");

        // Critical data (must succeed)
        match simulate_api_request("user_info", true, "").await {
            Ok(data) => dashboard.push_str(&format!("  User Info: {}\n", data)),
            Err(error) => {
                dashboard.push_str(&format!("  User Info: ERROR - {}\n", error));
                return dashboard; // Critical failure, return early
            }
        }

        // Optional data (can fail gracefully)
        match simulate_api_request("notifications", false, "timeout").await {
            Ok(data) => dashboard.push_str(&format!("  Notifications: {}\n", data)),
            Err(_) => dashboard.push_str("  Notifications: Unavailable (using cached count)\n"),
        }

        match simulate_api_request("analytics", false, "rate_limit").await {
            Ok(data) => dashboard.push_str(&format!("  Analytics: {}\n", data)),
            Err(_) => dashboard.push_str("  Analytics: Temporarily unavailable\n"),
        }

        // Always available fallback data
        dashboard.push_str("  System Status: Operational\n");

        dashboard
    }

    let dashboard = build_dashboard_data().await;
    println!("{}", dashboard);

    // Example 3: Circuit breaker pattern (simplified)
    println!("3. Circuit breaker pattern:");

    /// # Struct: SimpleCircuitBreaker
    ///
    /// A simplified circuit breaker implementation for demonstration.
    /// In production, you'd use a more sophisticated implementation.
    struct SimpleCircuitBreaker {
        failure_count: std::sync::Arc<std::sync::Mutex<u32>>,
        failure_threshold: u32,
        reset_timeout: Duration,
        last_failure: std::sync::Arc<std::sync::Mutex<Option<Instant>>>,
    }

    impl SimpleCircuitBreaker {
        fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
            Self {
                failure_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
                failure_threshold,
                reset_timeout,
                last_failure: std::sync::Arc::new(std::sync::Mutex::new(None)),
            }
        }

        async fn call<F, T, E>(&self, operation: F) -> Result<T, E>
        where
            F: std::future::Future<Output = Result<T, E>>,
            E: std::fmt::Display + Clone,
            T: std::fmt::Debug,
        {
            // Check if circuit is open
            {
                let failure_count = *self.failure_count.lock().unwrap();
                let last_failure = *self.last_failure.lock().unwrap();

                if failure_count >= self.failure_threshold {
                    if let Some(last_fail_time) = last_failure {
                        if last_fail_time.elapsed() < self.reset_timeout {
                            println!("   Circuit breaker OPEN - rejecting call");
                            // Create a dummy operation to get the error type
                            let dummy_result = operation.await;
                            if let Err(e) = dummy_result {
                                return Err(e);
                            } else {
                                // This shouldn't happen in our demo, but handle it gracefully
                                println!("   Unexpected success during circuit open state");
                                return dummy_result;
                            }
                        } else {
                            println!("   Circuit breaker HALF-OPEN - trying call");
                        }
                    }
                }
            }

            // Execute operation
            match operation.await {
                Ok(result) => {
                    // Reset on success
                    *self.failure_count.lock().unwrap() = 0;
                    *self.last_failure.lock().unwrap() = None;
                    println!("   Circuit breaker: Call succeeded");
                    Ok(result)
                }
                Err(error) => {
                    // Increment failure count
                    let mut failure_count = self.failure_count.lock().unwrap();
                    *failure_count += 1;
                    *self.last_failure.lock().unwrap() = Some(Instant::now());
                    println!(
                        "   Circuit breaker: Call failed (count: {})",
                        *failure_count
                    );
                    Err(error)
                }
            }
        }
    }

    let circuit_breaker = SimpleCircuitBreaker::new(2, Duration::from_millis(1000));

    // Make several calls to trigger circuit breaker
    for i in 1..=5 {
        println!("   Call {}: ", i);
        let _result = circuit_breaker
            .call(simulate_api_request("flaky_service", false, "network"))
            .await;

        if i == 3 {
            // Wait a bit to show the reset timeout
            sleep(Duration::from_millis(200)).await;
        }
    }
}

/// # Function: main
///
/// The main function orchestrates all error handling demonstrations,
/// showing progressively more advanced error handling patterns.
///
/// ## Learning Progression:
/// 1. Basic error handling with match statements
/// 2. Error propagation with ? operator
/// 3. Timeout handling and retry logic
/// 4. Concurrent error handling strategies
/// 5. Error recovery and resilience patterns
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚨 Error Handling in Async Rust Tutorial");
    println!("========================================");
    println!("This example demonstrates comprehensive error handling patterns in async Rust.");

    // Basic error handling patterns
    demonstrate_basic_error_handling().await;

    // Error propagation with ? operator
    demonstrate_error_propagation().await;

    // Timeout handling
    demonstrate_timeout_handling().await;

    // Concurrent error handling
    demonstrate_concurrent_error_handling().await;

    // Error recovery strategies
    demonstrate_error_recovery_strategies().await;

    println!("\n✅ Error Handling Tutorial completed!");
    println!("Key takeaways:");
    println!("  - Use custom error types with thiserror for better error handling");
    println!("  - The ? operator makes error propagation clean and readable");
    println!("  - Always handle timeouts in async operations");
    println!("  - Consider fail-fast vs. collect-all strategies for concurrent operations");
    println!("  - Implement fallbacks and graceful degradation for resilience");
    println!("  - Use circuit breakers for failing external services");
    println!("  - Add context to errors to make debugging easier");

    println!("\nNext: Try 'cargo run --bin real_world' to see real-world async patterns");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    /// Test basic error handling
    #[tokio::test]
    async fn test_api_error_handling() {
        // Test successful case
        let result = simulate_api_request("test", true, "").await;
        assert!(result.is_ok());

        // Test error cases
        let result = simulate_api_request("test", false, "network").await;
        assert!(matches!(result, Err(ApiError::NetworkError { .. })));

        let result = simulate_api_request("test", false, "auth").await;
        assert!(matches!(result, Err(ApiError::AuthenticationError { .. })));
    }

    /// Test timeout handling
    #[tokio::test]
    async fn test_timeout_handling() {
        // Test timeout
        let result = timeout(
            Duration::from_millis(50),
            simulate_api_request("slow", true, ""),
        )
        .await;

        assert!(result.is_err()); // Should timeout

        // Test successful completion within timeout
        let result = timeout(
            Duration::from_millis(200),
            simulate_api_request("fast", true, ""),
        )
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    /// Test error propagation
    #[tokio::test]
    async fn test_error_propagation() {
        async fn operation_that_fails() -> AnyhowResult<String> {
            simulate_api_request("test", false, "network")
                .await
                .context("Test operation failed")?;
            Ok("success".to_string())
        }

        let result = operation_that_fails().await;
        assert!(result.is_err());

        let error_message = format!("{}", result.unwrap_err());
        assert!(error_message.contains("Test operation failed"));
    }

    /// Test concurrent error handling
    #[tokio::test]
    async fn test_concurrent_errors() {
        // Test try_join! fail-fast behavior
        let result = tokio::try_join!(
            simulate_api_request("good", true, ""),
            simulate_api_request("bad", false, "network"),
            simulate_api_request("slow", true, "")
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NetworkError { .. }));
    }
}
