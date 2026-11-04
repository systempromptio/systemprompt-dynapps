// Remembers which upload a file belongs to, keyed by its content hash, so a
// reload or a dropped connection continues the same upload instead of sending
// every part again. Storage can be absent (private windows); resume is then
// simply unavailable and every upload starts fresh.
const key = (sha256) => `sp-upload:${sha256}`;

export const recall = (sha256) => {
  try {
    return localStorage.getItem(key(sha256));
  } catch {
    return null;
  }
};

export const remember = (sha256, uploadId) => {
  try {
    localStorage.setItem(key(sha256), uploadId);
  } catch {
    // Why: discard-ok: without storage the upload still completes, it just cannot resume.
  }
};

export const forget = (sha256) => {
  try {
    localStorage.removeItem(key(sha256));
  } catch {
    // Why: discard-ok: nothing was stored to remove.
  }
};
