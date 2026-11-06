// The browser half of core's document engine: loads the vendored Loro wasm
// (URL and SRI from the vendor head), and configures every document with the
// same mark expansion as the server (`systemprompt_engine_loro::rich::TEXT_STYLES`).
// A client that expands marks differently diverges from the server's copy.
import init, { LoroDoc, LoroMap, LoroMovableList, LoroText, VersionVector, Cursor } from 'loro-crdt';
import { rawResponse } from '../../services/api.js';

export const BLOCKS = 'blocks';

const TEXT_STYLES = {
  bold: { expand: 'after' },
  italic: { expand: 'after' },
  underline: { expand: 'after' },
  strike: { expand: 'after' },
  code: { expand: 'none' },
  link: { expand: 'none' },
  color: { expand: 'after' },
  highlight: { expand: 'after' },
  comment: { expand: 'none' },
};

export const FLAG_MARKS = new Set(['bold', 'italic', 'underline', 'strike', 'code']);

let ready = null;

const wasmLink = () => {
  const node = document.getElementById('sp-vendor-wasm');
  const links = node ? JSON.parse(node.textContent) : {};
  const link = links['loro-crdt'];
  if (!link) throw new Error('The Loro engine is not in the vendor head.');
  return link;
};

export const loadEngine = () => {
  ready ??= (async () => {
    const { url, integrity } = wasmLink();
    const response = await rawResponse(url, { headers: {}, integrity });
    if (!response.ok) throw new Error(`Loro engine failed to load (${response.status}).`);
    await init({ module_or_path: response });
  })();
  return ready;
};

export const newDoc = (snapshot) => {
  const doc = new LoroDoc();
  doc.configTextStyle(TEXT_STYLES);
  if (snapshot?.length) doc.import(snapshot);
  return doc;
};

export const blockList = (doc) => doc.getMovableList(BLOCKS);

export const newBlock = (list, at, id, kind) => {
  const map = list.insertContainer(at, new LoroMap());
  map.set('id', id);
  map.set('kind', JSON.stringify(kind));
  map.setContainer('text', new LoroText());
  return map;
};

// A table block is recognised by its `rows` (row Map {id, cells}, cell Map
// {id, props, blocks}); its text stays empty.
export const newTable = (list, at, id) => {
  const map = list.insertContainer(at, new LoroMap());
  map.set('id', id);
  map.set('kind', JSON.stringify({ type: 'table' }));
  map.setContainer('text', new LoroText());
  map.ensureMergeableMap('props');
  map.ensureMergeableMovableList('rows');
  return map;
};

export const newRow = (list, at, id) => {
  const map = list.insertContainer(at, new LoroMap());
  map.set('id', id);
  map.ensureMergeableMovableList('cells');
  return map;
};

export const newCell = (list, at, id) => {
  const map = list.insertContainer(at, new LoroMap());
  map.set('id', id);
  map.ensureMergeableMap('props');
  map.ensureMergeableMovableList('blocks');
  return map;
};

export const childList = (map, key) => {
  const value = map?.get(key);
  return value instanceof LoroMovableList ? value : null;
};

export const childMap = (map, key) => {
  const value = map?.get(key);
  return value instanceof LoroMap ? value : null;
};

// A lazily created child is mergeable: two replicas creating it at once get
// one container (core's `engine::loro::child`). An older regular child is
// kept until the server's write-back replaces it.
export const ensureMap = (map, key) => childMap(map, key) ?? map.ensureMergeableMap(key);

export const ensureList = (map, key) => childList(map, key) ?? map.ensureMergeableMovableList(key);

export const listItems = (list) => {
  const out = [];
  for (let i = 0; i < (list?.length ?? 0); i += 1) out.push(list.get(i));
  return out;
};

export const textOf = (map) => {
  const text = map.get('text');
  return text instanceof LoroText ? text : null;
};

export const toB64 = (bytes) => {
  let bin = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(bin);
};

export const fromB64 = (value) => Uint8Array.from(atob(value ?? ''), (c) => c.charCodeAt(0));

export const versionB64 = (doc) => toB64(doc.oplogVersion().encode());

export const decodeVersion = (value) => VersionVector.decode(fromB64(value));

// The version an update is exported from: everything other peers wrote is
// already on the server, so only this peer's ops past `sent` go out. Core
// refuses an update carrying any other peer's ops (`peer_not_assigned`).
export const sentWithPeers = (doc, sent) => {
  const own = doc.peerIdStr;
  const next = new Map(doc.oplogVersion().toJSON());
  const mine = sent.toJSON().get(own);
  if (mine == null) next.delete(own);
  else next.set(own, mine);
  return VersionVector.parseJSON(next);
};

export const encodeCursor = (text, offset) => {
  const cursor = text.getCursor(offset);
  return cursor ? toB64(cursor.encode()) : null;
};

export const cursorOffset = (doc, value) => {
  try {
    return doc.getCursorPos(Cursor.decode(fromB64(value)))?.offset ?? null;
  } catch {
    return null;
  }
};
