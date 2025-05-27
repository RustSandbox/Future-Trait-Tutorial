# Chapter 3: Async Runtime

## Understanding the Runtime

The async runtime is the system that drives futures to completion. It consists of several key components:

1. **Executor**: Drives futures to completion by polling them
2. **Reactor**: Handles I/O events and notifies the executor
3. **Scheduler**: Manages which tasks should run when

## The Executor

The executor is responsible for polling futures until they complete. Here's a simple executor implementation:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::collections::VecDeque;

struct Executor {
    tasks: VecDeque<Task>,
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
}

impl Executor {
    fn new() -> Self {
        Executor {
            tasks: VecDeque::new(),
        }
    }

    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.push_back(Task {
            future: Box::pin(future),
        });
    }

    fn run(&mut self) {
        let waker = waker_fn(|| {});
        let mut cx = Context::from_waker(&waker);

        while let Some(mut task) = self.tasks.pop_front() {
            match task.future.as_mut().poll(&mut cx) {
                Poll::Ready(()) => {
                    // Task completed
                }
                Poll::Pending => {
                    self.tasks.push_back(task);
                }
            }
        }
    }
}
```

## The Reactor

The reactor handles I/O events and notifies the executor when futures can make progress. Here's a simplified example:

```rust
use std::sync::{Arc, Mutex};
use std::task::Waker;

struct Reactor {
    wakers: Arc<Mutex<Vec<Waker>>>,
}

impl Reactor {
    fn new() -> Self {
        Reactor {
            wakers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn register_waker(&self, waker: Waker) {
        let mut wakers = self.wakers.lock().unwrap();
        wakers.push(waker);
    }

    fn wake_all(&self) {
        let mut wakers = self.wakers.lock().unwrap();
        for waker in wakers.drain(..) {
            waker.wake();
        }
    }
}
```

## Tokio Runtime

Tokio is the most popular async runtime for Rust. It provides a full-featured runtime with:

1. **Multi-threaded runtime**: Can utilize multiple CPU cores
2. **Work-stealing scheduler**: Efficiently distributes work across threads
3. **I/O driver**: Handles async I/O operations
4. **Timer**: Manages time-based operations

### Basic Usage

```rust
use tokio;

#[tokio::main]
async fn main() {
    // Spawn a new task
    let handle = tokio::spawn(async {
        // Do some work
        "Hello from spawned task"
    });

    // Wait for the task to complete
    let result = handle.await.unwrap();
    println!("{}", result);
}
```

## Runtime Features

### 1. Task Scheduling

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Spawn multiple tasks
    let mut handles = vec![];
    
    for i in 0..3 {
        let handle = tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            println!("Task {} completed", i);
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}
```

### 2. I/O Operations

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::open("foo.txt").await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    println!("File contents: {:?}", contents);
    Ok(())
}
```

### 3. Time Operations

```rust
use tokio::time::{sleep, Duration, timeout};

#[tokio::main]
async fn main() {
    // Sleep for a duration
    sleep(Duration::from_secs(1)).await;
    
    // Timeout an operation
    match timeout(Duration::from_secs(1), long_operation()).await {
        Ok(_) => println!("Operation completed in time"),
        Err(_) => println!("Operation timed out"),
    }
}

async fn long_operation() {
    sleep(Duration::from_secs(2)).await;
}
```

## Best Practices

1. **Choose the right runtime**: Use Tokio for most applications, but consider alternatives for specific needs
2. **Manage resources**: Be careful with blocking operations and resource limits
3. **Handle errors**: Properly propagate and handle errors in async code
4. **Monitor performance**: Use tools like `tokio-console` to monitor runtime performance

## Exercises

1. Implement a simple executor that can run multiple futures concurrently
2. Create a task that performs I/O operations using Tokio
3. Implement a timeout mechanism for long-running operations

## Further Reading

- [Tokio Documentation](https://docs.rs/tokio)
- [Async Runtime Design](https://tokio.rs/blog/2019-10-scheduler)
- [Tokio Console](https://github.com/tokio-rs/console) 