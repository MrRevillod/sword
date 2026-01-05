<script lang="ts">
  import { io, type Socket } from "socket.io-client";
  import msgpackParser from "socket.io-msgpack-parser";

  type Message = {
    id: string;
    content: string;
    timestamp: number;
  };

  let inputMessage = $state<string>("");

  let socket = $state<Socket>(
    io("http://localhost:8081/chat", {
      parser: msgpackParser,
    })
  );
  let messages = $state<Message[]>([]);

  socket.on("connect", () => {
    console.log("Connected to server");
  });

  socket.on("messages", (receivedMessages: Message[]) => {
    console.log("Message received:", receivedMessages);
    messages = receivedMessages;
  });

  async function sendMessage() {
    if (inputMessage.trim() === "") return;

    socket.emit("message", { content: inputMessage });

    console.log("Message sent:", inputMessage);
  }
</script>

<main>
  <h1>Sword Socket.IO Adapter Chat</h1>

  <div>
    {#each messages as message}
      <div class="message">
        <strong>{new Date(message.timestamp).toLocaleTimeString()}:</strong>
        <span>{message.content}</span>
      </div>
    {/each}
  </div>

  <div>
    <input
      type="text"
      bind:value={inputMessage}
      placeholder="Type your message..."
    />

    <button onclick={sendMessage}>Send</button>
  </div>
</main>
