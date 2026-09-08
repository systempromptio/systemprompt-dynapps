// Minimal .env loader (no dependency).
//
// Loads KEY=VALUE pairs into process.env without overwriting variables that are
// already set (real environment wins). Values may be quoted; inline `#` comments
// are stripped for unquoted values.

import { readFileSync, existsSync, statSync } from 'node:fs';
import { join } from 'node:path';

/**
 * Load a .env file. The target may be the .env file itself (e.g. a custom
 * env_path from config) or a directory that contains a .env file.
 * @param {string} target .env file path, or a directory containing one
 * @returns {boolean} whether a .env file was found and loaded
 */
export function loadEnv(target) {
  let path = target;
  try {
    if (existsSync(path) && statSync(path).isDirectory()) path = join(path, '.env');
  } catch {
    path = join(target, '.env');
  }
  if (!existsSync(path)) return false;

  const text = readFileSync(path, 'utf8');
  for (const rawLine of text.split('\n')) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) continue;

    const eq = line.indexOf('=');
    if (eq === -1) continue;

    const key = line.slice(0, eq).trim();
    if (!key || key in process.env) continue;

    let value = line.slice(eq + 1).trim();
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    } else {
      const hash = value.indexOf(' #');
      if (hash !== -1) value = value.slice(0, hash).trim();
    }

    process.env[key] = value;
  }
  return true;
}
