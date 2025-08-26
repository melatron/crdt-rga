# RGA CRDT Web Server

This module contains the Axum web server implementation that provides HTTP endpoints for interacting with the RGA CRDT.

## Architecture

The server module is organized as follows:

- `mod.rs` - Main server module with re-exports
- `routes.rs` - HTTP route handlers and response types

## Available Endpoints

### GET /
Returns a simple hello message.

**Response:**
```json
{
  "message": "Hello from Axum server!"
}
```

### GET /health
Health check endpoint for monitoring and load balancers.

**Response:**
```json
{
  "status": "ok",
  "message": "Server is running!"
}
```

### POST /messages
Creates a new message (example endpoint).

**Request:**
```json
{
  "content": "Your message here"
}
```

**Response:**
```json
{
  "id": 1,
  "content": "Your message here",
  "timestamp": "2023-12-07T10:30:00Z"
}
```

## Running the Server

From the project root:

```bash
cargo run
```

The server will start on `http://localhost:3000`.

## Testing the Endpoints

```bash
# Test hello endpoint
curl http://localhost:3000/

# Test health check
curl http://localhost:3000/health

# Test message creation
curl -X POST http://localhost:3000/messages \
  -H 'Content-Type: application/json' \
  -d '{"content":"Hello World"}'
```

## Future Extensions

This server module can be extended to include:

- RGA CRDT collaboration endpoints
- WebSocket support for real-time updates
- Authentication and authorization
- Database persistence
- Metrics and monitoring endpoints