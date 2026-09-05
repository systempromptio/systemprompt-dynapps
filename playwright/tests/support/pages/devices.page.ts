// The Devices & enrolments page.
//
// Four tabs over four credential tables, so the object's whole job is naming
// the tab and the one control that is not shared with every other listing:
// the filter chips, which are links rather than a select because every view
// on this page is bookmarkable.
import { expect, type Locator, type Page } from '@playwright/test';
import { BasePage } from './base.page';
import { SEL } from './selectors';
import { PATHS } from '../paths';

export type DevicesTab = 'bridges' | 'pats' | 'certs' | 'links';

export class DevicesPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.devices);
  }

  /** Open one tab directly, rather than clicking through from another. */
  async gotoTab(tab: DevicesTab): Promise<void> {
    await this.page.goto(PATHS.devicesTab(tab));
    await expect(this.page.locator('h1').first()).toBeVisible();
  }

  tab(label: string): Locator {
    return this.page.locator(SEL.tab).filter({ hasText: label }).first();
  }

  activeTab(): Locator {
    return this.page.locator(`${SEL.tab}[aria-selected="true"]`).first();
  }

  /** One filter chip in the toolbar, addressed by its visible text. */
  chip(label: string): Locator {
    return this.page.locator(SEL.toolbarActions).locator('..').getByRole('link', { name: label });
  }

  /** The inline SVG version histogram; one list item per bridge build. */
  versionBars(): Locator {
    return this.page.locator('.sp-p-devices__version');
  }

  /** Every revoke button currently rendered in the actions column. */
  revokeButtons(): Locator {
    return this.page.locator('[data-revoke-id]');
  }

  /** The name the first revocable row carries, as its button records it. */
  async firstRevocableName(): Promise<string> {
    return (await this.revokeButtons().first().getAttribute('data-revoke-name')) ?? '';
  }

  /** Revoke the first revocable credential and accept the confirmation.
   *
   *  The dialog is checked, not just dismissed. A destructive confirmation
   *  that does not name what it is about to destroy is the one that gets
   *  clicked through, so the name is asserted before the button is pressed.
   */
  async revokeFirst(): Promise<void> {
    const name = await this.firstRevocableName();
    await this.revokeButtons().first().click();
    await expect(this.confirmDialog()).toContainText(name);
    await this.confirm(true, /revoke/i);
  }
}
