// Drives the shared web components in a real browser and reports what they
// actually render, because both of them are shadow-DOM custom elements whose
// failures are silent: a toast that falls through to the wrong type still
// appears, and a dialog that never binds its buttons still opens. Reading the
// source proved neither. Mints its own cookie; point GATEWAY_URL at a server
// running the code under test.
import { chromium } from '@playwright/test';
import { mintToken } from '../setup/mint-token';
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8081';
(async () => {
  const jwks = (await (await fetch(`${BASE}/.well-known/jwks.json`)).json()) as { keys: { kid: string }[] };
  const token = await mintToken({
    userId: 'e2e-platform-admin', email: 'e2e-platform-admin@e2e.local',
    issuer: BASE, kid: jwks.keys[0]?.kid ?? '',
    pemPath: process.env.E2E_SIGNING_KEY ?? '', sessionId: 'e2e-session-platform-admin',
  });
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } });
  await ctx.addCookies([{ name: 'access_token', value: token, domain: 'localhost', path: '/' }]);
  const p = await ctx.newPage();
  const errors: string[] = [];
  p.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
  p.on('console', (m) => { if (m.type() === 'error') errors.push('console: ' + m.text()); });
  await p.goto(BASE + '/admin/users', { waitUntil: 'networkidle' });

  console.log('TOAST', await p.evaluate(async () => {
    const out: Record<string, unknown> = {};
    try {
      const mod = await import('/js/services/toast.js');
      (mod as { showToast: (m: string, t?: string) => void }).showToast('probe message', 'success');
      const host = document.querySelector('sp-toast');
      const root = host?.shadowRoot;
      out.hostPresent = Boolean(host);
      out.toastRendered = Boolean(root?.querySelector('.toast'));
      out.iconText = root?.querySelector('.icon')?.textContent ?? null;
      out.dataType = (root?.querySelector('.toast') as HTMLElement | null)?.dataset.type ?? null;
      out.messageText = root?.querySelector('.message')?.textContent ?? null;
    } catch (e) { out.threw = (e as Error).message; }
    return out;
  }));

  console.log('CONFIRM', await p.evaluate(async () => {
    const out: Record<string, unknown> = {};
    try {
      await import('/js/components/sp-confirm-dialog.js');
      const el = document.createElement('sp-confirm-dialog');
      document.body.append(el);
      const promise = (el as unknown as { confirm: (t: string, m: string) => Promise<boolean> }).confirm('Revoke?', 'This cannot be undone');
      out.opened = el.hasAttribute('is-open');
      const root = (el as HTMLElement & { shadowRoot: ShadowRoot }).shadowRoot;
      out.overlay = Boolean(root.querySelector('.overlay'));
      out.confirmBtnClass = (root.querySelector('[data-role="confirm"]') as HTMLElement)?.className ?? null;
      out.confirmBtnStyled = getComputedStyle(root.querySelector('[data-role="confirm"]') as Element).backgroundColor;
      (root.querySelector('[data-role="cancel"]') as HTMLElement).click();
      out.resolved = await promise;
    } catch (e) { out.threw = (e as Error).message; }
    return out;
  }));

  console.log('ERRORS', errors.length ? errors : 'none');
  await b.close();
})();
import { chromium } from '@playwright/test';
import { mintToken } from '../setup/mint-token';
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8081';
(async () => {
  const jwks = (await (await fetch(`${BASE}/.well-known/jwks.json`)).json()) as { keys: { kid: string }[] };
  const token = await mintToken({
    userId: 'e2e-platform-admin', email: 'e2e-platform-admin@e2e.local',
    issuer: BASE, kid: jwks.keys[0]?.kid ?? '',
    pemPath: process.env.E2E_SIGNING_KEY ?? '', sessionId: 'e2e-session-platform-admin',
  });
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } });
  await ctx.addCookies([{ name: 'access_token', value: token, domain: 'localhost', path: '/' }]);
  const p = await ctx.newPage();
  const errors: string[] = [];
  p.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
  p.on('console', (m) => { if (m.type() === 'error') errors.push('console: ' + m.text()); });
  await p.goto(BASE + '/admin/users', { waitUntil: 'networkidle' });

  console.log('TOAST', await p.evaluate(async () => {
    const out: Record<string, unknown> = {};
    try {
      const mod = await import('/js/services/toast.js');
      (mod as { showToast: (m: string, t?: string) => void }).showToast('probe message', 'success');
      const host = document.querySelector('sp-toast');
      const root = host?.shadowRoot;
      out.hostPresent = Boolean(host);
      out.toastRendered = Boolean(root?.querySelector('.toast'));
      out.iconText = root?.querySelector('.icon')?.textContent ?? null;
      out.dataType = (root?.querySelector('.toast') as HTMLElement | null)?.dataset.type ?? null;
      out.messageText = root?.querySelector('.message')?.textContent ?? null;
    } catch (e) { out.threw = (e as Error).message; }
    return out;
  }));

  console.log('CONFIRM', await p.evaluate(async () => {
    const out: Record<string, unknown> = {};
    try {
      await import('/js/components/sp-confirm-dialog.js');
      const el = document.createElement('sp-confirm-dialog');
      document.body.append(el);
      const promise = (el as unknown as { confirm: (t: string, m: string) => Promise<boolean> }).confirm('Revoke?', 'This cannot be undone');
      out.opened = el.hasAttribute('is-open');
      const root = (el as HTMLElement & { shadowRoot: ShadowRoot }).shadowRoot;
      out.overlay = Boolean(root.querySelector('.overlay'));
      out.confirmBtnClass = (root.querySelector('[data-role="confirm"]') as HTMLElement)?.className ?? null;
      out.confirmBtnStyled = getComputedStyle(root.querySelector('[data-role="confirm"]') as Element).backgroundColor;
      (root.querySelector('[data-role="cancel"]') as HTMLElement).click();
      out.resolved = await promise;
    } catch (e) { out.threw = (e as Error).message; }
    return out;
  }));

  console.log('ERRORS', errors.length ? errors : 'none');
  await b.close();
})();
