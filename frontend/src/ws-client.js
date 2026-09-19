export class WebSocketClient {
  constructor(url, callbacks = {}) {
    this.url = url;
    this.callbacks = callbacks;
    this.ws = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 10;
    this.reconnectDelay = 1000;
    this.messageQueue = [];
    this.channelCallbacks = new Map();
  }

  connect() {
    try {
      this.ws = new WebSocket(this.url);
      this.ws.binaryType = "arraybuffer";

      this.ws.onopen = (e) => this.onOpen(e);
      this.ws.onmessage = (e) => this.onMessage(e);
      this.ws.onclose = (e) => this.onClose(e);
      this.ws.onerror = (e) => this.onError(e);
    } catch (err) {
      this.scheduleReconnect();
    }
  }

  onOpen(e) {
    this.reconnectAttempts = 0;
    this.reconnectDelay = 1000;
    this.flushQueue();
    if (this.callbacks.onOpen) this.callbacks.onOpen(e);
  }

  onMessage(e) {
    try {
      const msg = JSON.parse(e.data);
      if (this.callbacks.onMessage) this.callbacks.onMessage(msg);
    } catch (err) {
      console.error("Failed to parse message:", err);
    }
  }

  onClose(e) {
    if (this.callbacks.onClose) this.callbacks.onClose(e);
    if (!e.wasClean) {
      this.scheduleReconnect();
    }
  }

  onError(e) {
    if (this.callbacks.onError) this.callbacks.onError(e);
  }

  scheduleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error("Max reconnect attempts reached");
      return;
    }

    this.reconnectAttempts++;
    const delay = Math.min(
      this.reconnectDelay * Math.pow(1.5, this.reconnectAttempts - 1),
      30000,
    );

    setTimeout(() => {
      this.connect();
    }, delay);
  }

  send(msg) {
    const data = JSON.stringify(msg);
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(data);
    } else {
      this.messageQueue.push(data);
    }
  }

  flushQueue() {
    while (
      this.messageQueue.length > 0 &&
      this.ws &&
      this.ws.readyState === WebSocket.OPEN
    ) {
      this.ws.send(this.messageQueue.shift());
    }
  }

  isConnected() {
    return this.ws && this.ws.readyState === WebSocket.OPEN;
  }

  close() {
    if (this.ws) {
      this.ws.close(1000, "Client disconnect");
      this.ws = null;
    }
  }

  on(channelId, callback) {
    this.channelCallbacks.set(channelId, callback);
  }

  off(channelId) {
    this.channelCallbacks.delete(channelId);
  }
}
