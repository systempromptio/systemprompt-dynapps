# dynwork-routines

DynWork routines in Governance: a systemprompt extension that keeps each person's routine definitions (the scheduled
or recurring agent tasks DynWork runs) so all of their DynWork desktops share them, plus a summary of each routine's
runs, and serves the "My routines" page. It also keeps the library of scheduled task templates an admin publishes for
groups of people, and serves each person the templates they may see.

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
| `GET /templates` | `If-None-Match?` | | `200 {templates: [template], links: [link]}` with `ETag`, or `304` |
| `GET /templates/{id}/revisions/{revision}` | | | `200 template` |

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

A template route answers `404 not_found` for a malformed template id or revision, and for a revision the caller has
no task linked to (see below), as if it did not exist.

## Change hint

After a write that succeeds, the caller's user's other connected desktops get the relay control frame
`{"t":"routines.changed"}` (see the relay README, "Change hints"). The caller names its own relay peer with
`X-Dynwork-Peer` (32 hex digits) to be skipped. The hint is advisory: DynWork keeps polling the feed.

## Templates

An admin publishes **templates** of scheduled tasks; each person who may see one finds it in DynWork (Scheduled →
Templates) and activates it as a task of their own. The wire shape is fixed by the contract DynWork implements too.

- **A template** answers `{id, revision, mandatory, name, description, where, definition, locks, personal}`:
  `definition` is the template's part of a task (`instructions`, `agent?`, `model?`, `schedule.cron`, `options`;
  the option defaults `recover` and `catchUp` are passed through as written),
  `locks` what the person may not change (`model`, `schedule`: `free` | `time` | `locked`, `timeLimitMax`) and
  `personal` what activation asks for (`folder`: `none` | `required`, `hint`). `where` is `local`. A template never
  carries a timezone, a folder, a task id or a device: those are the person's, and a body that names one is refused.
- **Who sees it.** A template names one or more anchor plugins. The caller sees a published template when core's
  access resolution admits at least one of them for the caller: `allowed_ids(EntityKind::Plugin, …)` with the roles
  `find_user_access_profile` reads, the services' subject attributes and the plugin → marketplace parent chain of the
  services config, the resolution the bridge manifest is filtered with. Access is managed where it always is
  (`rules.yaml`, `/admin/access`, a group's Access tab); there are no template groups.
- **`GET /templates`.** `templates` holds the published templates the caller sees, mandatory ones included, and each
  closed template the caller has a live task of (so that task's desktop can read its locks and merge its revisions);
  `links`
  holds one `{id, state, retire}` per template the caller has a live task of, seen or not. `state` is `published`,
  `closed` (no new activations; linked tasks keep following it), `retired` (`retire` says what that did to the task:
  `unlink`, `pause` or `remove`) or `no_access` (none of its anchor plugins admits the caller any more). The `ETag`
  is a hash of the answer; `If-None-Match` with it answers `304` and no body.
- **`GET /templates/{id}/revisions/{revision}`.** That revision as it was published, the base of the desktop's
  three-way merge, only for a caller who has or had (a tombstone counts) a task linked to the template.
- **The link.** `dynwork_routines.template_id` and `template_revision` name the template a task was activated from;
  a person has at most one live task per template (a unique partial index).
- **The library** (`library::save_draft`, `library::publish`, for the admin console and the YAML sync). Only a
  Governance admin writes it: the author's roles, read with `find_user_access_profile`, must grant management
  (`roles_grant_manage`). A draft is kept at revision 0 until it is published. A published template changes only by
  publishing again: each publish adds one to the revision and keeps the published copy in the history with its
  author; a closed template stays closed, a retired one takes no new revision. Publishing refuses a template with no
  anchor plugin; a blank name, schedule, plugin or connector; the connector `"all"`; "Don't ask"
  (`permissions: "bypass"`) without connectors, without `personal.folder: "required"`, or on a mandatory template; a
  mandatory template that requires a folder; and a `timeLimit` above `locks.timeLimitMax`.
- **Change hint.** Each publish sends `{"t":"templates.changed"}` to every connected desktop, of every person
  (`dynwork_relay::hint::publish_to_all`). Advisory: DynWork also reads `GET /templates` on its regular sync.

## "My routines" page

`/admin/routines`, in the admin shell, under Account in the sidebar, for every signed-in person (it is in the
non-admin account paths). It lists the person's live routines (name, schedule, the computer that runs it, on or
paused, the last run) and up to five recent runs each, with Pause or Resume and Delete. Each action is a form post to
`/admin/routines/{id}/pause`, `/resume` or `/delete` that writes a new revision (a delete needs no `If-Match`), sends
the change hint to all of the person's desktops and returns with a flash. There is no create or edit: that is
DynWork's form. The routes are in `page::routes`, merged by the admin router over the write pool.

## Tables

`schema/01_dynwork_routines.sql` and `schema/02_dynwork_routine_templates.sql`; migrations go in
`schema/migrations/NNN_<name>.sql` (`001_routine_templates` adds the template link to `dynwork_routines`).

- `dynwork_routines`: keyed by `(user_id, id)`; revision, `change_seq` (the feed cursor, unique per user), owner
  device and name, the definition (JSONB), `deleted_at`, `updated_at`. Deletes are tombstones with the definition
  emptied.
- `dynwork_routine_runs`: keyed by `(user_id, id)`; routine id, state, reason, started, ended, updated.
- `dynwork_routine_templates`: keyed by the template id; revision (0 for a draft), state (`draft`, `published`,
  `closed`, `retired`), retire mode (set exactly when retired), mandatory, anchor plugins (`TEXT[]`, never empty),
  source (`console` or `yaml`), the body (JSONB), created and updated by, updated at.
- `dynwork_routine_template_revisions`: keyed by `(template_id, revision)`; mandatory, the body as published, who
  published it and when.

Routines and runs cascade from `users`; runs cascade from their routine; revisions cascade from their template.

## Tests

In the `tests/` workspace, as everywhere in this repo:

- `tests/integration/dynwork-routines`: the router over a throwaway database with real RS256 desktop tokens —
  concurrency, the feed under racing writers, tombstones, user isolation, run ownership, the page's writes, the
  change hint through the relay's listener, and the template library (publish validation, revisions, admin-only
  writes), `GET /templates` under real access rules (allow, deny, two anchors), `links`, the `ETag`, the revision
  route's authorization and the `templates.changed` hint to every desktop;
- `tests/unit/dynwork-relay/src/hint.rs`: the hint frame and which desktops get it.
