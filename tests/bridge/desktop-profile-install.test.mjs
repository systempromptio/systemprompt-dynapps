import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

async function harness(warnings = []) {
  const calls = [];
  const source = await readFile(new URL('../../../systemprompt-core/bin/bridge/web/js/utils/host-actions.js', import.meta.url), 'utf8');
  const context = vm.createContext({
    console,
    bridge: {
      hostProfileGenerate: async () => ({ path: '/tmp/claude.mobileconfig' }),
      hostProfileInstall: async () => ({ warnings }),
      hostProbe: async () => { calls.push('probe'); },
    },
  });
  vm.runInContext(source.replace(/^import .*;$/gm, '').replace(/export /g, ''), context);
  return { context, calls };
}

test('manual profile approval is reported instead of successful installation', async () => {
  const { context, calls } = await harness(['Approval required: install the profile in System Settings.']);
  await context.repairHost('claude-desktop');
  assert.match(context.hostInstallMessage('claude-desktop', 'Installed'), /Approval required/);
  assert.deepEqual(calls, ['probe']);
});

test('completed host installations keep their normal success message', async () => {
  const { context } = await harness();
  await context.repairHost('opencode');
  assert.equal(context.hostInstallMessage('opencode', 'Installed'), 'Installed');
});
