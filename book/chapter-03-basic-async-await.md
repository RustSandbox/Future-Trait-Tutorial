# Chapter 3: Basic async/await

## Introduction

The `async` and `await` keywords are syntactic sugar that make working with futures much more ergonomic. Instead of manually implementing the `Future` trait, you can write code that looks almost synchronous while being fully asynchronous under the hood.

## The async Keyword

The `async` keyword transforms functions and blocks into futures. When you call an async function, it doesn't execute immediately - instead, it returns a future that can be awaited.

### Async Functions

```rust
// Regular synchronous function
fn sync_function() -> String {
    "Hello, World!".to_string()  // Executes immediately
}

// Async function
async fn async_function() -> String {
    "Hello, Async World!".to_string()  // Returns a Future<Output = String>
}
```

### What Happens Under the Hood

When you write an async function, the compiler transforms it:

```rust
// What you write:
async fn example() -> i32 {
    42
}

// What the compiler generates (simplified):
fn example() -> impl Future<Output = i32> {
    async move { 42 }
}
```

### Async Blocks

You can also create futures using async blocks:

```rust
let future = async {
    println!("This is an async block");
    42
};

// future is now a Future<Output = i32>
let result = future.await;  // result is 42
```

## The await Keyword

The `await` keyword is used to wait for a future to complete. It can only be used inside async functions or blocks.

### Basic Usage

```rust
async fn fetch_data() -> String {
    // Simulate an async operation
    tokio::time::sleep(Duration::from_millis(100)).await;
    "Data fetched!".to_string()
}

async fn main_example() {
    let data = fetch_data().await;  // Wait for the future to complete
    println!("{}", data);
}
```

### What await Does

When you use `.await`:

1. **Polls the future**: Calls the future's `poll` method
2. **If Ready**: Returns the value immediately
3. **If Pending**: Yields control back to the executor
4. **Resumes**: When woken, continues from the await point

```rust
async fn demonstrate_await() {
    println!("Before await");
    
    // This might yield control to other tasks
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("After await");  // Resumes here when timer completes
}
```

## Practical Examples

Let's explore practical examples using the code from our tutorial project.

### Example 1: Sequential vs Concurrent Execution

**File reference**: `src/examples/basic_future.rs`

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

async fn simulate_work(name: &str, duration: Duration) -> String {
    println!("Starting {}", name);
    sleep(duration).await;
    let result = format!("{} completed", name);
    println!("{}", result);
    result
}

// Sequential execution - one after another
async fn sequential_example() {
    let start = Instant::now();
    
    let result1 = simulate_work("Task 1", Duration::from_millis(100)).await;
    let result2 = simulate_work("Task 2", Duration::from_millis(150)).await;
    let result3 = simulate_work("Task 3", Duration::from_millis(200)).await;
    
    println!("Sequential total time: {:?}", start.elapsed());
    // Output: ~450ms (sum of all tasks)
}

// Concurrent execution - all at once
async fn concurrent_example() {
    let start = Instant::now();
    
    let (result1, result2, result3) = tokio::join!(
        simulate_work("Task 1", Duration::from_millis(100)),
        simulate_work("Task 2", Duration::from_millis(150)),
        simulate_work("Task 3", Duration::from_millis(200))
    );
    
    println!("Concurrent total time: {:?}", start.elapsed());
    // Output: ~200ms (time of longest task)
}
```

### Example 2: Error Handling with async/await

```rust
use anyhow::Result;

async fn might_fail(should_fail: bool) -> Result<String> {
    sleep(Duration::from_millis(50)).await;
    
    if should_fail {
        Err(anyhow::anyhow!("Something went wrong!"))
    } else {
        Ok("Success!".to_string())
    }
}

async fn error_handling_example() -> Result<()> {
    // Using ? operator with async functions
    let result1 = might_fail(false).await?;  // This succeeds
    println!("Result 1: {}", result1);
    
    let result2 = might_fail(true).await?;   // This fails and returns early
    println!("Result 2: {}", result2);       // This line won't execute
    
    Ok(())
}
```

### Example 3: Async Closures

```rust
async fn closure_example() {
    let async_closure = |name: &str| async move {
        sleep(Duration::from_millis(100)).await;
        format!("Hello, {}!", name)
    };
    
    let result = async_closure("Rust").await;
    println!("{}", result);
}
```

## Future Laziness in Practice

One of the most important concepts to understand is that futures are lazy:

```rust
async fn demonstrate_laziness() {
    println!("=== Demonstrating Future Laziness ===");
    
    // Creating futures does NOT start any work
    println!("Creating futures...");
    let future1 = simulate_work("Lazy 1", Duration::from_millis(100));
    let future2 = simulate_work("Lazy 2", Duration::from_millis(100));
    
    println!("Futures created, but no work started yet!");
    
    // Work only starts when we await
    println!("Now awaiting futures...");
    let result1 = future1.await;
    let result2 = future2.await;
    
    println!("Results: {}, {}", result1, result2);
}
```

## Compiler Transformations

Understanding how the compiler transforms async/await helps debug and optimize code.

### Simple Async Function

```rust
// What you write:
async fn simple() -> i32 {
    let x = 42;
    some_async_operation().await;
    x + 1
}

// What the compiler generates (conceptually):
fn simple() -> impl Future<Output = i32> {
    enum SimpleStateMachine {
        Start,
        WaitingForOperation { x: i32 },
        Done,
    }
    
    impl Future for SimpleStateMachine {
        type Output = i32;
        
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
            loop {
                match *self {
                    SimpleStateMachine::Start => {
                        let x = 42;
                        *self = SimpleStateMachine::WaitingForOperation { x };
                        // Start the async operation
                    }
                    SimpleStateMachine::WaitingForOperation { x } => {
                        // Poll the async operation
                        match some_async_operation_poll(cx) {
                            Poll::Ready(()) => {
                                *self = SimpleStateMachine::Done;
                                return Poll::Ready(x + 1);
                            }
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    SimpleStateMachine::Done => panic!("polled after completion"),
                }
            }
        }
    }
    
    SimpleStateMachine::Start
}
```

## Common Patterns

### Pattern 1: Conditional Awaiting

```rust
async fn conditional_await(should_wait: bool) -> String {
    if should_wait {
        sleep(Duration::from_millis(100)).await;
        "Waited".to_string()
    } else {
        "No wait".to_string()
    }
}
```

### Pattern 2: Loops with Await

```rust
async fn loop_with_await() {
    for i in 0..3 {
        println!("Iteration {}", i);
        sleep(Duration::from_millis(100)).await;
    }
}
```

### Pattern 3: Early Returns

```rust
async fn early_return(condition: bool) -> Result<String> {
    if condition {
        return Ok("Early return".to_string());
    }
    
    let result = some_async_operation().await?;
    Ok(format!("Normal return: {}", result))
}
```

## Working with Multiple Futures

### tokio::join! - Wait for All

```rust
async fn join_example() {
    let (a, b, c) = tokio::join!(
        async { 1 },
        async { 2 },
        async { 3 }
    );
    println!("Results: {}, {}, {}", a, b, c);
}
```

### tokio::select! - First to Complete

```rust
async fn select_example() {
    tokio::select! {
        result = async { "First" } => {
            println!("First completed: {}", result);
        }
        result = async { 
            sleep(Duration::from_millis(100)).await;
            "Second" 
        } => {
            println!("Second completed: {}", result);
        }
    }
}
```

### try_join! - All Must Succeed

```rust
async fn try_join_example() -> Result<()> {
    let (a, b, c) = tokio::try_join!(
        might_fail(false),
        might_fail(false),
        might_fail(false)
    )?;
    
    println!("All succeeded: {}, {}, {}", a, b, c);
    Ok(())
}
```

## Async Main Functions

To run async code, you need an async runtime. The most common approach is using `#[tokio::main]`:

```rust
// Option 1: tokio::main attribute
#[tokio::main]
async fn main() -> Result<()> {
    let result = some_async_function().await?;
    println!("Result: {}", result);
    Ok(())
}

// Option 2: Manual runtime creation
fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let result = some_async_function().await?;
        println!("Result: {}", result);
        Ok(())
    })
}
```

## Common Mistakes and How to Avoid Them

### Mistake 1: Forgetting to await

```rust
// ❌ Wrong - future is created but never executed
async fn wrong_example() {
    some_async_function();  // This does nothing!
}

// ✅ Correct - future is awaited
async fn correct_example() {
    some_async_function().await;  // This executes the future
}
```

### Mistake 2: Blocking in async context

```rust
// ❌ Wrong - blocks the async runtime
async fn wrong_blocking() {
    std::thread::sleep(Duration::from_secs(1));  // Blocks the executor!
}

// ✅ Correct - uses async sleep
async fn correct_async() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

### Mistake 3: Unnecessary sequential execution

```rust
// ❌ Inefficient - sequential when could be concurrent
async fn inefficient() {
    let a = fetch_data_a().await;
    let b = fetch_data_b().await;  // Waits for a to complete first
    let c = fetch_data_c().await;  // Waits for b to complete first
}

// ✅ Efficient - concurrent execution
async fn efficient() {
    let (a, b, c) = tokio::join!(
        fetch_data_a(),
        fetch_data_b(),
        fetch_data_c()
    );
}
```

## Testing Async Code

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert_eq!(result, "Hello, Async World!");
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let result = might_fail(true).await;
        assert!(result.is_err());
    }
}
```

## Performance Considerations

### When to Use Sequential vs Concurrent

```rust
// Use sequential when operations depend on each other
async fn sequential_dependency() {
    let user_id = authenticate_user().await?;
    let user_data = fetch_user_data(user_id).await?;
    let preferences = fetch_user_preferences(user_data.id).await?;
}

// Use concurrent when operations are independent
async fn concurrent_independent() {
    let (weather, news, stocks) = tokio::join!(
        fetch_weather(),
        fetch_news(),
        fetch_stock_prices()
    );
}
```

### Spawning Tasks

```rust
// For CPU-intensive work, spawn on a blocking thread pool
async fn cpu_intensive_work() {
    let result = tokio::task::spawn_blocking(|| {
        // Heavy computation here
        expensive_calculation()
    }).await?;
}

// For independent async work, spawn as a task
async fn spawn_independent_work() {
    let handle = tokio::spawn(async {
        background_processing().await
    });
    
    // Do other work...
    
    let result = handle.await?;
}
```

## What's Next?

Now that you understand async/await syntax and basic usage patterns, you're ready to dive deeper into implementing custom futures from scratch.

In [Chapter 4: Custom Future Implementation](./chapter-04-custom-futures.md), we'll explore:
- Implementing the Future trait manually
- Understanding state machines in detail
- Building reusable async components
- Advanced polling patterns

---

## Exercises

1. **Convert sync to async**: Take a synchronous function and convert it to async
2. **Experiment with timing**: Compare sequential vs concurrent execution times
3. **Error propagation**: Practice using `?` with async functions
4. **Build a simple async application**: Create a program that fetches data from multiple sources concurrently

## Key Takeaways

- `async` functions return futures, not values
- `await` is used to wait for futures to complete
- Futures are lazy - no work happens until awaited
- Use `tokio::join!` for concurrent execution
- Use `tokio::select!` for racing futures
- Always use async sleep in async contexts
- Test async code with `#[tokio::test]`
</rewritten_file> 