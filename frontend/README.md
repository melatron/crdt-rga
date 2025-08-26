# RGA CRDT Frontend

This directory contains the frontend client applications for the RGA CRDT collaborative text editor.

## Files

- `index.html` - Main collaborative editor web client

## Getting Started

### Prerequisites

- RGA CRDT server running on `http://localhost:3000`
- Modern web browser with WebSocket support

### Usage

1. **Start the Backend Server**
   ```bash
   cd .. # Go to project root
   cargo run
   ```

2. **Open the Client**
   ```bash
   # Open in your browser
   open frontend/index.html
   # Or simply double-click index.html
   ```

3. **Connect and Edit**
   - Click "Connect" to establish WebSocket connection
   - Enter single characters in the input field
   - Click "Insert Character" or press Enter
   - Watch the document update in real-time

## Features

### Current Features ✅

- **Real-time Connection** - WebSocket connection to RGA CRDT server
- **Character Insertion** - Insert individual characters into shared document
- **Live Document View** - See document updates immediately
- **Connection Status** - Visual feedback for connection state
- **Message Logging** - Debug log of all WebSocket messages

### Planned Features 🔄

- **Multi-user Collaboration** - Multiple clients editing simultaneously
- **Cursor Positioning** - Insert characters at specific positions
- **Text Deletion** - Delete characters from document
- **User Identification** - Show which user made which changes
- **Conflict Resolution** - Visual feedback for CRDT conflict resolution

## Architecture

### WebSocket Protocol

The client communicates with the server using JSON messages over WebSocket:

**Insert Character:**
```json
{
  "type": "insert",
  "character": "a",
  "after_id": "start"
}
```

**Get Content:**
```json
{
  "type": "get_content"
}
```

**Server Responses:**
```json
{
  "type": "init",
  "content": "hello"
}
```

```json
{
  "type": "update", 
  "content": "hello world"
}
```

### Client Structure

- **HTML Structure** - Clean, semantic markup
- **CSS Styling** - Responsive design with status indicators
- **JavaScript Logic** - WebSocket handling and UI updates
- **No Dependencies** - Pure vanilla JavaScript, no frameworks

## Development

### Adding New Features

1. **UI Changes** - Modify HTML structure and CSS styling
2. **Protocol Changes** - Update JavaScript to handle new message types
3. **Server Integration** - Ensure backend supports new operations

### Testing

- Test with multiple browser tabs to simulate multiple users
- Check WebSocket connection handling (connect/disconnect/reconnect)
- Verify message parsing and error handling
- Test with various character inputs (Unicode, symbols, etc.)

## Browser Compatibility

- **Chrome/Edge** - Full support
- **Firefox** - Full support  
- **Safari** - Full support
- **Mobile Browsers** - Basic support (touch-friendly)

## Troubleshooting

### Connection Issues

- **Cannot Connect** - Ensure backend server is running on port 3000
- **Connection Drops** - Check browser console for WebSocket errors
- **Messages Not Sending** - Verify JSON format in browser developer tools

### Common Problems

1. **CORS Issues** - Server runs on localhost, should work locally
2. **Port Conflicts** - Default server port is 3000, check if occupied
3. **Browser Cache** - Hard refresh (Ctrl+F5) to clear cached files

## Future Enhancements

- **Rich Text Editor** - Support for formatting, styles
- **File Operations** - Save/load documents
- **User Authentication** - Login system for multi-user sessions
- **Operational Transform UI** - Visual representation of CRDT operations
- **Performance Monitoring** - Metrics for operation latency and conflicts