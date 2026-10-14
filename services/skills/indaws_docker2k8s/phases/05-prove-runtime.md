# Phase 5 — Prove the runtime before trusting it with data

> **docker2k8s — Phase 5 of 7.** Before starting: Phase 4 finished: parts, custom modules and Python dependencies all carried across, and the environments deployed with their deploy targets wired.
> The router, the safety rules and the Prerequisites live in `SKILL.md`.


**AUTO.** Every migrated environment must be healthy **on its generated demo database**
before any customer data goes near it. A failure here is a build or platform problem; a
failure after a restore is ambiguous, and ambiguity in a migration is expensive.

- `platform_env_status` — phase Running, no red health issues, queue not blocked
- `platform_job_runs` — nothing failed on the way up
- The URL serves Odoo, and the customer's modules are in the Apps list at the expected
  versions

**VERIFY the module list against Phase 4.** An environment that boots but is missing a
module has a `dyn-build.yaml` problem, and finding it now costs minutes.

**VERIFY the connector pair is installed:** `dyncloud_api` and `dyncloud_auth` on the
fresh environment. They arrive on their own — the values overlay lists them in
`odoo.requiredModules` and the initContainer installs them at boot — so their absence
here is a platform problem, not a migration one, and it is far cheaper to find now.

**VERIFY THE SIZING — it is how you find out you picked an old release.** The environment
was never tuned by hand, so every number below came from the release catalogue's per-stage
defaults plus the gitops scaffold. If they do not match, the environment was created on a
release older than the fleet standard (§ Prerequisites (`SKILL.md`)) and it is **cheaper to recreate it
now than to discover it in production**, oversized and paying for a pooler nobody asked
for. Read `values.yaml` in the environment's gitops repo:

| | production | staging |
|---|---|---|
| Odoo pod requests, three containers summed | **165m / 902Mi** | **65m / 646Mi** |
| `postgresql.internal.pooler.enabled` | `false` | `false` |
| `odoo.worker.dbMaxConn` | `10` | `10` |
| `postgresql.internal.parameters.max_connections` | `"100"`, **declared** | `"100"`, **declared** |

The last row is the one that reads as fine when it is not: an absent `max_connections` is
not an error, the database simply inherits the operator's default — and then the whole
connection budget rests on a number nobody chose, which a CloudNativePG upgrade can move
without anyone noticing. It must be present, and it must be a **quoted string**: the CNPG
`parameters` map is string-to-string and an integer there is rejected by the CRD.

A customer whose real load genuinely needs more gets it deliberately, through the Tune
wizard, **after** the migration is proven — not by inheriting it from a stale default.

**This check does not carry over to Phase 6.** A restore replaces this database — module
list included — with the customer's, and the same boot-time install puts the pair back on
the next start. What does NOT come back on its own is the removal of `asoo_connector`:
that is § 6.3 (`phases/06-move-data.md`).

`asoo_connector` must NOT be here either: if it is, the drop list of § 4.2 (`phases/04a-parts.md`)
was not applied.

