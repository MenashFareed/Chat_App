# Rust Chat Application

A real-time chat application built with Rust using Warp for the web server and WebSocket communication.

## Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

## Dependencies

- tokio - Asynchronous runtime
- warp - Web server framework
- serde - Serialization/deserialization
- futures - Async utilities
- uuid - Unique identifier generation

## Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd rust_chat_app
   ```

2. Update your `Cargo.toml` to fix the futures dependency:
   ```toml
   futures = { version = "0.3", default-features = true }
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run the server:
   ```bash
   cargo run
   ```
   
## Features

- Real-time chat messaging
- WebSocket-based communication
- Unique client identification using UUIDs
