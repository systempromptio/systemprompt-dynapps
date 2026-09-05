// REQ-037 (data residency routing), REQ-038 (no-train / no-retain policy
// enforcement).
//
// Acceptance criteria: REQ-037 "routing rules can enforce that classified
// workloads remain within the configured jurisdiction/region/sovereign
// endpoint"; REQ-038 "policy can prevent classified requests from being routed
// to providers that do not satisfy the configured no-train/no-retain
// contractual requirement".
//
// What a browser can prove: the local profile's route `claude-star-4203d1`
// carries `requires: { no_retain: true }`, and the admin gateway API both
// surfaces that block and preserves it across an edit round-trip — the round-
// trip is the load-bearing assertion, because the admin UI does not edit
// `requires`, and an update path that dropped it would silently rewrite
// residency policy (the exact bug the opaque-passthrough fields in
// GatewayRouteView exist to prevent). The dispatch-side proof — a request to a
// provider whose governance block fails the `requires` predicate is DENIED —
// lives in the Rust tier against core's route resolution.
//
// There is no admin gateway-routes PAGE in extensions/web/admin/src/routes/
// (config is API-only), so the "routes page shows the requires block" evidence
// is asserted against the API, not a rendered page.
import { test, expect, apiAs } from '../support/fixtures';

interface RouteView {
  id: string;
  model_pattern: string;
  provider: string;
  requires?: { no_retain?: boolean } | null;
  [key: string]: unknown;
}

async function getRoutes(request: Parameters<typeof apiAs>[0]) {
  const res = await request.get('/api/public/admin/gateway', apiAs(request, 'admin'));
  expect(res.status()).toBe(200);
  const cfg = (await res.json()) as { routes: RouteView[] };
  return cfg.routes;
}

test.describe('@REQ-037 @REQ-038 residency and no-retain routing policy', () => {
  // services/ai/gateway.yaml no longer sets a route-level `requires:` block on
  // any route, and GatewayRouteView no longer carries the field, so there is
  // nothing to read back. The classification itself survives one level down as
  // `governance: { european, no_retain }` on each provider in
  // services/ai/providers.yaml, which has no admin API surface — the
  // dispatch-side proof (a request whose route requirement the provider fails
  // is DENIED) is the Rust tier's.
  test.fixme('@REQ-037 the anthropic route carries its no_retain requirement', async () => {});

  test.fixme('@REQ-038 the requires block survives an admin edit round-trip', async () => {});

  test('@REQ-038 a non-admin cannot rewrite routing policy', async ({ request }) => {
    const routes = await getRoutes(request);
    const idx = routes.findIndex((r) => r.id === 'claude-star-4203d1');
    expect(idx).toBeGreaterThanOrEqual(0);
    const res = await request.patch(`/api/public/admin/gateway/routes/${idx}`, {
      ...apiAs(request, 'user'),
      data: { ...routes[idx], provider: 'e2e-mock' },
    });
    expect(res.status()).toBe(403);

    // And the refusal must have refused: the route is untouched.
    const after = await getRoutes(request);
    expect(after[idx]?.provider).toBe('anthropic');
  });
});
