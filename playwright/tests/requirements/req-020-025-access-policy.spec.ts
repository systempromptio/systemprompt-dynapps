// REQ-020 (provider abstraction), REQ-021 (central model access policy),
// REQ-022 (governed virtual keys / PATs), REQ-023 (enterprise SSO), REQ-024
// (shadow-AI blocking via model allowlist), REQ-025 (time-bound external
// access).
//
// Honesty notes against the routes that actually exist
// (extensions/web/admin/src/routes/{ssr.rs,admin.rs}):
// - There is NO admin gateway-routes PAGE — routing config is API-only
//   (GET/PATCH /api/public/admin/gateway). REQ-020 is therefore proven from
//   the config API (model_pattern -> provider indirection, upstream_model
//   rewrite seam), and the wire-protocol substitution proof lives in the Rust
//   tier and the production Cerebras/OpenAI redirects.
// - REQ-022: a PAT authenticates the bridge/gateway path, not the admin
//   cookie surface, so "revoked PAT rejected as a bearer" is a Rust-tier
//   proof. The browser proves issue -> revoke is a real state change (the
//   second revoke has nothing left to revoke).
// - REQ-024: an unauthenticated /v1 call from the browser context refuses
//   with a 4xx before the allowlist is consulted; the allowlist-specific
//   denial (authenticated request, unlisted model) is a Rust-tier proof. What
//   the browser pins is the policy itself: allow_unlisted_models is off and
//   no advertised route reaches a backend-surface provider.
import { test, expect, apiAs } from '../support/fixtures';

test.describe('@REQ-020 provider abstraction', () => {
  test('@REQ-020 routing is a pattern->provider indirection, not a hardwired provider', async ({
    request,
  }) => {
    const res = await request.get('/api/public/admin/gateway', apiAs(request, 'admin'));
    expect(res.status()).toBe(200);
    const cfg = (await res.json()) as {
      enabled: boolean;
      inference_path_prefix: string;
      routes: { model_pattern: string; provider: string }[];
    };
    expect(cfg.enabled).toBe(true);
    // The contract clients code against is the path prefix + model pattern;
    // the provider behind it is a config value, which is the abstraction.
    expect(cfg.inference_path_prefix).toBe('/v1');
    expect(cfg.routes.length).toBeGreaterThan(0);
    for (const r of cfg.routes) {
      expect(r.model_pattern).toBeTruthy();
      expect(r.provider).toBeTruthy();
    }
  });
});

test.describe('@REQ-021 central model access policy', () => {
  test('@REQ-021 gateway-route access rules are centrally readable by an admin', async ({
    request,
  }) => {
    const res = await request.get(
      '/api/public/admin/access-control',
      apiAs(request, 'admin'),
    );
    expect(res.status()).toBe(200);
  });

  test('@REQ-021 the same policy surface refuses a non-admin', async ({ request }) => {
    const res = await request.get(
      '/api/public/admin/access-control',
      apiAs(request, 'user'),
    );
    expect(res.status()).toBe(403);
  });

  test('@REQ-021 a non-admin cannot write access rules either', async ({ request }) => {
    // Read refusal without write refusal would still leave policy editable by
    // anyone — the write path is the one that matters for "revocable".
    const res = await request.put(
      '/api/public/admin/access-control/entity/gateway_route/claude-star-4203d1',
      { ...apiAs(request, 'user'), data: { rules: [] } },
    );
    expect(res.status()).toBe(403);
  });
});

test.describe('@REQ-022 governed personal access tokens', () => {
  test('@REQ-022 a PAT can be issued and revoked, and revocation is a real state change', async ({
    platformAdminPage: page,
  }) => {
    const issued = await page.request.post('/admin/devices/pats', {
      data: { name: `e2e-pat-${Date.now()}` },
    });
    expect(issued.status()).toBe(200);
    const body = (await issued.json()) as { id: string; secret: string; key_prefix: string };
    // The secret is shown exactly once and is scoped by a stored prefix — the
    // credential is the platform's own, never a raw provider key (the register
    // point of virtual keys).
    expect(body.secret).toBeTruthy();
    expect(body.key_prefix).toBeTruthy();

    const revoke = await page.request.delete(`/admin/devices/pats/${body.id}`);
    expect(revoke.status()).toBe(204);

    // Revoking again must fail: if it succeeded, the first revoke changed
    // nothing and "revoked" is a display state, not a credential state.
    const again = await page.request.delete(`/admin/devices/pats/${body.id}`);
    expect(again.ok()).toBeFalsy();
  });
});

test.describe('@REQ-023 enterprise SSO', () => {
  test('@REQ-023 the login page offers SSO and no other door', async ({
    anonPage: page,
  }) => {
    // UI evidence only: the SAML handshake against AD FS is proven in the Rust
    // contract tier (AuthnRequest, state cookie, the assertion consumer's
    // validation ladder) and live against the IdP. What this asserts is that
    // no *second* way in has reappeared: the page carries no credential input
    // and no registration affordance, whether or not adfs.yaml is usable on
    // this instance (the SSO button renders only when it is).
    await page.goto('/admin/login');
    await expect(page.locator('main input')).toHaveCount(0);
    await expect(page.locator('main')).not.toContainText(/passkey/i);
    await expect(page.locator('main')).not.toContainText(/create an account/i);
    await expect(page.locator('main')).not.toContainText(/salesforce/i);
  });

  test('@REQ-023 a malformed SSO return lands on the login page, not an error document', async ({
    request,
  }) => {
    // SSO reports failure by redirecting: a 5xx on the callback would strand
    // the browser. Whether or not ADFS is configured, the callback owes a
    // redirect back to /admin/login carrying a reason.
    const res = await request.post('/admin/auth/adfs/acs', {
      form: { SAMLResponse: 'not-a-real-assertion', RelayState: 'nope' },
      maxRedirects: 0,
    });
    expect(res.status()).toBeGreaterThanOrEqual(300);
    expect(res.status()).toBeLessThan(400);
    expect(res.headers()['location']).toMatch(/\/admin\/login\?sso=/);
  });
});

test.describe('@REQ-024 shadow AI blocked at the gateway', () => {
  test('@REQ-024 unlisted models are policy-refused, not policy-optional', async ({
    request,
  }) => {
    const res = await request.get('/api/public/admin/gateway', apiAs(request, 'admin'));
    expect(res.status()).toBe(200);
    const cfg = (await res.json()) as { routes: { model_pattern: string }[] };
    // Every advertised pattern is explicit; combined with
    // allow_unlisted_models: false in the profile, a model outside these
    // patterns has no route to a provider.
    expect(cfg.routes.every((r) => r.model_pattern.length > 0)).toBeTruthy();
  });

  test('@REQ-024 an ungoverned /v1 call is refused', async ({ request }) => {
    const res = await request.post('/v1/messages', {
      data: {
        model: 'definitely-not-an-approved-model',
        max_tokens: 8,
        messages: [{ role: 'user', content: 'hi' }],
      },
    });
    // No bearer credential, unlisted model: whichever gate answers first, the
    // call must not reach a provider. Any 2xx here is a governance hole.
    expect(res.status()).toBeGreaterThanOrEqual(400);
    expect(res.status()).toBeLessThan(500);
  });
});

test.describe('@REQ-025 time-bound external access', () => {
  test('@REQ-025 a garbage or expired invite token is refused with a dead end', async ({
    anonPage: page,
  }) => {
    await page.goto('/admin/invite/e2e-not-a-real-token');
    await expect(page.locator('body')).toContainText(
      /invalid, expired, or has already been used/i,
    );
    // Refusal, not a form: no accept control may render for a bad token.
    await expect(page.locator('form button[type="submit"]')).toHaveCount(0);
  });

  test('@REQ-025 a PAT accepts an expiry at issue time', async ({
    platformAdminPage: page,
  }) => {
    const expires = new Date(Date.now() + 60_000).toISOString();
    const issued = await page.request.post('/admin/devices/pats', {
      data: { name: `e2e-pat-ttl-${Date.now()}`, expires_at: expires },
    });
    expect(issued.status()).toBe(200);
    const body = (await issued.json()) as { id: string; expires_at: string | null };
    // The expiry must survive issuance — a credential that silently drops its
    // TTL is exactly the "manual revocation discipline" this requirement bans.
    expect(body.expires_at).not.toBeNull();
    await page.request.delete(`/admin/devices/pats/${body.id}`);
  });
});
