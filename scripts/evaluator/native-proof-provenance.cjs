'use strict';
const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');
const { execFileSync } = require('node:child_process');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const encode = value => Buffer.from(JSON.stringify(value, null, 2) + '\n');
function source(root) {
  const names = execFileSync('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard'], { cwd: root, maxBuffer: 16 * 1024 * 1024 }).toString().split('\0').filter(Boolean);
  const files = [];
  for (const name of [...new Set(names)].sort()) {
    if (path.isAbsolute(name) || name.split('/').includes('..')) throw Error('Unsafe source path');
    const filename = path.join(root, name);
    if (!fs.existsSync(filename)) { files.push({ path: name, deleted: true }); continue; }
    const stat = fs.lstatSync(filename);
    if (stat.isSymbolicLink()) files.push({ path: name, mode: 'symlink', sha256: hash(fs.readlinkSync(filename)) });
    else if (stat.isFile() && stat.size <= 64 * 1024 * 1024) files.push({ path: name, mode: stat.mode & 0o111 ? 'executable' : 'regular', sha256: hash(fs.readFileSync(filename)) });
    else throw Error(`Unsupported source entry: ${name}`);
  }
  if (!files.length) throw Error('Empty source manifest');
  return { schema_version: 1, files };
}
function capture(root, report, exporter, liveExporter) {
  const manifests = { core: source(path.resolve(root, '../systemprompt-core')), astound: source(root) };
  const digests = {};
  for (const [name, manifest] of Object.entries(manifests)) {
    const bytes = encode(manifest); fs.writeFileSync(path.join(report, `${name}-source.json`), bytes); digests[`${name}_source_digest`] = hash(bytes);
  }
  return { ...digests, exporter_sha256: hash(fs.readFileSync(exporter)), live_exporter_sha256: liveExporter ? hash(fs.readFileSync(liveExporter)) : null };
}
function unchanged(root, captured, exporter, liveExporter) {
  return captured.core_source_digest === hash(encode(source(path.resolve(root, '../systemprompt-core')))) && captured.astound_source_digest === hash(encode(source(root))) && captured.exporter_sha256 === hash(fs.readFileSync(exporter)) && captured.live_exporter_sha256 === (liveExporter ? hash(fs.readFileSync(liveExporter)) : null);
}
module.exports = { capture, unchanged, hash, encode };
