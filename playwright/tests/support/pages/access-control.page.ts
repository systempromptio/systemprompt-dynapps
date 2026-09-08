// The access-control page: the rule ledger.
//
// Everything on it is server-rendered, so a filter is a form submission and
// the wait is for the navigation it causes. Editing lives with the subject:
// a group's band on its Access tab, a person's overrides on theirs.
import { type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { TableHandle } from './table';

export class AccessControlPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.accessControl);
  }

  ledger(): TableHandle {
    return new TableHandle(this.page);
  }

  async search(term: string): Promise<void> {
    await this.page.locator('#ac-filter-q').fill(term);
    await this.page.getByRole('button', { name: 'Apply' }).first().click();
    await this.page.waitForLoadState('networkidle');
  }

  async filterBy(name: string, value: string): Promise<void> {
    await this.page.locator(`form select[name="${name}"]`).first().selectOption(value);
    await this.page.getByRole('button', { name: 'Apply' }).first().click();
    await this.page.waitForLoadState('networkidle');
  }


  // The sort indicator lives inside the `th`, so a sorted column's text is
  // "Person ▲" and an exact header match misses it. Matching on the label
  // prefix reads the column whether or not it is the one being sorted on.
  async columnValues(header: string, table = 0): Promise<string[]> {
    const root = this.page.locator('.sp-table').nth(table);
    const headers = await root.locator('.sp-table__el thead th').allInnerTexts();
    const index = headers.findIndex((t) => t.trim().toLowerCase().startsWith(header.toLowerCase()));
    if (index < 0) {
      throw new Error(`no column "${header}"; saw ${headers.join(' | ')}`);
    }
    return root.locator(`.sp-table__el tbody tr td:nth-child(${index + 1})`).allInnerTexts();
  }

  groupRulesLink(): Locator {
    return this.page.locator('.sp-page-header a[href="/admin/groups"]');
  }
}
