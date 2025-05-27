# Chapter 4: Streams and Sinks

## Understanding Streams

A `Stream` is like a `Future` that can yield multiple values over time. It's the async equivalent of an `Iterator`.

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Stream {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}
```

### Key Concepts

1. **Multiple Values**: Unlike `Future`, which produces a single value, `Stream` can produce multiple values
2. **Async Iterator**: Similar to `Iterator`, but with async operations
3. **Backpressure**: Built-in support for handling backpressure through the `Poll` type

## Creating Streams

### 1. From Iterators

```rust
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() {
    let stream = stream::iter(vec![1, 2, 3]);
    
    // Process the stream
    let sum = stream
        .map(|x| x * 2)
        .fold(0, |acc, x| async move { acc + x })
        .await;
        
    println!("Sum: {}", sum);
}
```

### 2. From Futures

```rust
use futures::stream::{self, StreamExt};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let stream = stream::unfold(0, |state| async move {
        if state < 3 {
            sleep(Duration::from_millis(100)).await;
            Some((state, state + 1))
        } else {
            None
        }
    });

    // Collect all values
    let values: Vec<_> = stream.collect().await;
    println!("Values: {:?}", values);
}
```

## Processing Streams

### 1. Basic Operations

```rust
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() {
    let stream = stream::iter(1..=5);
    
    // Map
    let doubled: Vec<_> = stream
        .map(|x| x * 2)
        .collect()
        .await;
    println!("Doubled: {:?}", doubled);
    
    // Filter
    let even: Vec<_> = stream::iter(1..=5)
        .filter(|x| futures::future::ready(*x % 2 == 0))
        .collect()
        .await;
    println!("Even: {:?}", even);
}
```

### 2. Chaining Operations

```rust
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() {
    let result = stream::iter(1..=10)
        .map(|x| x * 2)
        .filter(|x| futures::future::ready(*x % 3 == 0))
        .fold(0, |acc, x| async move { acc + x })
        .await;
        
    println!("Result: {}", result);
}
```

## Understanding Sinks

A `Sink` is the opposite of a `Stream` - it's an async consumer that can receive multiple values over time.

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Sink<Item> {
    type Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error>;
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
}
```

### Example: Channel Sink

```rust
use futures::sink::SinkExt;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    
    // Send values through the sink
    let mut sink = tx;
    for i in 0..5 {
        sink.send(i).await.unwrap();
    }
    
    // Receive values
    while let Some(value) = rx.recv().await {
        println!("Received: {}", value);
    }
}
```

## Combining Streams and Sinks

### 1. Forwarding

```rust
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    
    // Create a stream
    let stream = futures::stream::iter(1..=5);
    
    // Forward the stream to the sink
    stream.forward(tx).await.unwrap();
    
    // Receive values
    while let Some(value) = rx.recv().await {
        println!("Received: {}", value);
    }
}
```

### 2. Processing Pipeline

```rust
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    
    // Create a processing pipeline
    let stream = futures::stream::iter(1..=5)
        .map(|x| x * 2)
        .filter(|x| futures::future::ready(*x % 3 == 0));
        
    // Forward to sink
    stream.forward(tx).await.unwrap();
    
    // Receive processed values
    while let Some(value) = rx.recv().await {
        println!("Processed: {}", value);
    }
}
```

## Best Practices

1. **Handle backpressure**: Always check `poll_ready` before sending to a sink
2. **Use appropriate buffer sizes**: Choose buffer sizes that match your application's needs
3. **Process in chunks**: Use `chunks` or `buffer_unordered` for batch processing
4. **Handle errors**: Properly propagate and handle errors in stream processing

## Exercises

1. Implement a stream that generates Fibonacci numbers
2. Create a sink that writes to a file
3. Build a processing pipeline that transforms and filters data

## Further Reading

- [Futures Documentation](https://docs.rs/futures)
- [Tokio Streams](https://docs.rs/tokio-stream)
- [Async Streams RFC](https://rust-lang.github.io/rfcs/2996-async-iterator.html) 