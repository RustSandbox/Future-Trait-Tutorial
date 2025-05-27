# Chapter 5: Error Handling

## Understanding Async Error Handling

Error handling in async Rust combines the power of the `Result` type with the async/await syntax. This chapter covers various patterns and best practices for handling errors in asynchronous code.

## Basic Error Handling

### 1. Using `?` Operator

```rust
use std::io;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn read_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let contents = read_file("example.txt").await?;
    println!("File contents: {}", contents);
    Ok(())
}
```

### 2. Custom Error Types

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum AppError {
    IoError(std::io::Error),
    ParseError(String),
    NetworkError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::ParseError(e) => write!(f, "Parse error: {}", e),
            AppError::NetworkError(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}
```

## Error Handling Patterns

### 1. Error Propagation

```rust
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn process_file(path: &str) -> Result<String, AppError> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    
    if contents.is_empty() {
        return Err(AppError::ParseError("File is empty".into()));
    }
    
    Ok(contents)
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    match process_file("example.txt").await {
        Ok(contents) => println!("Success: {}", contents),
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}
```

### 2. Error Mapping

```rust
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn read_and_parse(path: &str) -> Result<i32, AppError> {
    let contents = File::open(path)
        .await
        .map_err(|e| AppError::IoError(e))?
        .read_to_string()
        .await
        .map_err(|e| AppError::IoError(e))?;
        
    contents
        .trim()
        .parse::<i32>()
        .map_err(|e| AppError::ParseError(e.to_string()))
}
```

## Error Handling in Streams

### 1. Stream Error Handling

```rust
use futures::stream::{self, StreamExt};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn process_files(paths: Vec<String>) -> Result<Vec<String>, AppError> {
    stream::iter(paths)
        .then(|path| async move {
            let mut file = File::open(&path).await?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).await?;
            Ok::<String, AppError>(contents)
        })
        .collect()
        .await
}
```

### 2. Error Recovery

```rust
use futures::stream::{self, StreamExt};

async fn process_with_retry(path: String) -> Result<String, AppError> {
    let mut attempts = 0;
    let max_attempts = 3;
    
    while attempts < max_attempts {
        match process_file(&path).await {
            Ok(contents) => return Ok(contents),
            Err(e) => {
                attempts += 1;
                if attempts == max_attempts {
                    return Err(e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
    
    Err(AppError::NetworkError("Max retries exceeded".into()))
}
```

## Best Practices

1. **Use Custom Error Types**: Create specific error types for your application
2. **Implement Error Conversion**: Use `From` trait for automatic error conversion
3. **Handle Errors Early**: Deal with errors as close to their source as possible
4. **Provide Context**: Include relevant information in error messages
5. **Use Error Recovery**: Implement retry logic for transient errors

## Error Handling Utilities

### 1. Error Context

```rust
use std::error::Error;
use std::fmt;

struct ErrorContext<E> {
    error: E,
    context: String,
}

impl<E: Error> Error for ErrorContext<E> {}

impl<E: Error> fmt::Display for ErrorContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.error)
    }
}

fn add_context<E: Error>(error: E, context: impl Into<String>) -> ErrorContext<E> {
    ErrorContext {
        error,
        context: context.into(),
    }
}
```

### 2. Error Logging

```rust
use tracing::{error, info};

async fn process_with_logging(path: &str) -> Result<String, AppError> {
    info!("Processing file: {}", path);
    
    match process_file(path).await {
        Ok(contents) => {
            info!("Successfully processed file: {}", path);
            Ok(contents)
        }
        Err(e) => {
            error!("Failed to process file {}: {}", path, e);
            Err(e)
        }
    }
}
```

## Exercises

1. Implement a custom error type for a network application
2. Create a function that handles multiple types of errors
3. Implement retry logic for a network operation

## Further Reading

- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [anyhow Documentation](https://docs.rs/anyhow)
- [thiserror Documentation](https://docs.rs/thiserror) 