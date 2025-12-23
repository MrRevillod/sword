import io from "socket.io-client";

const socket = io("http://localhost:8081/test")

socket.on("connect", () => {
  console.log("Connected to server");
  socket.emit("ping");
});

socket.on("pong", (data: any) => {
    console.log("Received from server:", data);
});

socket.on("connect_error", (err: any) => {
  console.error("Connection error:", err);
});
