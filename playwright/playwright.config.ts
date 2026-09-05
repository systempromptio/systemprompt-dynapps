import { defineConfig, devices } from '@playwright/test';

// Runs against an already-running gateway (`just start`); it never boots one —
// this clone's server may be shared with other agents, so global-setup pings
// /health and fails fast instead of starting anything. GATEWAY_URL overrides
// the default local port.
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  retries: Number(process.env.E2E_RETRIES ?? 0),
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]],
  globalSetup: './setup/global-setup.ts',
  snapshotPathTemplate: '{testFileDir}/__screenshots__/{testFileName}/{projectName}-{platform}/{arg}{ext}',
  expect: {
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.01,
      animations: 'disabled',
      caret: 'hide',
      stylePath: './tests/support/screenshot.css',
    },
  },
  use: {
    baseURL: process.env.GATEWAY_URL ?? 'http://localhost:8080',
    trace: 'retain-on-failure',
    // Determinism knobs for the visual suite; harmless for functional specs.
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
    colorScheme: 'light',
    timezoneId: 'UTC',
    locale: 'en-US',
    contextOptions: { reducedMotion: 'reduce' },
  },
  projects: [
    {
      name: 'chromium',
      testIgnore: ['**/demo/**', '**/_*.spec.ts'],
      use: { ...devices['Desktop Chrome'], viewport: { width: 1440, height: 900 } },
    },
    // The demo project asserts nothing: it walks the product in narrative order
    // and screenshots each step, and `just e2e-demo` renders those into the
    // demo book. It is excluded from `chromium` so a normal `just e2e` neither
    // runs it nor is slowed by it.
    {
      name: 'demo',
      testDir: './tests/demo',
      testMatch: '**/*.demo.ts',
      testIgnore: '**/_*.demo.ts',
      fullyParallel: false,
      // A chapter is one test that walks a whole section, and each stop waits
      // for the network to settle and takes a full-page screenshot. Under the
      // default 30s a long chapter is torn down mid-step, which surfaces as
      // "target page has been closed" on whatever it was doing — an error that
      // says nothing about the timeout that caused it.
      timeout: 180_000,
      use: { ...devices['Desktop Chrome'], viewport: { width: 1440, height: 900 } },
    },
  ],
});
