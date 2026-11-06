// A file's SHA-256, read in slices so memory stays flat for any size. Small
// files go through WebCrypto in one call; the threshold is where reading the
// whole file at once stops being cheap.
import { createSha256 } from './sha256.js';

const SLICE = 8 * 1024 * 1024;
const WHOLE_FILE_LIMIT = 64 * 1024 * 1024;

const hex = (buffer) => Array.from(new Uint8Array(buffer), (b) => b.toString(16).padStart(2, '0')).join('');

export const hashFile = async (file, { signal, onProgress } = {}) => {
  if (file.size <= WHOLE_FILE_LIMIT) {
    const digest = await crypto.subtle.digest('SHA-256', await file.arrayBuffer());
    onProgress?.(file.size);
    return hex(digest);
  }
  const sha = createSha256();
  for (let offset = 0; offset < file.size; offset += SLICE) {
    signal?.throwIfAborted();
    sha.update(new Uint8Array(await file.slice(offset, offset + SLICE).arrayBuffer()));
    onProgress?.(Math.min(offset + SLICE, file.size));
  }
  return sha.digestHex();
};
