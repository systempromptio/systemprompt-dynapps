// The governance page: three URL-driven tabs over one window.
//
// Every control is a link or a GET form, so switching a tab or a filter is a
// navigation. The accessors below therefore return after the page has
// re-rendered rather than after a click, which is what makes an assertion on
// the resulting rows meaningful.
import { type Locator, type Page } from "@playwright/test";
import { PATHS } from "../paths";
import { BasePage } from "./base.page";

export type GovernanceTab = "decisions" | "safety" | "hooks";

export class GovernancePage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.governance);
  }

  async openTab(tab: GovernanceTab, preset = "30d"): Promise<void> {
    await this.page.goto(`${PATHS.governance}?tab=${tab}&preset=${preset}`);
    await this.page.waitForLoadState("networkidle");
  }

  // The sort control is the anchor inside the header cell, not the cell. On a
  // wide table the cell's centre is empty space, so a click on the cell itself
  // lands on nothing and the page never navigates.
  async sortBy(label: string): Promise<string | null> {
    const th = this.page
      .locator("th")
      .filter({ has: this.page.locator("a.sp-table__sort") })
      .filter({ hasText: new RegExp(`^${label}`, "i") })
      .first();
    await th.locator("a.sp-table__sort").click();
    await this.page.waitForLoadState("networkidle");
    return this.page
      .locator("th")
      .filter({ hasText: new RegExp(`^${label}`, "i") })
      .first()
      .getAttribute("aria-sort");
  }

  /** Narrow the log through the toolbar's GET form. */
  async filterBy(name: string, value: string): Promise<void> {
    await this.page
      .locator(`form[data-governance-filters] select[name="${name}"]`)
      .selectOption(value);
    await this.page.waitForLoadState("networkidle");
  }

  reasonCells(): Locator {
    return this.page.locator(".sp-p-governance__reason");
  }

  rankLists(): Locator {
    return this.page.locator(".sp-rank-list");
  }

  csvLink(): Locator {
    return this.page
      .locator(".sp-page-header__actions")
      .getByRole("link", { name: /export csv/i });
  }
}
