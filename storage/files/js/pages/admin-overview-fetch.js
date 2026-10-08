// Fetching the overview body as a fragment. The server answers with the same
// partial the page embeds; here it is parsed into a detached document and the
// live root handed back. A redirect or a 401/403 means the session is gone,
// and that is reported as its own kind so the poller stops rather than
// retrying into a login page.

import { rawResponse } from '../services/api.js';

const FRAGMENT_URL = '/admin/api/overview';

const failure = (kind, message) => Object.assign(new Error(message), { kind });

export const fetchFragment = async (preset, signal) => {
  let resp;
  try {
    resp = await rawResponse(`${FRAGMENT_URL}?preset=${encodeURIComponent(preset)}`, {
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
  if (!resp.ok) throw failure('http', `Refresh failed with status ${resp.status}`);
  const html = await resp.text();
  const doc = new DOMParser().parseFromString(html, 'text/html');
  const root = doc.querySelector('[data-overview-live]');
  if (!root) throw failure('http', 'Refresh returned no overview body');
  return root;
};
