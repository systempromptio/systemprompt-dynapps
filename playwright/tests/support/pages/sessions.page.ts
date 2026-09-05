// The sessions list and one session's detail page.
//
// Two objects rather than one: the list is a scoped, sorted, paginated table,
// and the detail page is four linked tables plus the two judgement panels
// (the stored AI analysis and the human ratings). They share only the shell.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export class SessionsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.sessions);
  }

  /** Every session id link in the table, in row order. */
  sessionLinks(): Locator {
    return this.page.locator(`${SEL.tableRow} a[href^="/admin/sessions/"]`);
  }

  /** Open the first session and return the detail object standing on it. */
  async openFirst(): Promise<SessionDetailPage> {
    await this.sessionLinks().first().click();
    await expect(this.page).toHaveURL(/\/admin\/sessions\/.+/);
    return new SessionDetailPage(this.page, this.page.url());
  }

  /** The errors-only KPI, which is also the filter it counts. */
  errorsKpi(): Locator {
    return this.page.locator(SEL.kpi).filter({ hasText: 'With errors' }).first();
  }

  /** The group and project columns, which name the exclusive attribution. */
  async attribution(): Promise<{ groups: string[]; projects: string[] }> {
    return {
      groups: await this.table().column('Group'),
      projects: await this.table().column('Project'),
    };
  }
}

export class SessionDetailPage extends BasePage {
  constructor(page: Page, path: string) {
    super(page, path);
  }

  /** The stored AI assessment panel, absent until the run has been summarised. */
  analysis(): Locator {
    return this.page.locator('.sp-p-session-detail__analysis');
  }

  verdictBadges(): Locator {
    return this.page.locator(`.sp-p-session-detail__verdict ${SEL.badge}`);
  }

  /** The four linked tables, addressed by their screen-reader caption. */
  tableCaptioned(caption: RegExp): Locator {
    return this.page.locator(SEL.table).filter({ has: this.page.locator('caption', { hasText: caption }) });
  }

  ratingsTable(): Locator {
    return this.tableCaptioned(/Human ratings/i);
  }

  requestsTable(): Locator {
    return this.tableCaptioned(/Requests in this session/i);
  }
}
