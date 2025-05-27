# Chapter 7: Real-World Applications

## Web Server with Tokio

Let's build a simple but production-ready web server using Tokio and Axum.

```rust
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// Application state
#[derive(Clone)]
struct AppState {
    counter: Arc<RwLock<i32>>,
}

// Request and response types
#[derive(Deserialize)]
struct IncrementRequest {
    amount: i32,
}

#[derive(Serialize)]
struct CounterResponse {
    value: i32,
}

// Route handlers
async fn get_counter(
    State(state): State<AppState>,
) -> Json<CounterResponse> {
    let counter = state.counter.read().await;
    Json(CounterResponse { value: *counter })
}

async fn increment_counter(
    State(state): State<AppState>,
    Json(req): Json<IncrementRequest>,
) -> Json<CounterResponse> {
    let mut counter = state.counter.write().await;
    *counter += req.amount;
    Json(CounterResponse { value: *counter })
}

#[tokio::main]
async fn main() {
    // Initialize application state
    let state = AppState {
        counter: Arc::new(RwLock::new(0)),
    };

    // Build router
    let app = Router::new()
        .route("/counter", get(get_counter))
        .route("/counter/increment", post(increment_counter))
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
```

## Database Client

Here's an example of an async database client using SQLx:

```rust
use sqlx::postgres::PgPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    async fn create_user(&self, name: &str, email: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
            name,
            email
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn get_user(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn list_users(&self) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as!(
            User,
            "SELECT * FROM users ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}
```

## Message Queue Consumer

Here's an example of a message queue consumer using RabbitMQ:

```rust
use lapin::{
    options::*,
    types::FieldTable,
    Connection, ConnectionProperties, Result,
};
use tokio::time::Duration;

async fn consume_messages() -> Result<()> {
    // Connect to RabbitMQ
    let addr = "amqp://guest:guest@localhost:5672";
    let conn = Connection::connect(
        addr,
        ConnectionProperties::default(),
    ).await?;

    // Create channel
    let channel = conn.create_channel().await?;

    // Declare queue
    let queue = channel
        .queue_declare(
            "my_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // Set up consumer
    let consumer = channel
        .basic_consume(
            "my_queue",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // Process messages
    for delivery in consumer {
        let (_, delivery) = delivery?;
        
        // Process message
        if let Ok(message) = String::from_utf8(delivery.data.clone()) {
            println!("Received message: {}", message);
        }

        // Acknowledge message
        delivery.ack(BasicAckOptions::default()).await?;
    }

    Ok(())
}
```

## File Processing Pipeline

Here's an example of a file processing pipeline using streams:

```rust
use futures::stream::{self, StreamExt};
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};

async fn process_files(paths: Vec<String>) -> io::Result<()> {
    stream::iter(paths)
        .map(|path| async move {
            let file = File::open(&path).await?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();
            
            let mut count = 0;
            while let Some(line) = lines.next_line().await? {
                if line.contains("error") {
                    count += 1;
                }
            }
            
            Ok::<_, io::Error>((path, count))
        })
        .buffer_unordered(10) // Process up to 10 files concurrently
        .for_each(|result| async {
            match result {
                Ok((path, count)) => println!("{}: {} errors", path, count),
                Err(e) => eprintln!("Error: {}", e),
            }
        })
        .await;

    Ok(())
}
```

## WebSocket Server

Here's an example of a WebSocket server using Tokio and Warp:

```rust
use futures::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};

struct ChatServer {
    users: broadcast::Sender<String>,
}

impl ChatServer {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { users: tx }
    }

    async fn handle_connection(&self, ws: WebSocket) {
        let (mut ws_sender, mut ws_receiver) = ws.split();
        let mut rx = self.users.subscribe();

        // Spawn task to forward messages to WebSocket
        let forward_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if ws_sender.send(Message::text(msg)).await.is_err() {
                    break;
                }
            }
        });

        // Handle incoming messages
        while let Some(result) = ws_receiver.next().await {
            if let Ok(msg) = result {
                if let Ok(text) = msg.to_str() {
                    let _ = self.users.send(text.to_string());
                }
            }
        }

        forward_task.abort();
    }
}

#[tokio::main]
async fn main() {
    let chat_server = ChatServer::new();
    let chat_server = warp::any().map(move || chat_server.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(chat_server)
        .map(|ws: warp::ws::Ws, server: ChatServer| {
            ws.on_upgrade(move |socket| server.handle_connection(socket))
        });

    warp::serve(ws_route)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
```

## Best Practices

1. **Error Handling**: Use proper error types and propagation
2. **Resource Management**: Clean up resources properly
3. **Concurrency Control**: Use appropriate concurrency limits
4. **Monitoring**: Implement logging and metrics
5. **Graceful Shutdown**: Handle shutdown signals properly

## Exercises

1. Implement a rate-limited API client
2. Create a file upload service with progress tracking
3. Build a real-time notification system

## Further Reading

- [Tokio Documentation](https://docs.rs/tokio)
- [Axum Documentation](https://docs.rs/axum)
- [SQLx Documentation](https://docs.rs/sqlx)
- [Warp Documentation](https://docs.rs/warp) 