# Chapter 6: Testing Async Code

## Testing Strategies

Testing async code requires special consideration for handling asynchronous operations, timeouts, and concurrent behavior. This chapter covers various approaches to testing async Rust code.

## Basic Async Tests

### 1. Using `#[tokio::test]`

```rust
use tokio;

#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert_eq!(result, "expected value");
}

async fn async_function() -> &'static str {
    "expected value"
}
```

### 2. Testing with Timeouts

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_with_timeout() {
    let result = timeout(
        Duration::from_secs(1),
        long_running_operation()
    ).await;
    
    assert!(result.is_ok());
}

async fn long_running_operation() -> &'static str {
    tokio::time::sleep(Duration::from_millis(500)).await;
    "operation completed"
}
```

## Mocking Async Code

### 1. Using Mock Traits

```rust
use std::future::Future;
use std::pin::Pin;

trait Database {
    fn query(&self, id: i32) -> Pin<Box<dyn Future<Output = Result<String, String>> + '_>>;
}

struct MockDatabase {
    data: Vec<(i32, String)>,
}

impl Database for MockDatabase {
    fn query(&self, id: i32) -> Pin<Box<dyn Future<Output = Result<String, String>> + '_>> {
        let data = self.data.clone();
        Box::pin(async move {
            data.iter()
                .find(|(i, _)| *i == id)
                .map(|(_, value)| Ok(value.clone()))
                .unwrap_or(Err("Not found".to_string()))
        })
    }
}

#[tokio::test]
async fn test_database_query() {
    let db = MockDatabase {
        data: vec![(1, "test".to_string())],
    };
    
    let result = db.query(1).await;
    assert_eq!(result.unwrap(), "test");
}
```

### 2. Using Mock Streams

```rust
use futures::stream::{self, StreamExt};

struct MockStream {
    items: Vec<i32>,
}

impl Stream for MockStream {
    type Item = i32;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.items.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(this.items.remove(0)))
        }
    }
}

#[tokio::test]
async fn test_stream() {
    let mut stream = MockStream {
        items: vec![1, 2, 3],
    };
    
    let result: Vec<_> = stream.collect().await;
    assert_eq!(result, vec![1, 2, 3]);
}
```

## Integration Testing

### 1. Testing with Real Services

```rust
use tokio::net::TcpListener;

#[tokio::test]
async fn test_server() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    // Start server in background
    tokio::spawn(async move {
        server::run(listener).await;
    });
    
    // Test client
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}", addr))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
}
```

### 2. Testing with Test Containers

```rust
use testcontainers::*;
use tokio_postgres::NoTls;

#[tokio::test]
async fn test_with_postgres() {
    let docker = clients::Cli::default();
    let postgres = docker.run(images::postgres::Postgres::default());
    
    let port = postgres.get_host_port(5432);
    let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
    
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls)
        .await
        .unwrap();
        
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    
    // Run tests with the database client
    let rows = client
        .query("SELECT 1", &[])
        .await
        .unwrap();
        
    assert_eq!(rows[0].get::<_, i32>(0), 1);
}
```

## Property-Based Testing

### 1. Using Proptest

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_async_property(x in 0..1000u32) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async_function(x));
        assert!(result > 0);
    }
}

async fn async_function(x: u32) -> u32 {
    tokio::time::sleep(Duration::from_millis(1)).await;
    x + 1
}
```

### 2. Testing Concurrent Properties

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_concurrent_access() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = counter.clone();
        handles.push(tokio::spawn(async move {
            let mut lock = counter.lock().await;
            *lock += 1;
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let final_count = *counter.lock().await;
    assert_eq!(final_count, 10);
}
```

## Best Practices

1. **Use Appropriate Timeouts**: Always set timeouts for async tests
2. **Mock External Dependencies**: Use mocks for external services
3. **Test Error Cases**: Include tests for error conditions
4. **Test Concurrent Behavior**: Verify correct behavior under concurrency
5. **Use Test Containers**: For integration tests with real services

## Testing Utilities

### 1. Async Test Helpers

```rust
use tokio::time::{sleep, Duration};

async fn with_timeout<F, T>(duration: Duration, future: F) -> Option<T>
where
    F: Future<Output = T>,
{
    match timeout(duration, future).await {
        Ok(result) => Some(result),
        Err(_) => None,
    }
}

#[tokio::test]
async fn test_with_helper() {
    let result = with_timeout(
        Duration::from_secs(1),
        async_function()
    ).await;
    
    assert!(result.is_some());
}
```

### 2. Test Fixtures

```rust
struct TestFixture {
    db: MockDatabase,
    client: reqwest::Client,
}

impl TestFixture {
    async fn new() -> Self {
        TestFixture {
            db: MockDatabase::new(),
            client: reqwest::Client::new(),
        }
    }
    
    async fn setup(&self) {
        // Setup test data
    }
    
    async fn teardown(&self) {
        // Cleanup test data
    }
}

#[tokio::test]
async fn test_with_fixture() {
    let fixture = TestFixture::new().await;
    fixture.setup().await;
    
    // Run tests
    
    fixture.teardown().await;
}
```

## Exercises

1. Write tests for an async function that makes HTTP requests
2. Create a mock for an async database interface
3. Implement property-based tests for a concurrent data structure

## Further Reading

- [Tokio Testing](https://docs.rs/tokio-test)
- [Testcontainers](https://docs.rs/testcontainers)
- [Proptest](https://docs.rs/proptest) 