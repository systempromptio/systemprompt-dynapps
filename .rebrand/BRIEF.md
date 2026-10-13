# Dynapps rebrand — shared brief (read fully before editing)

Repo: /var/www/html/systemprompt-dynapps (branch `next`, forked from Astound 0.65.0).
Several agents edit this tree concurrently, each owning a DISJOINT file set. Edit ONLY the
files/dirs in your assignment. If you need a change outside your set, write it to
`.rebrand/handoff-<yourletter>.md` (path, line, exact change) and move on.

## Hard rules
- NO cargo / just / npm builds, tests, clippy, fmt. The orchestrator runs one validation pass.
- Never touch: `extensions/web/schema/migrations/**`, `tests/fixtures/schema/*.sql` BODIES
  (line 1 header is agent R's), `.sqlx/**`, `CHANGELOG.md`, `docs/ops/**`, `docs/status/**`,
  `Cargo.lock` files (R regenerates), `.rebrand/BRIEF.md`.
- sed is fine for ids, slugs, env prefixes, paths, binary names. Prose (CLAUDE.md, README,
  theme, homepage, skills, docs, UI copy) is rewritten by hand for Dynapps.
- Rust comments: do not write "renamed", "formerly", "previously", "was astound" (a gate
  rejects archaeology). `//!` must stay on line 1 of every .rs. Files max 300 lines.
- `// Why:` comments: none inside fn bodies unless naming an external system; max 6 per file.
- Do not add new image filenames under storage/files/images (asset-reachability gate); swap
  content under existing names.
- Keep `@RELEASE_VERSION@` tokens in docs. Keep `/admin/auth/adfs/*` route paths and the
  `adfs.yaml` filename (contract baseline), even though the IdP is now Entra ID.
- Commit nothing. Leave the working tree for the orchestrator.
- When done, write `.rebrand/done-<letter>.md`: files changed, anything skipped and why,
  handoffs, blockers for requirements.txt.

## Identifier table (verbatim)
| Astound | Dynapps |
|---|---|
| `Astound Digital` (display) | `Dynapps` |
| `astounddigital.com`, `www.astounddigital.com`, `astound.digital`, `sp-dev.astound.digital`, `astoundcommerce.com`, `astound.local` | `dynapps.es` / `www.dynapps.es` (email domains: `dynapps.es`, `wearedynapps.com`) |
| `astound.systemprompt.io` | `dynapps.systemprompt.io` |
| `@astounddigital` | `@dynapps` |
| `systemprompt-astound` (crate, repo, tarball, helm, image) | `systemprompt-dynapps` |
| `systemprompt_astound` | `systemprompt_dynapps` |
| `astound-test-common`/`astound_test_common` | already renamed to `dynapps-test-common`/`dynapps_test_common` |
| `astound-commons`, `astound-salesforce`, `astound-super-admin` | `dynapps-commons`, `dynapps-salesforce`, `dynapps-super-admin` (dirs already moved) |
| `astound-admin` (legacy id in tests/scripts), `astound-admin-reports` | `dynapps-admin`, `dynapps-admin-reports` |
| `astound-bridge`, `astound-bridge-*` assets | `dynapps-bridge`, `dynapps-bridge-*` |
| `Astound Bridge`, `AstoundBridge.app`, `AstoundBridgeSync`, `AstoundBridgeGui` | `Dynapps Bridge`, `DynappsBridge.app`, `DynappsBridgeSync`, `DynappsBridgeGui` |
| `com.astounddigital.bridge[-sync|-gui]` | `com.dynapps.bridge[-sync|-gui]` |
| `ASTOUND_*` (env, GitHub secrets: `ASTOUND_BUNDLE_SIGNING_SEED`, `ASTOUND_RELEASE_TOKEN`, `ASTOUND_STATS_PAT`, `ASTOUND_BRIDGE_*`) | `DYNAPPS_*` |
| secret key `astound_release_token` | `dynapps_release_token` |
| `~/.config/astound`, `astound-bridge.{toml,pat,oauth-client}`, `/etc/astound`, `.astound/`, workspace `Astound` | `~/.config/dynapps`, `dynapps-bridge.*`, `/etc/dynapps`, `.dynapps/`, `Dynapps` |
| `ghcr.io/astound-digital/systemprompt-astound[-base]` | `ghcr.io/systempromptio/systemprompt-dynapps[-base]` |
| `Astound-Digital/systemprompt-astound`, `systempromptio/systemprompt-astound` | `systempromptio/systemprompt-dynapps` |
| `ghcr.io/astound-digital/sfnext-claude-dev-kit`, `sfscore-claude-india-kit`, marketplaces `astound-india-dev`, `astound-europe-dev`, plugins `astound-india-ba`, `astound-india-sfcore`, bundles `sfnext`, `india`, `astound-ba` | REMOVED (delete the reference and whatever only existed for it) |
| Helm helpers `astound.*` | `dynapps.*` |
| `astound:analysis_report`, `astound:conversation_judge` hash seeds | `dynapps:analysis_report`, `dynapps:conversation_judge` |
| `login.astoundcommerce.com/adfs/...`, "Astound SSO", "Active Directory" | Microsoft Entra ID (`https://login.microsoftonline.com/<tenant-id>/saml2`), "Dynapps SSO (Microsoft Entra ID)" |
| `astound.my.salesforce.com`, Astound Salesforce org | removed; registry empty, placeholder `<my-domain>.my.salesforce.com` in docs |
| `astound-knowledge-uk-sales`, GCP project 148780456827, google_workspace MCP | REMOVED |
| `ed@systemprompt.io` as author/support | keep |
| KEEP AS IS: `astound.feedback`, `astound.feedback_retention`, `astound.conversations`, `astound.plugin_usage_events` producer keys and seeded ids inside migrations/fixtures and the Rust constants naming them (internal DB keys, not brand) | unchanged |

## Dynapps brand facts (from https://www.dynapps.es/en)
- Name: **Dynapps** (Spanish entity "Dynapps Spain", formerly Indaws). Europe's leading Odoo
  implementation partner: 280+ Odoo experts across 5 countries (Spain HQ Valencia; Belgium
  Geel HQ, Antwerp, Ghent, Namur; Netherlands Rotterdam, Tilburg; France Toulouse, Bayonne,
  Nantes, Strasbourg; Switzerland Bulle, Martigny). Services: implement / recover / run &
  evolve Odoo; customise & extend, integrate, train, host, front-end, AI & automation.
  Sectors: construction, distribution, manufacturing, logistics, retail, food & beverage,
  health & pharma, professional services, green energy.
- Tagline: "Odoo, shaped around how you work."
- Meta: "Dynapps - Europe's leading Odoo implementation partner"; description "280+ Odoo
  experts across 5 countries. Industry-first Odoo implementation, migration, integration
  and support."
- Sites: dynapps.es, wearedynapps.com. Phone +34 960 20 29 42. Address Plaza de las Bandas de
  Música de la Comunidad Valenciana 7, 46013 Valencia.
- Colours: brand gradient `#7B00DC` (purple) → `#F13C42` (red) → `#FF911F` (orange);
  purple variants `#951CF3`, `#670BDF`; red/orange `#FF6C12`/`#FF8F1E`; body text `#535862`;
  near-white `#FAFAFA`. Primary accent for UI: red `#F13C42`; secondary purple `#951CF3`
  (a March-2026 Dynapps demo in this GitHub repo used the same pair). Font: **Readex Pro**
  (Google Fonts), fallback system sans.
- Assets in `brand/`: `dynapps-wordmark.svg` (152x32, gradient mark + `currentColor` text),
  `icon.svg` (512, embedded PNG), `icon.png` (96), `apple-icon.png` (180), `favicon.ico`.
- Ops contact for this instance: Ed (ed@systemprompt.io). Entra ID SSO wanted by customer.

## What this instance is
A systemprompt.io gateway/governance instance for Dynapps staff: Claude Code / Cowork
through a governed gateway, skills marketplaces, admin console. 0.65.0 core from crates.io.
Deployed as a systemprompt.io cloud tenant (Fly) — the hostname is `<hash>.systemprompt.io`
until `dynapps.systemprompt.io` is CNAMEd; write URLs as `https://dynapps.systemprompt.io`.

## Tenant (provisioned 2026-10-12)
Fly app `sp-e43527c1cbb5`, tenant hostname `https://e43527c1cbb5.systemprompt.io` (resolves
now). Canonical brand URL in docs/config: `https://dynapps.systemprompt.io` (DNS resolves to
the platform edge; routing to the tenant is confirmed by the orchestrator). Never reference
`astound.systemprompt.io` (that is Astound's live production).
