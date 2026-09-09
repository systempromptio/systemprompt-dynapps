// Page object for /admin/profile — the signed-in person's own account page.
//
// The page is identity-scoped, so there is nothing to filter and no row to
// act on: what it offers is the connect-code button, which mints a bearer
// credential and is therefore the one action worth driving from a spec.
import { expect, type Locator } from '@playwright/test';
import { BasePage } from './base.page';
import { PATHS } from '../paths';

export class ProfilePage extends BasePage {
  constructor(page: import('@playwright/test').Page) {
    super(page, PATHS.profile);
  }

  connectButton(): Locator {
    return this.page.locator('#issue-connect-code');
  }

  connectOutput(): Locator {
    return this.page.locator('#connect-code-status');
  }

  /** The install command the issued code is pasted into. */
  installCommand(): Locator {
    return this.page.getByRole('tabpanel', { name: 'Claude Code', exact: true })
      .locator('[data-connect-field="install_command"]');
  }

  /** Ask for a connect code and wait for the block to appear. */
  async issueConnectCode(): Promise<void> {
    const issued = this.page.waitForResponse(response =>
      response.url().endsWith('/admin/api/profile/bridge-code') &&
      response.request().method() === 'POST',
    );
    await this.connectButton().click();
    expect((await issued).status()).toBe(200);
    await expect(this.connectOutput()).toBeVisible();
  }

  /** The named section's table, addressed by its screen-reader caption. */
  tableCaptioned(caption: string): Locator {
    return this.page.locator('.sp-table__el').filter({ has: this.page.locator(`caption:text-is("${caption}")`) });
  }
}
