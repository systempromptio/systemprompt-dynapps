// The roles and permissions page.
//
// Two tables share the page — assignments first, then role entitlements — so
// every accessor names which one it means rather than relying on document
// order. The role cards are links that filter the assignment table, which is
// why selecting one is a navigation and not a click on a control.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { TableHandle } from './table';

export class RolesPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.roles);
  }

  cards(): Locator {
    return this.page.locator('.sp-p-roles__card');
  }

  card(label: string): Locator {
    return this.cards().filter({ hasText: label }).first();
  }

  assignments(): TableHandle {
    return new TableHandle(this.page);
  }

  // The entitlement listing is the second .sp-table card on the page; the
  // design system's table partial takes no name, so position is the only
  // handle there is.
  entitlementRows(): Locator {
    return this.page.locator('.sp-table').nth(1).locator('tbody tr');
  }

  async selectRole(label: string): Promise<void> {
    await this.card(label).click();
    await this.page.waitForLoadState('networkidle');
  }

  async filterBy(name: string, value: string): Promise<void> {
    await this.page.locator(`form select[name="${name}"]`).first().selectOption(value);
    await this.page.getByRole('button', { name: 'Apply' }).first().click();
    await this.page.waitForLoadState('networkidle');
  }

  grantButton(): Locator {
    return this.page.getByRole('button', { name: 'Grant a role' });
  }

  revokeButtons(): Locator {
    return this.page.locator('[data-action="revoke-role"]');
  }

  async expectRowsFor(role: string): Promise<void> {
    const values = await this.assignments().column('Role');
    expect(values.length).toBeGreaterThan(0);
    for (const value of values) expect(value.trim()).toBe(role);
  }
}
