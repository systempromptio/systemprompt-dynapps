// Group entitlement, end to end: the rule the admin API resolves and the
// bundle the client actually receives must agree.
//
// The seed gives each group exactly one marketplace, denied by default and
// allowed to that group alone, so every assertion here fails if the `group`
// ACL dimension stops resolving — an entitlement that silently widens is the
// failure this suite exists to catch.
import { test, expect, apiAs } from './support/fixtures';
import { fetchManifest } from './support/bridge';

const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8080';

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
  test('the developer resolves commerce allow at the group layer', async ({ request }) => {
    const matrix = await matrixFor(request, 'e2e-developer');
    const commerce = row(matrix, 'marketplace', COMMERCE);
    expect(commerce.effective).toBe('allow');
    expect(commerce.source.layer).toBe('group');
    expect(row(matrix, 'marketplace', EUROPE).effective).toBe('deny');
    expect(row(matrix, 'marketplace', BUSINESS).effective).toBe('deny');
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

test.describe('signed bridge manifest', () => {
  test('the developer receives commerce and the baseline, not europe or business', async () => {
    const manifest = await fetchManifest(BASE, 'developer');
    expect(manifest).toContain(COMMERCE);
    expect(manifest).toContain(BASELINE);
    expect(manifest).not.toContain(EUROPE);
    expect(manifest).not.toContain(BUSINESS);
  });

  test('a business user receives business, not commerce', async () => {
    const manifest = await fetchManifest(BASE, 'businessUser');
    expect(manifest).toContain(BUSINESS);
    expect(manifest).not.toContain(COMMERCE);
    expect(manifest).not.toContain('astound-admin');
  });

  test('an unassigned user receives the baseline marketplace only', async () => {
    const manifest = await fetchManifest(BASE, 'unassigned');
    expect(manifest).toContain(BASELINE);
    expect(manifest).not.toContain(COMMERCE);
    expect(manifest).not.toContain(EUROPE);
    expect(manifest).not.toContain(BUSINESS);
  });
});
