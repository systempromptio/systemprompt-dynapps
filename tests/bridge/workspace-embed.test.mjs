import test from 'node:test';
import assert from 'node:assert/strict';
import { availableCommands, registerCommand, runCommand } from '../../storage/files/js/workspace/registry.js';

const ran = [];
registerCommand({ id: 'embed-test-stays', label: 'Stays', run: () => ran.push('stays') });
registerCommand({ id: 'embed-test-leaves', label: 'Leaves', leaves: true, run: () => ran.push('leaves') });

const idsOf = (ctx) => availableCommands(ctx).map((command) => command.id);

test('a page that is not embedded lists and runs every command it may', async () => {
  const ctx = { root: {}, embed: false };
  assert.ok(idsOf(ctx).includes('embed-test-stays'));
  assert.ok(idsOf(ctx).includes('embed-test-leaves'));
  ran.length = 0;
  await runCommand('embed-test-leaves', ctx);
  assert.deepEqual(ran, ['leaves']);
});

test('an embedded page neither lists nor runs a command that leaves the page', async () => {
  const ctx = { root: {}, embed: true };
  assert.ok(idsOf(ctx).includes('embed-test-stays'));
  assert.ok(!idsOf(ctx).includes('embed-test-leaves'));
  ran.length = 0;
  await runCommand('embed-test-leaves', ctx);
  await runCommand('embed-test-stays', ctx);
  assert.deepEqual(ran, ['stays']);
});
