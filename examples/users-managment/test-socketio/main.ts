import io from "socket.io-client";

const socket = io("http://localhost:8081/socket")

socket.on("connect", async () => {
  console.log("Connected to server");

  socket.emit("message-try-data", { content:  "Hello, Server!" });


  let response = await socket.emitWithAck("message-with-ack", { title: "Test Title", count: 42 })

  console.log("Response from message-with-ack:", response);
});

socket.on("connect_error", (err: any) => {
  console.error("Connection error:", err);
});
