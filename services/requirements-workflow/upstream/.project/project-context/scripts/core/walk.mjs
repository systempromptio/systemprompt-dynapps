// Recursive directory walk shared by the indexer and the normalizer.

import { readdir } from 'node:fs/promises';
import { join } from 'node:path';

/**
 * Yield every file path under `dir`, recursively, in a stable sorted order.
 * Hidden entries (dotfiles/dotdirs) are skipped. Callers apply their own file
 * filtering. A missing or unreadable directory yields nothing.
 * @param {string} dir
 * @returns {AsyncGenerator<string>}
 */
export async function* walk(dir) {
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
    if (entry.name.startsWith('.')) continue;
    const full = join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(full);
    else yield full;
  }
}
