// Global setup: fail fast on a dead stack, seed deterministic e2e data, mint
// one JWT per principal, and write storageState files the fixtures consume.
//
// It never boots a server (the stack may be shared with other agents on this
// clone — `just start` is the operator's job) and the minted cookies live only
// in the gitignored playwright/.auth/ directory.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { FullConfig } from '@playwright/test';
import { mintToken } from './mint-token';
import { startMockInference } from './mock-inference';
import { E2E, E2E_SESSIONS, seed } from './seed';

const REPO = join(__dirname, '..', '..');
const AUTH_DIR = join(__dirname, '..', '.auth');

export const AUTH = {
  admin: join(AUTH_DIR, 'admin.json'),
  platformAdmin: join(AUTH_DIR, 'platform-admin.json'),
  developer: join(AUTH_DIR, 'developer.json'),
  businessUser: join(AUTH_DIR, 'business-user.json'),
  unassigned: join(AUTH_DIR, 'unassigned.json'),
  pm: join(AUTH_DIR, 'pm.json'),
  user: join(AUTH_DIR, 'user.json'),
  bridgeEurope: join(AUTH_DIR, 'bridge-europe.json'),
  bridgeIndia: join(AUTH_DIR, 'bridge-india.json'),
  bridgeKnowledgeWorker: join(AUTH_DIR, 'bridge-knowledge-worker.json'),
  bridgeUnassigned: join(AUTH_DIR, 'bridge-unassigned.json'),
  bridgeAdmin: join(AUTH_DIR, 'bridge-admin.json'),
};

// The issuer must be the one the server under test verifies against, and this
// clone's profile is not always that server's profile: pointed at another
// worktree's port, reading jwt_issuer from here mints tokens a perfectly
// healthy server rejects with a 401 that says nothing about why.
//
// So: an explicit override wins, then the profile — but only when it names the
// same origin we are testing. Otherwise the base URL is the better guess,
// because jwt_issuer is the deployment's own address.
function resolveIssuer(baseURL: string): string {
  if (process.env.E2E_JWT_ISSUER) return process.env.E2E_JWT_ISSUER;
  try {
    const profile = readFileSync(
      join(REPO, '.systemprompt', 'profiles', 'local', 'profile.yaml'),
      'utf8',
    );
    const m = profile.match(/^\s*jwt_issuer:\s*(\S+)/m);
    if (m && new URL(m[1]).origin === new URL(baseURL).origin) return m[1];
  } catch {
    // fall through to the base URL
  }
  return new URL(baseURL).origin;
}

function storageState(baseURL: string, token: string): string {
  const host = new URL(baseURL).hostname;
  return JSON.stringify(
    {
      cookies: [
        {
          name: 'access_token',
          value: token,
          domain: host,
          path: '/',
          httpOnly: true,
          secure: false,
          sameSite: 'Lax',
          expires: Math.floor(Date.now() / 1000) + 2 * 60 * 60,
        },
      ],
      origins: [],
    },
    null,
    2,
  );
}

async function sanityCheck(baseURL: string, statePath: string, path: string, expect: number[]) {
  const state = JSON.parse(readFileSync(statePath, 'utf8'));
  const cookie = state.cookies[0];
  const res = await fetch(`${baseURL}${path}`, {
    headers: { cookie: `${cookie.name}=${cookie.value}` },
    redirect: 'manual',
  });
  if (!expect.includes(res.status)) {
    // A 404 is a different fault from a rejected token and wants a different
    // fix: the route moved or was never mounted on this build. Saying
    // "the key was rotated" about a missing page sends the reader to the wrong
    // place, which is how a stale path in this very check cost an afternoon.
    const cause =
      res.status === 404
        ? `that route does not exist on this build — the console's paths moved, so update this check`
        : `a token minted from ${process.env.E2E_SIGNING_KEY ?? 'signing_key.pem'} was rejected; ` +
          `if the key was rotated, restart the stack, and check E2E_JWT_ISSUER matches the server`;
    throw new Error(
      `auth sanity check failed: ${path} returned ${res.status}, expected ${expect.join('/')} — ${cause}.`,
    );
  }
}

export default async function globalSetup(config: FullConfig) {
  const baseURL =
    config.projects[0]?.use?.baseURL ?? process.env.GATEWAY_URL ?? 'http://localhost:8080';

  let health: Response;
  try {
    health = await fetch(`${baseURL}/health`);
  } catch {
    throw new Error(
      `E2E needs a running stack at ${baseURL}: run 'just start' (or set GATEWAY_URL). ` +
        `Not booting one here — the server may be shared with other agents.`,
    );
  }
  if (!health.ok) throw new Error(`health check at ${baseURL}/health returned ${health.status}`);

  // The mock lives in the runner process and dies with it; the gateway
  // reaches it through the `e2e-mock` provider in the local profile.
  await startMockInference();

  await seed();

  const jwks = (await (await fetch(`${baseURL}/.well-known/jwks.json`)).json()) as {
    keys: { kid: string }[];
  };
  const kid = jwks.keys[0]?.kid;
  if (!kid) throw new Error('no kid in /.well-known/jwks.json');

  const issuer = resolveIssuer(baseURL);
  const pemPath = process.env.E2E_SIGNING_KEY ?? join(REPO, 'signing_key.pem');

  mkdirSync(AUTH_DIR, { recursive: true });
  const principals = [
    { p: E2E.admin, out: AUTH.admin },
    { p: E2E.platformAdmin, out: AUTH.platformAdmin },
    { p: E2E.developer, out: AUTH.developer },
    { p: E2E.businessUser, out: AUTH.businessUser },
    { p: E2E.unassigned, out: AUTH.unassigned },
    { p: E2E.pm, out: AUTH.pm },
    { p: E2E.user, out: AUTH.user },
    // The bridge principals get a cookie too, for the spec that walks the
    // non-console PAT path from a signed-in profile page.
    { p: E2E.bridge.europe, out: AUTH.bridgeEurope },
    { p: E2E.bridge.india, out: AUTH.bridgeIndia },
    { p: E2E.bridge.knowledgeWorker, out: AUTH.bridgeKnowledgeWorker },
    { p: E2E.bridge.unassigned, out: AUTH.bridgeUnassigned },
    { p: E2E.bridge.admin, out: AUTH.bridgeAdmin },
  ];
  for (const { p, out } of principals) {
    const token = await mintToken({
      userId: p.id,
      email: p.email,
      issuer,
      kid,
      pemPath,
      sessionId: E2E_SESSIONS[p.id],
    });
    writeFileSync(out, storageState(baseURL, token));
  }

  // Fail loudly here rather than as thirty cryptic spec failures: the DB (not
  // the token) decides roles, so these three responses prove seeding + minting
  // + cookie extraction all line up.
  await sanityCheck(baseURL, AUTH.admin, '/api/public/admin/users', [200]);
  await sanityCheck(baseURL, AUTH.platformAdmin, '/api/public/admin/users', [200]);
  await sanityCheck(baseURL, AUTH.user, '/api/public/admin/users', [401, 403]);
  // The developer role is deliberately NOT a console role: it reads its own
  // profile and nothing else. The project manager IS a console role (read
  // only), which is what makes the read/write split on the group pages
  // observable at all.
  await sanityCheck(baseURL, AUTH.developer, '/api/public/admin/users', [401, 403]);
  await sanityCheck(baseURL, AUTH.developer, '/admin/users', [303, 302]);
  await sanityCheck(baseURL, AUTH.developer, '/admin/profile', [200]);
  await sanityCheck(baseURL, AUTH.pm, '/api/public/admin/groups', [200]);
  await sanityCheck(baseURL, AUTH.bridgeEurope, '/admin/profile', [200]);
}
