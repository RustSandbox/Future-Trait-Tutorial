use std::fmt;

#[derive(Debug)]
struct ErrorContext<E> {
    context: String,
    error: E,
}

impl<E: std::error::Error> std::error::Error for ErrorContext<E> {}

impl<E: std::error::Error> fmt::Display for ErrorContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.error)
    }
}

#[tokio::test]
async fn test_error_context() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let context = ErrorContext {
        context: "Failed to read file".to_string(),
        error: io_error,
    };

    assert!(context.to_string().contains("Failed to read file"));
    assert!(context.to_string().contains("file not found"));
}

use futures::stream::StreamExt;
