# Summary

[Introduction](./introduction.md)

# Part I: Fundamentals

- [Chapter 1: Understanding Async Programming](./chapter-01-async-fundamentals.md)
  - [Why Async?](./chapter-01-async-fundamentals.md#why-async)
  - [Key Concepts](./chapter-01-async-fundamentals.md#key-concepts)
  - [Async vs Sync](./chapter-01-async-fundamentals.md#async-vs-sync)

- [Chapter 2: The Future Trait](./chapter-02-future-trait.md)
  - [Core Components](./chapter-02-future-trait.md#core-components)
  - [Poll States](./chapter-02-future-trait.md#poll-states)
  - [Waker and Context](./chapter-02-future-trait.md#waker-and-context)

- [Chapter 3: Basic async/await](./chapter-03-basic-async-await.md)
  - [Your First Async Function](./chapter-03-basic-async-await.md#first-async-function)
  - [Sequential vs Concurrent](./chapter-03-basic-async-await.md#sequential-vs-concurrent)
  - [Future Laziness](./chapter-03-basic-async-await.md#future-laziness)

# Part II: Implementation

- [Chapter 4: Custom Future Implementation](./chapter-04-custom-futures.md)
  - [Delay Future](./chapter-04-custom-futures.md#delay-future)
  - [Shared State](./chapter-04-custom-futures.md#shared-state)
  - [Thread Safety](./chapter-04-custom-futures.md#thread-safety)

- [Chapter 5: Future Combinators](./chapter-05-combinators.md)
  - [Map and AndThen](./chapter-05-combinators.md#map-and-then)
  - [Join and Select](./chapter-05-combinators.md#join-and-select)
  - [Collection Combinators](./chapter-05-combinators.md#collection-combinators)

- [Chapter 6: Error Handling](./chapter-06-error-handling.md)
  - [Result in Async](./chapter-06-error-handling.md#result-in-async)
  - [Custom Error Types](./chapter-06-error-handling.md#custom-error-types)
  - [Error Propagation](./chapter-06-error-handling.md#error-propagation)

# Part III: Real-World Applications

- [Chapter 7: HTTP Client](./chapter-07-http-client.md)
  - [Async HTTP Requests](./chapter-07-http-client.md#async-http-requests)
  - [JSON Processing](./chapter-07-http-client.md#json-processing)
  - [Caching and Rate Limiting](./chapter-07-http-client.md#caching-and-rate-limiting)

- [Chapter 8: Advanced Patterns](./chapter-08-advanced-patterns.md)
  - [Stream Processing](./chapter-08-advanced-patterns.md#stream-processing)
  - [Backpressure](./chapter-08-advanced-patterns.md#backpressure)
  - [Cancellation](./chapter-08-advanced-patterns.md#cancellation)

# Part IV: Best Practices

- [Chapter 9: Testing Async Code](./chapter-09-testing.md)
  - [Unit Testing](./chapter-09-testing.md#unit-testing)
  - [Integration Testing](./chapter-09-testing.md#integration-testing)
  - [Mocking](./chapter-09-testing.md#mocking)

- [Chapter 10: Performance](./chapter-10-performance.md)
  - [Benchmarking](./chapter-10-performance.md#benchmarking)
  - [Memory Usage](./chapter-10-performance.md#memory-usage)
  - [Runtime Selection](./chapter-10-performance.md#runtime-selection)

- [Chapter 11: Common Pitfalls](./chapter-11-pitfalls.md)
  - [Await Points](./chapter-11-pitfalls.md#await-points)
  - [Blocking Code](./chapter-11-pitfalls.md#blocking-code)
  - [Send + Sync](./chapter-11-pitfalls.md#send-sync)

# Appendices

- [Appendix A: Code Examples](./appendix-a-examples.md)
- [Appendix B: API Reference](./appendix-b-api.md)
- [Appendix C: Resources](./appendix-c-resources.md) 