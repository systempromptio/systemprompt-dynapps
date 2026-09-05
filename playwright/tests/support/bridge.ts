// Bridge-manifest helper shared by the group entitlement specs.
//
// A signed manifest is the only place the *effective* marketplace entitlement
// of a principal is observable end to end: the admin API says what the rules
// are, the manifest says what the client actually receives. Both are asserted,
// because a rule that resolves correctly and still ships the wrong bundle is
// the failure mode this whole slice exists to prevent.
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
