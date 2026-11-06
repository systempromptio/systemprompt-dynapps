import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';

const root = fileURLToPath(new URL('../../storage/files/js/', import.meta.url));
// Core serves no document modules: this install vendors the two it needs
// under js/documents/, so no script may reach for core's /assets/documents/.
const CORE_PREFIX = 'assets/documents/';

async function javascriptFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...await javascriptFiles(file));
    else if (entry.name.endsWith('.js')) files.push(file);
  }
  return files;
}

// Link without evaluation: the actual JS engine checks named exports and
// syntax without needing a browser DOM or executing application operations.
async function linkGraph(directory, published = false) {
  const context = vm.createContext({});
  const modules = new Map();
  const currentPaths = new Set((await javascriptFiles(root)).map(file => path.relative(root, file)));
  for (const file of await javascriptFiles(directory)) {
    if (published && !currentPaths.has(path.relative(directory, file))) continue;
    modules.set(file, new vm.SourceTextModule(await readFile(file, 'utf8'), {
      context, identifier: file,
    }));
  }
  // Bare specifiers resolve as the workspace import map resolves them: to the
  // one module under vendor/<name>@<version>/ (services/web/vendor.yaml).
  const importMap = new Map();
  for (const file of modules.keys()) {
    const rel = path.relative(path.join(directory, 'vendor'), file);
    const match = /^((?:@[^/]+\/)?[^/@]+)@[^/]+\/index\.[a-f0-9]+\.js$/.exec(rel.split(path.sep).join('/'));
    if (match) importMap.set(match[1], file);
  }
  const resolve = (specifier, parent) => {
    if (importMap.has(specifier)) return modules.get(importMap.get(specifier));
    if (published) {
      assert.match(specifier, /\?v=[a-f0-9]{64}$/, `${parent.identifier} imports an unversioned dependency: ${specifier}`);
    }
    const pathname = specifier.split('?')[0];
    let file = path.resolve(path.dirname(parent.identifier), pathname);
    if (pathname.startsWith('/js/')) file = path.join(directory, pathname.slice(4));
    assert.ok(modules.has(file), `${parent.identifier} imports missing module ${specifier}`);
    return modules.get(file);
  };
  for (const module of [...modules.values()]) {
    if (module.status === 'unlinked') await module.link(resolve);
  }
  return modules.size;
}

test('every shipped JavaScript module links with its actual dependencies', async () => {
  assert.ok(await linkGraph(root) > 50);
});

test('the install imports no core document modules at all', async () => {
  for (const file of await javascriptFiles(root)) {
    const source = await readFile(file, 'utf8');
    assert.ok(!source.includes(CORE_PREFIX), `${path.relative(root, file)} imports from core's /${CORE_PREFIX}`);
  }
});

test('the graph check rejects the missing-export failure seen on Connectors', async () => {
  const context = vm.createContext({});
  const dependency = new vm.SourceTextModule('export const rawResponse = () => {};', { context });
  const page = new vm.SourceTextModule("import { errorMessage } from './api.js';", { context });
  await assert.rejects(page.link(() => dependency), /does not provide an export named 'errorMessage'/);
});

if (process.env.SYSTEMPROMPT_JS_DIST) {
  test('published JavaScript modules link with their published dependencies', async () => {
    assert.ok(await linkGraph(path.resolve(process.env.SYSTEMPROMPT_JS_DIST), true) > 50);
  });
}
