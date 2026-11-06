// One file through the resumable upload API: hash, begin (or resume a known
// upload), send the missing parts a few at a time, complete. Parts are
// numbered from 1 and each is exactly part_size bytes but the last; the
// server's missing_parts says which still need sending. Progress is reported
// as { phase, done, total } in bytes. A terminal failure aborts the upload so
// it stops counting against the user's open uploads and quota; a network
// failure keeps it, because that is the case resume exists for.
import { abortUpload, beginUpload, putPart, readUpload } from './transport.js';
import { hashFile } from './hash.js';
import { forget, recall, remember } from './resume.js';
import { awaitCompletion, finishUpload } from './complete.js';

const CONCURRENCY = 3;
const MISSING_ROUNDS = 3;

const isResumable = (upload) => upload.status === 'open' && Date.parse(upload.expires_at) > Date.now();

const resumeOrBegin = async ({ file, mime, sha256 }, signal) => {
  const known = recall(sha256);
  if (known) {
    try {
      const upload = await readUpload(known, signal);
      if (isResumable(upload)) return upload;
    } catch {
      // Why: discard-ok: an unknown or expired upload id just means starting over.
    }
    forget(sha256);
  }
  const upload = await beginUpload({ name: file.name, mime, size: file.size, sha256 }, signal);
  remember(sha256, upload.upload_id);
  return upload;
};

const missingParts = ({ missing_parts: missing, received_parts: received = [], part_count: count }) => {
  if (missing) return [...missing];
  const have = new Set(received);
  return Array.from({ length: count }, (_, i) => i + 1).filter((n) => !have.has(n));
};

const sendParts = async (file, plan, { signal, report }) => {
  const { upload_id: uploadId, part_size: partSize } = plan;
  const pending = missingParts(plan);
  const sizeOf = (n) => Math.min(partSize, file.size - (n - 1) * partSize);
  let done = file.size - pending.reduce((sum, n) => sum + sizeOf(n), 0);
  report(done);
  const worker = async () => {
    for (let n = pending.shift(); n != null; n = pending.shift()) {
      const start = (n - 1) * partSize;
      await putPart(uploadId, n, file.slice(start, start + partSize), signal);
      done += sizeOf(n);
      report(done);
    }
  };
  await Promise.all(Array.from({ length: CONCURRENCY }, worker));
};

// Why: core answers complete with 409 missing_parts when a part it acked was
// lost; re-reading the upload and re-sending what it lists recovers it. A 409
// on a part means another tab's complete closed the upload: wait for it.
const deliver = async (file, plan, ctx) => {
  let current = plan;
  for (let round = 1; ; round += 1) {
    try {
      await sendParts(file, current, ctx);
    } catch (err) {
      if (err.status !== 409 || err.key === 'missing_parts') throw err;
      return awaitCompletion(current.upload_id, ctx.signal);
    }
    try {
      return await finishUpload(current.upload_id, ctx.signal);
    } catch (err) {
      if (err.key !== 'missing_parts' || round >= MISSING_ROUNDS) throw err;
      current = await readUpload(current.upload_id, ctx.signal);
    }
  }
};

const settleFailure = async ({ err, plan, sha256, signal }) => {
  if (!signal?.aborted && !err.isTerminal) return;
  forget(sha256);
  try {
    await abortUpload(plan.upload_id);
  } catch {
    // The session expires server-side anyway, so a failed abort only delays
    // releasing its quota and is not worth surfacing over the real failure.
  }
};

export const uploadFile = async (file, { mime, signal, onProgress = () => {} }) => {
  const sha256 = await hashFile(file, {
    signal,
    onProgress: (done) => onProgress({ phase: 'hashing', done, total: file.size }),
  });
  const plan = await resumeOrBegin({ file, mime, sha256 }, signal);
  const report = (done) => onProgress({ phase: 'uploading', done, total: file.size });
  try {
    const blobRef = await deliver(file, plan, { signal, report });
    forget(sha256);
    return { blobRef, sha256 };
  } catch (err) {
    await settleFailure({ err, plan, sha256, signal });
    throw err;
  }
};
