# Claude Code access and connection state

## Jay Dave: 9 September 2026

The supplied screenshots show an active developer/user account in `india-devs`,
India Development allowed by that group, and Claude Code listing 34 skills as
`on` from the India BA and Salesforce Core plugins. Jay reports that the skills
are blocked and that he installed the Windows bridge app. No invocation error,
correlated failed request, or diagnostic output from his device was supplied.
The root cause remains unconfirmed. These screenshots do not justify changing
his roles or granting administrator access.

`Locked by plugin` appears in Claude Code's skill-management list alongside
`on`; the footer directs plugin skill management to `/plugin`. It is a control
on individual skill enablement, not sufficient evidence of an authorization
failure. Plugin-managed skills can still fail during execution.

## Separate admin enrollment evidence

The supplied live test for `e.burton@astound.local` authenticated successfully
against `https://sp-dev.astound.digital`. The saved
[current access](../demo/india-skills/20260909-admin-haiku/current-access.json)
shows roles `admin`, `platform_admin`, and `user`, but only 11 skills and no
`astound-india-dev` workspace. The supplied investigation reports no group
membership or direct India workspace grant.

The workspace IDs in that file are `enterprise-demo` and `systemprompt-admin`;
`astound-dev` and `astound-admin` are their **plugin** IDs. The
[installation comparison](../demo/india-skills/20260909-admin-haiku/installation-before-india-grant.json)
confirms that the client registry and skills match the granted manifest while
all 34 India skills are absent. It also reports separate MCP-set and bundled
reference mismatches; it is not an overall installation pass.

This explains the admin account's missing India content under the current
explicit group grant. Administrator status alone did not include that workspace.
It does not explain Jay's reported execution block: his screenshot already
lists the 34 India skills. No policy change or automatic admin bypass follows
from either observation. The dashboard presents the currently resolved grants;
a change to the administrator entitlement policy is a separate decision.

## Installation and runtime are separate checks

The bridge signs the user in, obtains their filtered catalog, and mirrors its
marketplaces and enabled plugins into Claude Code's local configuration. Model
requests travel through the bridge proxy. Connector authentication is a further,
independent requirement. The Linux shell installer is not a Windows repair
procedure; Windows app enrollment and policy installation must be checked on
that device and version.

The inspected marketplace doctor check verifies local marketplace JSON and a
nonempty plugin list. It does not establish successful skill execution. Server
heartbeats likewise establish reported activity, not a healthy Claude session.

The server's marketplace filter removes managed India connectors unless their
configuration, entitlement, and verified account state permit delivery. Skills
can remain included while a connector needs sign-in. Some BA skills also require
specific local trackers or project inputs and stop if they are missing. Neither
condition has been established as Jay's failure.

To finish that diagnosis, capture the exact namespaced skill invocation, its
response, Windows bridge and Claude Code versions, and attempt time. Match that
time to the user's request/trace evidence if a server request was made. Review
connector state separately. A failure before any request reaches the server
requires client evidence; absence of a request is not proof of a permission deny.
Do not collect tokens or provider credentials.

## Dashboard interpretation

Access & connections separates current entitlement, included content, verified
provider connections, and device activity. Plugin counts use the manifest
assembly and filtering path; they describe content available for sync, not an
observed installation. Permission details show individual resource rules and
retain the existing editor; parent inheritance is reflected in included content.
Unavailable reads are shown explicitly. Existing account identity links are not
used as evidence of MCP connector readiness.
