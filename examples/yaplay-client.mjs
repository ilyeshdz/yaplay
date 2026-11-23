const allActions = ['set_mute', 'set_deafen', 'leave_call'];
const validLabel = value => typeof value === 'string' && value.trim().length > 0 &&
  new TextEncoder().encode(value).length <= 128 && !/[\u0000-\u001f\u007f-\u009f]/u.test(value);

/** Small transport wrapper. Inject `ws` in Node or use a host's WebSocket. */
export class YaplayClient {
  constructor({ port = 8080, WebSocketImpl = globalThis.WebSocket, retryMs = 1500, onStatus = () => {}, onControl = null, onControlError = () => {}, name, clientVersion, actions = onControl ? allActions : [] } = {}) {
    if (!Number.isInteger(port) || port < 1 || port > 65535) throw new RangeError('Invalid port');
    if (!WebSocketImpl) throw new Error('Provide a WebSocket implementation');
    this.url = `ws://127.0.0.1:${port}`;
    this.WebSocketImpl = WebSocketImpl;
    this.retryMs = retryMs;
    if (onControl !== null && typeof onControl !== 'function') throw new TypeError('onControl must be a function');
    if (name !== undefined && !validLabel(name)) throw new TypeError('Invalid integration name');
    if (clientVersion !== undefined && (!name || !validLabel(clientVersion))) throw new TypeError('Invalid clientVersion');
    if (!Array.isArray(actions) || actions.some(a => !onControl || !allActions.includes(a))) throw new TypeError('Invalid actions');
    this.actions = new Set(actions);
    if (!name && onControl && this.actions.size !== allActions.length) throw new TypeError('Partial controls require an integration name');
    this.name = name;
    this.clientVersion = clientVersion;
    this.onStatus = onStatus;
    this.onControl = onControl;
    this.onControlError = onControlError;
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
    let accepted = new Set();
    let awaitingAck = false;
    let fallbackTimer;
    socket.addEventListener('open', () => {
      if (this.socket !== socket || this.stopped) return;
      const enableV1 = () => {
        accepted = new Set(allActions);
        this.send({ type: 'hello', version: 1, controls: true });
      };
      if (this.name) {
        awaitingAck = true;
        this.send({ type: 'hello', version: 2, name: this.name, client_version: this.clientVersion, actions: [...this.actions] });
        if (this.actions.size === allActions.length) {
          fallbackTimer = setTimeout(() => {
            if (!awaitingAck || this.socket !== socket || socket.readyState !== 1) return;
            awaitingAck = false;
            enableV1();
            this.replay();
          }, 1500);
        }
      } else if (this.onControl) enableV1();
      this.replay();
      this.onStatus('connected');
    });
    let pending = Promise.resolve();
    socket.addEventListener('message', (event) => {
      if (this.socket !== socket || this.stopped) return;
      let action;
      try { action = JSON.parse(String(event.data)); } catch { return; }
      if (!action || typeof action !== 'object') return;
      if (action.type === 'hello_ack' && awaitingAck && action.version === 2 &&
          Array.isArray(action.actions) && action.actions.every(a => typeof a === 'string')) {
        awaitingAck = false;
        clearTimeout(fallbackTimer);
        accepted = new Set(action.actions.filter(a => this.actions.has(a)));
        return;
      }
      if (!accepted.has(action.type)) return;
      const valid = (action.type === 'set_mute' && typeof action.muted === 'boolean') ||
        (action.type === 'set_deafen' && typeof action.deafened === 'boolean') || action.type === 'leave_call';
      if (!valid) return;
      // Keep host actions in order; discard requests from an old connection.
      pending = pending.then(() => {
        if (this.socket === socket && socket.readyState === 1 && !this.stopped) return this.onControl(action);
      }).catch((error) => { try { this.onControlError(error); } catch { /* Host error callbacks must not break delivery. */ } });
    });
    socket.addEventListener('error', () => {
      // Closing also covers hosts that do not automatically close on error.
      socket.close();
    });
    socket.addEventListener('close', () => {
      clearTimeout(fallbackTimer);
      if (this.socket !== socket) return;
      this.socket = null;
      this.retry();
    });
  }
  replay() {
    for (const type of ['voice_state', 'mic_state', 'deafen_state']) {
      if (this.state.has(type)) this.send(this.state.get(type));
    }
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
