// A handle over one rendered data table: rows, sorting, filtering, paging and
// CSV export. Every admin listing is the same component, so every listing spec
// drives it through this one object rather than re-deriving row selectors.
import { expect, type Download, type Locator, type Page } from '@playwright/test';
import { SEL } from './selectors';

export class TableHandle {
  readonly root: Locator;

  constructor(
    private readonly page: Page,
    readonly name?: string,
  ) {
    this.root = name ? page.locator(`${SEL.table}[data-table="${name}"]`) : page.locator(SEL.table).first();
  }

  rows(): Locator {
    return this.root.locator('tbody tr');
  }

  async rowCount(): Promise<number> {
    return this.rows().count();
  }

  headers(): Locator {
    return this.root.locator('thead th');
  }

  /** Row values for one column, addressed by its visible header text. */
  async column(header: string): Promise<string[]> {
    const index = await this.columnIndex(header);
    return this.rows().locator(`td:nth-child(${index + 1})`).allInnerTexts();
  }

  private async columnIndex(header: string): Promise<number> {
    const texts = await this.headers().allInnerTexts();
    const index = texts.findIndex((t) => t.trim().toLowerCase() === header.trim().toLowerCase());
    if (index < 0) throw new Error(`no column "${header}" in table ${this.name ?? '(first)'}; saw ${texts.join(' | ')}`);
    return index;
  }

  /** Click a column header to sort, then wait for the table to settle. */
  async sortBy(header: string): Promise<void> {
    const index = await this.columnIndex(header);
    await this.headers().nth(index).click();
    await this.page.waitForLoadState('networkidle');
  }

  async filter(text: string): Promise<void> {
    const input = this.page.locator(SEL.filterInput).first();
    await input.fill(text);
    await input.press('Enter');
    await this.page.waitForLoadState('networkidle');
  }

  async gotoPage(n: number): Promise<void> {
    await this.page.locator(`${SEL.pagination} [data-page="${n}"]`).click();
    await this.page.waitForLoadState('networkidle');
  }

  async nextPage(): Promise<void> {
    await this.page.locator(SEL.paginationNext).click();
    await this.page.waitForLoadState('networkidle');
  }

  /** Trigger the CSV export and return the downloaded file's text. */
  async csv(): Promise<string> {
    const [download] = await Promise.all([
      this.page.waitForEvent('download'),
      this.page.locator(SEL.csvLink).first().click(),
    ]);
    return readDownload(download);
  }

  async expectEmpty(): Promise<void> {
    await expect(this.page.locator(SEL.empty)).toBeVisible();
    expect(await this.rowCount()).toBe(0);
  }
}

async function readDownload(download: Download): Promise<string> {
  const stream = await download.createReadStream();
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(Buffer.from(chunk));
  return Buffer.concat(chunks).toString('utf8');
}
