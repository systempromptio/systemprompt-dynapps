// The approvals queue.
//
// The decision buttons only render for an admin, so a spec that drives them
// must use an admin context; a project manager sees the same rows with the
// read-only notice instead. Both are page states worth asserting, which is why
// the notice has an accessor of its own.
import { type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';

export class ApprovalsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.approvals);
  }

  row(callId: string): Locator {
    return this.page.locator(`tr[data-call-id="${callId}"]`);
  }

  approveButton(callId: string): Locator {
    return this.row(callId).locator('[data-action="approve"]');
  }

  denyButton(callId: string): Locator {
    return this.row(callId).locator('[data-action="deny"]');
  }

  readOnlyNotice(): Locator {
    return this.page.locator('.sp-notice');
  }

  async statusTab(label: string): Promise<void> {
    await this.page.locator('.sp-tab', { hasText: label }).first().click();
    await this.page.waitForLoadState('networkidle');
  }
}
