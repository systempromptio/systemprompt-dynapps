---
title: "Connect Salesforce: CRM and Development"
description: "Salesforce Hosted MCP and DX MCP requirements, current blockers, and secure credential provisioning for Astound clients."
author: "Astound Digital"
slug: "salesforce"
kind: "guide"
public: true
tags: ["documentation", "salesforce", "connectors", "development"]
published_at: "2026-09-09"
updated_at: "2026-09-11"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Connect Cowork"
    url: "/documentation/connect-cowork"
  - title: "Connect OpenCode"
    url: "/documentation/connect-opencode"
---

# Connect Salesforce: CRM and Development

Salesforce CRM access and Salesforce development use different MCP integrations. The **Salesforce** connector in the Astound dashboard is Salesforce Hosted MCP for records and queries. The **official Salesforce DX MCP server** is a separate integration for development against an authorized org and a project checkout. Connecting one does not configure the other.

This guide is for developers, Salesforce administrators, and the operators of your Astound gateway. The requirements below describe what must be provided and verified; they are not a live inventory of missing secrets on any deployment. A visible connector or installed skill does not prove that Salesforce access works.

## Choose the integration

| Integration | What it provides | Where it runs |
|---|---|---|
| Salesforce Hosted MCP — SObject All | Record creation, reads, updates, deletes, SOQL and search, within the user's Salesforce permissions | Salesforce-managed HTTPS endpoint |
| Salesforce DX MCP — `@salesforce/mcp` | Development tools for metadata retrieval and deployment, tests, org management and data operations | On the machine running the development tools, alongside the project and CLI authorization |
| Other Salesforce MCP servers | Scoped record access, Data 360, Tableau, and custom tools backed by Apex, Flows or APIs | Separate servers with their own capabilities and setup |

The hosted SObject All connector does not provide the local project environment needed for DX development. Salesforce also offers separate B2C Commerce developer tooling; it is not the Salesforce Core DX integration described here. See the [official DX server](https://github.com/salesforcecli/mcp), [hosted server catalogue](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/references/reference/servers-reference.html), [custom hosted servers](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/custom-servers.html), and [B2C Commerce tooling](https://developer.salesforce.com/blogs/2026/04/introducing-the-b2c-commerce-cli-and-on-demand-sandbox-cloning).

## Which clients can use it?

| Client | Local Salesforce DX MCP | Salesforce Hosted MCP |
|---|---|---|
| Claude Code | Supported through a local MCP configuration and org authorization on the execution machine | Supported through remote MCP; verify Astound connector delivery after account linking |
| OpenCode | Supported through a local stdio MCP configuration | Supported through remote MCP; verify Astound connector delivery after account linking |
| Cowork | Ordinary Claude Desktop local MCP configuration is not available in Cowork | Use a reachable remote connector; verify tool discovery and a real Salesforce read in Cowork |

Anthropic documents that local servers configured through `claude_desktop_config.json` are unavailable in Cowork. Installing DX on a laptop therefore does not make it a Cowork connector. A working Bridge or model request also does not prove Salesforce tool connectivity. See [Claude's remote connector requirements](https://support.claude.com/en/articles/11175166-get-started-with-custom-connectors-using-remote-mcp), [Claude Code MCP support](https://code.claude.com/docs/en/mcp), and [OpenCode MCP support](https://opencode.ai/v2/docs/mcp-servers).

## How the India skills use Salesforce

The India marketplace import includes 18 Salesforce Core skills and 16 business-analysis skills. Salesforce instructions cover Apex tests, Flow patterns, permissions, integration, deployment, and project context. The Salesforce plugin currently declares the hosted Salesforce and Atlassian connectors; it does not install DX MCP.

The client loads a skill's instructions, discovers its available tools, and invokes those tools when needed. Skills do not install runtimes, start DX servers, or authorize orgs. Local source review and artifact generation can proceed without a Salesforce connection. Org queries, retrieval, tests, and deployments need the appropriate authenticated tools.

The imported execution contract requires approved Systemprompt tool access and prohibits bypassing it with direct provider requests. The DX path must be explicitly supported in that contract and in the integration before it is described as governed. Adding a local MCP entry alone is not proof that its calls pass through Systemprompt policy and audit controls.

The retained India test matrix excludes live Salesforce execution. Local artifact results must not be presented as successful org validation or deployment.

## Requirements and what remains blocked

| Required input or capability | Who provides it | Blocked until provided and verified |
|---|---|---|
| Org My Domain, org ID, and environment type: production/developer org or sandbox/scratch org | Salesforce administrator | Selecting the correct authorization domain and hosted endpoint |
| The packaged `Systemprompt` External Client App installed in the org, with its permission set assigned | Salesforce administrator | Hosted browser authorization |
| Activated hosted MCP server and appropriate user/app permissions | Salesforce administrator | Tool discovery and Salesforce operations |
| Application settings in the correct gateway secret store | Gateway operator | Completing the dashboard connection flow |
| Individual Salesforce login and consent, or separately configured administrative pre-authorization | Each user, with administrator support | That user's hosted Salesforce access |
| Salesforce CLI, compatible Node/npm runtime, DX MCP, project checkout, and local org authorization | Developer | Local DX development operations |
| An entry in the org registry (`services/config/salesforce-orgs.yaml`), a matching `services/mcp/salesforce-<slug>.yaml` server and an access rule for every org, sandbox included | Gateway operator | That org appearing on the profile page at all |
| Bridge execution, readiness reporting, and policy/audit integration for DX | Systemprompt maintainer | Dashboard-managed, governed local DX |
| Reachable remote service with an isolated development workspace, or an authenticated relay to the execution machine | Systemprompt maintainer | Cowork invoking DX remotely |

Credentials can unblock hosted authorization and local DX authorization. They cannot supply the missing DX integration: the Bridge delivers MCP servers as gateway URLs only, and a stdio process such as `@salesforce/mcp` cannot be expressed in its manifest, so DX remains a local, per-machine setup.

## Hosted Salesforce: administrator setup

Use browser OAuth for the first connection. No private signing key or signing certificate is required for this path.

One **External Client App**, `Systemprompt`, serves every org. It is created once in the source org (Astound US) and delivered to every other org as a managed package built from `deploy/salesforce/` in the gateway repository, whose README is the operator runbook. An org administrator therefore never creates an app or handles a consumer secret:

1. Identify the target Salesforce org, its HTTPS My Domain origin, org ID, and environment type. Use the My Domain origin, such as `https://example.my.salesforce.com`, without a Lightning page path, and send them to the gateway operator.
2. Install the `Systemprompt` package from the install link the operator provides (for a sandbox, sign in via `test.salesforce.com`). The installed app references the source org's OAuth settings — callback, PKCE, `mcp_api`, `refresh_token` and `openid` scopes, named-user JWT access tokens — so nothing is configured by hand.
3. Assign the packaged **Systemprompt MCP Access** permission set to the people who may connect. That assignment is the entitlement: a user without it, or without a login in this org, is refused by Salesforce at consent.
4. In Salesforce Setup, find **MCP Servers** under API Catalog and activate **SObject All** for the org. Confirm the intended users can access the required objects, fields, and records; the hosted server enforces their own permissions.
5. Allow for Salesforce's documented propagation delay of up to 30 minutes after the app first appears in the org.

Only the source org's administrator handles credentials: the app's consumer key and consumer secret are delivered once, using the secure process below, and stored on the gateway as `salesforce_mcp_client_id` and `salesforce_mcp_client_secret`.

Test environment (`https://astound.systemprompt.io`) callback:

```text
https://astound.systemprompt.io/api/public/connectors/salesforce/callback
```

Astound endpoint (`https://sp-dev.astound.digital`) callback:

```text
https://sp-dev.astound.digital/api/public/connectors/salesforce/callback
```

The site hosting this guide does not determine which gateway your account uses. Confirm the target with your gateway operator. A Salesforce org's production/sandbox type is also independent of whether the Astound gateway is a test deployment.

Follow Salesforce's [External Client App setup](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/create-external-client-app.html) and [MCP activation instructions](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/activate-mcp-servers.html). These provider requirements are separate from Astound's account and marketplace permissions.

### One entry per org: the org registry

Salesforce has no central authorization server. Every org is its own login, holds its own user records, and a sandbox is served by a different hosted MCP endpoint. Astound therefore declares the org in `services/config/salesforce-orgs.yaml`, keyed by the id of the MCP server that fronts it (`salesforce`; `salesforce-<slug>` for an org added later). A user picks the connection on the profile page before signing in; the grant, verification and every tool call are bound to it. Every entry shares one callback, `/api/public/connectors/salesforce/callback` — the server id travels in the OAuth `state` — so adding an entry never touches the app.

```yaml
orgs:
  salesforce-client-a-uat:
    label: "Client A — UAT sandbox"
    my_domain: https://client-a--uat.sandbox.my.salesforce.com
    environment: sandbox                      # production | sandbox
    server_kind: headless_360                 # sobject_all (default) | headless_360
    org_id: 00DXXXXXXXXXXXXXXX                # optional; a consent in another org is refused
    groups: [Systemprompt-Commerce]           # empty = every active user
    # client_id_secret / client_secret: only for an org running its own app
```

Each entry needs a matching `services/mcp/salesforce-<slug>.yaml` (type `external`, listed in `services/config/config.yaml`) whose endpoint matches both the environment and `server_kind`:

```text
production:  https://api.salesforce.com/platform/mcp/v1/platform/sobject-all
sandbox:     https://api.salesforce.com/platform/mcp/v1/sandbox/platform/sobject-all
headless:    https://api.salesforce.com/platform/mcp/v1/platform/headless-360
headless sb: https://api.salesforce.com/platform/mcp/v1/sandbox/platform/headless-360
```

The gateway refuses to start if an org names a server that is missing, disabled, internal, or on the wrong endpoint, or if `my_domain` does not match `environment`. It does **not** refuse to start when the shared secrets are absent: every org is shown as **Not provisioned** on the profile page until the operator has stored the packaged app's key and secret. `groups` controls who sees the row; a user outside those groups sees the org as not entitled and cannot connect, even by URL.

Because every org is its own MCP server, governance sees the org: an audit row for a SOQL query names `salesforce-client-a-uat`, and a blocklist rule on production writes does not touch UAT.

### After a sandbox refresh

A sandbox refresh recreates the org without the installed package and with it every user's consent. Once the administrator has reinstalled the package and reactivated the MCP server, reset the org in one step:

```text
POST /api/public/admin/connectors/salesforce-client-a-uat/reprovision
```

Every account on that org moves to **Reconnect required** with the reason shown on the profile page; no other org is affected. See [Salesforce endpoint troubleshooting](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/connection-issues.html).

## How to provide credentials securely

There are two handoffs: the Salesforce administrator provides application credentials to the gateway operator; each employee then signs in to Salesforce themselves. Employees do not need the application's consumer secret.

### Salesforce administrator: deliver the application settings

Create a restricted item in your organization's approved secret manager or approved secret-sharing facility. Grant access only to the operator of the intended gateway. Alternatively, an authorized operator can enter the values directly into that deployment's secret manager. Use these exact field names so the operator can map them without guessing:

| Field | What to provide | Required for browser OAuth? |
|---|---|---|
| My Domain | HTTPS My Domain origin of the target org — goes into the registry entry, not a secret | Yes |
| consumer key | stored under the `client_id_secret` name in the org's registry entry (`salesforce_mcp_client_id` for the default org) | Yes |
| consumer secret | stored under the `client_secret` name in the org's registry entry (`salesforce_mcp_client_secret` for the default org) | Yes |

Alongside the restricted item, provide the target gateway URL, Salesforce org ID, environment type, registered callback, app name, intended test user's Salesforce username, and confirmation that SObject All and the required permissions are enabled. The username can differ from the employee's Astound email.

Send the operator the restricted item's reference and non-secret setup details through the normal support process. **Do not paste credential values into Slack, email, tickets, this documentation, repository files, or AI conversations.** If no approved sharing facility is available, have the gateway operator arrange direct entry through the deployment's secret-management interface before proceeding. Do not invent an upload URL or use a public form.

Do not provide Salesforce passwords, MFA codes, browser session cookies, personal access/refresh tokens, or local `.sf`/`.sfdx` credential directories. The dashboard's Salesforce connection uses OAuth; it is not a form for uploading the application's client secret or a manual Salesforce token.

### Gateway operator: provision the target deployment

Add or update the org's entry in `services/config/salesforce-orgs.yaml` with its My Domain and environment, then store the two credentials in the active server profile's secret store under the names that entry declares (or as the equivalent upper-cased environment variables, e.g. `SALESFORCE_MCP_CLIENT_ID`).

Enter values using the secret manager's protected value fields, not literal secrets in shell history. Environment variables must reach the running server service; setting them in an unrelated terminal is insufficient. Keep credentials out of service YAML, client MCP configuration, marketplace bundles, and rendered web content.

Confirm that the gateway's configured external API URL matches the registered callback. Verify that the existing `encryption_master_key` and `mcp_credential_broker_secret` are provisioned: these are gateway infrastructure secrets, not inputs to request from Salesforce users. Preserve existing encryption and manifest-signing keys and stored grants. Do not rotate or replace them to connect Salesforce.

Restart the target service after provisioning so it loads the settings. Provision each gateway separately; do not copy a user's saved OAuth grants between deployments. A connector labelled configured or enabled is not proof that all settings are present or accepted by Salesforce.

### Employee: connect your Salesforce account

1. Sign in to the intended Astound gateway and open **Account → Connectors** at `/admin/connectors`. In Bridge, **Manage on server** opens the server account page.
2. Find the connection by its label — one row per surface you are entitled to — and select **Connect**. Sign in on Salesforce's own page with the account authorized for that org and complete its MFA and consent flow. A row marked **Not provisioned** is waiting on its administrator; nothing you do on the page changes that.
3. Return to Astound and use **Test connection**. Confirm the connected Salesforce identity and org are the expected ones.
4. Sync Bridge and reopen the client if needed. Confirm Salesforce tools are available and perform a small read against a known record or object you are permitted to access.

If the connector is missing or denied, ask the gateway operator to check account status, marketplace/tool access, application configuration, and the verification failure. Do not substitute another employee's account. A successful login without a successful tool read is incomplete setup.

### Optional: administrative JWT-bearer pre-authorization

Astound also contains a separate path for administrators to map a Systemprompt user to their exact Salesforce username and obtain a token using an application's signing certificate. This requires Salesforce-side JWT-bearer configuration and user pre-authorization, the matching public certificate registered with the app, and the corresponding `salesforce_private_key` stored only on the gateway (`SALESFORCE_PRIVATE_KEY` when supplied through the environment).

The operator maps the username per org through the admin Salesforce-identity endpoint (`POST /api/public/admin/users/{user_id}/salesforce-identity` with `sf_username` and `provider: <server id>`), the org's registry entry names the key under `private_key_secret`, and the org is named under the server's `allow.salesforce` band in `services/access-control/rules.yaml` so linked users see it. Generate and retain the private key in the approved server secret-management process; provide Salesforce with the public certificate. This optional path requires its own verification and is not a prerequisite for browser OAuth.

Neither path converts an arbitrary Astound login token into DX CLI authorization. Different Salesforce and Astound usernames do not prevent account linking, but hosted MCP authorization and local DX authorization remain separate.

## Local Salesforce DX setup

A developer can establish a local DX connection with administrator support where Salesforce org policies require it. This describes native client capability; the dashboard-managed DX and governance integration remains blocked as listed above.

1. Install Salesforce CLI and a Node/npm version supported by the chosen DX MCP release on the machine that will run the tools. Use an approved, tested package version for a managed rollout.
2. Open the actual Salesforce project checkout containing `sfdx-project.json`.
3. Authorize the intended org through Salesforce CLI. For a sandbox, for example:

```bash
sf org login web --alias india-dev --instance-url https://test.salesforce.com
```

Use the appropriate production login or My Domain URL for other orgs. Complete login in Salesforce's browser flow; do not copy credentials into the skill.

4. Configure the client to launch `@salesforce/mcp` over stdio. Salesforce publishes a Claude Code `.mcp.json` example. OpenCode uses its own `mcp` configuration with `type: "local"` and a command array. Select the required org, metadata, data, and testing tools supported by the chosen release.
5. Set `--orgs` to explicit approved aliases or usernames, such as `india-dev`. Avoid granting every locally authorized org by default. Confirm the server's advertised tool names and schemas rather than assuming a skill's example names exist.
6. Verify the org identity, retrieve a known metadata component, and run an appropriate Apex test before claiming the development path is ready. Deployment needs its own authorization and retained validation evidence.

If Claude Code or OpenCode runs on a remote development machine or in a container, DX, the project files, and org authorization must be available there. A laptop login does not authorize that separate environment. See [Salesforce's DX client configuration and org authorization](https://github.com/salesforcecli/mcp#configure-orgs).

To make DX available through the dashboard, maintainers must add machine/project readiness and governed execution. To make it available in Cowork, they must additionally provide a reachable remote execution path with the right project and org context. A local `npx` command cannot simply replace the existing hosted connector URL.

## Verify completion and report failures

| Check | Evidence to retain, without secrets |
|---|---|
| Hosted authorization | Correct Salesforce user and org; successful connection test |
| Client delivery | Salesforce tools visible in each intended client after sync |
| Hosted execution | Successful read of a known permitted record in Cowork and any other intended client |
| Local DX capability | Correct org identity, retrieved metadata, and actual Apex test result in Claude Code/OpenCode |
| Governed execution | Corresponding Systemprompt identity, tool call, policy outcome and audit trace; local DX requires the additional integration first |
| Access lifecycle | Disconnect/revocation stops subsequent access; expiry or missing permission produces an explicit failure |

For a failure report, supply the gateway URL, client, Salesforce environment type, failing step, timestamp, and redacted error or trace ID. Do not attach OAuth redirect URLs containing authorization codes, tokens, or credential files. The operator can distinguish an app/callback problem from a permissions problem, unsupported sandbox routing, or missing DX integration.

Publishing this guide, installing a skill, connecting Bridge, and completing Salesforce authentication are separate milestones. Salesforce is ready only when the intended client performs the required operation against the intended org and the promised audit evidence exists.
