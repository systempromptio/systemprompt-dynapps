// The access-control page: the rule ledger above, the three-pane editor below.
//
// The ledger is server-rendered and the editor is not, so the two need
// different waits — a ledger filter is a form submission and a tree selection
// is a click that swaps a client-rendered pane.
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

  async filterBy(name: string, value: string): Promise<void> {
    await this.page.locator(`form select[name="${name}"]`).first().selectOption(value);
    await this.page.getByRole('button', { name: 'Apply' }).first().click();
    await this.page.waitForLoadState('networkidle');
  }

  tree(): Locator {
    return this.page.locator('.sp-ac-tree');
  }

  groupRow(label: string): Locator {
    return this.page.locator('[data-action="select-group"]').filter({ hasText: label }).first();
  }

  editorPane(): Locator {
    return this.page.locator('[data-pane="editor"]');
  }

  matrixPane(): Locator {
    return this.page.locator('[data-pane="matrix"]');
  }

  async selectGroup(label: string): Promise<void> {
    await this.groupRow(label).click();
  }
}
