// The demo-book recorder.
//
// A demo spec is a narrated walk through one section of the admin console: it
// asserts nothing, it drives the product in the order a person would and calls
// step() at each point worth showing. Each call screenshots the page and
// appends one record to playwright/demo/steps.jsonl, which scripts/demo-book.ts
// renders into a self-contained page.
//
// The login guard is the whole reason this is a helper rather than three lines
// in each spec: a demo book full of screenshots of the sign-in page looks
// plausible at a glance and is worthless, and that is exactly what a demo run
// against an unseeded or unauthenticated stack produces.
import { test, type Page } from '@playwright/test';
import { appendFileSync, mkdirSync } from 'node:fs';
import { basename, join } from 'node:path';

export const DEMO_DIR = join(__dirname, '..', '..', 'demo');
const SHOTS_DIR = join(DEMO_DIR, 'shots');
const STEPS_FILE = join(DEMO_DIR, 'steps.jsonl');

/** The six chapters of the book, in the order they are presented. */
export const SECTIONS = [
  'overview',
  'people-and-access',
  'ai-activity',
  'governance',
  'platform',
  'account',
] as const;

export type Section = (typeof SECTIONS)[number];

export interface StepMeta {
  /** What the reader is looking at, in one sentence. */
  what: string;
  /** What it does for them — the job it is on the page to do. */
  does: string;
  /** The design intent: why it looks and behaves the way it does. */
  design: string;
}

export interface StepRecord extends StepMeta {
  section: Section;
  title: string;
  url: string;
  shot: string;
  order: number;
}

// Why a per-step deadline rather than only a per-chapter one: chapters share a
// worker, so one step waiting on a selector that will never appear takes down
// every chapter after it in the same file — five authors lost their chapters to
// one bad selector, and none of them could see the cause from their own file.
// A chapter asserts nothing, so no step has a legitimate reason to wait three
// minutes for anything. Capping each step keeps the blast radius to the chapter
// that caused it and names it in the failure.
const STEP_DEADLINE_MS = 45_000;

async function withDeadline<T>(title: string, work: Promise<T>): Promise<T> {
  let timer: NodeJS.Timeout | undefined;
  const deadline = new Promise<never>((_, reject) => {
    timer = setTimeout(
      () =>
        reject(
          new Error(
            `demo step "${title}" exceeded ${STEP_DEADLINE_MS / 1000}s. A chapter asserts ` +
              `nothing, so a step should navigate and screenshot and little else — a wait ` +
              `this long means it is waiting for something that will never appear. Note the ` +
              `page may simply be erroring; check it in a browser before changing the step.`,
          ),
        ),
      STEP_DEADLINE_MS,
    );
  });
  try {
    return await Promise.race([work, deadline]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

const counters = new Map<string, number>();

function sectionOfCurrentFile(): Section {
  const file = basename(test.info().file).replace(/\.demo\.spec\.ts$|\.demo\.ts$/, '');
  const match = SECTIONS.find((s) => s === file);
  if (!match) {
    throw new Error(
      `demo spec ${file} does not name a section; rename it to one of: ${SECTIONS.join(', ')}`,
    );
  }
  return match;
}

function slug(section: string, title: string, order: number): string {
  const body = title.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  return `${section}-${String(order).padStart(2, '0')}-${body}`;
}

/** Run one narrated step: perform `fn`, then screenshot and record it. */
export async function step(
  page: Page,
  title: string,
  meta: StepMeta,
  fn: () => Promise<void>,
): Promise<void> {
  const section = sectionOfCurrentFile();
  await withDeadline(title, fn());
  await withDeadline(title, page.waitForLoadState('networkidle'));

  const url = new URL(page.url()).pathname;
  if (url.startsWith('/admin/login')) {
    throw new Error(
      `demo step "${title}" landed on ${url}: the principal is not signed in, so every ` +
        `figure after this one would show the login page. Re-run the seed and global setup.`,
    );
  }

  const order = (counters.get(section) ?? 0) + 1;
  counters.set(section, order);
  const name = slug(section, title, order);

  mkdirSync(SHOTS_DIR, { recursive: true });
  await page.screenshot({ path: join(SHOTS_DIR, `${name}.png`), fullPage: true });

  const record: StepRecord = { section, title, url, shot: `${name}.png`, order, ...meta };
  appendFileSync(STEPS_FILE, `${JSON.stringify(record)}\n`);
}
