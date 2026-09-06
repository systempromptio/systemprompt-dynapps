// The secrets audit trail.
//
// The one column that carries a judgement is "Third party" — the actor was not
// the secret's owner — so it gets an accessor rather than being read out of the
// generic table handle by position.
import { type Locator, type Page } from "@playwright/test";
import { PATHS } from "../paths";
import { BasePage } from "./base.page";

export class SecretsAuditPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.secretsAudit);
  }

  async filterByAction(action: string): Promise<void> {
    await this.page.locator('form select[name="action"]').selectOption(action);
    await this.page.getByRole("button", { name: "Apply" }).first().click();
    await this.page.waitForLoadState("networkidle");
  }

  thirdPartyBadges(): Locator {
    return this.page.locator("tbody .sp-badge--err");
  }

  csvLink(): Locator {
    return this.page
      .locator(".sp-page-header__actions")
      .getByRole("link", { name: /export csv/i });
  }
}
