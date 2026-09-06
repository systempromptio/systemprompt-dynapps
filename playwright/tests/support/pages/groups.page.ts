// Page objects for the two Groups pages.
//
// The listing is one table, so it drives the shared TableHandle. The detail
// page carries a table per tab and several on the Usage tab, and the design
// system's table partial emits no name attribute — so sections there are
// addressed by their accessible name, which is the label a reader sees.
import { expect, type Locator, type Page } from '@playwright/test';
import { SEL } from './selectors';
import { BasePage } from './base.page';
import { PATHS } from '../paths';

export class GroupsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.groups);
  }

  /** One group's row, by its id. The remainder row answers to `unattributed`. */
  row(groupId: string): Locator {
    return this.page.locator(`tr[data-group-id="${groupId}"]`);
  }

  /** The remainder row: traffic no group accounts for. */
  remainderRow(): Locator {
    return this.row('unattributed');
  }

  /** Every group row, excluding the remainder. */
  groupRows(): Locator {
    return this.page.locator('tr[data-group-id]:not([data-group-id="unattributed"])');
  }

  /** A row's numeric cell, addressed by the column's visible header text. */
  async cell(groupId: string, header: string): Promise<string> {
    const headers = await this.page.locator(SEL.tableHeaderCell).allInnerTexts();
    const index = headers.findIndex((t) => t.trim().toLowerCase().startsWith(header.toLowerCase()));
    if (index < 0) throw new Error(`no column "${header}"; saw ${headers.join(' | ')}`);
    return (await this.row(groupId).locator('td').nth(index).innerText()).trim();
  }

  /** One row's cell as a number. Several cells carry a headline figure over a
   *  smaller qualifier, so the raw text is not `Number`-able. */
  async cellNumber(groupId: string, header: string): Promise<number> {
    return toNumber(await this.cell(groupId, header));
  }

  /** The group rows excluding Unassigned, which the page pins last whatever
   *  the sort, so an ordering assertion has to leave it out. */
  sortableRows(): Locator {
    return this.page.locator(
      'tr[data-group-id]:not([data-group-id="unattributed"]):not([data-group-id="unassigned"])',
    );
  }

  async sortableColumn(header: string): Promise<number[]> {
    const ids = await this.sortableRows().evaluateAll((rows) =>
      rows.map((r) => (r as HTMLElement).dataset.groupId ?? ''),
    );
    const out: number[] = [];
    for (const id of ids) out.push(await this.cellNumber(id, header));
    return out;
  }

  /** Every value of one column across the group rows, as numbers. */
  async columnNumbers(header: string): Promise<number[]> {
    const ids = await this.groupRows().evaluateAll((rows) =>
      rows.map((r) => (r as HTMLElement).dataset.groupId ?? ''),
    );
    const out: number[] = [];
    for (const id of ids) out.push(toNumber(await this.cell(id, header)));
    return out;
  }

  async remainder(header: string): Promise<number> {
    return toNumber(await this.cell('unattributed', header));
  }

  /** Click a column header to sort. The header text carries a sort indicator,
   *  so the column is matched on its label prefix rather than exactly. */
  async sortBy(header: string): Promise<string> {
    const th = this.page
      .locator(SEL.tableHeaderCell)
      .filter({ hasText: new RegExp(`^${header}`, 'i') })
      .first();
    await th.getByRole('link').click();
    await this.page.waitForLoadState('networkidle');
    return (await th.getAttribute('aria-sort')) ?? 'none';
  }

  /** Follow one of the source filter links. */
  async filterBySource(value: string): Promise<void> {
    await this.page.locator(`[data-source-filter="${value}"]`).first().click();
    await this.page.waitForLoadState('networkidle');
  }

  headerAction(name: RegExp): Locator {
    return this.page.locator(SEL.headerActions).getByRole('button', { name });
  }

  dialog(name: string): Locator {
    return this.page.locator(`dialog[data-dialog="${name}"]`);
  }
}

export class GroupDetailPage extends BasePage {
  constructor(
    page: Page,
    readonly groupId: string,
  ) {
    super(page, PATHS.group(groupId));
  }

  async gotoTab(tab: string): Promise<void> {
    await this.goto({ tab });
    await expect(this.page.locator(`${SEL.tabActive}`)).toHaveCount(1);
  }

  tabLink(tab: string): Locator {
    return this.page.locator(`${SEL.tabs} a[href*="tab=${tab}"]`);
  }

  section(label: string): Locator {
    return this.page.locator(`section[aria-label="${label}"]`);
  }

  /** The rows of the table inside one named section. */
  sectionRows(label: string): Locator {
    return this.section(label).locator(SEL.tableRow);
  }

  memberRow(userId: string): Locator {
    return this.page.locator(`tr[data-user-id="${userId}"]`);
  }

  marketplaceRow(id: string): Locator {
    return this.page.locator(`tr[data-marketplace-id="${id}"]`);
  }

  marketplaceCheckbox(id: string): Locator {
    return this.marketplaceRow(id).locator('[data-field="marketplace"]');
  }
}

// The first number in the cell. Several cells carry a headline figure and a
// smaller qualifier beneath it ("6" over "5 active"), and stripping every
// non-digit would concatenate the two into a number that never existed.
function toNumber(text: string): number {
  const match = text.replace(/,/g, '').match(/-?\d+(\.\d+)?/);
  return match ? Number(match[0]) : 0;
}
