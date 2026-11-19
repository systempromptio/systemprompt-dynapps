# Microsoft 365 MCP server

The internal `microsoft-365` server (`extensions/mcp/microsoft-365`, binary
`systemprompt-mcp-microsoft-365`) serves Microsoft Graph to signed-in users
as themselves: mail, calendar, contacts, To Do, OneDrive, SharePoint, Teams,
chats, presence, meetings, Planner, OneNote, groups, users and search. It
has 147 tools with the names and annotations of the Belgian `dynapps-365`
server (`d365_*`, pinned by `tests/unit/mcp/fixtures/m365_tools.json`),
except four annotations that are stricter on purpose (see
[Governance](#governance)).

Each person connects their own Microsoft 365 account on `/admin/connectors`.
Every tool call asks the connector broker for that person's Graph token, so
Microsoft's own permissions decide what each person reads or changes.
Configuration lives in `services/mcp/microsoft-365.yaml`. The file ships with
`enabled: false`, and the server stays disabled until the Entra app below
exists.

## What to ask the Entra administrator (Daan) for

One **Web** app registration in Entra ID:

1. **Supported account types**: *Accounts in any organizational directory*
   (multi-tenant) for the shared Dynapps app, or *this directory only* for a
   customer's own app.
2. **Redirect URI** (Web), one per installation:
   `https://YOUR-SERVER/api/public/connectors/microsoft-365/callback`
3. **A client secret.** Record its expiry date, because rotating the secret
   does not force anyone to reconnect.
4. **Delegated Graph permissions**: the list in
   `extensions/web/admin/platform/src/services/connector_oauth/entra.rs`
   (those of the Belgian server, plus `offline_access openid profile email`).
   Each customer tenant's admin grants consent once.
5. **The tenant ids of the customers** to admit on each installation.

No application permissions are needed. The server only acts as the
signed-in user.

## Secrets

Set these in the installation's secrets store. They never go in `services/`.

| Secret | Required | Value |
| --- | --- | --- |
| `microsoft_365_client_id` | yes | Application (client) id. The server is granted this secret too, because Graph wants it as the session id of a presence the app sets (`d365_set_my_presence`). |
| `microsoft_365_client_secret` | yes | The client secret value. Only the API holds it. |
| `microsoft_365_tenant` | no | `organizations` (default, the shared app) or one tenant id (a customer's own app). |
| `microsoft_365_allowed_tenants` | shared mode | The tenant ids admitted on this installation, separated by commas or spaces. |
| `microsoft_365_authority` | no | Authority origin. Defaults to `https://login.microsoftonline.com`. |
| `microsoft_365_graph_url` | no | Graph root. Defaults to `https://graph.microsoft.com/v1.0`. It exists for the local simulation. |
| `mcp_credential_broker_secret` | yes (already set) | Shared by the API and the MCP servers that call the broker. |

The server's YAML grants it `mcp_credential_broker_secret` and
`microsoft_365_client_id`. Core refuses to spawn a server whose granted
secret is missing, so `microsoft_365_graph_url` is not granted in the shipped
file. The simulation adds that grant to its own copy.

An installation whose file storage is S3 (`storage.backend: s3`, credential
`files_s3`) must also grant `storage`, as it does for `documents` and
`systemprompt`: the server builds the profile's file layer at startup, and
without that grant it exits before it serves anything. A smoke boot on local
storage does not show it.

## Shared and dedicated mode

- **Shared** (`microsoft_365_tenant` unset or `organizations`): one Dynapps
  multi-tenant app serves many customers. `microsoft_365_allowed_tenants` is
  mandatory, and an account from any other tenant is refused when it
  connects.
- **Dedicated** (`microsoft_365_tenant` = a tenant id): the customer's own
  app. The allow-list defaults to that tenant.

Personal Microsoft accounts are never admitted. `common`, `consumers` and the
tenant id of personal accounts, `9188040d-6c67-4c5b-b112-36a304b66dad`
(well-formed, but it behaves like `consumers`), are refused as
`microsoft_365_tenant` and in `microsoft_365_allowed_tenants`: either one is a
configuration error, and Microsoft 365 sign-ins and tokens fail until it is
corrected. An ID token whose `tid` is that tenant is refused with "personal
Microsoft accounts are not supported on this instance".

The tenant is admitted **per installation, not per person**. Every person on
an installation may connect an account from any tenant on its list. Removing
a tenant from the list cuts its users off at their next call: their saved
grant is deleted, refresh token included, and the account shows *Reconnect
required*.

Every refusal is logged (target `connector_oauth`) with the tenant id involved
and a reason: `tenant_not_allowed`, `personal_account`, or the ID token check
that failed. Tenant ids are not secrets; tokens and usernames are never logged.

What the code does enforce between companies on one installation:

- a tenant outside the list is never linked;
- a consent can only be completed by the person who started it, because the
  `state` is bound to its owner;
- each tool call uses the caller's own grant, so a token of tenant A never
  reads tenant B's data;
- file references resolve only for their owner.

**Open (for the PR, to decide with Aylen):** a person of company A who also
holds an account in tenant B (a guest, or a second job) can link tenant B
when both tenants share one installation. Closing that gap needs a binding
between tenant and company, for example "group `empresa-a` may only link
tenant A", checked at the exchange beside the allow-list. That binding is not
implemented. Until then, two companies that must never cross need separate
installations, or separate allow-lists.

Connector details (ID token checks, refresh, rotation) are in
`docs/CONFIGURED-CONNECTORS.md`, section "Microsoft 365 (Entra)".

## Files

Files never travel through the model as large base64. The server uses the
shared file layer (`docs/integrations/mcp-files.md`):

- **Into Graph** (attachments, OneDrive uploads, upload-session chunks): a
  `file` reference. It can be an `upload_id` or a `blob_ref` from the
  documents server's `documents_upload`, or a `document_id` of the caller's.
  Base64 in the arguments is accepted up to 64 KB, as on the Belgian server.
  A reference resolves only for the person who owns it.
- **Out of Graph** (attachments, file and transcript downloads): saved as a
  `file` document of the caller. The tool returns the reference and a signed
  download link valid for 10 minutes. The exception is `d365_get_file_content`
  on a text file of up to 256 KB: the text comes back inline for the model to
  read, and the download is audited like a saved one (`inline`).
  A mail attachment is read raw from `/attachments/{id}/$value`, after its
  metadata is read without the base64. That way one of up to 25 MB fits
  the response limit. As base64 inside JSON, the limit was about 18.7 MB.
  Item and reference attachments have no file to download, so the tool
  returns Graph's description of them.

The API signs those links only for a server that declares the `documents`
toolset, so `microsoft-365.yaml` and `documents.yaml` both declare it. The
server has no upload tool of its own: `documents_upload` covers that (the
documents server is always on in this installation), and its resumable
upload is the path for files over 64 KB. Every use and every refusal of a file is
written to `user_activity` (`mcp_access`, `entity_type = file`).

### Large mail attachments

`d365_add_mail_attachment` with a `file` does the whole upload on the
server, upload session included. The Belgian pair
`d365_create_mail_attachment_upload_session` and
`d365_upload_attachment_chunk` is kept for parity, with two limits:

- **Only sessions this server opened for the same person.** Every tenant
  shares Microsoft's upload hosts, so a host check alone would let a
  session opened in another tenant receive the person's file. When the
  server opens a session it records the `uploadUrl` against the person who
  asked. The chunk tool accepts a URL only if it was recorded for the same
  person and has not expired. A session expires at Graph's
  `expirationDateTime`, or one hour after it was opened if that comes first.
  Any other URL is refused before the file is opened or Graph is called, and
  the model is told to open a new session. The host check (Microsoft's
  upload hosts, without the bearer) still applies behind it.
- **At most 4 MiB per chunk**, which is Graph's limit. A larger chunk, as a
  `file` or as base64, is refused with that message before it is sent.

The sessions live in the memory of the server process: at most 16 per
person and 1,024 in total, and expired ones are dropped. A restart forgets
them, and so does a second replica of the server, which never sees sessions
the first one opened. Either way the chunk is refused and the model opens a
new session.

## Governance

`services/governance/config.yaml` gates these tools (`role_gated`,
`contains:` on `mcp__microsoft-365__`):

- every tool that sends, replies, forwards or deletes;
- `d365_batch`, whose requests may include any write, `DELETE` or send;
- the tools that move data out without sending: `d365_create_mail_rule`
  (a forward or redirect rule), `d365_share_file` (a sharing link) and
  `d365_upload_attachment_chunk` (it puts a file into an upload session);
- the event tools that make Exchange email someone: `d365_create_event` and
  `d365_update_event` invite the attendees, body included, and
  `d365_accept_event`, `d365_decline_event` and
  `d365_tentatively_accept_event` answer the organizer;
- `d365_move_mail` (a move to `deleteditems` is a soft delete) and
  `d365_add_planner_roster_member` (it grants access to a plan).

`contains:` matches a substring of the whole tool name, so an entry that
would also catch a read tool is written as the full name: `event` alone would
catch `d365_list_events` and `d365_get_event`, `roster_member` would catch
`d365_list_planner_roster_members`.
`tests/unit/web/src/governance_microsoft_365_gate.rs` checks the gated list
and that every read tool of the pinned surface still passes.

These tools pass only for admin or the `knowledge_worker` role, and the same
role is exempt from the delete blocklist on this prefix. Only the gateway
name (`mcp__microsoft-365__`) is listed. If the server is ever added to a
plugin, its `mcp__plugin_<plugin>_microsoft-365__` form needs the same
rules. Read tools pass for everyone entitled to the server
(`mcp_server/microsoft-365` in `services/access-control/rules.yaml`). The
tool plane runs in warn mode, so today a gated call is recorded, not
refused. `tool_policy: prompt` makes the client ask before each call.

### Annotations that differ from the Belgian server

Four tools that the Belgian `_annotations.py` marks as a plain write are
served with `destructiveHint: true`. A client that auto-approves
non-destructive tools still asks before any of them:

| Tool | Why it is destructive here |
|------|----------------------------|
| `d365_batch` | its requests may carry any write, delete or send |
| `d365_share_file` | it opens the file to people outside the user |
| `d365_create_mail_rule` | a rule can forward, redirect or delete incoming mail |
| `d365_upload_file` | it replaces an existing file of the same name |

The parity test (`tests/unit/mcp/src/m365_parity.rs`, `HINT_DEVIATIONS`)
records them with these reasons. Every other title and hint still has to
match Belgium.

## Local simulation

```bash
just m365-sim
```

The simulation needs Postgres (the local profile's database URL), `python3`
and `openssl`, and no real account. It runs a throwaway database, Entra and
Graph simulated per tenant on loopback (`tests/fixtures/oauth/m365_sim.py`),
the API, and the `documents` and `microsoft-365` binaries as separate
processes. Two people in two companies each connect their own tenant, read
their mail, upload through `documents_upload`, attach the file, send, mark
mail as read, put a file on OneDrive and save an attachment as a document.

The simulation also checks that these crossings are refused:

- a tenant that is not admitted;
- another person's consent;
- another tenant's message;
- another person's file references.

Each case prints as it passes. The throwaway CA is trusted through
`SSL_CERT_FILE` for that run only. This works on Linux. On macOS, rustls
reads the keychain instead, so the CA has to be trusted by hand.
