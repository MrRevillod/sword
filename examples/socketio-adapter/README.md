# socketio-adapter

This example demonstrates a real-time chat application using Sword's SocketIO adapter. Clients can connect and send messages that are broadcasted to all connected clients.

## Running the Example

1. Navigate to this directory: `cd examples/socketio-adapter`
2. Run the server: `cargo run`
3. In another terminal, navigate to the client directory: `cd examples/socketio-adapter/client`
4. Install dependencies: `npm install`
5. Run the client: `npm run dev`

Open http://localhost:5173 in your browser to interact with the chat application. The server runs on port 8081.