// The contexts list and one context's detail page.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class ContextsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.contexts);
  }

  contextLinks(): Locator {
    return this.page.locator(`${SEL.tableRow} a[href^="/admin/contexts/"]`);
  }

  async openFirst(): Promise<ContextDetailPage> {
    await this.contextLinks().first().click();
    await expect(this.page).toHaveURL(/\/admin\/contexts\/.+/);
    return new ContextDetailPage(this.page, this.page.url());
  }

  /** The "By user" / "Contexts" view switch. */
  viewTab(name: 'contexts' | 'users'): Locator {
    return this.page.locator(`${SEL.tab}[data-view="${name}"]`);
  }

  /** Click a sortable column header by its label and wait for the re-render.
   *
   *  Not `TableHandle.sortBy`: the design system renders the sort indicator
   *  inside the `th`, so the header's text is "COST ↕" and the shared handle's
   *  exact-match column lookup never finds it.
   */
  async sortBy(label: string): Promise<string | null> {
    const th = this.page.locator(`${SEL.tableHeaderCell}`).filter({ hasText: new RegExp(`^\\s*${label}`, 'i') }).first();
    await th.getByRole('link').first().click();
    await this.page.waitForLoadState('networkidle');
    return th.getAttribute('aria-sort');
  }

  /** Narrow the listing to one group through the page's own scope form.
   *
   *  The shell's `[data-scope-selector]` bar carries the window and the scope
   *  chips but no selects, so `BasePage.scope` has nothing to drive here.
   */
  async scopeTo(field: 'group' | 'project', value: string): Promise<void> {
    const form = this.page.locator('form.sp-analytics-filters');
    await form.locator(`select[name="${field}"]`).selectOption(value);
    await form.getByRole('button', { name: /apply/i }).click();
    await this.page.waitForLoadState('networkidle');
  }

}

export class ContextDetailPage extends BasePage {
  constructor(page: Page, path: string) {
    super(page, path);
  }

  /** What the session touched: files, tools and repositories, busiest first. */
  entityTable(): Locator {
    return this.page
      .locator(SEL.table)
      .filter({ has: this.page.locator('caption', { hasText: /Entities linked/i }) });
  }

  entityBars(): Locator {
    return this.page.locator('.sp-p-context-detail__bar');
  }
}
