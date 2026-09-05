// Page objects for the projects listing and one project's detail page.
//
// Everything the shell already owns — nav, breadcrumb, KPI tiles, tables,
// pagination, the confirm dialog — comes from BasePage. Only what is genuinely
// specific to projects lives here: the create dialog, the three detail tabs,
// and the two settings fields.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';

export class ProjectsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.projects);
  }

  row(projectId: string): Locator {
    return this.page.locator(`tr[data-project-id="${projectId}"]`);
  }

  search(): Locator {
    return this.page.locator('#project-search');
  }

  /** Type into the toolbar search and submit it, then wait for the re-render. */
  async searchFor(text: string): Promise<void> {
    await this.search().fill(text);
    await this.search().press('Enter');
    await this.page.waitForLoadState('networkidle');
  }

  /** Click a sortable column header by its label and return the resulting
   *  aria-sort. The shared TableHandle matches header text exactly, which a
   *  sort header defeats by carrying its direction indicator inside the cell.
   */
  async sortBy(label: string): Promise<string | null> {
    const th = this.page.locator('th').filter({ hasText: new RegExp(`^\\s*${label}`, 'i') }).first();
    await th.getByRole('link').click();
    await this.page.waitForLoadState('networkidle');
    return this.page
      .locator('th')
      .filter({ hasText: new RegExp(`^\\s*${label}`, 'i') })
      .first()
      .getAttribute('aria-sort');
  }

  /** One column's cell text, addressed by its header label. */
  async column(label: string): Promise<string[]> {
    const headers = await this.page.locator('th').allInnerTexts();
    const index = headers.findIndex((t) => t.trim().toLowerCase().startsWith(label.toLowerCase()));
    if (index < 0) throw new Error(`no column "${label}"; saw ${headers.join(' | ')}`);
    return this.page.locator(`tbody tr td:nth-child(${index + 1})`).allInnerTexts();
  }

  createDialog(): Locator {
    return this.page.locator('dialog[data-dialog="new-project"]');
  }

  async openCreate(): Promise<void> {
    await this.page.locator('[data-action="new-project"]').click();
    await expect(this.createDialog()).toBeVisible();
  }

  /** Fill and submit the create dialog. Does not wait for the redirect. */
  async create(id: string, name: string, description = ''): Promise<void> {
    await this.openCreate();
    await this.page.locator('[data-field="new-project-id"]').fill(id);
    await this.page.locator('[data-field="new-project-name"]').fill(name);
    await this.page.locator('[data-field="new-project-description"]').fill(description);
    await this.page.locator('[data-action="confirm-new-project"]').click();
  }
}

export type ProjectTab = 'members' | 'usage' | 'settings';

export class ProjectDetailPage extends BasePage {
  constructor(
    page: Page,
    readonly projectId: string,
  ) {
    super(page, PATHS.project(projectId));
  }

  /** Open one tab directly; every tab is its own URL. */
  async gotoTab(tab: ProjectTab): Promise<void> {
    await this.goto({ tab });
  }

  memberRow(userId: string): Locator {
    return this.page.locator(`tr[data-user-id="${userId}"]`);
  }

  sessionRows(): Locator {
    return this.page.locator('tr[data-session-id]');
  }

  groupChips(): Locator {
    return this.page.locator('.sp-group-chips__chip');
  }

  nameField(): Locator {
    return this.page.locator('[data-field="project-name"]');
  }

  descriptionField(): Locator {
    return this.page.locator('[data-field="project-description"]');
  }

  async save(): Promise<void> {
    await this.page.locator('[data-action="save-project"]').click();
  }

  async removeMember(userId: string): Promise<void> {
    await this.memberRow(userId).locator('[data-action="remove-member"]').click();
    await this.confirm();
  }
}
