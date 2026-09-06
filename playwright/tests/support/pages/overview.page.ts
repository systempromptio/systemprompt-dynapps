// The console landing page.
//
// The overview stacks four panels rather than one listing, so this object
// addresses each by the accessible name of the section it lives in. That is
// the same handle a screen-reader user navigates by, which keeps the spec
// honest about the structure actually being shipped.
import { type Locator, type Page } from '@playwright/test';
import { BasePage } from './base.page';
import { SEL } from './selectors';
import { PATHS } from '../paths';

export class OverviewPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.overview);
  }

  /** Every panel is a landmark named by its heading. */
  section(label: string): Locator {
    return this.page.locator(`section[aria-label="${label}"]`);
  }

  rangeLink(label: string): Locator {
    return this.page.getByRole('link', { name: label, exact: true }).first();
  }

  activeRange(): Locator {
    return this.page.locator('.sp-p-overview__range[aria-current="page"]');
  }

  kpiTiles(): Locator {
    return this.page.locator(`${SEL.kpiGrid} ${SEL.kpi}`);
  }

  async kpiLabels(): Promise<string[]> {
    const raw = await this.page.locator(SEL.kpiLabel).allInnerTexts();
    return raw.map((t) => t.trim());
  }

  rowsIn(sectionLabel: string): Locator {
    return this.section(sectionLabel).locator(SEL.tableRow);
  }

  /** The state badge on one MCP server's row, addressed by the server id. */
  mcpState(serverId: string): Locator {
    return this.rowsIn('MCP servers')
      .filter({ hasText: serverId })
      .locator(SEL.badge)
      .first();
  }

  /** The visible text of one column of a board, top row first. */
  async boardColumn(boardTitle: string, index: number): Promise<string[]> {
    return this.rowsIn(boardTitle).locator(`td:nth-child(${index})`).allInnerTexts();
  }
}
