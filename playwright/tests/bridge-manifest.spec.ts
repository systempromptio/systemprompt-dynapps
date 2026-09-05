// The signed bridge manifest, per entitlement band, down to the exact set.
//
// Every other access surface in the suite is a projection the admin reads;
// this is the bundle the client installs. Five UUID-keyed principals — one per
// band the marketplaces are cut along — each fetch their manifest with a PAT
// the seed minted, and the plugin, skill, MCP-server and marketplace sets are
// asserted for EQUALITY against setup/seed/entitlements.json. A set that grew
// is as much a failure as one that shrank: an entitlement that silently widens
// is the regression a marketplace exists to prevent.
//
// The expected table is itself checked against services/ here, so it cannot
// drift from the descriptors it summarises without this file going red.
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test, expect, cookieFor } from './support/fixtures';
import {
  bridgeToken,
  fetchManifest,
  seededPat,
  sortedUnique,
  stable,
  type BridgeRole,
  type ManifestPayload,
} from './support/manifest';
import { marketplaceIncludes, pluginIncludes, SERVICES } from './support/services-yaml';

const REPO = join(__dirname, '..', '..');

interface Entitlement {
  marketplaces: string[];
  plugins: string[];
  skills: string[];
  mcp_servers: string[];
}

const ENTITLEMENTS = JSON.parse(
  readFileSync(join(__dirname, '..', 'setup', 'seed', 'entitlements.json'), 'utf8'),
) as Record<BridgeRole, Entitlement> & { _comment?: string };

const ROLES: BridgeRole[] = ['europe', 'india', 'knowledgeWorker', 'unassigned', 'admin'];

// The knowledge-bank and salesforce servers carry their own role rules and are
// not reached through a marketplace's include list, so the YAML derivation
// below covers plugins -> skills only and leaves mcp_servers to the table.
test.describe('@REQ-marketplaces the expected-entitlements table', () => {
  test('lists, for every principal, exactly the skills its plugins declare', () => {
    const plugins = pluginIncludes();
    for (const role of ROLES) {
      const e = ENTITLEMENTS[role];
      const derived = e.plugins.flatMap((p) => {
        const plugin = plugins.get(p);
        if (!plugin) throw new Error(`${role}: plugin ${p} has no services/plugins descriptor`);
        return plugin.skills;
      });
      expect(sortedUnique(e.skills), `${role} skills`).toEqual(sortedUnique(derived));
    }
  });

  test('lists, for every principal, exactly the plugins its marketplaces include', () => {
    const marketplaces = marketplaceIncludes();
    for (const role of ROLES) {
      const e = ENTITLEMENTS[role];
      const derived = e.marketplaces.flatMap((m) => {
        const marketplace = marketplaces.get(m);
        if (!marketplace) throw new Error(`${role}: no services/marketplaces/${m}`);
        return marketplace.plugins;
      });
      // The admin plugin is a member of enterprise-demo but role-gated to
      // admins (services/access-control/roles.yaml), so it is the one member a
      // non-admin's marketplace includes and their manifest does not carry.
      const adminOnly = new Set(role === 'admin' ? [] : ['astound-admin']);
      expect(sortedUnique(e.plugins), `${role} plugins`).toEqual(
        sortedUnique(derived.filter((p) => !adminOnly.has(p))),
      );
    }
  });

  test('names only MCP servers that exist under services/mcp', () => {
    for (const role of ROLES) {
      for (const server of ENTITLEMENTS[role].mcp_servers) {
        expect(existsSync(join(SERVICES, 'mcp', `${server}.yaml`)), `${role}: ${server}`).toBe(true);
      }
    }
  });
});

function skillIds(payload: ManifestPayload): string[] {
  return sortedUnique(payload.skills.map((s) => s.id));
}

test.describe('@REQ-marketplaces the signed manifest per principal', () => {
  for (const role of ROLES) {
    test(`${role}: plugins, skills and MCP servers are exactly the entitled set`, async ({
      baseURL,
    }) => {
      const e = ENTITLEMENTS[role];
      const { payload } = await fetchManifest(baseURL!, seededPat(role).secret);

      expect(payload.user_id).toBe(seededPat(role).userId);
      expect(sortedUnique(payload.plugins.map((p) => p.id))).toEqual(sortedUnique(e.plugins));
      expect(skillIds(payload)).toEqual(sortedUnique(e.skills));
      expect(sortedUnique(payload.managed_mcp_servers.map((m) => m.name))).toEqual(
        sortedUnique(e.mcp_servers),
      );
    });

    test(`${role}: the marketplace set is exactly the entitled set`, async ({ baseURL }) => {
      const e = ENTITLEMENTS[role];
      const { payload } = await fetchManifest(baseURL!, seededPat(role).secret);

      if (payload.marketplaces === undefined) {
        // An older core omits the key. The plugin set is then the only
        // observable projection of marketplace membership, so assert that
        // transitively and say so in the report rather than passing silently.
        test.info().annotations.push({
          type: 'note',
          description:
            'manifest payload carries no `marketplaces` key (older core); asserted transitively via plugins',
        });
        const includes = marketplaceIncludes();
        const viaPlugins = e.marketplaces.flatMap((m) => includes.get(m)?.plugins ?? []);
        for (const p of payload.plugins) expect(viaPlugins).toContain(p.id);
        return;
      }
      expect(sortedUnique(payload.marketplaces.map((m) => m.id))).toEqual(
        sortedUnique(e.marketplaces),
      );
      // Each marketplace entry lists only plugins this principal received.
      const received = new Set(payload.plugins.map((p) => p.id));
      for (const m of payload.marketplaces) {
        for (const p of m.plugin_ids) expect(received, `${m.id} lists ${p}`).toContain(p);
      }
    });
  }

  test('a plugin outside the principal\'s set is not served, file by file', async ({ baseURL }) => {
    const pat = seededPat('europe').secret;
    const { payload } = await fetchManifest(baseURL!, pat);
    const granted = payload.plugins[0];
    expect(granted, 'the europe developer holds at least one plugin').toBeTruthy();
    const file = granted.files[0]?.path ?? '.claude-plugin/plugin.json';

    const token = await bridgeToken(baseURL!, pat);
    const own = await fetch(`${baseURL}/v1/bridge/plugins/${granted.id}/${file}`, {
      headers: { authorization: `Bearer ${token}` },
    });
    expect(own.status).toBe(200);

    // astound-india-sfcore is india-devs only; a europe developer never sees it.
    const foreign = await fetch(
      `${baseURL}/v1/bridge/plugins/astound-india-sfcore/.claude-plugin/plugin.json`,
      { headers: { authorization: `Bearer ${token}` } },
    );
    expect(foreign.status).toBe(404);
  });
});

test.describe('@REQ-marketplaces the non-console PAT path', () => {
  // A developer cannot open /admin/devices, so the desktop bridge signs in
  // through a connect code issued from the profile page. The manifest that
  // token receives must be the one the seeded PAT receives.
  test('europe developer: connect code -> device PAT -> bridge JWT -> same manifest', async ({
    baseURL,
  }) => {
    const issued = await fetch(`${baseURL}/admin/api/profile/bridge-code`, {
      method: 'POST',
      headers: { ...cookieFor('bridgeEurope'), 'content-type': 'application/json' },
    });
    expect(issued.status, await issued.clone().text()).toBe(200);
    const { code } = (await issued.json()) as { code: string };
    expect(code).toBeTruthy();

    const exchanged = await fetch(`${baseURL}/v1/auth/bridge/session-pat`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ code, device_name: 'e2e bridge-manifest' }),
    });
    expect(exchanged.status, await exchanged.clone().text()).toBe(200);
    const { pat } = (await exchanged.json()) as { pat: string };
    expect(pat).toMatch(/^sp-live-[0-9a-f]{12}\.[0-9a-f]{52}$/);

    const minted = await fetch(`${baseURL}/v1/auth/bridge/pat`, {
      method: 'POST',
      headers: { authorization: `Bearer ${pat}` },
    });
    expect(minted.status, await minted.clone().text()).toBe(200);
    const { token } = (await minted.json()) as { token: string };
    expect(token).toBeTruthy();

    const viaJwt = await fetchManifest(baseURL!, token);
    const viaSeed = await fetchManifest(baseURL!, seededPat('europe').secret);
    expect(stable(viaJwt.payload)).toEqual(stable(viaSeed.payload));
  });
});

test.describe('@REQ-marketplaces the Windows leg', () => {
  const exe = join(REPO, 'storage', 'files', 'downloads', 'astound-bridge-windows.exe');

  test('the Windows installer is downloadable from the gateway', async ({ baseURL }) => {
    test.skip(
      !existsSync(exe),
      `storage/files/downloads/astound-bridge-windows.exe is not in this checkout; nothing to serve`,
    );
    const res = await fetch(`${baseURL}/files/downloads/astound-bridge-windows.exe`, {
      method: 'HEAD',
    });
    expect(res.status).toBe(200);
  });

  test('a Windows bridge receives the same bundle as a Linux one', async ({ baseURL }) => {
    const pat = seededPat('europe').secret;
    const linux = await fetchManifest(baseURL!, pat, {
      'user-agent': 'astound-bridge/0.0.0 (linux)',
    });
    const windows = await fetchManifest(baseURL!, pat, {
      'user-agent': 'astound-bridge/0.0.0 (windows)',
    });
    expect(stable(windows.payload)).toEqual(stable(linux.payload));
  });

  test('/v1/bridge/latest answers for windows exactly as bridge_releases is configured', async ({
    baseURL,
  }) => {
    const gateway = readFileSync(join(SERVICES, 'ai', 'gateway.yaml'), 'utf8');
    const configured = /^\s*bridge_releases:\s*\S/m.test(gateway);
    const res = await fetch(`${baseURL}/v1/bridge/latest?platform=windows`, {
      headers: { authorization: `Bearer ${await bridgeToken(baseURL!, seededPat('europe').secret)}` },
    });
    if (!configured) {
      expect(res.status).toBe(404);
      expect(await res.text()).toBe('bridge releases are not configured on this gateway');
      return;
    }
    expect(res.status).toBe(200);
    const body = (await res.json()) as { version: string; sha256: string; size: number };
    expect(body.version).toMatch(/^\d+\.\d+\.\d+/);
    expect(body.sha256).toMatch(/^[0-9a-f]{64}$/);
    expect(body.size).toBeGreaterThan(0);
  });
});
