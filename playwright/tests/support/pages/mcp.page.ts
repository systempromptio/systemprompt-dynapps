// The MCP servers list and one server's detail page.
//
// The list's rows are keyed by the server id the runtime uses, which is not
// always an id the catalog declares — a server serving under an undeclared
// name still gets a row, and `row()` finds it the same way.
import { type Locator, type Page } from '@playwright/test';
import { BasePage } from './base.page';
import { PATHS } from '../paths';

export class McpListPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.mcp);
  }

  row(serverId: string): Locator {
    return this.page.locator(`tr[data-mcp-id="${serverId}"]`);
  }

  /** The status badge text for one server: Alive, Stale, Never connected, … */
  async statusOf(serverId: string): Promise<string> {
    return (await this.row(serverId).locator('.sp-badge').first().innerText()).trim();
  }

  serverIds(): Promise<string[]> {
    return this.page
      .locator('tr[data-mcp-id]')
      .evaluateAll((rows) => rows.map((r) => (r as HTMLElement).dataset.mcpId ?? ''));
  }
}

export class McpDetailPage extends BasePage {
  constructor(page: Page, readonly serverId: string) {
    super(page, PATHS.mcpServer(serverId));
  }

  /** One of the page's named tables, addressed by its section's label. */
  section(label: string): Locator {
    return this.page.locator(`section[aria-label="${label}"]`);
  }

  toolRows(): Locator {
    return this.section('Tools').locator('tbody tr');
  }

  executionRows(): Locator {
    return this.section('Call log').locator('tbody tr');
  }

  sessionRows(): Locator {
    return this.section('Sessions').locator('tbody tr');
  }

  grantRows(): Locator {
    return this.section('Access').locator('tbody tr');
  }
}
