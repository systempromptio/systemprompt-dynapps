// Bridge-manifest helper.
//
// Currently unused: the manifest payload carries no `marketplaces` key, so
// group marketplace entitlement is asserted against the admin matrix and the
// marketplaces audience grid instead (tests/groups-access.spec.ts). Two further
// blockers stand in the way of driving it from a spec, both recorded here so
// the next attempt does not rediscover them:
//   * `POST /admin/devices/pats` sits behind the admin-console gate, so a
//     developer or business user is bounced with 303 and cannot mint a PAT.
//     The non-console path is `/admin/api/profile/bridge-code` followed by
//     `/v1/auth/bridge/session-pat` and `/v1/auth/bridge/pat`.
//   * that last exchange validates the subject as a UUID, and the e2e
//     principals use readable ids (`e2e-developer`), so it answers 400
//     "Invalid user UUID". Manifest coverage needs UUID-id seed principals.
import { readFileSync } from 'node:fs';
import { AUTH, type Principal } from './fixtures';

function jwtFor(principal: Principal): string {
  const state = JSON.parse(readFileSync(AUTH[principal], 'utf8')) as {
    cookies: { name: string; value: string }[];
  };
  const cookie = state.cookies.find((c) => c.name === 'access_token');
  if (!cookie) throw new Error(`no access_token cookie in ${AUTH[principal]}`);
  return cookie.value;
}

/** Issue a personal access token for a principal, or null if refused. */
export async function issuePat(baseURL: string, principal: Principal): Promise<string | null> {
  const res = await fetch(`${baseURL}/admin/devices/pats`, {
    method: 'POST',
    headers: { cookie: `access_token=${jwtFor(principal)}`, 'content-type': 'application/json' },
    body: JSON.stringify({ name: `e2e-groups-${principal}-${Date.now()}` }),
  });
  if (!res.ok || !res.headers.get('content-type')?.includes('json')) return null;
  const body = (await res.json()) as { secret?: string };
  return body.secret ?? null;
}

/** The raw signed manifest envelope for a principal, as text. */
export async function fetchManifest(baseURL: string, principal: Principal): Promise<string> {
  const pat = await issuePat(baseURL, principal);
  if (!pat) throw new Error(`could not issue a PAT for ${principal}`);
  const res = await fetch(`${baseURL}/v1/bridge/manifest`, {
    headers: { authorization: `Bearer ${pat}` },
  });
  if (!res.ok) throw new Error(`/v1/bridge/manifest returned ${res.status} for ${principal}`);
  return res.text();
}
