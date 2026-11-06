// Completes an upload exactly once. complete is never re-sent: after a
// timeout or 5xx the server may still be assembling the blob, so the upload
// is polled while it is `open` or `completing`, and its blob_ref is taken
// from there. A 409 upload_closed can mean another complete is already
// assembling it, so it is polled too; the status then decides.
import { completeUpload, readUpload } from './transport.js';

const POLL_MS = 2000;
const POLL_LIMIT = 90;
const IN_PROGRESS = new Set(['open', 'completing']);

const pause = (ms, signal) => new Promise((resolve, reject) => {
  const timer = setTimeout(resolve, ms);
  signal?.addEventListener('abort', () => {
    clearTimeout(timer);
    reject(signal.reason);
  }, { once: true });
});

export const awaitCompletion = async (id, signal) => {
  for (let n = 0; n < POLL_LIMIT; n += 1) {
    await pause(POLL_MS, signal);
    const upload = await readUpload(id, signal);
    if (upload.status === 'completed') return upload.blob_ref;
    if (!IN_PROGRESS.has(upload.status)) {
      throw Object.assign(new Error(upload.refusal ?? `The upload was ${upload.status}.`), { isTerminal: true });
    }
  }
  throw new Error('The server is still finishing this upload. Try again shortly.');
};

export const finishUpload = async (id, signal) => {
  try {
    return (await completeUpload(id, signal)).blob_ref;
  } catch (err) {
    if (signal?.aborted) throw err;
    if (err.isTerminal && err.key !== 'upload_closed') throw err;
    return awaitCompletion(id, signal);
  }
};
