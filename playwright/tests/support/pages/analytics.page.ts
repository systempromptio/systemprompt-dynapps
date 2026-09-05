// The analytics dashboard: six tabs over one scope and one window.
//
// The tabs are links, not client-side panels, so switching one is a
// navigation. Everything else on the page is either the shared shell (scope
// selector, time range, breadcrumbs) or a plain table, both already driven by
// BasePage — so this object adds only the tab bar, the attribution toggle and
// the cost audience switch.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';

export type AnalyticsTab = 'overview' | 'models' | 'skills' | 'tools' | 'sessions' | 'cost';

export const ANALYTICS_TABS: AnalyticsTab[] = [
  'overview',
  'models',
  'skills',
  'tools',
  'sessions',
  'cost',
];

export class AnalyticsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.analytics);
  }

  /** Open one tab directly. Preset defaults to 30d so the seeded window is covered. */
  async openTab(tab: AnalyticsTab, extra: Record<string, string> = {}): Promise<void> {
    await this.goto({ tab, preset: '30d', ...extra });
  }

  activeTab(): Locator {
    return this.page.locator('.sp-tab[aria-selected="true"]');
  }

  async activeTabName(): Promise<string> {
    return (await this.activeTab().innerText()).trim().toLowerCase();
  }

  /** The attribution toggle, whose two readings are Exclusive and Member view. */
  attribution(label: 'Exclusive' | 'Member view'): Locator {
    return this.page
      .locator('[aria-label="Attribution"]')
      .getByRole('link', { name: label, exact: true });
  }

  async setAttribution(label: 'Exclusive' | 'Member view'): Promise<void> {
    await this.attribution(label).click();
    await this.page.waitForLoadState('networkidle');
  }

  /** The warn notice the page shows whenever the rows deliberately overlap. */
  overlapNotice(): Locator {
    return this.page.locator('.sp-notice--warn').filter({ hasText: 'overlaps' });
  }

  async setAudience(label: 'Internal' | 'Customer'): Promise<void> {
    await this.page
      .locator('[aria-label="Report audience"]')
      .getByRole('link', { name: label, exact: true })
      .click();
    await this.page.waitForLoadState('networkidle');
  }

  /** The named table on the current tab, matched by its screen-reader caption. */
  tableWithCaption(fragment: string | RegExp): Locator {
    return this.page.locator('.sp-table').filter({
      has: this.page.locator('caption', { hasText: fragment }),
    });
  }

  /** Every drill link a table row offers, as its href. */
  async drillHrefs(): Promise<string[]> {
    const links = this.page.locator('.sp-table__el tbody a[href^="/admin/requests?"]');
    return links.evaluateAll((nodes) => nodes.map((n) => n.getAttribute('href') ?? ''));
  }

  async expectRendered(): Promise<void> {
    await expect(this.heading()).toBeVisible();
    await expect(this.activeTab()).toHaveCount(1);
  }
}
