# Chapter 2: The Future Trait

## Core Components

The `Future` trait is the foundation of asynchronous programming in Rust. Let's break down its core components:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

### Key Concepts

1. **Lazy Evaluation**: Unlike futures in some other languages, Rust futures are lazy - they don't do anything until they're polled.
2. **Pin**: The `Pin` type ensures that futures can't be moved while they're being polled, which is crucial for self-referential futures.
3. **Context**: Contains the `Waker` that allows the future to notify the executor when it's ready to make progress.

## Poll States

A future can be in one of two states:

1. **Ready**: The future has completed and produced a value
2. **Pending**: The future is not yet complete and needs to be polled again later

```rust
pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

### Example: Simple Future

Here's a simple future that completes after a delay:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            // Get the waker from the context
            let waker = cx.waker().clone();
            
            // Spawn a thread to wait for the duration
            let when = self.when;
            std::thread::spawn(move || {
                let now = Instant::now();
                if now < when {
                    std::thread::sleep(when - now);
                }
                waker.wake();
            });

            Poll::Pending
        }
    }
}
```

## Waker and Context

The `Waker` is a crucial component that allows futures to notify the executor when they're ready to make progress. Here's how it works:

1. When a future returns `Poll::Pending`, it should store the `Waker` from the `Context`
2. When the future is ready to make progress, it calls `wake()` on the stored `Waker`
3. The executor then knows to poll the future again

### Example: Using Waker

```rust
use std::sync::{Arc, Mutex};
use std::task::Waker;

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

struct MyFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Future for MyFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut shared_state = self.shared_state.lock().unwrap();
        
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
```

## Async/Await and Futures

The `async` keyword in Rust is syntactic sugar that transforms your code into a state machine that implements the `Future` trait. Here's how it works:

```rust
async fn my_async_function() -> String {
    // This gets transformed into a future that implements Future<Output = String>
    "Hello, async world!".to_string()
}

// The above is roughly equivalent to:
fn my_async_function() -> impl Future<Output = String> {
    // The compiler generates a state machine that implements Future
    async {
        "Hello, async world!".to_string()
    }
}
```

## Best Practices

1. **Always wake the waker**: When returning `Poll::Pending`, ensure you've arranged for the waker to be called when the future can make progress
2. **Handle cancellation**: Futures should handle being dropped without completing
3. **Avoid blocking**: Never block the thread in a future's poll method
4. **Use Pin correctly**: Understand when and why futures need to be pinned

## Exercises

1. Implement a future that counts down from a number to zero
2. Create a future that combines two other futures using `join`
3. Implement a future that can be cancelled

## Further Reading

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial/async)
- [Rust Async Book](https://rust-lang.github.io/async-book/02_execution/01_chapter.html)
- [Future Trait Documentation](https://doc.rust-lang.org/std/future/trait.Future.html) 