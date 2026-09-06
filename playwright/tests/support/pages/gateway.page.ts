// The gateway route editor.
//
// Route order is the policy — the first matching pattern wins — so the row
// ordinal and the reorder controls are first-class here rather than incidental
// styling, and `routeIds()` returns them in dispatch order.
import { type Locator, type Page } from '@playwright/test';
import { BasePage } from './base.page';
import { PATHS } from '../paths';

export class GatewayPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.gateway);
  }

  routeRow(routeId: string): Locator {
    return this.page.locator(`tr[data-route-index][data-route-id="${routeId}"]`);
  }

  /** The declared route ids, in dispatch order.
   *
   *  Returns an empty list rather than throwing when the page is mid-navigation:
   *  a write reloads the page, and `expect.poll` around this call is how a spec
   *  waits for the new order. Letting the destroyed-context error escape would
   *  fail the poll on the reload it is waiting for.
   */
  async routeIds(): Promise<string[]> {
    try {
      return await this.page
        .locator('tr[data-route-index]')
        .evaluateAll((rows) => rows.map((r) => (r as HTMLElement).dataset.routeId ?? ''));
    } catch {
      return [];
    }
  }

  dialog(): Locator {
    return this.page.locator('#gateway-route-dialog');
  }

  async openNewRoute(): Promise<void> {
    await this.page.locator('[data-action="route-new"]').first().click();
    await this.dialog().waitFor({ state: 'visible' });
  }

  async submitRoute(fields: Record<string, string>): Promise<void> {
    for (const [name, value] of Object.entries(fields)) {
      await this.dialog().locator(`[name="${name}"]`).fill(value);
    }
    await this.dialog().getByRole('button', { name: /save route/i }).click();
  }

  routeError(): Locator {
    return this.page.locator('[data-route-error]');
  }

  async probe(userId: string): Promise<string> {
    await this.page.locator('#gw-probe-user').selectOption(userId);
    await this.page.locator('#gateway-probe').getByRole('button', { name: /probe/i }).click();
    const result = this.page.locator('[data-probe-result]');
    await result.locator('pre, :text("could not")').first().waitFor();
    return (await result.innerText()).trim();
  }
}
