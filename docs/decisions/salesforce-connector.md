# Salesforce connector: one package, one connector, N orgs

**Status:** decided 2026-09-17 — Option A; stage 1 = Astound US (source org) + Astound UK, proven on the Developer Edition org first · **Date:** 2026-09-17 · **Supersedes in part:** the 2026-09-11
connector design ([artifact](https://claude.ai/code/artifact/5c590947-7438-44bd-a2d1-e25777c6c57a),
commit 1079d707)

## Decision required

Nothing in the Salesforce connector has reached production. Before the first org connects we
must choose its runtime shape. Two things prompted this:

1. The 2026-09-11 design assumes **one External Client App (ECA) per org**, hand-created by
   each org's administrator, with per-org secret names in `services/config/salesforce-orgs.yaml` and
   one `services/mcp/salesforce-<slug>.yaml` per org. Salesforce supports packaging an ECA so
   one app is installed into every org, which removes that per-org credential work.
2. The product framing was "Salesforce needs RAG". It does not: the agent queries Salesforce
   live through an MCP connector; a copied corpus adds staleness, a second permission model
   and an ingestion pipeline for structured data that is better queried than embedded.

Orgs in scope are **both** Astound-owned (production + sandboxes) **and** client-owned.

## Facts the decision rests on (verified 2026-09-17)

| Fact | Source | Consequence |
|------|--------|-------------|
| An ECA with distribution state *Packaged* can ship in a 2GP managed package. A subscriber org either **references the source org's OAuth settings** (same consumer key/secret/callbacks everywhere) or generates independent global settings. Deleting the app in the source org breaks every subscriber. Requires a Dev Hub. | [Package an External Client App](https://help.salesforce.com/s/articleView?id=xcloud.configure_packageable_external_client_apps.htm), [Trailhead](https://trailhead.salesforce.com/content/learn/modules/external-client-app-basics/package-and-distribute-external-client-apps) | One consumer key can serve N orgs. The Dev Hub org becomes production infrastructure. |
| Hosted MCP (`api.salesforce.com/platform/mcp/…/sobject-all`) accepts only authorization-code + PKCE via an ECA with scope `mcp_api`. Tokens are org-scoped. Salesforce states MCP "doesn't support multitenant systems like Salesforce", so the user must be logged into the target org before consent. | [Hosted MCP: create an ECA](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/create-external-client-app.html), [log into your org](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/log-into-org.html) | Org selection cannot be pushed past consent while we proxy the hosted server. |
| Astound owns **no** Salesforce MCP code. `services/mcp/salesforce.yaml` is `type: external` pointing at Salesforce's hosted `sobject-all`; core's external proxy injects a per-user bearer minted by `/api/public/connectors/<id>/token`. | `extensions/web/admin/src/services/salesforce_orgs.rs`, `connector_oauth/` | The tool surface is Salesforce's. Every org is a separate MCP server id. |
| The registry only stores secret **names**; nothing rejects two orgs naming the same secret. Only one org (`salesforce`, production) is declared. | `salesforce_orgs.rs::validate` | A shared client id needs no registry change. |
| Salesforce is not indexed by the knowledge bank; the docs already say "live query". | `services/content/documentation/enterprise-knowledge-rag.md` | "No Salesforce RAG" is a decision to record, not work to undo. |

## Adopt regardless of option

### One packaged ECA, not N hand-made apps

Astound builds a 2GP managed package from its Dev Hub containing:

- the ECA (`distributionState: Packaged`, global OAuth settings **referenced from the source
  org**, PKCE required, web-server flow, scopes `mcp_api refresh_token openid` — plus `api`
  if Option B is ever taken), callback URLs for each gateway (test, production);
- a permission set that grants the app and is assigned to the intended users;
- for JWT-bearer pre-authorisation: the certificate is *not* packaged (secrets never are); the
  subscriber admin uploads it after install if they want the non-interactive path.

Installing the package and assigning the permission set becomes the whole per-org operation.
Every `orgs.yaml` entry names the same `salesforce_mcp_client_id` / `salesforce_mcp_client_secret`.

Client-owned orgs install from the package URL (an AppExchange listing is not required). Their
security review is the gating step. If a client refuses to reference Astound's source-org
credentials, they generate independent global settings and hand over that consumer key — the
existing per-org secret names cover that case without code.

### No Salesforce RAG

Salesforce data is structured, permissioned by Salesforce, and changes minute to minute. The
agent gets it live through tools; the org remains the authority for both freshness and ACLs.
Confluence stays the only indexed corpus. This follows Anthropic's guidance that capable agents
should fetch context just-in-time via tools rather than pre-embedding everything, and the
pattern OpenAI documented for its internal data agent (live queries for current data, embeddings
only for static institutional docs). Cross-system questions ("what is happening with Acme?") are
answered by the agent calling Salesforce, Jira and Confluence tools in turn — retrieval, not RAG.

## Option A — Salesforce Hosted MCP behind the gateway (current code + package)

**How it works.** Unchanged proxy model. One MCP server id per org (`salesforce`,
`salesforce-<slug>`). The token broker keys on `(user, server id)`. The package replaces the
per-org ECA; the org's My Domain still hosts the authorize/token endpoints, so consent runs
against the org the server id names.

**Code delta (small).**

- One shared callback `/api/public/connectors/salesforce/callback` that recovers the real
  provider id from the sealed state (`connector_oauth/transport.rs::callback`,
  `handlers/connector_auth/mod.rs`). Otherwise the packaged ECA needs a callback URL added for
  every new org, which means a change on the source org — and whether that propagates to
  subscribers without a package version is unverified.
- `orgs.yaml` entries shrink to `label, my_domain, environment, groups[, org_id]`; secret names
  default to the shared pair (`SalesforceOrg` in `salesforce_orgs.rs`).

**Per-org onboarding.** Install package → assign permission set → activate hosted MCP server
(Setup → API Catalog) → Astound adds an `orgs.yaml` entry and `services/mcp/<id>.yaml` → deploy
or kit import → users press Connect.

**Pros.**

- No tool code to own; Salesforce maintains `sobject-all` and its least-privilege `mcp_api`
  scope, which is the easier story for a client security team.
- Per-org governance already exists: audit rows and `services/access-control/rules.yaml`
  address the server id (`allow.salesforce: [salesforce-<slug>]`), and
  `POST …/connectors/<id>/reprovision` handles sandbox refresh.
- The only new code is the shared callback.

**Cons.**

- An org still needs a config entry in this repo. "The org automatically appears" is not
  achievable: core's MCP server list is static YAML resolved at boot.
- No cross-org fan-out; the agent must call each org's server separately (it can, but every
  org's tools land in the client's tool list).
- Tool surface is whatever Salesforce ships; `mcp_api` does not reach metadata or DX.

## Option B — Astound-owned Salesforce MCP crate, org as a first-class argument

**How it works.** A new internal server `extensions/mcp/salesforce` exposing tools such as
`list_orgs`, `search(org, query)`, `query(org, soql)`, `get_record(org, id)`,
`list_cases(org, account)` and optionally `search_all_orgs(query)`, implemented over the
Salesforce REST API with scope `api refresh_token`. One server id. Consent starts at
`login.salesforce.com` / `test.salesforce.com`; after the exchange the gateway reads
`organization_id` and `instance_url` and inserts an org row — orgs **do** appear
automatically. Fan-out is limited to the orgs the caller has grants for.

**Code delta (large).**

- New crate, tool schemas, REST client, rate-limit handling, `.sqlx` cache.
- Credential store re-keyed from `(user, provider)` to `(user, org id)`; reprovision by org id;
  `salesforce_user_identities` and the `salesforce:` access-control band re-keyed to org ids.
- Governance: policy rules and audit today address the server id. The org would become a
  **tool argument**, so per-org rules (blocklist, rate limit) need the chain to read that
  argument — a core change, or a convention that every tool audits `org` into the trace.
- Docs, kit, marketplace references.

**Pros.**

- Matches the "one connector, N grants" product story exactly; auto-discovery; cross-org
  queries; we control tool ergonomics and are not bound to hosted-MCP feature cadence.
- One MCP server in every client's tool list regardless of org count.

**Cons.**

- We own SOQL/CRUD correctness, API versioning, rate limits and the security review of the
  tool surface.
- Needs the broad `api` scope instead of `mcp_api` — harder to justify to client security teams
  than "the Salesforce-hosted, Salesforce-scoped server".
- Per-org governance has to be rebuilt on an argument rather than an entity id.
- Weeks, not days, before the first client org is live.

## Comparison

| | Option A — hosted MCP + package | Option B — own crate |
|---|---|---|
| Time to first client org | Days (package build + shared callback) | Weeks |
| Per-org onboarding | Install package, permission set, activate hosted MCP, one config entry here | Install package, permission set, user connects |
| Per-org config in repo | Yes (`orgs.yaml` + `services/mcp/<id>.yaml`) | No |
| Org auto-discovery | No | Yes |
| Cross-org queries | Agent calls N servers | `search_all_orgs` |
| Scope requested from the org | `mcp_api refresh_token openid` | `api refresh_token openid` |
| Who maintains tools | Salesforce | Astound |
| Governance granularity | Per server id (exists) | Per tool argument (to build) |
| Sandbox refresh | `reprovision` per server id (exists) | Per org id (to build) |
| Exposure to Salesforce change | Hosted MCP feature cadence | REST API versioning |
| Client tool-list size | One server per org | One server |
| Cost to the org | No per-call price; each call draws on the org's daily API allocation (Enterprise+: 100k/day + per-licence); hosted MCP included with Enterprise+ | Same allocation, same editions (REST needs Enterprise+ or the API add-on); we control how many API calls one tool call spends |

### Cost is not a differentiator

Salesforce publishes no per-call price for hosted MCP servers and includes them with
Enterprise Edition and above; a tool call is metered only through the org's existing daily
API request allocation, exactly as a REST call is. The REST API is available on the same
editions (Professional needs the API add-on). Usage fees begin only when a call invokes
Agentforce, Data 360 or Prompt Builder, which neither option does. The one real difference is
control: in Option B we decide how many REST requests a tool call spends (composite/batch), in
Option A that is Salesforce's choice.

## Stage 1 (implemented 2026-09-17)

- `deploy/salesforce/`: the packaged External Client App as an SFDX project, package and
  install scripts, and the runbook. Astound US is the source org; UK installs the package; the
  Developer Edition org `orgfarm-141013b2b4-dev-ed.develop.my.salesforce.com` proves it first.
- Gateway: one shared callback `/api/public/connectors/salesforce/callback` (org in the OAuth
  `state` prefix), shared secret pair by default, instance URL stored on every grant.
- Config: servers `salesforce-us`, `salesforce-uk`, `salesforce-dev`; US/UK My Domains are
  placeholders until the orgs are wired up. Entitlement is Salesforce's (permission set).
- Stage 2 groundwork only: the shared callback and the stored `(organization_id, instance_url)`
  are what a one-to-many client connector keys on; nothing else is built.

## Recommendation

Adopt the two "regardless" items now. Ship **Option A** first: build the package, add the
shared callback, and connect Astound's production org and one sandbox through it. It gets both
Astound-owned and client-owned orgs live with the least new surface, and the package, the
permission set and the org-side runbook carry over unchanged if Option B is taken later.

Take **Option B** only when connected orgs show a concrete need for cross-org fan-out or for
onboarding orgs without a repo change. Do not start it speculatively: its cost is the tool
surface and the governance rework, neither of which the package saves.

## Open questions

1. **Callback propagation.** On a source-referenced packaged ECA, does adding a callback URL on
   the source org reach subscribers without a new package version? Thirty-minute test on two
   Astound sandboxes (carried over from the 09-11 design). If not, the shared callback in
   Option A is mandatory rather than convenient.
2. **Package ownership.** Which Dev Hub org and namespace own the package, and who holds the
   consumer secret. The source org is now production infrastructure: deleting the app there
   breaks every subscriber.
3. **Client acceptance.** Whether client security teams accept referencing Astound's source-org
   credentials, or will insist on independent global settings (falls back to per-org secrets).
4. **Org inventory.** The launch list of orgs with label, My Domain, environment, org id and
   entitled groups — still outstanding from 09-11.

## References

- Current implementation: `extensions/web/admin/src/services/salesforce_orgs.rs`,
  `extensions/web/admin/src/services/connector_oauth/`, `services/config/salesforce-orgs.yaml`,
  `services/mcp/salesforce.yaml`, `docs/integrations/centralized-mcp.md`,
  `services/content/documentation/salesforce.md`.
- Salesforce: [Hosted MCP Servers guide](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/hosted-mcp-servers-overview.html),
  [External Client Apps](https://help.salesforce.com/s/articleView?id=sf.external_client_apps.htm).
