# Seeding an existing subtree

Bringing a subtree that already exists into a territory, without moving it and
without touching a single body. **Dry run is the default.** Applying the report is a
separate step that needs explicit human confirmation, because the content belongs to
somebody else.

## The two-step shape

You cannot write values before the definition exists. So:

1. **Provision the definition** on **every node of the subtree** — `to_provision` in
   the report, which is all of them, not only the ones with children today. Merge
   with `merge_definition()`, never replace. Re-read each one with
   `assert_written()`.
2. **Write the values**, built by `activate_canonicity()`. Re-read again.

Skipping step 1, or doing it on only the current parents, loses data **in silence**:
the write returns `ok: true` and stores nothing.

## What the report says

`build_seed_report(subtree, territory, owner, verified_on, verified_by)`:

| Field | Meaning |
| --- | --- |
| `total` / `native` / `synced` | The real count. Report it; never hard-code it |
| `writes` | `{"id", "props"}` per article, built by `activate_canonicity()` |
| `to_provision` | Every node id — the definition goes on all of them |
| `skipped_roots` | Counted, provisioned, never written: a root cannot carry values |
| `skipped_managed` | Already canonical. Seeding twice changes nothing |
| `applied` | Always `False`. This is a report |

**Synced articles become `derived`** and never get an expiry: their importer
maintains them, and a date here would put a person on the hook for something they do
not control. Native ones become `decided` and get the annual floor. Everything
written is `verified`, because managed-and-not-verified retires the article from the
index.

## Never move anything

`assert_no_parent_change(writes)` before applying, always. A Git-imported subtree is
re-anchored by its importer on every sync — if the seed moved an article, the next
sync would undo the work and nobody would find out until a domain search came back
empty.

Territory membership is `canon_territory`, a **field**. That is the whole reason
imported content can belong to a territory while staying exactly where the importer
put it.

## Reporting it to a person

Show the three counts and the two skip lists before asking. "144 articles: 1 native
and 143 synced; 1 root skipped, 0 already managed" is a sentence somebody can say yes
to. "Seed Dyncloud?" is not.

If the count differs from what the requester expected, **say so and stop**. The
Dyncloud subtree went from 143 to 144 in two days; a mismatch usually means the
repository grew, but it can also mean you are looking at the wrong subtree.

## Dyncloud, specifically

Not seeded in this delivery, by decision. It would need the definition written to
**29 Git-synced articles**, and whether that survives the next sync of the
`Plataforma Dyncloud` repository is unverified. Its 143 synced articles stay indexed
exactly as they are today, outside canonicity.

Before seeding it, test the durability on **one** synced article: provision, re-read,
trigger a sync, re-read again.
