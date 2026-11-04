// Core answers 429 `update_contended` when other writers hold the document.
// The replica is still in step, so the same update is resent after the
// server's `retry_after_ms`, jittered so contenders spread out; a body that
// does not parse waits the fallback. Local edits queue until the wait ends.
export const retryDelay = async (resp, fallback) => {
  let wait = fallback;
  try {
    wait = Number((await resp.json()).retry_after_ms) || fallback;
  } catch {
    wait = fallback;
  }
  return wait + Math.random() * wait * 0.25;
};
