# dynwork-routines

DynWork routines in Governance: a systemprompt extension that keeps each person's routine definitions (the scheduled
or recurring agent tasks DynWork runs) so all of their DynWork desktops share them, plus a summary of each routine's
runs, and serves the "My routines" page.

- **Governance is the source of truth for definitions.** Desktops read them from a change feed and write them with
  optimistic concurrency.
- **A routine runs on the computer that created it**, its owner device. Only that device reports its runs.
- **The conversation of a run never leaves that computer.** Governance stores a run's state, reason and timestamps
  only.

## Routes

All routes sit under `/api/dynwork/routines/v1`, under `/api/` for the same reason as the relay's: core's session
middleware would otherwise treat each call as a tracked site page.

| Method and path | Headers | Body | Answer |
| --- | --- | --- | --- |
| `GET /changes?since=<cursor>&limit=<n>` | | | `200 {routines: [routine], cursor, more}` |
| `PUT /routines/{id}` | `If-Match: <revision>` (absent = create); `X-Dynwork-Peer?` | `{owner_device, owner_name?, definition}` | `200 routine` with `ETag: "<revision>"` |
| `DELETE /routines/{id}` | `If-Match: <revision>` (required); `X-Dynwork-Peer?` | | `200 routine` (the tombstone) |
| `PUT /runs/{id}` | | `{routine_id, device, state, reason?, started_at?, ended_at?}` | `200 run` |

A **routine** is `{id, revision, owner_device, owner_name, definition, deleted, deleted_at, updated_at}`. A tombstone
has `deleted: true` and `definition: null`. A **run** is `{id, routine_id, state, reason, started_at, ended_at,
updated_at}`. Times are RFC 3339.

- **Ids.** A routine or run id is chosen by the desktop: 1 to 128 letters, digits, `-`, `_`, `.` or `:`. It is scoped
  to its user, so two people may use the same id.
- **Definition.** DynWork's `Definition`, stored as sent. It must be a JSON object with a non-empty `name` and a
  boolean `enabled`, at most 64 KiB; the page also reads `schedule.cron` and `schedule.timezone`.
- **Feed.** `since` is the `cursor` of the previous page (absent: from the start). `limit` defaults to 200, at most
  500. `more: true` means another page is ready now. The feed returns each routine's latest revision once, oldest
  change first, tombstones included. Every write takes its user's row lock and numbers its change one past the
  user's highest, so changes commit in cursor order and a reader never skips one that commits after it read.
- **Revisions.** A create starts at revision 1; each update, delete, pause or resume adds one. `If-Match` takes the
  bare number or an entity tag (`"3"`). Creating an id that exists (a tombstone included), updating or deleting a
  tombstone, or naming another revision answers `409` with the current row.
- **Owner.** `owner_device` is required on create and kept when an update omits it; an update that sends it moves
  the routine to that computer. `owner_name` is the computer's display name.
- **Runs.** `device` must equal the routine's `owner_device`. The device is declared by the caller, not proven: it
  keeps a person's other desktops from reporting runs by mistake, not a person from writing their own rows. A run id
  stays with the routine it was first written for. `state` is one of `queued`, `running`, `ok`, `late`, `failed`,
  `blocked`, `missed`, `skipped`, `cancelled`; `reason` is at most 500 characters.

### Credentials

The desktop presents its DynWork JWT in `Authorization: Bearer`, checked by the relay's own
`dynwork_relay::auth::verify_desktop`: audience `bridge` or `api`, user type `user` or `admin`, no `act`, no
`plugin_id`, no `purpose`, not expired. Every row read or written belongs to that token's user. Bodies are parsed
only after the token is accepted.

### Errors

JSON `{"error": <key>, "message": <text>}`:

| Status | `error` | When |
| --- | --- | --- |
| 400 | `bad_request` | a malformed body, cursor, limit, `If-Match` or `X-Dynwork-Peer`; a `DELETE` without `If-Match`; a create without `owner_device`; a definition without `name` or `enabled`, or over 64 KiB |
| 401 | `unauthorized` | a missing, forged, expired or wrong-audience token; a token that is not a signed-in person's; a deleted account |
| 403 | `not_owner_device` | a run reported by a device that is not the routine's owner |
| 404 | `not_found` | an id that is malformed or not the caller's; an update of a routine that never existed; a run of a deleted routine |
| 409 | `revision_conflict` | the revision check failed; the body also carries `current`, the routine as it is now |
| 409 | `run_conflict` | the run id already belongs to another routine |

## Change hint

After a write that succeeds, the caller's user's other connected desktops get the relay control frame
`{"t":"routines.changed"}` (see the relay README, "Change hints"). The caller names its own relay peer with
`X-Dynwork-Peer` (32 hex digits) to be skipped. The hint is advisory: DynWork keeps polling the feed.

## "My routines" page

`/admin/routines`, in the admin shell, under Account in the sidebar, for every signed-in person (it is in the
non-admin account paths). It lists the person's live routines (name, schedule, the computer that runs it, on or
paused, the last run) and up to five recent runs each, with Pause or Resume and Delete. Each action is a form post to
`/admin/routines/{id}/pause`, `/resume` or `/delete` that writes a new revision (a delete needs no `If-Match`), sends
the change hint to all of the person's desktops and returns with a flash. There is no create or edit: that is
DynWork's form. The routes are in `page::routes`, merged by the admin router over the write pool.

## Tables

`schema/01_dynwork_routines.sql`; migrations, when one is needed, go in `schema/migrations/NNN_<name>.sql`.

- `dynwork_routines`: keyed by `(user_id, id)`; revision, `change_seq` (the feed cursor, unique per user), owner
  device and name, the definition (JSONB), `deleted_at`, `updated_at`. Deletes are tombstones with the definition
  emptied.
- `dynwork_routine_runs`: keyed by `(user_id, id)`; routine id, state, reason, started, ended, updated.

Both cascade from `users`; runs cascade from their routine.

## Tests

In the `tests/` workspace, as everywhere in this repo:

- `tests/integration/dynwork-routines`: the router over a throwaway database with real RS256 desktop tokens —
  concurrency, the feed under racing writers, tombstones, user isolation, run ownership, the page's writes and the
  change hint through the relay's listener;
- `tests/unit/dynwork-relay/src/hint.rs`: the hint frame and which desktops get it.
