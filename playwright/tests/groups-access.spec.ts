// Group entitlement, end to end: the rule the admin API resolves and the
// bundle the client actually receives must agree.
//
// The seed gives each development group exactly one marketplace, denied by
// default and allowed to that group alone, so every assertion here fails if
// the `group` ACL dimension stops resolving — an entitlement that silently
// widens is the failure this suite exists to catch. The Cowork marketplace is
// the counter-example: it is ROLE-gated (`knowledge_worker`), as production
// declares it, so the business-users GROUP reaches nothing there.
import { test, expect, apiAs } from './support/fixtures';

const COMMERCE = 'astound-europe-dev';
const EUROPE = 'astound-india-dev';
const BUSINESS = 'astound-cowork';
const BASELINE = 'enterprise-demo';

interface MatrixRow {
  entity_id: string;
  effective: string;
  source: { layer: string; detail: string };
  default_included: boolean;
}
interface Matrix {
  sections: { entity_type: string; rows: MatrixRow[] }[];
}

function row(matrix: Matrix, entityType: string, entityId: string): MatrixRow {
  const section = matrix.sections.find((s) => s.entity_type === entityType);
  if (!section) throw new Error(`no ${entityType} section in the matrix`);
  const found = section.rows.find((r) => r.entity_id === entityId);
  if (!found) throw new Error(`no ${entityType} row for ${entityId}`);
  return found;
}

async function matrixFor(request: Parameters<typeof apiAs>[0], userId: string): Promise<Matrix> {
  const res = await request.get(
    `/api/public/admin/access-control/users/${userId}/matrix`,
    apiAs(request, 'platformAdmin'),
  );
  expect(res.status()).toBe(200);
  return (await res.json()) as Matrix;
}

test.describe('group marketplace entitlement', () => {
  // e2e-user is in europe-devs and in no project, so the allow can only come
  // from the group layer. (e2e-developer is in europe-devs AND storefront,
  // and the seed allows commerce to both — the project rule wins there, which
  // the next test pins.)
  test('a commerce group member resolves commerce allow at the group layer', async ({
    request,
  }) => {
    const matrix = await matrixFor(request, 'e2e-user');
    const commerce = row(matrix, 'marketplace', COMMERCE);
    expect(commerce.effective).toBe('allow');
    expect(commerce.source.layer).toBe('group');
    expect(row(matrix, 'marketplace', EUROPE).effective).toBe('deny');
    expect(row(matrix, 'marketplace', BUSINESS).effective).toBe('deny');
  });

  test('a project rule outranks the group rule that agrees with it', async ({ request }) => {
    const matrix = await matrixFor(request, 'e2e-developer');
    const commerce = row(matrix, 'marketplace', COMMERCE);
    expect(commerce.effective).toBe('allow');
    expect(commerce.source.layer).toBe('project');
    expect(commerce.source.detail).toContain('storefront');
  });

  test('a business user is denied the admin plugin at the group layer', async ({ request }) => {
    const matrix = await matrixFor(request, 'e2e-business-user');
    const admin = row(matrix, 'plugin', 'astound-admin');
    expect(admin.effective).toBe('deny');
    expect(admin.source.layer).toBe('group');
    // The Cowork allow is the knowledge_worker ROLE, not the business group.
    const cowork = row(matrix, 'marketplace', BUSINESS);
    expect(cowork.effective).toBe('allow');
    expect(cowork.source.layer).toBe('role');
    expect(cowork.source.detail).toContain('knowledge_worker');
    expect(row(matrix, 'marketplace', COMMERCE).effective).toBe('deny');
  });

  test('an unassigned user gets no group marketplace, and the default says so', async ({
    request,
  }) => {
    const matrix = await matrixFor(request, 'e2e-unassigned');
    for (const id of [COMMERCE, EUROPE, BUSINESS]) {
      const r = row(matrix, 'marketplace', id);
      expect(r.effective).toBe('deny');
      // No rule matches, so the deny comes from the entity default rather than
      // from any group/project/role rule.
      expect(r.source.layer).toBe('default');
    }
    expect(row(matrix, 'marketplace', BASELINE).effective).toBe('allow');
  });
});

test.describe('the audience matrix on the marketplaces page', () => {
  // The same resolution, rendered per subject rather than per user: one row
  // per group, one cell per marketplace, each cell carrying the rule that
  // decided it. This is the operator-facing proof of the group layer.
  const cell = (group: string, marketplace: string) =>
    `tr[data-subject="group:${group}"] td[data-marketplace="${marketplace}"]`;

  test('each development group is allowed exactly its own marketplace', async ({
    adminPage: page,
  }) => {
    await page.goto('/admin/marketplaces?tab=audience');

    const pairs: [string, string][] = [
      ['europe-devs', COMMERCE],
      ['india-devs', EUROPE],
    ];
    for (const [group, own] of pairs) {
      await expect(page.locator(cell(group, own))).toHaveClass(/is-allow/);
      for (const other of [COMMERCE, EUROPE, BUSINESS].filter((m) => m !== own)) {
        await expect(page.locator(cell(group, other))).toHaveClass(/is-deny/);
      }
    }
  });

  test('the Cowork marketplace is a role grant, so the business group reaches none', async ({
    adminPage: page,
  }) => {
    await page.goto('/admin/marketplaces?tab=audience');
    for (const m of [COMMERCE, EUROPE, BUSINESS]) {
      await expect(page.locator(cell('business-users', m))).toHaveClass(/is-deny/);
    }
    const roleCell = page.locator(
      `tr[data-subject="role:knowledge_worker"] td[data-marketplace="${BUSINESS}"]`,
    );
    await expect(roleCell).toHaveClass(/is-allow/);
    await expect(roleCell).toHaveAttribute('title', /role:knowledge_worker allow/);
  });

  test('the allow cell names the rule that granted it', async ({ adminPage: page }) => {
    await page.goto('/admin/marketplaces?tab=audience');
    await expect(page.locator(cell('europe-devs', COMMERCE))).toHaveAttribute(
      'title',
      /group:europe-devs allow/,
    );
  });

  test('the unassigned group reaches no group marketplace', async ({ adminPage: page }) => {
    await page.goto('/admin/marketplaces?tab=audience');
    for (const m of [COMMERCE, EUROPE, BUSINESS]) {
      await expect(page.locator(cell('unassigned', m))).toHaveClass(/is-deny/);
    }
    await expect(page.locator(cell('unassigned', BASELINE))).toHaveClass(/is-deny|is-allow/);
  });
});

// The bundle a client actually receives — the signed manifest, per principal,
// down to the exact plugin, skill and MCP-server set — is asserted in
// tests/bridge-manifest.spec.ts (@REQ-marketplaces) against the UUID-keyed
// bridge principals, because `/v1/auth/bridge/pat` cannot mint for the
// readable ids the matrix cases above use.
