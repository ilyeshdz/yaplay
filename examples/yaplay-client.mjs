/** Small transport wrapper. Inject `ws` in Node or use a host's WebSocket. */
export class YaplayClient {
  constructor({ port = 8080, WebSocketImpl = globalThis.WebSocket, retryMs = 1500, onStatus = () => {} } = {}) {
    if (!Number.isInteger(port) || port < 1 || port > 65535) throw new RangeError('Invalid port');
    if (!WebSocketImpl) throw new Error('Provide a WebSocket implementation');
    this.url = `ws://127.0.0.1:${port}`;
    this.WebSocketImpl = WebSocketImpl;
    this.retryMs = retryMs;
    this.onStatus = onStatus;
    this.state = new Map();
    this.stopped = true;
    this.socket = null;
    this.timer = null;
  }
  connect() {
    if (!this.stopped) return;
    this.stopped = false;
    this.open();
  }
  open() {
    if (this.stopped) return;
    this.onStatus('connecting');
    let socket;
    try { socket = new this.WebSocketImpl(this.url); }
    catch { this.retry(); return; }
    this.socket = socket;
    socket.addEventListener('open', () => {
      if (this.socket !== socket || this.stopped) return;
      // Replay a snapshot, never old notifications, after reconnecting.
      for (const type of ['voice_state', 'mic_state', 'deafen_state']) {
        if (this.state.has(type)) this.send(this.state.get(type));
      }
      this.onStatus('connected');
    });
    socket.addEventListener('error', () => {
      // Closing also covers hosts that do not automatically close on error.
      socket.close();
    });
    socket.addEventListener('close', () => {
      if (this.socket !== socket) return;
      this.socket = null;
      this.retry();
    });
  }
  retry() {
    if (this.stopped) return;
    this.onStatus('disconnected');
    clearTimeout(this.timer);
    this.timer = setTimeout(() => this.open(), this.retryMs);
  }
  send(message) {
    if (this.socket?.readyState !== 1) return false;
    try { this.socket.send(JSON.stringify(message)); return true; }
    catch { this.socket.close(); return false; }
  }
  remember(message) {
    // Copy values so later host mutations cannot change the reconnect snapshot.
    const copy = JSON.parse(JSON.stringify(message));
    this.state.set(copy.type, copy);
    return this.send(copy);
  }
  setRoster(members, inCall = true) {
    if (!inCall) {
      this.state.delete('mic_state');
      this.state.delete('deafen_state');
    }
    return this.remember({ type: 'voice_state', in_call: inCall, members: inCall ? members : [] });
  }
  setMic(muted) { return this.remember({ type: 'mic_state', muted }); }
  setDeafen(deafened) { return this.remember({ type: 'deafen_state', deafened }); }
  notify(title, body) { return this.send({ type: 'notification', title, body }); }
  disconnect() {
    this.stopped = true;
    clearTimeout(this.timer);
    const socket = this.socket;
    this.socket = null;
    socket?.close();
    this.onStatus('stopped');
  }
}
