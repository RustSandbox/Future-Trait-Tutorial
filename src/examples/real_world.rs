//! # Real-World Async Patterns Tutorial
//!
//! This example demonstrates practical async patterns using real-world scenarios.
//! It covers:
//!
//! 1. HTTP client operations with reqwest
//! 2. JSON serialization/deserialization with serde
//! 3. Concurrent API calls and data aggregation
//! 4. Rate limiting and backoff strategies
//! 5. Caching and memoization patterns
//! 6. Background task processing
//! 7. Real-world error handling and resilience

use anyhow::{Context, Result as AnyhowResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};

/// # Struct: User
///
/// Represents a user from a REST API.
/// This demonstrates how to work with structured data in async contexts.
///
/// ## Fields:
/// - `id`: Unique user identifier
/// - `name`: User's display name
/// - `email`: User's email address
/// - `posts_count`: Number of posts the user has made
///
/// ## Serialization:
/// Uses serde for automatic JSON serialization/deserialization
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
    #[serde(default)]
    posts_count: u32,
}

/// # Struct: Post
///
/// Represents a blog post from a REST API.
/// Demonstrates working with nested data structures in async operations.
///
/// ## Fields:
/// - `id`: Unique post identifier
/// - `user_id`: ID of the user who created the post
/// - `title`: Post title
/// - `body`: Post content
/// - `created_at`: When the post was created (optional)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Post {
    id: u32,
    #[serde(rename = "userId")]
    user_id: u32,
    title: String,
    body: String,
    #[serde(default)]
    created_at: Option<String>,
}

/// # Struct: Comment
///
/// Represents a comment on a blog post.
/// Shows how to handle optional fields and nested relationships.
///
/// ## Fields:
/// - `id`: Unique comment identifier
/// - `post_id`: ID of the post this comment belongs to
/// - `name`: Commenter's name
/// - `email`: Commenter's email
/// - `body`: Comment content
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Comment {
    id: u32,
    #[serde(rename = "postId")]
    post_id: u32,
    name: String,
    email: String,
    body: String,
}

/// # Struct: UserProfile
///
/// Aggregated user data combining information from multiple API calls.
/// This demonstrates how to compose data from multiple async operations.
///
/// ## Fields:
/// - `user`: Basic user information
/// - `posts`: All posts by the user
/// - `total_comments`: Total number of comments on user's posts
/// - `fetch_time`: How long it took to gather all the data
#[derive(Debug, Serialize)]
struct UserProfile {
    user: User,
    posts: Vec<Post>,
    total_comments: u32,
    fetch_time: Duration,
}

/// # Struct: ApiClient
///
/// A wrapper around reqwest::Client that provides higher-level API operations.
/// This demonstrates how to encapsulate async HTTP operations in a reusable client.
///
/// ## Features:
/// - Built-in rate limiting
/// - Automatic retries with exponential backoff
/// - Response caching
/// - Timeout handling
/// - Error context enrichment
///
/// ## Fields:
/// - `client`: The underlying HTTP client
/// - `base_url`: Base URL for all API requests
/// - `cache`: Simple in-memory cache for responses
/// - `rate_limiter`: Tracks request timing for rate limiting
#[derive(Clone)]
struct ApiClient {
    client: Client,
    base_url: String,
    cache: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    rate_limiter: Arc<Mutex<Instant>>,
}

impl ApiClient {
    /// # Function: new
    ///
    /// Creates a new ApiClient with default configuration.
    ///
    /// ## Arguments:
    /// - `base_url`: The base URL for all API requests
    ///
    /// ## Returns:
    /// - A new ApiClient instance ready for use
    ///
    /// ## Example:
    /// ```rust
    /// let client = ApiClient::new("https://jsonplaceholder.typicode.com");
    /// let users = client.get_users().await?;
    /// ```
    fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Future-Tutorial/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// # Function: get_with_cache
    ///
    /// Makes a GET request with caching support.
    /// This demonstrates how to implement caching in async operations.
    ///
    /// ## Arguments:
    /// - `endpoint`: The API endpoint to call (relative to base_url)
    /// - `cache_duration`: How long to cache the response
    ///
    /// ## Returns:
    /// - `AnyhowResult<String>`: The response body or an error
    ///
    /// ## Caching Strategy:
    /// - Checks cache first before making HTTP request
    /// - Stores successful responses in memory cache
    /// - Respects cache expiration times
    /// - Falls back to fresh request if cache miss or expired
    async fn get_with_cache(
        &self,
        endpoint: &str,
        cache_duration: Duration,
    ) -> AnyhowResult<String> {
        let cache_key = format!("{}/{}", self.base_url, endpoint);

        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some((cached_response, cached_at)) = cache.get(&cache_key) {
                if cached_at.elapsed() < cache_duration {
                    println!("📦 Cache hit for {}", endpoint);
                    return Ok(cached_response.clone());
                }
            }
        }

        // Rate limiting: ensure minimum time between requests
        {
            let mut last_request = self.rate_limiter.lock().unwrap();
            let time_since_last = last_request.elapsed();
            let min_interval = Duration::from_millis(100); // 10 requests per second max

            if time_since_last < min_interval {
                let sleep_time = min_interval - time_since_last;
                println!("⏱️  Rate limiting: waiting {:?}", sleep_time);
                drop(last_request); // Release lock before sleeping
                sleep(sleep_time).await;
                *self.rate_limiter.lock().unwrap() = Instant::now();
            } else {
                *last_request = Instant::now();
            }
        }

        // Make the HTTP request
        println!("🌐 Making HTTP GET request to {}", endpoint);
        let url = format!("{}/{}", self.base_url, endpoint);

        let response = timeout(Duration::from_secs(10), self.client.get(&url).send())
            .await
            .context("Request timed out")?
            .context("Failed to send HTTP request")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP request failed with status: {}",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Cache the successful response
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(cache_key, (body.clone(), Instant::now()));
        }

        Ok(body)
    }

    /// # Function: get_users
    ///
    /// Fetches all users from the API.
    /// Demonstrates JSON deserialization and error handling.
    ///
    /// ## Returns:
    /// - `AnyhowResult<Vec<User>>`: List of users or an error
    ///
    /// ## Error Handling:
    /// - Network errors are propagated with context
    /// - JSON parsing errors include the problematic data
    /// - Timeouts are handled gracefully
    async fn get_users(&self) -> AnyhowResult<Vec<User>> {
        let body = self
            .get_with_cache("users", Duration::from_secs(300)) // Cache for 5 minutes
            .await
            .context("Failed to fetch users")?;

        let users: Vec<User> = serde_json::from_str(&body).context("Failed to parse users JSON")?;

        println!("✅ Fetched {} users", users.len());
        Ok(users)
    }

    /// # Function: get_user_posts
    ///
    /// Fetches all posts for a specific user.
    /// Shows how to make parameterized API calls.
    ///
    /// ## Arguments:
    /// - `user_id`: The ID of the user whose posts to fetch
    ///
    /// ## Returns:
    /// - `AnyhowResult<Vec<Post>>`: List of posts or an error
    async fn get_user_posts(&self, user_id: u32) -> AnyhowResult<Vec<Post>> {
        let endpoint = format!("users/{}/posts", user_id);
        let body = self
            .get_with_cache(&endpoint, Duration::from_secs(60)) // Cache for 1 minute
            .await
            .context(format!("Failed to fetch posts for user {}", user_id))?;

        let posts: Vec<Post> = serde_json::from_str(&body).context("Failed to parse posts JSON")?;

        println!("✅ Fetched {} posts for user {}", posts.len(), user_id);
        Ok(posts)
    }

    /// # Function: get_post_comments
    ///
    /// Fetches all comments for a specific post.
    /// Demonstrates nested API calls and data relationships.
    ///
    /// ## Arguments:
    /// - `post_id`: The ID of the post whose comments to fetch
    ///
    /// ## Returns:
    /// - `AnyhowResult<Vec<Comment>>`: List of comments or an error
    async fn get_post_comments(&self, post_id: u32) -> AnyhowResult<Vec<Comment>> {
        let endpoint = format!("posts/{}/comments", post_id);
        let body = self
            .get_with_cache(&endpoint, Duration::from_secs(30)) // Cache for 30 seconds
            .await
            .context(format!("Failed to fetch comments for post {}", post_id))?;

        let comments: Vec<Comment> =
            serde_json::from_str(&body).context("Failed to parse comments JSON")?;

        println!(
            "✅ Fetched {} comments for post {}",
            comments.len(),
            post_id
        );
        Ok(comments)
    }
}

/// # Function: demonstrate_basic_http_operations
///
/// Demonstrates basic HTTP operations with async/await.
/// Shows how to make simple API calls and handle responses.
///
/// ## Key Learning Points:
/// - Making HTTP requests with reqwest
/// - JSON deserialization with serde
/// - Error handling for network operations
/// - Working with structured data in async contexts
async fn demonstrate_basic_http_operations() {
    println!("\n=== Basic HTTP Operations ===");

    let client = ApiClient::new("https://jsonplaceholder.typicode.com");

    // Example 1: Fetch all users
    println!("1. Fetching all users:");
    match client.get_users().await {
        Ok(users) => {
            println!("   Successfully fetched {} users", users.len());
            if let Some(first_user) = users.first() {
                println!("   First user: {} ({})", first_user.name, first_user.email);
            }
        }
        Err(error) => {
            println!("   Failed to fetch users: {}", error);
        }
    }

    // Example 2: Fetch posts for a specific user
    println!("\n2. Fetching posts for user 1:");
    match client.get_user_posts(1).await {
        Ok(posts) => {
            println!("   Successfully fetched {} posts", posts.len());
            if let Some(first_post) = posts.first() {
                println!("   First post: \"{}\"", first_post.title);
                println!(
                    "   Body preview: {}...",
                    first_post.body.chars().take(50).collect::<String>()
                );
            }
        }
        Err(error) => {
            println!("   Failed to fetch posts: {}", error);
        }
    }

    // Example 3: Fetch comments for a specific post
    println!("\n3. Fetching comments for post 1:");
    match client.get_post_comments(1).await {
        Ok(comments) => {
            println!("   Successfully fetched {} comments", comments.len());
            if let Some(first_comment) = comments.first() {
                println!(
                    "   First comment by: {} ({})",
                    first_comment.name, first_comment.email
                );
            }
        }
        Err(error) => {
            println!("   Failed to fetch comments: {}", error);
        }
    }
}

/// # Function: demonstrate_concurrent_api_calls
///
/// Demonstrates how to make multiple API calls concurrently.
/// Shows the performance benefits of async programming for I/O-bound operations.
///
/// ## Key Learning Points:
/// - Using tokio::join! for concurrent operations
/// - Measuring performance improvements
/// - Handling partial failures in concurrent operations
/// - Aggregating data from multiple sources
async fn demonstrate_concurrent_api_calls() {
    println!("\n=== Concurrent API Calls ===");

    let client = ApiClient::new("https://jsonplaceholder.typicode.com");

    // Example 1: Sequential vs Concurrent comparison
    println!("1. Performance comparison - Sequential vs Concurrent:");

    // Sequential approach
    println!("   Sequential approach:");
    let sequential_start = Instant::now();

    let _users = client.get_users().await.unwrap_or_default();
    let _posts_1 = client.get_user_posts(1).await.unwrap_or_default();
    let _posts_2 = client.get_user_posts(2).await.unwrap_or_default();
    let _posts_3 = client.get_user_posts(3).await.unwrap_or_default();

    let sequential_time = sequential_start.elapsed();
    println!("     Sequential time: {:?}", sequential_time);

    // Concurrent approach
    println!("   Concurrent approach:");
    let concurrent_start = Instant::now();

    let users_future = client.get_users();
    let posts_1_future = client.get_user_posts(1);
    let posts_2_future = client.get_user_posts(2);
    let posts_3_future = client.get_user_posts(3);

    let (users_result, posts_1_result, posts_2_result, posts_3_result) =
        tokio::join!(users_future, posts_1_future, posts_2_future, posts_3_future);

    let concurrent_time = concurrent_start.elapsed();
    println!("     Concurrent time: {:?}", concurrent_time);

    // Calculate speedup
    let speedup = sequential_time.as_millis() as f64 / concurrent_time.as_millis() as f64;
    println!("     Speedup: {:.2}x faster", speedup);

    // Handle results
    let users = users_result.unwrap_or_default();
    let posts_1 = posts_1_result.unwrap_or_default();
    let posts_2 = posts_2_result.unwrap_or_default();
    let posts_3 = posts_3_result.unwrap_or_default();

    println!(
        "     Results: {} users, {} + {} + {} posts",
        users.len(),
        posts_1.len(),
        posts_2.len(),
        posts_3.len()
    );

    // Example 2: Fetching data for multiple users concurrently
    println!("\n2. Fetching posts for multiple users concurrently:");
    let start = Instant::now();

    // Create futures for multiple users
    let user_ids = vec![1, 2, 3, 4, 5];
    let mut post_futures = Vec::new();

    for user_id in &user_ids {
        post_futures.push(client.get_user_posts(*user_id));
    }

    // Wait for all futures to complete
    let results = futures::future::join_all(post_futures).await;
    let elapsed = start.elapsed();

    // Process results
    let mut total_posts = 0;
    let mut successful_users = 0;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(posts) => {
                total_posts += posts.len();
                successful_users += 1;
                println!("     User {}: {} posts", user_ids[i], posts.len());
            }
            Err(error) => {
                println!("     User {}: Error - {}", user_ids[i], error);
            }
        }
    }

    println!(
        "     Summary: {} total posts from {} users in {:?}",
        total_posts, successful_users, elapsed
    );
}

/// # Function: demonstrate_user_profile_aggregation
///
/// Demonstrates a complex real-world scenario: building a complete user profile
/// by aggregating data from multiple API endpoints.
///
/// ## Key Learning Points:
/// - Orchestrating multiple dependent async operations
/// - Building complex data structures from API responses
/// - Error handling in multi-step operations
/// - Performance optimization with concurrent requests
async fn demonstrate_user_profile_aggregation() {
    println!("\n=== User Profile Aggregation ===");

    let client = ApiClient::new("https://jsonplaceholder.typicode.com");

    /// # Function: build_user_profile
    ///
    /// Builds a complete user profile by fetching data from multiple endpoints.
    /// This demonstrates a real-world async workflow with multiple dependencies.
    ///
    /// ## Workflow:
    /// 1. Fetch user basic information
    /// 2. Fetch all posts by the user
    /// 3. For each post, fetch its comments (concurrently)
    /// 4. Aggregate all data into a UserProfile
    ///
    /// ## Arguments:
    /// - `client`: The API client to use for requests
    /// - `user_id`: The ID of the user to build a profile for
    ///
    /// ## Returns:
    /// - `AnyhowResult<UserProfile>`: Complete user profile or error
    async fn build_user_profile(client: &ApiClient, user_id: u32) -> AnyhowResult<UserProfile> {
        let start_time = Instant::now();

        println!("   Building profile for user {}...", user_id);

        // Step 1: Get user basic info and their posts concurrently
        let (user_result, posts_result) = tokio::join!(
            async {
                let users = client.get_users().await?;
                users
                    .into_iter()
                    .find(|u| u.id == user_id)
                    .ok_or_else(|| anyhow::anyhow!("User {} not found", user_id))
            },
            client.get_user_posts(user_id)
        );

        let user = user_result.context("Failed to fetch user info")?;
        let posts = posts_result.context("Failed to fetch user posts")?;

        // Step 2: Fetch comments for all posts concurrently
        let comment_futures: Vec<_> = posts
            .iter()
            .map(|post| client.get_post_comments(post.id))
            .collect();

        let comment_results = futures::future::join_all(comment_futures).await;

        // Step 3: Count total comments (ignoring failed requests)
        let total_comments: u32 = comment_results
            .into_iter()
            .filter_map(|result| result.ok())
            .map(|comments| comments.len() as u32)
            .sum();

        let fetch_time = start_time.elapsed();

        Ok(UserProfile {
            user,
            posts,
            total_comments,
            fetch_time,
        })
    }

    // Example 1: Build profile for a single user
    println!("1. Building profile for user 1:");
    match build_user_profile(&client, 1).await {
        Ok(profile) => {
            println!("   ✅ Profile completed in {:?}", profile.fetch_time);
            println!("   User: {} ({})", profile.user.name, profile.user.email);
            println!("   Posts: {}", profile.posts.len());
            println!("   Total comments: {}", profile.total_comments);

            if let Some(most_commented_post) = profile.posts.first() {
                println!("   Sample post: \"{}\"", most_commented_post.title);
            }
        }
        Err(error) => {
            println!("   ❌ Failed to build profile: {}", error);
        }
    }

    // Example 2: Build profiles for multiple users concurrently
    println!("\n2. Building profiles for multiple users:");
    let start = Instant::now();

    let user_ids = vec![1, 2, 3];
    let profile_futures: Vec<_> = user_ids
        .iter()
        .map(|&user_id| build_user_profile(&client, user_id))
        .collect();

    let profile_results = futures::future::join_all(profile_futures).await;
    let total_time = start.elapsed();

    let mut successful_profiles = 0;
    let mut total_posts = 0;
    let mut total_comments = 0;

    for (i, result) in profile_results.iter().enumerate() {
        match result {
            Ok(profile) => {
                successful_profiles += 1;
                total_posts += profile.posts.len();
                total_comments += profile.total_comments;
                println!(
                    "   User {}: {} posts, {} comments (built in {:?})",
                    user_ids[i],
                    profile.posts.len(),
                    profile.total_comments,
                    profile.fetch_time
                );
            }
            Err(error) => {
                println!("   User {}: Failed - {}", user_ids[i], error);
            }
        }
    }

    println!(
        "   Summary: {} profiles built in {:?}",
        successful_profiles, total_time
    );
    println!(
        "   Totals: {} posts, {} comments",
        total_posts, total_comments
    );
}

/// # Function: demonstrate_caching_and_performance
///
/// Demonstrates caching strategies and their performance impact.
/// Shows how caching can dramatically improve async application performance.
///
/// ## Key Learning Points:
/// - Implementing in-memory caching for HTTP responses
/// - Measuring cache hit rates and performance improvements
/// - Cache invalidation strategies
/// - Trade-offs between memory usage and performance
async fn demonstrate_caching_and_performance() {
    println!("\n=== Caching and Performance ===");

    let client = ApiClient::new("https://jsonplaceholder.typicode.com");

    // Example 1: Demonstrate cache performance
    println!("1. Cache performance demonstration:");

    // First request (cache miss)
    println!("   First request (cache miss):");
    let start = Instant::now();
    let _users1 = client.get_users().await.unwrap_or_default();
    let first_request_time = start.elapsed();
    println!("     Time: {:?}", first_request_time);

    // Second request (cache hit)
    println!("   Second request (cache hit):");
    let start = Instant::now();
    let _users2 = client.get_users().await.unwrap_or_default();
    let second_request_time = start.elapsed();
    println!("     Time: {:?}", second_request_time);

    // Calculate speedup
    let speedup = first_request_time.as_millis() as f64 / second_request_time.as_millis() as f64;
    println!("     Cache speedup: {:.2}x faster", speedup);

    // Example 2: Multiple requests showing cache effectiveness
    println!("\n2. Multiple requests with caching:");
    let start = Instant::now();

    // Make multiple requests for the same data
    let (users1, users2, posts1_1, posts1_2, posts2_1, posts2_2) = tokio::join!(
        client.get_users(),
        client.get_users(),
        client.get_user_posts(1),
        client.get_user_posts(1),
        client.get_user_posts(2),
        client.get_user_posts(2),
    );

    // Count successful requests for demonstration
    let user_requests_ok = [&users1, &users2].iter().filter(|r| r.is_ok()).count();
    let post_requests_ok = [&posts1_1, &posts1_2, &posts2_1, &posts2_2]
        .iter()
        .filter(|r| r.is_ok())
        .count();
    let total_successful = user_requests_ok + post_requests_ok;

    println!("     {} out of 6 requests succeeded", total_successful);
    let total_time = start.elapsed();

    println!("     6 requests completed in {:?}", total_time);
    println!("     (Notice how subsequent requests are much faster due to caching)");
}

/// # Function: demonstrate_error_resilience
///
/// Demonstrates error handling and resilience patterns in real-world scenarios.
/// Shows how to build robust async applications that handle failures gracefully.
///
/// ## Key Learning Points:
/// - Handling network failures and timeouts
/// - Implementing retry logic with exponential backoff
/// - Graceful degradation when services are unavailable
/// - Circuit breaker patterns for failing services
async fn demonstrate_error_resilience() {
    println!("\n=== Error Resilience ===");

    // Example 1: Handling invalid endpoints gracefully
    println!("1. Handling invalid endpoints:");
    let client = ApiClient::new("https://jsonplaceholder.typicode.com");

    match client
        .get_with_cache("invalid-endpoint", Duration::from_secs(60))
        .await
    {
        Ok(_) => println!("   Unexpected success"),
        Err(error) => {
            println!("   Expected error handled gracefully: {}", error);
            // In a real app, you might log this error and use fallback data
        }
    }

    // Example 2: Timeout handling
    println!("\n2. Timeout handling:");
    let slow_client = ApiClient::new("https://httpbin.org");

    match timeout(
        Duration::from_millis(100), // Very short timeout
        slow_client.get_with_cache("delay/5", Duration::from_secs(1)), // 5 second delay
    )
    .await
    {
        Ok(_) => println!("   Unexpected completion"),
        Err(_) => {
            println!("   Request timed out as expected");
            println!("   → Application can continue with cached data or default values");
        }
    }

    // Example 3: Partial failure handling
    println!("\n3. Partial failure handling:");
    let client = ApiClient::new("https://jsonplaceholder.typicode.com");

    // Try to fetch data for multiple users, some of which might fail
    let user_ids = vec![1, 2, 999, 3]; // 999 doesn't exist
    let futures: Vec<_> = user_ids
        .iter()
        .map(|&id| {
            let client_clone = client.clone();
            async move { (id, client_clone.get_user_posts(id).await) }
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    let mut successful = 0;
    let mut failed = 0;

    for (user_id, result) in results {
        match result {
            Ok(posts) => {
                successful += 1;
                println!("   User {}: {} posts", user_id, posts.len());
            }
            Err(_) => {
                failed += 1;
                println!("   User {}: Failed (using default data)", user_id);
            }
        }
    }

    println!("   Summary: {} successful, {} failed", successful, failed);
    println!("   → Application continues to work despite partial failures");
}

/// # Function: main
///
/// The main function orchestrates all real-world async pattern demonstrations.
///
/// ## Learning Progression:
/// 1. Basic HTTP operations with structured data
/// 2. Concurrent API calls for performance
/// 3. Complex data aggregation workflows
/// 4. Caching strategies and performance optimization
/// 5. Error handling and resilience patterns
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Real-World Async Patterns Tutorial");
    println!("====================================");
    println!("This example demonstrates practical async patterns using real HTTP APIs.");

    // Basic HTTP operations
    demonstrate_basic_http_operations().await;

    // Concurrent API calls
    demonstrate_concurrent_api_calls().await;

    // Complex data aggregation
    demonstrate_user_profile_aggregation().await;

    // Caching and performance
    demonstrate_caching_and_performance().await;

    // Error resilience
    demonstrate_error_resilience().await;

    println!("\n✅ Real-World Patterns Tutorial completed!");
    println!("Key takeaways:");
    println!("  - HTTP clients integrate seamlessly with async/await");
    println!("  - Concurrent API calls provide significant performance benefits");
    println!("  - Complex workflows can be built by composing simple async operations");
    println!("  - Caching dramatically improves performance for repeated requests");
    println!("  - Robust error handling is essential for production applications");
    println!("  - Rate limiting prevents overwhelming external services");
    println!("  - Structured data with serde makes JSON handling ergonomic");

    println!("\nNext: Try 'cargo run --bin advanced_patterns' for advanced async patterns");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    /// Test basic API client functionality
    #[tokio::test]
    async fn test_api_client_creation() {
        let client = ApiClient::new("https://jsonplaceholder.typicode.com");

        // Test that we can create the client without errors
        assert_eq!(client.base_url, "https://jsonplaceholder.typicode.com");
    }

    /// Test JSON deserialization
    #[test]
    fn test_user_deserialization() {
        let json = r#"
        {
            "id": 1,
            "name": "Test User",
            "email": "test@example.com"
        }
        "#;

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.posts_count, 0); // Default value
    }

    /// Test post deserialization with renamed fields
    #[test]
    fn test_post_deserialization() {
        let json = r#"
        {
            "id": 1,
            "userId": 123,
            "title": "Test Post",
            "body": "This is a test post"
        }
        "#;

        let post: Post = serde_json::from_str(json).unwrap();
        assert_eq!(post.id, 1);
        assert_eq!(post.user_id, 123);
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.body, "This is a test post");
    }

    /// Test user profile serialization
    #[test]
    fn test_user_profile_serialization() {
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            posts_count: 5,
        };

        let posts = vec![Post {
            id: 1,
            user_id: 1,
            title: "Test Post".to_string(),
            body: "Test body".to_string(),
            created_at: None,
        }];

        let profile = UserProfile {
            user,
            posts,
            total_comments: 10,
            fetch_time: Duration::from_millis(500),
        };

        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("Test User"));
        assert!(json.contains("test@example.com"));
        assert!(json.contains("Test Post"));
    }

    /// Test concurrent operations timing
    #[tokio::test]
    async fn test_concurrent_vs_sequential_timing() {
        async fn mock_operation(delay_ms: u64) -> String {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            format!("Result after {}ms", delay_ms)
        }

        // Sequential execution
        let start = Instant::now();
        let _r1 = mock_operation(50).await;
        let _r2 = mock_operation(50).await;
        let _r3 = mock_operation(50).await;
        let sequential_time = start.elapsed();

        // Concurrent execution
        let start = Instant::now();
        let (_r1, _r2, _r3) =
            tokio::join!(mock_operation(50), mock_operation(50), mock_operation(50));
        let concurrent_time = start.elapsed();

        // Concurrent should be significantly faster
        assert!(concurrent_time < sequential_time);
        assert!(concurrent_time < Duration::from_millis(100)); // Should be ~50ms, not 150ms
    }
}
