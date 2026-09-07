import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const html = (id: string) => readFileSync(join(__dirname, '../../services/artifacts', id, 'view.html'), 'utf8');
const report = {
  report: 'costs', title: 'AI Cost & Adoption', checked_at: '2026-09-07T12:00:00Z',
  period: '7 days', complete: true, metrics: {}, request: { report: 'costs', days: 7 },
  sources: [{ source: 'Models', complete: true, warning: null }],
  tables: [{ title: 'Models', columns: ['model', 'total_cost_microdollars'], rows: [
    { model: '<img src=x onerror="window.injected=true">', total_cost_microdollars: 1234567 },
    { model: 'model-b', total_cost_microdollars: 0 },
  ] }],
};

test('structured reports render costs, escape data, filter and refresh', async ({ page }) => {
  await page.setContent(html('admin-usage-costs').replace('/*REPORT_DATA*/null', JSON.stringify(report).replaceAll('<', '\\u003c')));
  await expect(page.locator('tbody tr')).toHaveCount(2);
  await expect(page.locator('tbody')).toContainText('$1.234567');
  await expect(page.locator('tbody img')).toHaveCount(0);
  await page.locator('#filter').fill('model-b');
  await expect(page.locator('tbody tr')).toHaveCount(1);
  await page.evaluate((value) => {
    (window as any).calls = [];
    (window as any).cowork = { callMcpTool: async (tool: string, args: unknown) => {
      (window as any).calls.push({ tool, args });
      return { structuredContent: { ...value, complete: false, sources: [{ source: 'Models', complete: false, warning: 'Capped sample' }] } };
    } };
  }, report);
  await page.locator('#refresh').click();
  await expect(page.locator('#state')).toContainText('Partial data');
  await expect(page.locator('#sources')).toContainText('Capped sample');
  expect(await page.evaluate(() => (window as any).calls)).toEqual([{ tool: 'mcp__systemprompt__admin_report', args: { report: 'costs', days: 7 } }]);
});

test('project artifact selects site and project without a skill', async ({ page }) => {
  await page.setContent(html('admin-project-delivery'));
  await page.evaluate((value) => {
    (window as any).calls = [];
    (window as any).cowork = { callMcpTool: async (tool: string, args: any) => {
      (window as any).calls.push({ tool, args });
      if (tool.endsWith('catalog')) return { structuredContent: { ...value, tables: args.cloud_id ? [
        { title: 'Projects', rows: [{ key: 'DEMO', name: 'Demo' }] },
        { title: 'Spaces', rows: [{ id: '42', name: 'Delivery' }] },
      ] : [{ title: 'Sites', rows: [{ id: 'cloud-1', name: 'Astound' }] }] } };
      return { structuredContent: { ...value, report: 'projects', tables: [], metrics: { 'Overdue unresolved': 2 } } };
    } };
  }, report);
  await page.locator('#catalog').click();
  await page.locator('#site').selectOption('cloud-1');
  await expect(page.locator('#projects option')).toHaveCount(1);
  await page.locator('#project').fill('DEMO');
  await page.locator('#space').fill('42');
  await page.locator('#refresh').click();
  await expect(page.locator('#metrics')).toContainText('Overdue unresolved');
  expect((await page.evaluate(() => (window as any).calls)).at(-1)).toEqual({ tool: 'mcp__systemprompt__admin_report', args: { report: 'projects', days: 7, cloud_id: 'cloud-1', project_key: 'DEMO', space_id: '42' } });
});

test('host theme overrides system theme and errors preserve prior data', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.setContent(html('admin-usage-costs').replace('/*REPORT_DATA*/null', JSON.stringify(report).replaceAll('<', '\\u003c')));
  await page.evaluate(() => window.postMessage({ params: { hostContext: { theme: 'light' } } }, '*'));
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await page.evaluate(() => { (window as any).cowork = { callMcpTool: async () => ({ isError: true, content: [{ text: 'Connector denied' }] }) }; });
  await page.locator('#refresh').click();
  await expect(page.locator('#state')).toContainText('Connector denied');
  await expect(page.locator('tbody tr')).toHaveCount(2);
});

test('MCP Apps viewer relays deterministic refresh through the sandbox', async ({ page }) => {
  const root = join(__dirname, '../..');
  const bridge = readFileSync(join(root, 'storage/files/js/admin-report-bridge.js'), 'utf8');
  const relay = readFileSync(join(root, 'storage/files/js/admin-report-shell.js'), 'utf8');
  const shell = readFileSync(join(root, '.vendor/systemprompt-core/crates/domain/mcp/src/services/ui_renderer/templates/assets/html/artifact-shell.html'), 'utf8')
    .replace('/*MCP_UI_CONSTANTS*/', `const MCP_UI={INITIALIZE:'ui/initialize',INITIALIZED:'ui/notifications/initialized',TOOL_RESULT:'ui/notifications/tool-result',SIZE_CHANGED:'ui/notifications/size-changed',HOST_CONTEXT_CHANGED:'ui/notifications/host-context-changed',PROTOCOL_VERSION:'2026-01-26'};`)
    .replace('    request(MCP_UI.INITIALIZE, {', relay + '\n    request(MCP_UI.INITIALIZE, {');
  const artifact = html('admin-usage-costs').replace('/*MCP_BRIDGE*/', bridge)
    .replace('/*REPORT_DATA*/null', JSON.stringify(report).replaceAll('<', '\\u003c'));
  const literal = (value: unknown) => JSON.stringify(value).replaceAll('<', '\\u003c');
  await page.setContent(`<iframe id="viewer" style="width:100%;height:800px"></iframe><script>
    window.calls=[];
    const frame=document.getElementById('viewer');
    window.addEventListener('message', event=>{
      if(event.source!==frame.contentWindow)return;
      const message=event.data;
      if(message.method==='ui/initialize'){
        frame.contentWindow.postMessage({jsonrpc:'2.0',id:message.id,result:{hostContext:{theme:'light'}}},'*');
        frame.contentWindow.postMessage({jsonrpc:'2.0',method:'ui/notifications/tool-result',params:{content:[{type:'resource',resource:{uri:'ui://systemprompt/artifact/example',mimeType:'text/html;profile=mcp-app',text:${literal(artifact)}}}]}},'*');
      }
      if(message.method==='tools/call'){
        window.calls.push(message.params);
        frame.contentWindow.postMessage({jsonrpc:'2.0',id:message.id,result:{structuredContent:${literal(report)}}},'*');
      }
    });
    frame.srcdoc=${literal(shell)};
  </script>`);
  const app = page.frameLocator('#viewer').frameLocator('#frame');
  await expect(app.locator('h1')).toHaveText('AI Cost & Adoption');
  await expect(app.locator('html')).toHaveAttribute('data-theme', 'light');
  await app.locator('#refresh').click();
  await expect(app.locator('#state')).toHaveText('Sources loaded');
  expect(await page.evaluate(() => (window as any).calls)).toEqual([{ name: 'admin_report', arguments: { report: 'costs', days: 7 } }]);
  const denial = await app.locator('body').evaluate(() => new Promise(resolve => {
    const handler = (event: MessageEvent) => {
      if (event.data?.id === 'forbidden-write') { window.removeEventListener('message', handler); resolve(event.data.error?.message); }
    };
    window.addEventListener('message', handler);
    window.parent.postMessage({ jsonrpc: '2.0', id: 'forbidden-write', method: 'tools/call', params: { name: 'systemprompt', arguments: { command: 'admin users delete example' } } }, '*');
  }));
  expect(denial).toBe('Reporting tool not allowed');
  expect((await page.evaluate(() => (window as any).calls)).length).toBe(1);
});
