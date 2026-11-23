import { test } from 'node:test';
import assert from 'node:assert/strict';
import { YaplayClient } from './yaplay-client.mjs';

class Socket extends EventTarget {
  static instances = [];
  readyState = 0;
  messages = [];
  constructor(url) { super(); this.url = url; Socket.instances.push(this); }
  open() { this.readyState = 1; this.dispatchEvent(new Event('open')); }
  send(data) { this.messages.push(JSON.parse(data)); }
  close() { this.readyState = 3; this.dispatchEvent(new Event('close')); }
}

test('replays the latest snapshot in protocol order, not notifications', (t) => {
  const client = new YaplayClient({ WebSocketImpl: Socket });
  t.after(() => client.disconnect());
  client.setMic(true);
  const members = [{ id: 'self', name: 'You' }];
  client.setRoster(members);
  members[0].name = 'Changed externally';
  assert.equal(client.notify('Old', 'Do not replay'), false);
  client.connect();
  const socket = Socket.instances.at(-1);
  client.connect();
  assert.equal(Socket.instances.at(-1), socket);
  socket.open();
  assert.deepEqual(socket.messages.map(m => m.type), ['voice_state', 'mic_state']);
  assert.equal(socket.messages[0].members[0].name, 'You');
  assert.equal(client.notify('New', 'Live'), true);
  assert.equal(socket.messages.at(-1).type, 'notification');
});

test('reconnects with updated state and cancels retries on unload', (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const client = new YaplayClient({ WebSocketImpl: Socket, retryMs: 10 });
  t.after(() => client.disconnect());
  client.connect();
  const first = Socket.instances.at(-1);
  first.open();
  client.setMic(true);
  client.setDeafen(true);
  first.close();
  client.setRoster([], false);
  t.mock.timers.tick(10);
  const second = Socket.instances.at(-1);
  assert.notEqual(first, second);
  second.open();
  assert.deepEqual(second.messages, [{ type: 'voice_state', in_call: false, members: [] }]);
  second.close();
  client.disconnect();
  t.mock.timers.tick(100);
  assert.equal(Socket.instances.at(-1), second);
});

test('validates configuration and retries failed construction', (t) => {
  assert.throws(() => new YaplayClient({ port: 0 }), RangeError);
  t.mock.timers.enable({ apis: ['setTimeout'] });
  let attempts = 0;
  class Broken { constructor() { attempts++; throw new Error('unavailable'); } }
  const client = new YaplayClient({ WebSocketImpl: Broken, retryMs: 10 });
  client.connect();
  t.mock.timers.tick(10);
  assert.equal(attempts, 2);
  client.disconnect();
  t.mock.timers.tick(20);
  assert.equal(attempts, 2);
});

const receive = (socket, data) => socket.dispatchEvent(new MessageEvent('message', { data }));
const flush = () => new Promise(resolve => setImmediate(resolve));

test('negotiates controls before replay and delivers validated host actions in order', async (t) => {
  const actions = [];
  const errors = [];
  const client = new YaplayClient({ WebSocketImpl: Socket, onControl: async action => {
    actions.push(action);
    if (action.type === 'set_deafen') throw new Error('Host rejected');
  }, onControlError: error => errors.push(error.message) });
  t.after(() => client.disconnect());
  client.setRoster([{ id: '1', avatar_url: 'https://example.com/avatar.png' }]);
  client.connect();
  const socket = Socket.instances.at(-1);
  socket.open();
  assert.deepEqual(socket.messages.map(m => m.type), ['hello', 'voice_state']);
  assert.equal(socket.messages[1].members[0].avatar_url, 'https://example.com/avatar.png');
  for (const data of ['bad', 'null', '{"type":"set_mute","muted":"yes"}', '{"type":"unknown"}',
    '{"type":"set_mute","muted":true}', '{"type":"set_deafen","deafened":true}', '{"type":"leave_call"}']) receive(socket, data);
  await flush();
  assert.deepEqual(actions.map(a => a.type), ['set_mute', 'set_deafen', 'leave_call']);
  assert.deepEqual(errors, ['Host rejected']);
  assert.equal(socket.messages.length, 2); // Host confirmation must be explicit.
});

test('drops stale queued controls and negotiates again after reconnect', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const actions = [];
  const client = new YaplayClient({ WebSocketImpl: Socket, retryMs: 10, onControl: action => actions.push(action) });
  t.after(() => client.disconnect());
  client.connect();
  const old = Socket.instances.at(-1);
  old.open();
  receive(old, '{"type":"leave_call"}');
  old.close();
  t.mock.timers.tick(10);
  const current = Socket.instances.at(-1);
  current.open();
  receive(old, '{"type":"leave_call"}');
  await flush();
  assert.deepEqual(actions, []);
  assert.deepEqual(current.messages, [{ type: 'hello', version: 1, controls: true }]);
  receive(current, '{"type":"set_mute","muted":true}');
  await flush();
  assert.equal(actions.length, 1);
});

test('named clients accept only acknowledged actions for this socket', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const actions = [];
  const client = new YaplayClient({ WebSocketImpl: Socket, name: 'Demo', clientVersion: '1.0',
    actions: ['set_mute', 'leave_call'], onControl: action => actions.push(action.type) });
  t.after(() => client.disconnect());
  client.connect();
  const socket = Socket.instances.at(-1);
  socket.open();
  assert.deepEqual(socket.messages, [{ type: 'hello', version: 2, name: 'Demo', client_version: '1.0', actions: ['set_mute', 'leave_call'] }]);
  const mute = '{"type":"set_mute","muted":true}';
  receive(socket, mute);
  t.mock.timers.tick(2000);
  assert.equal(socket.messages.length, 1);
  receive(socket, '{"type":"hello_ack","version":2,"actions":["set_mute","set_deafen"]}');
  receive(socket, mute);
  receive(socket, '{"type":"set_deafen","deafened":true}');
  receive(socket, '{"type":"leave_call"}');
  await flush();
  assert.deepEqual(actions, ['set_mute']);
});

test('full control clients fall back to v1 and replay the current snapshot only on timeout', (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const client = new YaplayClient({ WebSocketImpl: Socket, name: 'Demo', onControl: () => {} });
  t.after(() => client.disconnect());
  client.setRoster([]);
  client.connect();
  const first = Socket.instances.at(-1);
  first.open();
  t.mock.timers.tick(1500);
  assert.deepEqual(first.messages.map(m => m.version || m.type), [2, 'voice_state', 1, 'voice_state']);
  first.close();
  t.mock.timers.tick(1500);
  const second = Socket.instances.at(-1);
  second.open();
  receive(second, '{"type":"hello_ack","version":2,"actions":[]}');
  t.mock.timers.tick(2000);
  assert.deepEqual(second.messages.map(m => m.version || m.type), [2, 'voice_state']);
});
