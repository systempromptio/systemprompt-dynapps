# Security Policy

This repository is the Dynapps installation of systemprompt: the
extensions, services configuration, desktop-bridge white-label and deployment
bundles built on the published `systemprompt-*` core crates. It runs inside
Dynapps's own environment, and a vulnerability here can expose governed AI
traffic, audit data or credentials, so reports are handled by coordinated
disclosure.

## Reporting a Vulnerability

Email: **ed@systemprompt.io**

Include, where possible:

- Affected version(s) and commit hash
- Reproduction steps or proof-of-concept
- Impact assessment (confidentiality / integrity / availability)
- Any suggested mitigation

Do not open public GitHub issues for security reports. If you need an encrypted
channel, ask for one in your first email. A report that turns out to sit in core
is handled under the
[core security policy](https://github.com/systempromptio/systemprompt-core/blob/main/SECURITY.md)
with the same contact and targets.

## Response Targets

| Stage | Target |
|-------|--------|
| Acknowledgement of report | 48 hours |
| Triage and severity assignment | 5 business days |
| Fix for Critical (CVSS 9.0+) | 14 days |
| Fix for High (CVSS 7.0–8.9) | 30 days |
| Fix for Medium / Low | Next scheduled release |
| Coordinated public disclosure | 90 days from report, or on patch release, whichever is earlier |

If a fix needs longer than this, we will say why and agree a timeline in writing.

## Supported Versions

Dynapps releases track core minor versions (this release is 0.63.1 on core 0.63.1).

| Version | Security fixes |
|---------|----------------|
| 0.63.x (current minor) | Yes |
| 0.62.x (previous minor) | Critical and High only |
| < 0.62 | No |

The previous minor receives Critical and High fixes until the next minor ships,
then leaves support, mirroring core's policy. Deployments should run the latest
0.63.x release.

## Scope

In scope:

- This repository: the extensions under `extensions/`, the configuration under
  `services/`, the scripts and deployment bundles (`deploy/`), the published
  container image and the white-labelled desktop bridge built from `bridge/`.
- The `systemprompt-*` core crates this repository builds on, as consumed here.
  Core fixes are made and released in core and adopted by a core pin bump.

Out of scope:

- Third-party kits and MCP servers not maintained in this repository.
- Upstream dependency vulnerabilities we cannot remediate (we forward them to
  their maintainers).
- Denial of service through unrealistic resource consumption against a local
  development build.

## Audit tamper evidence

Audit tables (`ai_requests`, `ai_safety_findings` and the other audit logs)
are guarded by database triggers that refuse, or in `warn` mode record, any
update, delete or truncate outside the retention floors. That refusal is
immediate: it runs inside the offending statement. The hash chain over
`ai_requests` and `ai_safety_findings` is eventually consistent: the
`audit_chain_seal` job links new rows every 10 seconds, so a row can be
unsealed for up to one seal interval, and `audit_chain_verify` reports the
waiting count as `unsealed_rows`. See `extensions/web/schema/51_audit_integrity.sql`.

## Service-account credentials

Workflow write-backs and farm runs act as service accounts that core refuses
for every interactive login. Their machine credentials (GitHub App keys,
Salesforce client secrets, Atlassian API tokens) are sealed with the master
key, bound to one account and provider, never returned by any endpoint, and
issued as a bearer only to a `workflow:<id>:write` or `farm:<id>` caller. See
`extensions/web/schema/53_connector_machine_credentials.sql`.

## Telemetry

A self-hosted Dynapps instance sends nothing to the vendor. Neither this
repository nor the core it builds on has crash reporting, usage or licence
pings, or an update check. Every outbound connection is one the operator
configures:

1. **Model providers.** The upstream endpoints in `services/ai/providers.yaml`
   (Vertex AI, Anthropic, Gemini), called when a gateway request is routed to them.
2. **Vertex rate-card discovery.** At boot, the Vertex model list is read from
   the configured Vertex project to price and list the models it serves.
3. **OpenTelemetry export.** The collector named in the profile's
   `observability.otlp` block. Unset in the shipped profiles, so nothing is
   exported until a deployment names one.
4. **Container images from GHCR.** The deployment pulls the instance image from
   `ghcr.io`; the server process itself does not.
5. **Bridge release feed.** When the services manifest declares a bridge
   releases repository, the gateway resolves the newest bridge release from
   that GitHub repository on a desktop bridge's request.
6. **Identity and connectors.** Microsoft Entra ID (SAML) for sign-in, and the
   Atlassian and Salesforce connectors when a user uses them.

The desktop bridge checks for its own updates against the gateway it is
enrolled with, never against the vendor. A cloud profile (`target: cloud`) or
the `systemprompt cloud` commands additionally contact `api.systemprompt.io`
for tenant credentials and deployment; a self-hosted profile never does.

## Further Reading

- [Branching](docs/BRANCHING.md) and [Releasing](docs/RELEASING.md)
- [Observability Export](services/content/documentation/observability-export.md)
- [Governance Modes](services/content/documentation/governance-modes.md)
