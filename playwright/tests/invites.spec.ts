// Invites are a removed concept.
//
// The invite mint/list/revoke surface and `/api/public/admin/invites` were
// retired with the organization + seat spine (migration 041): the roster has no
// invite panel and every invite route answers 404. Onboarding is now directory
// driven — a user arrives through ADFS, lands in the derived `unassigned`
// group, and is granted membership from the user detail page (covered by
// user-membership.spec.ts) or the groups API (groups-pages.spec.ts).
//
// Kept as fixmes rather than deleted so the requirement stays visible if a
// time-bound external-access surface is reintroduced.
import { test } from './support/fixtures';

test.fixme('invites panel mints an invite from the UI', async () => {
  // Removed concept: no invite panel on /admin/access/users.
});

test.fixme('the invites API returns an invite_path with the raw token', async () => {
  // Removed concept: POST /api/public/admin/invites answers 404.
});

test.fixme('a second live invite for the same email conflicts', async () => {
  // Removed concept: POST /api/public/admin/invites answers 404.
});

test.fixme('non-admins cannot mint invites', async () => {
  // Removed concept: the route is gone, so 404 precedes any authz decision.
});
