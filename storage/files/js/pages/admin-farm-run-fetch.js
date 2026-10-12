// Fetching a run's live block after a known sequence number. The server
// answers with the farm/run-live partial; it is parsed into a detached
// document and its root handed back. A redirect or a 401/403 means the
// session is gone, reported as its own kind so the poller stops rather than
// retrying into a login page.

import { rawResponse } from '../services/api.js';

const failure = (kind, message) => Object.assign(new Error(message), { kind });

export const fetchRunLive = async (pollUrl, afterSeq, signal) => {
  let resp;
  try {
    resp = await rawResponse(`${pollUrl}?after=${encodeURIComponent(afterSeq)}`, {
      signal,
      headers: { Accept: 'text/html' },
    });
  } catch (error) {
    if (error?.name === 'AbortError') throw error;
    throw failure('offline', error?.message || 'Network unavailable');
  }
  if (resp.redirected || resp.status === 401 || resp.status === 403) {
    throw failure('auth', `Session no longer valid (${resp.status})`);
  }
  if (resp.status === 404) throw failure('auth', 'Run no longer exists');
  if (!resp.ok) throw failure('http', `Refresh failed with status ${resp.status}`);
  const html = await resp.text();
  const doc = new DOMParser().parseFromString(html, 'text/html');
  const root = doc.querySelector('[data-farm-run-live]');
  if (!root) throw failure('http', 'Refresh returned no run block');
  return root;
};
