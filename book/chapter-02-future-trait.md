# Chapter 2: The Future Trait

## Introduction

The `Future` trait is the cornerstone of Rust's async ecosystem. It represents a computation that will complete at some point in the future, potentially yielding a value. Understanding this trait deeply is essential for mastering async programming in Rust.

## The Future Trait Definition

Let's start by examining the actual trait definition:

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

This simple definition contains several important concepts:

### Associated Type: Output
```rust
type Output;
```

The `Output` associated type specifies what type of value this Future will produce when it completes. This is similar to the `Item` type in the `Iterator` trait.

Examples:
- `Future<Output = String>` - produces a String when complete
- `Future<Output = Result<i32, Error>>` - produces a Result when complete
- `Future<Output = ()>` - produces nothing meaningful (unit type)

### The poll Method
```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
```

The `poll` method is where all the work happens. Let's break down each part:

#### Parameters

1. **`self: Pin<&mut Self>`**
   - The future itself, pinned in memory
   - Pin ensures the future won't move after being polled
   - Required for memory safety with self-referential futures

2. **`cx: &mut Context<'_>`**
   - Provides access to the current task's context
   - Contains a `Waker` for efficient scheduling
   - Allows the future to signal when it's ready to make progress

#### Return Type: Poll<Self::Output>

The `Poll` enum represents the current state of the future:

```rust
pub enum Poll<T> {
    Ready(T),    // Future has completed with value T
    Pending,     // Future is not ready yet, will be woken later
}
```

## Understanding Poll States

### Poll::Ready(value)

When a future returns `Poll::Ready(value)`:
- The future has completed its work
- The value is the final result
- The future should not be polled again
- The executor consumes the value and considers the task complete

```rust
// Example: A future that immediately returns a value
struct ImmediateFuture {
    value: Option<i32>,
}

impl Future for ImmediateFuture {
    type Output = i32;
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.value.take() {
            Poll::Ready(value)  // Complete immediately
        } else {
            panic!("Future polled after completion!");
        }
    }
}
```

### Poll::Pending

When a future returns `Poll::Pending`:
- The future is not ready to complete yet
- It's waiting for some external event (I/O, timer, etc.)
- The future MUST arrange for the task to be woken when ready
- The executor will stop polling until woken

```rust
// Example: A future that's never ready (don't do this!)
struct NeverReady;

impl Future for NeverReady {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending  // Always pending - this future never completes!
    }
}
```

## The Waker: Efficient Scheduling

The `Waker` is crucial for efficient async execution. It allows futures to signal when they're ready to make progress.

### How Waker Works

```rust
use std::task::Waker;

// When a future returns Poll::Pending, it should store the waker
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // Check if work is complete
    if self.is_complete() {
        return Poll::Ready(self.get_result());
    }
    
    // Not ready yet - store the waker for later
    self.store_waker(cx.waker().clone());
    
    Poll::Pending
}

// Later, when the external event occurs:
fn on_external_event(&self) {
    if let Some(waker) = self.get_stored_waker() {
        waker.wake();  // Tell executor to poll this future again
    }
}
```

### Waker Properties

The `Waker` has several important properties:

1. **Clone**: Can be duplicated and stored
2. **Send + Sync**: Can be shared across threads
3. **wake()**: Signals the executor to re-poll the future
4. **wake_by_ref()**: More efficient when you don't need to consume the waker

## Pin: Memory Safety for Self-Referential Futures

The `Pin<&mut Self>` parameter requires explanation. Pin is necessary because async functions can create self-referential structures.

### Why Pin is Needed

When the compiler transforms async functions into state machines, it can create structures that reference themselves:

```rust
async fn example() {
    let x = 42;
    let y = &x;  // y references x
    some_async_operation().await;
    println!("{}", y);  // y is still used after the await point
}
```

The compiler generates something like:

```rust
struct ExampleFuture {
    x: i32,
    y: *const i32,  // Points to x field - self-referential!
    state: State,
}
```

If this struct moves in memory, the pointer becomes invalid. Pin prevents this by guaranteeing the future won't move.

### Pin in Practice

Most of the time, you don't need to worry about Pin directly:

```rust
// When implementing Future, just use Pin as shown
impl Future for MyFuture {
    type Output = String;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Pin ensures this future won't move in memory
        // You can safely create self-references here
        todo!()
    }
}
```

## A Complete Example: Timer Future

Let's implement a simple timer future to see all concepts in action:

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

// Shared state between the future and the timer thread
struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // Spawn a thread to handle the timing
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut state = thread_shared_state.lock().unwrap();
            state.completed = true;
            if let Some(waker) = state.waker.take() {
                waker.wake();  // Wake the future!
            }
        });

        TimerFuture { shared_state }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        
        if shared_state.completed {
            Poll::Ready(())  // Timer has completed
        } else {
            // Store the waker so the timer thread can wake us
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending    // Still waiting
        }
    }
}
```

### Using the Timer Future

```rust
#[tokio::main]
async fn main() {
    println!("Starting timer...");
    TimerFuture::new(Duration::from_secs(2)).await;
    println!("Timer completed!");
}
```

## Future Lifecycle

Understanding the lifecycle of a future is crucial:

1. **Creation**: Future is created but no work starts (lazy)
2. **First Poll**: Executor calls poll(), work begins
3. **Pending**: Future returns Pending, stores waker
4. **External Event**: Something happens (I/O completes, timer expires)
5. **Wake**: Waker.wake() is called
6. **Re-poll**: Executor polls the future again
7. **Ready**: Future returns Ready with final value
8. **Completion**: Future is consumed, task ends

## Common Patterns

### Immediate Completion
```rust
impl Future for ImmediateValue {
    type Output = i32;
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(42)  // Always ready immediately
    }
}
```

### State Machine Pattern
```rust
enum State {
    Start,
    Waiting,
    Complete(String),
}

impl Future for StateMachineFuture {
    type Output = String;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match &mut self.state {
                State::Start => {
                    // Start some async work
                    self.state = State::Waiting;
                    // Continue to next state
                }
                State::Waiting => {
                    if self.is_work_complete() {
                        self.state = State::Complete("Done!".to_string());
                        // Continue to next state
                    } else {
                        // Store waker and return Pending
                        return Poll::Pending;
                    }
                }
                State::Complete(result) => {
                    return Poll::Ready(result.clone());
                }
            }
        }
    }
}
```

## Key Insights

### Futures are Lazy
```rust
// Creating a future does NO work
let future = async { expensive_computation() };

// Work only starts when polled
future.await;  // Now the work happens
```

### Polling Contract
- Futures must not be polled after returning `Poll::Ready`
- When returning `Poll::Pending`, futures must arrange to be woken
- Executors will only re-poll after `wake()` is called

### Memory Safety
- Pin ensures futures don't move after being polled
- This allows safe self-referential structures
- The compiler handles most Pin complexity automatically

## What's Next?

Now that we understand the Future trait fundamentals, let's see how the async/await syntax makes working with futures much more ergonomic.

In [Chapter 3: Basic async/await](./chapter-03-basic-async-await.md), we'll explore:
- How async functions work
- The await keyword and when to use it
- How the compiler transforms async/await into Future implementations
- Practical examples of async/await usage

---

## Exercises

1. **Implement a Counter Future**: Create a future that counts from 0 to N, yielding each number
2. **Analyze the Timer**: Trace through the TimerFuture example step by step
3. **Experiment with Waker**: What happens if you don't store the waker in Poll::Pending?

## Key Takeaways

- The Future trait represents asynchronous computations
- `poll()` is called by executors to advance the future
- `Poll::Ready` means complete, `Poll::Pending` means waiting
- Waker enables efficient scheduling without busy-waiting
- Pin ensures memory safety for self-referential futures
- Futures are lazy - no work happens until polled 