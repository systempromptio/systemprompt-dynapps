import { test } from '@playwright/test';
import { join } from 'node:path';
const OUT = '/tmp/claude-1000/-var-www-html-systemprompt-astound/ed190528-5ac4-42f6-b10f-bf8d067262aa/scratchpad/shots';
const KW = 'e2e00001-0000-4000-8000-000000000003';
test.use({ storageState: join(__dirname, '..', '.auth', 'platform-admin.json'), viewport: { width: 1440, height: 1000 } });
for (const [name, path] of [
  ['traces', '/admin/traces'],
  ['requests', '/admin/requests'],
  ['skills', '/admin/skills'],
  ['mcp', '/admin/mcp'],
  ['kw-usage', '/admin/users/' + KW + '?tab=usage'],
  ['governance', '/admin/governance'],
] as const) {
  test('shot ' + name, async ({ page }) => {
    const res = await page.goto(path);
    console.log(name, path, res?.status());
    await page.waitForTimeout(500);
    await page.screenshot({ path: join(OUT, name + '.png'), fullPage: true });
  });
}
