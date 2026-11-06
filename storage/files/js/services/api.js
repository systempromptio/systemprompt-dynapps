import { showToast } from './toast.js';

export const BASE = '/admin';
export const API_BASE = '/api/public/admin';

export const rawResponse = (url, options = {}) => fetch(url, {
  headers: { 'Content-Type': 'application/json' },
  ...options
});

export const errorMessage = async (resp) => {
  const text = await resp.text();
  try {
    const json = JSON.parse(text);
    const message = json.error || json.message || text || resp.statusText;
    return json.error_key ? `${message} (${json.error_key})` : message;
  } catch {
    return text || resp.statusText;
  }
};

const SIGN_IN = '/admin/login';
const REAUTH = 'reauth';

// A session minted before the signed browser-session claim is refused with
// `error_key: reauth`; the only recovery is signing in again, so the page
// goes there and comes back rather than toasting an error it cannot fix.
const isReauth = async (resp) => {
  if (resp.status !== 401) return false;
  try {
    return (await resp.clone().json()).error_key === REAUTH;
  } catch {
    return false;
  }
};

const signInAgain = () => {
  const back = `${window.location.pathname}${window.location.search}`;
  window.location.assign(`${SIGN_IN}?redirect=${encodeURIComponent(back)}`);
};

const request = async (url, options = {}) => {
  const resp = await rawResponse(url, options);
  if (await isReauth(resp)) {
    signInAgain();
    throw new Error('Signing in again');
  }
  if (!resp.ok) {
    const message = await errorMessage(resp);
    showToast(message, 'error');
    throw new Error(message);
  }
  const ct = resp.headers.get('content-type') || '';
  if (resp.status === 204 || !ct.includes('application/json')) return null;
  return resp.json();
};

export const apiFetch = (path, options = {}) => request(API_BASE + path, options);

export const apiGet = (path) => apiFetch(path);

export const rawFetch = (url, options = {}) => request(url, options);
