// The measured bar: what a dense page actually looks like, in numbers.
//
// "Information-dense" is the whole point of this console and the easiest claim
// to lose quietly — a row grows two pixels a week and nobody notices until half
// the table is below the fold. These are the measurements that catch that, taken
// in the browser against the real rendered page rather than against the CSS.
//
// Every selector here is the design system's. If one is renamed, the metric it
// feeds silently reads zero and the bar passes — so they are named once, in
// SEL, and read from there.
import type { Page } from '@playwright/test';
import { SEL } from './pages/selectors';

export interface DensityMetrics {
  /** Table rows lying wholly inside the viewport. */
  rowsVisible: number;
  /** Rows present in the DOM, so a thin dataset is distinguishable from a sparse layout. */
  rowsTotal: number;
  /** Height of the first row, the density figure itself. */
  rowH: number | null;
  /** Height of the KPI strip. */
  kpiH: number | null;
  /** Cells whose content visibly spills out of their own box. */
  overflowCells: number;
  /** Adjacent first-row cells whose rectangles intersect. */
  overlap: number;
  /** Scroll containers wider than themselves. */
  hscroll: number;
  bodyScroll: boolean;
  tabs: number;
  dupTabs: boolean;
  /** Sub-labels reading a bare "0", which is a placeholder rather than a fact. */
  zeroSubs: number;
  /** Stacked filter rows. Two is a toolbar and a scope row; three is a wall. */
  filterRows: number;
  crumbs: boolean;
  noHeader: boolean;
}

export async function densityMetrics(page: Page): Promise<DensityMetrics> {
  return page.evaluate((sel) => {
    const viewportHeight = window.innerHeight;
    const wholly = (el: Element) => {
      const b = el.getBoundingClientRect();
      return b.top >= 0 && b.bottom <= viewportHeight;
    };
    const rows = Array.from(document.querySelectorAll(sel.tableRow));
    const cells = Array.from(document.querySelectorAll(`${sel.tableEl} td`));
    const firstRowCells = Array.from(
      document.querySelectorAll(`${sel.tableEl} tbody tr:first-child td`),
    );
    let overlap = 0;
    for (let i = 1; i < firstRowCells.length; i += 1) {
      const a = firstRowCells[i - 1].getBoundingClientRect();
      const b = firstRowCells[i].getBoundingClientRect();
      if (a.right > b.left + 2) overlap += 1;
    }
    const kpi = document.querySelector(sel.kpiGrid);
    const tabs = Array.from(document.querySelectorAll('[role=tab]')).map((t) =>
      t.textContent?.trim(),
    );
    return {
      rowsVisible: rows.filter(wholly).length,
      rowsTotal: rows.length,
      rowH: rows[0] ? Math.round(rows[0].getBoundingClientRect().height) : null,
      kpiH: kpi ? Math.round(kpi.getBoundingClientRect().height) : null,
      // A cell clamped with overflow:hidden and an ellipsis has scrollWidth
      // greater than clientWidth by design — that is the truncation working,
      // not a defect. Only a cell whose overflow is visible actually spills its
      // content across its neighbour, so the computed overflow decides.
      overflowCells: cells.filter((td) => {
        const bigger =
          td.scrollWidth > td.clientWidth + 2 || td.scrollHeight > td.clientHeight + 2;
        if (!bigger) return false;
        const style = getComputedStyle(td);
        return style.overflow === 'visible' || style.overflowX === 'visible';
      }).length,
      overlap,
      hscroll: Array.from(document.querySelectorAll(`${sel.tableScroll}, .sp-shell__main`)).filter(
        (el) => el.scrollWidth > el.clientWidth + 2,
      ).length,
      bodyScroll: document.documentElement.scrollWidth > document.documentElement.clientWidth,
      tabs: tabs.length,
      dupTabs: new Set(tabs).size < tabs.length,
      zeroSubs: Array.from(
        document.querySelectorAll('.sp-kpi__note, .sp-kpi__delta, .sp-section__count'),
      ).filter((e) => e.textContent?.trim() === '0').length,
      filterRows: document.querySelectorAll(
        '.sp-toolbar, .sp-scope-filter, .sp-filter-ribbon, form.sp-filters',
      ).length,
      crumbs: !!document.querySelector(`${sel.breadcrumb}, nav[aria-label=Breadcrumb]`),
      noHeader: !document.querySelector(sel.pageHeader),
    };
  }, {
    tableRow: SEL.tableRow,
    tableEl: SEL.tableEl,
    tableScroll: SEL.tableScroll,
    kpiGrid: SEL.kpiGrid,
    breadcrumb: SEL.breadcrumb,
    pageHeader: SEL.pageHeader,
  });
}
