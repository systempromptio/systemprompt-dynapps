// Group entitlement, end to end: the rule the admin API resolves and the
// bundle the client actually receives must agree.
//
// The seed gives each group exactly one marketplace, denied by default and
// allowed to that group alone, so every assertion here fails if the `group`
// ACL dimension stops resolving — an entitlement that silently widens is the
// failure this suite exists to catch.
import { test, expect, apiAs } from './support/fixtures';

const COMMERCE = 'astound-commerce-dev';
const EUROPE = 'astound-europe-dev';
const BUSINESS = 'astound-business';
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
  // e2e-user is in commerce-devs and in no project, so the allow can only come
  // from the group layer. (e2e-developer is in commerce-devs AND storefront,
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
    expect(row(matrix, 'marketplace', BUSINESS).effective).toBe('allow');
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

  test('each group is allowed exactly its own marketplace', async ({ adminPage: page }) => {
    await page.goto('/admin/catalog/marketplaces');

    const pairs: [string, string][] = [
      ['commerce-devs', COMMERCE],
      ['europe-devs', EUROPE],
      ['business-users', BUSINESS],
    ];
    for (const [group, own] of pairs) {
      await expect(page.locator(cell(group, own))).toHaveClass(/is-allow/);
      for (const other of [COMMERCE, EUROPE, BUSINESS].filter((m) => m !== own)) {
        await expect(page.locator(cell(group, other))).toHaveClass(/is-deny/);
      }
    }
  });

  test('the allow cell names the rule that granted it', async ({ adminPage: page }) => {
    await page.goto('/admin/catalog/marketplaces');
    await expect(page.locator(cell('commerce-devs', COMMERCE))).toHaveAttribute(
      'title',
      /group:commerce-devs allow/,
    );
  });

  test('the unassigned group reaches no group marketplace', async ({ adminPage: page }) => {
    await page.goto('/admin/catalog/marketplaces');
    for (const m of [COMMERCE, EUROPE, BUSINESS]) {
      await expect(page.locator(cell('unassigned', m))).toHaveClass(/is-deny/);
    }
    await expect(page.locator(cell('unassigned', BASELINE))).toHaveClass(/is-deny|is-allow/);
  });
});

test.describe('signed bridge manifest', () => {
  // The manifest payload carries `plugins`, `skills`, `agents` and
  // `artifacts` but no `marketplaces` key, so group MARKETPLACE entitlement is
  // not observable in the bundle a client receives — the admin matrix above is
  // the only surface that shows it. Reinstate these once the manifest carries
  // the resolved marketplace set (reported alongside this pass).
  test.fixme('the developer receives commerce and the baseline, not europe or business', async () => {});
  test.fixme('a business user receives business, not commerce', async () => {});
  test.fixme('an unassigned user receives the baseline marketplace only', async () => {});
});
