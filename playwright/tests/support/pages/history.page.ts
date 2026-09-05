// Page object for /admin/history — the viewer's own conversations, searchable.
import { type Locator } from '@playwright/test';
import { BasePage } from './base.page';
import { SEL } from './selectors';
import { PATHS } from '../paths';

export class HistoryPage extends BasePage {
  constructor(page: import('@playwright/test').Page) {
    super(page, PATHS.history);
  }

  searchInput(): Locator {
    return this.page.locator('#history-q');
  }

  /** Run a full-text search and wait for the reloaded page. */
  async search(term: string): Promise<void> {
    await this.searchInput().fill(term);
    await Promise.all([
      this.page.waitForLoadState('networkidle'),
      this.page.locator('form[action="/admin/history"] button[type="submit"]').click(),
    ]);
  }

  pagination(): Locator {
    return this.page.locator(SEL.pagination);
  }
}
