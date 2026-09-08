// Concurrency primitives shared across the indexer and the Bedrock client.

/**
 * Run `fn` over every item with at most `limit` calls in flight at once.
 * Results are returned in input order.
 * @template T, R
 * @param {T[]} items
 * @param {number} limit maximum concurrent calls
 * @param {(item: T, index: number) => Promise<R>} fn
 * @returns {Promise<R[]>}
 */
export async function mapLimit(items, limit, fn) {
  const out = new Array(items.length);
  let next = 0;
  const workers = Array.from({ length: Math.min(limit, items.length) }, async () => {
    while (next < items.length) {
      const i = next++;
      out[i] = await fn(items[i], i);
    }
  });
  await Promise.all(workers);
  return out;
}

/**
 * A concurrency limiter (semaphore). However many callers run in parallel, the
 * number of in-flight tasks is capped at `max`, so a shared resource (the
 * Bedrock account quota) is saturated without being permanently exceeded.
 * @param {number} max
 * @returns {<T>(fn: () => Promise<T>) => Promise<T>}
 */
export function createLimiter(max) {
  let active = 0;
  const queue = [];
  const next = () => {
    if (active >= max || queue.length === 0) return;
    active++;
    const { fn, resolve, reject } = queue.shift();
    fn().then(resolve, reject).finally(() => {
      active--;
      next();
    });
  };
  return (fn) =>
    new Promise((resolve, reject) => {
      queue.push({ fn, resolve, reject });
      next();
    });
}

/**
 * Serialize async writes. Returns `run(fn)` that executes tasks one at a time
 * in call order, regardless of how many callers invoke it concurrently.
 * @returns {<T>(fn: () => Promise<T>) => Promise<T>}
 */
export function createMutex() {
  let tail = Promise.resolve();
  return (fn) => {
    const result = tail.then(fn);
    tail = result.then(
      () => undefined,
      () => undefined,
    );
    return result;
  };
}
