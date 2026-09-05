// The base page object every admin page spec extends.
//
// It owns the shell — nav, breadcrumb, heading, toasts, the confirm dialog —
// plus the two controls that appear on nearly every page (the group/project
// scope switcher and the time range). Page-specific objects add only what is
// genuinely unique to their page.
import { expect, type Locator, type Page } from '@playwright/test';
import { SEL } from './selectors';
import { TableHandle } from './table';

export class BasePage {
  constructor(
    readonly page: Page,
    readonly path: string,
  ) {}

  async goto(query: Record<string, string> = {}): Promise<void> {
    const qs = new URLSearchParams(query).toString();
    await this.page.goto(qs ? `${this.path}?${qs}` : this.path);
    await expect(this.heading()).toBeVisible();
  }

  nav(): Locator {
    return this.page.locator(SEL.nav);
  }

  activeNavItem(): Locator {
    return this.page.locator(SEL.navLinkActive);
  }

  breadcrumb(): Locator {
    return this.page.locator(SEL.breadcrumb);
  }

  heading(): Locator {
    return this.page.locator(SEL.heading).first();
  }

  toast(): Locator {
    return this.page.locator(SEL.toast).first();
  }

  emptyState(): Locator {
    return this.page.locator(SEL.empty).first();
  }

  tabs(): Locator {
    return this.page.locator(SEL.tab);
  }

  table(name?: string): TableHandle {
    return new TableHandle(this.page, name);
  }

  /** The KPI tile whose label matches `label`, as value + locator. */
  kpi(label: string): KpiHandle {
    return new KpiHandle(this.page, label);
  }

  /** Set the group/project scope and wait for the page to re-render. */
  async scope(value: string): Promise<void> {
    await this.page.locator(SEL.scopeSelect).first().selectOption(value);
    await this.page.waitForLoadState('networkidle');
  }

  /** Set the time range (`7d`, `30d`, …) and wait for the page to re-render. */
  async range(value: string): Promise<void> {
    await this.page.locator(SEL.rangeSelect).first().selectOption(value);
    await this.page.waitForLoadState('networkidle');
  }

  /** Answer the confirm dialog. Fails if no dialog is open. */
  async confirm(accept = true): Promise<void> {
    const dialog = this.page.locator(SEL.dialog);
    await expect(dialog).toBeVisible();
    await this.page.locator(accept ? SEL.dialogConfirm : SEL.dialogCancel).click();
    await expect(dialog).toBeHidden();
  }

  /** True when the document is wider than the viewport at the current size. */
  async hasHorizontalScroll(): Promise<boolean> {
    return this.page.evaluate(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
    );
  }
}

export class KpiHandle {
  readonly root: Locator;

  constructor(page: Page, readonly label: string) {
    this.root = page.locator(SEL.kpi).filter({ hasText: label }).first();
  }

  async value(): Promise<string> {
    return (await this.root.locator(SEL.kpiValue).innerText()).trim();
  }

  /** The KPI's value parsed as a number, with thousands separators and units stripped. */
  async number(): Promise<number> {
    const raw = (await this.value()).replace(/[^0-9.\-]/g, '');
    const n = Number(raw);
    if (Number.isNaN(n)) throw new Error(`KPI "${this.label}" is not numeric: ${await this.value()}`);
    return n;
  }
}
