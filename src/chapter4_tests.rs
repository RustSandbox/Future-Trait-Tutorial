use futures::stream::StreamExt;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_stream_forwarding() {
    let (tx, mut rx) = mpsc::channel(10);
    let stream = futures::stream::iter(1..=5);

    // Use for_each instead of forward
    stream
        .for_each(|item| {
            let mut tx = tx.clone();
            async move {
                tx.send(item).await.unwrap();
            }
        })
        .await;

    // Verify received items
    let mut received = Vec::new();
    while let Some(item) = rx.recv().await {
        received.push(item);
    }
    assert_eq!(received, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_stream_processing() {
    let (tx, mut rx) = mpsc::channel(10);
    let stream = futures::stream::iter(1..=10)
        .map(|x| x * 2)
        .filter(|x| x % 3 == 0);

    // Use for_each instead of forward
    stream
        .for_each(|item| {
            let mut tx = tx.clone();
            async move {
                tx.send(item).await.unwrap();
            }
        })
        .await;

    // Verify received items
    let mut received = Vec::new();
    while let Some(item) = rx.recv().await {
        received.push(item);
    }
    assert_eq!(received, vec![6, 12, 18]);
}
