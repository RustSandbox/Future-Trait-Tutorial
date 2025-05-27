use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MockStream {
    items: Vec<i32>,
    index: usize,
}

impl Stream for MockStream {
    type Item = i32;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.index < self.items.len() {
            let item = self.items[self.index];
            self.index += 1;
            Poll::Ready(Some(item))
        } else {
            Poll::Ready(None)
        }
    }
}

#[tokio::test]
async fn test_mock_stream() {
    let stream = MockStream {
        items: vec![1, 2, 3],
        index: 0,
    };

    let result: Vec<_> = stream.collect().await;
    assert_eq!(result, vec![1, 2, 3]);
}
