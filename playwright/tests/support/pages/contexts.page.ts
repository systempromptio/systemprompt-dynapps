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
