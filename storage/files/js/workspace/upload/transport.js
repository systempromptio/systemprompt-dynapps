// The resumable upload API: begin, put a part, read progress, complete, abort,
// and import a completed blob as a document. Every call retries transient
// failures (network errors, 408, 429, 5xx) with exponential backoff; any
// other 4xx is the server's verdict and is not retried. A 429 that names a
// used-up quota, too many open uploads or a spent part budget will not clear
// by waiting, so it fails at once rather than retrying.
import { rawResponse } from '../../services/api.js';
import { describeError } from './errors.js';

const UPLOADS = '/api/v1/uploads';
const IMPORT = '/api/v1/documents/import';
const MAX_ATTEMPTS = 5;
const BASE_DELAY_MS = 500;

const LASTING_429 = new Set(['quota_exceeded', 'too_many_open_uploads', 'part_budget_exhausted']);

const errorKey = async (resp) => {
  try {
    return (await resp.clone().json()).error_key;
  } catch {
    return null;
  }
};

const isTransient = async (resp) => {
  if (resp.status === 429) return !LASTING_429.has(await errorKey(resp));
  return resp.status === 408 || resp.status >= 500;
};

const wait = (ms, signal) => new Promise((resolve, reject) => {
  const timer = setTimeout(resolve, ms);
  signal?.addEventListener('abort', () => {
    clearTimeout(timer);
    reject(signal.reason);
  }, { once: true });
});

const attempt = async (url, options) => {
  try {
    const resp = await rawResponse(url, options);
    return { resp, isRetryable: await isTransient(resp) };
  } catch (err) {
    if (options.signal?.aborted) throw err;
    return { err, isRetryable: true };
  }
};

// `isTerminal` marks the server's verdict on the request itself; an error
// without it (network, exhausted retries) leaves the upload worth resuming.
const failure = async (resp, err, isRetryable) => {
  const key = resp ? await errorKey(resp) : null;
  return Object.assign(new Error(resp ? await describeError(resp) : err.message), {
    key,
    status: resp?.status ?? null,
    isTerminal: Boolean(resp) && !isRetryable,
  });
};

const send = async (url, options = {}, attempts = MAX_ATTEMPTS) => {
  for (let n = 1; ; n += 1) {
    const { resp, err, isRetryable } = await attempt(url, options);
    if (resp?.ok) return resp.status === 204 ? null : resp.json();
    if (!isRetryable || n >= attempts) throw await failure(resp, err, isRetryable);
    await wait(BASE_DELAY_MS * 2 ** (n - 1) + Math.random() * BASE_DELAY_MS, options.signal);
  }
};

const json = (method, body, signal) => ({ method, body: JSON.stringify(body), signal });
const at = (id) => `${UPLOADS}/${encodeURIComponent(id)}`;

export const beginUpload = (meta, signal) => send(UPLOADS, json('POST', meta, signal));

export const putPart = (id, n, bytes, signal) => send(`${at(id)}/parts/${n}`, {
  method: 'PUT',
  headers: { 'Content-Type': 'application/octet-stream' },
  body: bytes,
  signal,
});

export const readUpload = (id, signal) => send(at(id), { signal });

// Why: a proxy timeout on a large complete leaves core still assembling the
// blob; a second POST races it, so the caller polls readUpload instead.
export const completeUpload = (id, signal) => send(`${at(id)}/complete`, json('POST', {}, signal), 1);

export const abortUpload = (id) => send(at(id), { method: 'DELETE' });

export const importDocument = (body, signal) => send(IMPORT, json('POST', body, signal));
