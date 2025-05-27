# The Future Trait

The `Future` trait is the core abstraction for asynchronous programming in Rust. It represents a computation that may not have completed yet.

## Basic Structure

A `Future` in Rust has this basic structure:

```rust
pub trait Future {
    type Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

<InteractiveExample title="Basic Future Implementation" exampleId="basic-future">
  <template #code>
```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Delay {
    duration: std::time::Duration,
}

impl Future for Delay {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // In a real implementation, this would check if the delay has elapsed
        Poll::Ready(())
    }
}

async fn example() {
    let delay = Delay {
        duration: std::time::Duration::from_secs(1),
    };
    delay.await;
    println!("Delay completed!");
}
```
  </template>
</InteractiveExample>

## Key Concepts

### Poll States

A `Future` can be in one of two states:

1. **Pending**: The computation is not yet complete
2. **Ready**: The computation has completed with a value

```rust
pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

### Waker and Context

The `Context` provides a `Waker` that allows the future to notify the executor when it's ready to make progress.

<InteractiveExample title="Waker Example" exampleId="waker-example">
  <template #code>
```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::sync::{Arc, Mutex};

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

struct MyFuture {
    state: Arc<Mutex<SharedState>>,
}

impl Future for MyFuture {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.completed {
            Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
```
  </template>
</InteractiveExample>

## Common Patterns

### Chaining Futures

Futures can be chained using combinators like `and_then` and `map`:

<InteractiveExample title="Future Chaining" exampleId="future-chaining">
  <template #code>
```rust
use futures::future::{self, FutureExt};

async fn example() {
    let future = future::ready(1)
        .map(|x| x + 1)
        .and_then(|x| future::ready(x * 2));
    
    let result = future.await;
    println!("Result: {}", result); // Output: 4
}
```
  </template>
</InteractiveExample>

### Error Handling

Futures can handle errors using the `Result` type:

<InteractiveExample title="Error Handling" exampleId="error-handling">
  <template #code>
```rust
use std::io;

async fn read_file() -> Result<String, io::Error> {
    // Simulate file reading
    Ok("File contents".to_string())
}

async fn example() -> Result<(), io::Error> {
    let contents = read_file().await?;
    println!("{}", contents);
    Ok(())
}
```
  </template>
</InteractiveExample>

## Best Practices

1. **Keep futures small and focused**
2. **Use appropriate error types**
3. **Consider cancellation**
4. **Handle backpressure**
5. **Use appropriate combinators**

## Next Steps

In the next chapter, we'll explore how to use the `async/await` syntax to make working with futures more ergonomic.

---

## Exercises

1. Implement a `Future` that completes after a random delay
2. Create a future that combines multiple futures using `join!`
3. Implement error handling for a future that might fail

## Further Reading

- [The Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/docs/overview/)
- [Futures Documentation](https://docs.rs/futures) 