// /admin/governance/approvals — the calls a policy is holding for a person.
//
// The queue is a waiting room, not a log: every pending row is a caller
// blocked on it right now. That is why the age column and the oldest-wait tile
// are asserted here, and why the decision buttons are gated on the admin role
// while the rows themselves are readable by any console role.
import {
  AUTH,
  test,
  expect,
  dbRow,
  snapshot,
  type Principal,
} from "../support/fixtures";
import { expectAccessible, expectDensity } from "../support/a11y";
import { PATHS } from "../support/paths";
import { SEL, TOKENS } from "../support/pages/selectors";
import { ApprovalsPage } from "../support/pages/approvals.page";

const PATH = PATHS.approvals;
const SEEDED = "e2e-dappr-000";

const ACCESS: { principal: Principal | "anon"; status: number }[] = [
  { principal: "platformAdmin", status: 200 },
  { principal: "admin", status: 200 },
  { principal: "pm", status: 200 },
  { principal: "developer", status: 303 },
  { principal: "businessUser", status: 303 },
  { principal: "unassigned", status: 303 },
  { principal: "user", status: 303 },
  // Anonymous is 307, not 303: the shell redirects a caller with no session
  // at all, before the gate that sends a signed-in non-console user to their
  // profile.
  { principal: "anon", status: 307 },
];

test.describe("renders", () => {
  test("lists the seeded pending requests", async ({ adminPage }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    await expect(page.row(SEEDED)).toBeVisible();
  });

  test("leads with what is pending and how long it has waited", async ({
    adminPage,
  }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto();
    expect(await page.kpi("Pending").number()).toBeGreaterThan(0);
    expect(await page.kpi("Oldest wait").value()).not.toBe("");
  });

  test("shows the arguments the approver is authorising", async ({
    adminPage,
  }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto();
    const args = await page.table().column("Arguments");
    expect(args.join(" ")).toContain("/etc/hosts");
  });
});

test.describe("actions", () => {
  // Deciding a row changes it for good, so these cases own the rows they touch
  // and run in order: the conflict case is the second decision on the first
  // case's row, which is the property being asserted.
  test.describe.configure({ mode: "serial" });

  const own = async (suffix: string): Promise<string> => {
    const callId = `e2e-spec-appr-${suffix}-${Date.now()}`;
    await dbRow(
      `INSERT INTO approval_requests
           (call_id, tool_name, server_name, arguments, args_digest, requested_by,
            rule, status, expires_at)
       VALUES ($1, 'write_file', 'systemprompt', '{"path":"/etc/hosts"}', $1,
               'e2e-developer', 'require_approval:write_file', 'pending',
               NOW() + INTERVAL '1 hour')`,
      [callId],
    );
    return callId;
  };

  test("a project manager sees the queue but no decision buttons", async ({
    pmPage,
  }) => {
    const page = new ApprovalsPage(pmPage);
    await page.goto();
    await expect(page.readOnlyNotice()).toBeVisible();
    expect(await page.approveButton(SEEDED).count()).toBe(0);
  });

  let decided = "";

  test("an admin denies a held call and the row becomes Denied", async ({
    adminPage,
  }) => {
    decided = await own("deny");
    const page = new ApprovalsPage(adminPage);
    await page.goto();
    await page.denyButton(decided).click();
    await page.confirm(true, /deny/i);
    await expect(page.row(decided).locator(SEL.badge).first()).toHaveText(
      /denied/i,
    );
  });

  test("deciding an already-decided call is refused, not silently overwritten", async ({
    adminPage,
  }) => {
    const res = await adminPage.request.post(
      `/api/public/admin/approvals/${decided}/approve`,
    );
    expect(res.status()).toBe(409);
  });

  test("filtering by status narrows the queue", async ({ adminPage }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto({ status: "pending" });
    const statuses = await page.table().column("Status");
    for (const status of statuses) expect(status.trim()).toBe("Pending");
  });
});

test.describe("authorization", () => {
  for (const { principal, status } of ACCESS) {
    test(`${principal} gets ${status}`, async ({ browser }) => {
      const context = await browser.newContext(
        principal === "anon" ? {} : { storageState: AUTH[principal] },
      );
      const res = await context.request.get(PATH, { maxRedirects: 0 });
      expect(res.status()).toBe(status);
      await context.close();
    });
  }

  test("a project manager cannot decide through the API either", async ({
    pmPage,
  }) => {
    const res = await pmPage.request.post(
      "/api/public/admin/approvals/e2e-dappr-001/approve",
    );
    expect(res.status()).toBe(403);
  });

  test("an unknown call id is a 404, not a 500", async ({ adminPage }) => {
    const res = await adminPage.request.post(
      "/api/public/admin/approvals/no-such-call/approve",
    );
    expect(res.status()).toBe(404);
  });
});

test.describe("design language", () => {
  test("wears the shell", async ({ adminPage }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute("href", PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test("paints from design tokens", async ({ adminPage }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe("");
  });

  // Nothing in the seed or in these specs approves a call, so the approved
  // filter is the one status reliably empty on a fresh database.
  test("shows an empty state when a status matches nothing", async ({
    adminPage,
  }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto({ status: "approved" });
    await page.table().expectEmpty();
  });

  test("keeps a visible focus ring", async ({ adminPage }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto();
    await adminPage.keyboard.press("Tab");
    if (await adminPage.locator(".sp-skip-link:focus").count()) {
      await adminPage.keyboard.press("Tab");
    }
    const ring = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body) return null;
      const s = getComputedStyle(el);
      return {
        outline: `${s.outlineStyle} ${s.outlineWidth}`,
        shadow: s.boxShadow,
        tag: el.tagName,
      };
    });
    expect(ring, "nothing took focus on Tab").not.toBeNull();
    const hasOutline =
      !ring?.outline.startsWith("none") &&
      Number.parseFloat(ring?.outline.split(" ")[1] ?? "0") > 0;
    const hasShadow = !!ring?.shadow && ring.shadow !== "none";
    expect(
      hasOutline || hasShadow,
      `no focus ring on <${ring?.tag}>: ${JSON.stringify(ring)}`,
    ).toBe(true);
  });

  test("does not scroll horizontally at 1440 or 1024", async ({
    adminPage,
  }) => {
    const page = new ApprovalsPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(
        await page.hasHorizontalScroll(),
        `horizontal scroll at ${width}px`,
      ).toBe(false);
    }
  });

  test("meets the density bar", async ({ adminPage }) => {
    const page = new ApprovalsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await expectDensity(adminPage, "list");
  });

  test("has no serious or critical axe violations", async ({ adminPage }) => {
    const page = new ApprovalsPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test("matches the committed screenshot", async ({ adminPage }) => {
    const page = new ApprovalsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await snapshot(adminPage, "approvals");
  });
});
