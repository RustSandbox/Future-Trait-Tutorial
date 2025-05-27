# Chapter 8: Error Handling

## Introduction

Error handling in async Rust requires special consideration due to the nature of futures and concurrent execution. This chapter explores comprehensive error handling patterns using our `error_handling.rs` example.

## Error Handling Fundamentals

**File reference**: `src/examples/error_handling.rs`

```bash
# Run the error handling example
cargo run --bin error_handling

# Test error handling patterns
cargo test --bin error_handling
```

### Basic Async Error Handling

```rust
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AsyncError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Timeout after {duration:?}")]
    Timeout { duration: Duration },
    #[error("Validation failed: {reason}")]
    Validation { reason: String },
}

async fn might_fail() -> Result<String, AsyncError> {
    // Simulate potential failure
    if random_condition() {
        Err(AsyncError::Network("Connection refused".to_string()))
    } else {
        Ok("Success!".to_string())
    }
}
```

## Error Propagation with ?

The `?` operator works seamlessly with async functions:

```rust
async fn chain_operations() -> Result<String> {
    let data = fetch_data().await?;
    let processed = process_data(data).await?;
    let result = save_result(processed).await?;
    Ok(result)
}
```

## Timeout Patterns

### Basic Timeout

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout() -> Result<String> {
    let result = timeout(
        Duration::from_secs(5),
        slow_operation()
    ).await??; // Note the double ?
    
    Ok(result)
}
```

### Custom Timeout Implementation

```rust
async fn custom_timeout<F, T>(
    future: F,
    duration: Duration,
) -> Result<T, AsyncError>
where
    F: Future<Output = Result<T, AsyncError>>,
{
    tokio::select! {
        result = future => result,
        _ = tokio::time::sleep(duration) => {
            Err(AsyncError::Timeout { duration })
        }
    }
}
```

## Retry Patterns

### Simple Retry

```rust
async fn retry_operation<F, T, E>(
    mut operation: F,
    max_attempts: usize,
) -> Result<T, E>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
{
    let mut attempts = 0;
    
    loop {
        attempts += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts >= max_attempts => return Err(e),
            Err(_) => {
                // Wait before retry
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
```

### Exponential Backoff

```rust
async fn retry_with_backoff<F, T, E>(
    operation: F,
    max_attempts: usize,
    base_delay: Duration,
) -> Result<T, E>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
{
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

## Circuit Breaker Pattern

```rust
use std::sync::atomic::{AtomicU32, Ordering};

pub struct CircuitBreaker {
    failure_count: AtomicU32,
    failure_threshold: u32,
    reset_timeout: Duration,
    last_failure: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            failure_threshold,
            reset_timeout,
            last_failure: Mutex::new(None),
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, AsyncError>
    where
        F: Future<Output = Result<T, E>>,
        E: Into<AsyncError>,
    {
        // Check if circuit is open
        if self.is_open().await {
            return Err(AsyncError::CircuitOpen);
        }

        match operation.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e.into())
            }
        }
    }

    async fn is_open(&self) -> bool {
        let failure_count = self.failure_count.load(Ordering::Relaxed);
        
        if failure_count >= self.failure_threshold {
            let last_failure = self.last_failure.lock().unwrap();
            if let Some(last_failure_time) = *last_failure {
                last_failure_time.elapsed() < self.reset_timeout
            } else {
                true
            }
        } else {
            false
        }
    }

    fn on_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
    }

    async fn on_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        *self.last_failure.lock().unwrap() = Some(Instant::now());
    }
}
```

## Concurrent Error Handling

### Fail Fast with try_join!

```rust
async fn fail_fast_example() -> Result<(String, String, String)> {
    let (r1, r2, r3) = tokio::try_join!(
        operation_1(),
        operation_2(),
        operation_3()
    )?;
    
    Ok((r1, r2, r3))
}
```

### Collect All Results

```rust
async fn collect_all_results() -> Vec<Result<String, AsyncError>> {
    let futures = vec![
        operation_1(),
        operation_2(),
        operation_3(),
    ];
    
    futures::future::join_all(futures).await
}
```

### Partial Success Handling

```rust
async fn partial_success_example() -> Result<Vec<String>> {
    let results = collect_all_results().await;
    
    let mut successes = Vec::new();
    let mut errors = Vec::new();
    
    for result in results {
        match result {
            Ok(value) => successes.push(value),
            Err(e) => errors.push(e),
        }
    }
    
    if successes.is_empty() {
        Err(anyhow::anyhow!("All operations failed: {:?}", errors))
    } else {
        println!("Partial success: {} succeeded, {} failed", 
                successes.len(), errors.len());
        Ok(successes)
    }
}
```

## Error Context and Debugging

### Adding Context

```rust
use anyhow::Context;

async fn operation_with_context() -> Result<String> {
    let data = fetch_data()
        .await
        .context("Failed to fetch initial data")?;
    
    let processed = process_data(data)
        .await
        .context("Failed to process data")?;
    
    Ok(processed)
}
```

### Custom Error Context

```rust
async fn detailed_error_context() -> Result<String> {
    let user_id = get_user_id().await?;
    
    let user_data = fetch_user_data(user_id)
        .await
        .with_context(|| format!("Failed to fetch data for user {}", user_id))?;
    
    Ok(user_data)
}
```

## Testing Error Scenarios

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_error() {
        let result = timeout(
            Duration::from_millis(10),
            tokio::time::sleep(Duration::from_millis(100))
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_success() {
        let mut attempts = 0;
        
        let operation = || {
            attempts += 1;
            async move {
                if attempts < 3 {
                    Err(AsyncError::Network("Temporary failure".to_string()))
                } else {
                    Ok("Success!".to_string())
                }
            }
        };
        
        let result = retry_operation(operation, 5).await;
        assert!(result.is_ok());
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let circuit_breaker = CircuitBreaker::new(2, Duration::from_secs(1));
        
        // Trigger failures to open circuit
        for _ in 0..3 {
            let _ = circuit_breaker.call(async { 
                Err::<(), _>(AsyncError::Network("Failure".to_string()))
            }).await;
        }
        
        // Circuit should be open now
        let result = circuit_breaker.call(async { Ok("Success") }).await;
        assert!(matches!(result, Err(AsyncError::CircuitOpen)));
    }
}
```

## Best Practices

### 1. Use Appropriate Error Types

```rust
// ✅ Good - specific error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Query failed: {query}")]
    Query { query: String },
    #[error("Transaction rolled back")]
    Transaction,
}

// ❌ Avoid - generic string errors
async fn bad_example() -> Result<String, String> {
    // Hard to handle programmatically
    Err("Something went wrong".to_string())
}
```

### 2. Provide Context

```rust
// ✅ Good - rich context
async fn with_good_context() -> Result<User> {
    let user_id = extract_user_id()
        .context("Failed to extract user ID from request")?;
    
    let user = database.fetch_user(user_id)
        .await
        .with_context(|| format!("Failed to fetch user {}", user_id))?;
    
    Ok(user)
}
```

### 3. Handle Partial Failures

```rust
async fn robust_batch_operation(items: Vec<Item>) -> BatchResult {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    
    for item in items {
        match process_item(item).await {
            Ok(result) => results.push(result),
            Err(e) => {
                errors.push(e);
                // Continue processing other items
            }
        }
    }
    
    BatchResult { results, errors }
}
```

### 4. Implement Graceful Degradation

```rust
async fn with_fallback() -> Result<String> {
    match primary_service().await {
        Ok(result) => Ok(result),
        Err(_) => {
            println!("Primary service failed, trying fallback");
            fallback_service().await
        }
    }
}
```

## What's Next?

Now that you understand error handling patterns, let's explore how to build complex concurrent systems that handle multiple async operations gracefully.

In [Chapter 10: Autonomous Agent Example](./chapter-10-autonomous-agent.md), we'll see how all these concepts come together in a real-world example.

---

## Key Takeaways

- Use `?` operator for clean error propagation
- Implement timeouts for all external operations
- Use retry patterns with exponential backoff
- Implement circuit breakers for resilience
- Handle partial failures gracefully
- Provide rich error context for debugging
- Test error scenarios thoroughly 