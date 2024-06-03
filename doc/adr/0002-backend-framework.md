# 2. Backend Framework: Actix-web

Date: 2024-04-29

## Status

Proposed

## Context

The backend system for the project requires robust handling of movie file uploads and subsequent processing, which includes splitting movies into frames and creating a "long exposure" style images. The chosen backend framework needs to provide reliable performance and easy integration with external utilities like FFmpeg. As Rust currently is proposed as the programming language I need to decide on a suitable web framework within the Rust ecosystem. The current decision might be superseded in the future as it is not 100% clear yet if Rust will remain the Programming language of choice for this project.

## Considered Options

* Actix-web
* Rocket
* Warp
* Axum

## Decision

I have chosen Actix-web as the backend framework for the following reasons:

- Performance and efficiency in handling concurrent requests.
- Comprehensive set of features including WebSockets, HTTP/2 support, and middleware.
- Support for async/await syntax, aligning with Rust’s concurrency model.
- A growing community and a range of plugins and extensions.

## Consequences

### Actix-web

**Pros:**

- High performance in handling many concurrent connections.
- Allows an easy switch from previously known "Express" Framework.
- Robust middleware support and other essential web application features.
- Compile-time guarantees through Rust’s type system.
- A substantial set of community-contributed libraries and plugins.

**Cons:**

- The comprehensive feature set can introduce complexity.
- Active development can lead to breaking changes and the need for frequent updates.

### Rocket

**Pros:**

- Ergonomic API for ease of use and readability.
- Compile-time guarantees through Rust’s type system.
- Good documentation and active community support.

**Cons:**

- Historically affected by stability issues due to reliance on nightly Rust features.
- Performance is good but not as high as Actix-web in high-concurrency scenarios.
- Did not work with it before

### Warp

**Pros:**

- Composability through a filter-based system.
- Inherently asynchronous design leveraging Rust’s async/await capabilities.

**Cons:**

- The filter system can be complex and unintuitive.
- Smaller community and fewer plugins compared to Actix-web.
- Did not work with it before
- Unfamiliar way of defining routes which is inconvenient imho.


### Axum

**Pros:**

- Seamless integration with Tokio for efficient I/O operations.
- Modular design for flexible and scalable architecture.

**Cons:**

- Relatively new framework, potentially lacking maturity and extensive community support.
- Ecosystem is not as extensive as Actix-web’s.

## Conclusion

Actix-web is chosen as the backend framework due to its performance, feature set, and alignment with Rust’s strengths in concurrency and memory safety. Despite its complexity, the benefits it offers in handling our specific requirements make it the most suitable choice for this project so far.
