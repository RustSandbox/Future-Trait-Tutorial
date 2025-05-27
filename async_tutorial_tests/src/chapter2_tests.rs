use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Simple Delay Future
struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
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

// Shared State Future
use std::sync::{Arc, Mutex};

struct SharedState {
    completed: bool,
    waker: Option<std::task::Waker>,
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

#[tokio::test]
async fn test_delay_future() {
    let when = Instant::now() + Duration::from_millis(100);
    let delay = Delay { when };

    let result = delay.await;
    assert_eq!(result, "done");
}

#[tokio::test]
async fn test_shared_state_future() {
    let shared_state = Arc::new(Mutex::new(SharedState {
        completed: false,
        waker: None,
    }));

    let future = MyFuture {
        shared_state: shared_state.clone(),
    };

    // Spawn a task to complete the future
    let shared_state_clone = shared_state.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        let mut state = shared_state_clone.lock().unwrap();
        state.completed = true;
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    });

    future.await;
    assert!(shared_state.lock().unwrap().completed);
}

// Async function test
async fn my_async_function() -> String {
    "Hello, async world!".to_string()
}

#[tokio::test]
async fn test_async_function() {
    let result = my_async_function().await;
    assert_eq!(result, "Hello, async world!");
}
