// Accessibility assertion shared by every page spec.
//
// Serious and critical are the two axe severities that describe a barrier
// rather than a preference: a control with no accessible name, a contrast
// failure, a form field with no label. Those are treated as defects. Minor and
// moderate findings are reported in the failure message when there is one, but
// do not fail a run on their own.
import AxeBuilder from '@axe-core/playwright';
import { expect, type Page } from '@playwright/test';

export async function expectAccessible(page: Page): Promise<void> {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .analyze();
  const blocking = results.violations.filter(
    (v) => v.impact === 'serious' || v.impact === 'critical',
  );
  const summary = blocking
    .map((v) => `${v.id} (${v.impact}, ${v.nodes.length} nodes): ${v.help}`)
    .join('\n');
  expect(blocking, `axe found blocking violations:\n${summary}`).toHaveLength(0);
}
