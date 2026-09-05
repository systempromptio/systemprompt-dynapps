// Deterministic clock, PRNG and id vocabulary shared by every seed module.
//
// Why a fixed T0 rather than SQL NOW(): every row the declarative dataset
// writes is placed at a known offset from the instant the seed started, so two
// runs an hour apart produce the same shape shifted in time. That is what lets
// a visual baseline, a demo-book figure and a KPI assertion all be written
// against exact numbers while the dashboards still show data that looks recent.
import type { Client } from 'pg';

export const T0 = new Date();

/** A timestamp `days` days and `hours` hours before T0. */
export function ago(days: number, hours = 0): Date {
  return new Date(T0.getTime() - days * 86_400_000 - hours * 3_600_000);
}

/** A timestamp `minutes` minutes before T0, for intra-session ordering. */
export function minutesAgo(minutes: number): Date {
  return new Date(T0.getTime() - minutes * 60_000);
}

/** mulberry32 — small, fast, and identical on every platform and Node version. */
export function prng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Zero-padded ordinal, so ids sort the way the rows were generated. */
export function pad(n: number, width = 3): string {
  return String(n).padStart(width, '0');
}

// Every id the declarative dataset owns. All start `e2e-` so the reset
// predicates (LIKE 'e2e-%') cover them without a second vocabulary.
export const ID = {
  session: (n: number) => `e2e-dsess-${pad(n, 2)}`,
  request: (n: number) => `e2e-dreq-${pad(n)}`,
  toolCall: (n: number) => `e2e-dtoolcall-${pad(n)}`,
  execution: (n: number) => `e2e-dtool-${pad(n)}`,
  context: (n: number) => `e2e-dctx-${pad(n, 2)}`,
  finding: (n: number) => `e2e-dfind-${pad(n)}`,
  decision: (n: number) => `e2e-dgov-${pad(n)}`,
  skill: (n: number) => `e2e-dskill-${pad(n)}`,
  approval: (n: number) => `e2e-dappr-${pad(n)}`,
  bridge: (n: number) => `e2e-dbridge-${pad(n, 2)}`,
  device: (n: number) => `e2e-ddevice-${pad(n, 2)}`,
  cert: (n: number) => `e2e-dcert-${pad(n, 2)}`,
  apiKey: (n: number) => `e2e-dkey-${pad(n, 2)}`,
  mcpSession: (n: number) => `e2e-dmcp-${pad(n, 2)}`,
};

/** True when `table` exists in the public schema. */
export async function hasTable(db: Client, table: string): Promise<boolean> {
  const { rows } = await db.query<{ n: number }>(
    `SELECT count(*)::int AS n FROM pg_class c
       JOIN pg_namespace ns ON ns.oid = c.relnamespace
      WHERE ns.nspname = 'public' AND c.relname = $1`,
    [table],
  );
  return (rows[0]?.n ?? 0) > 0;
}
