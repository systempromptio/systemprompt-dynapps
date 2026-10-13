# done-K

Scope: CLAUDE.md, README.md, AGENTS.md, SECURITY.md, docs/** (except ops/, status/), deploy/helm/systemprompt-dynapps/**, deploy/self-host/**, deploy/loadtest/**, deploy/salesforce/**, demo/**.

Verification: `git grep -i astound` over that set returns zero; also zero for india|sfnext. Helm: every include/template name resolves to a define in _helpers.tpl (all `dynapps.*`).

## Changes
- Mechanical identifiers per brief table (repo/image/GHCR org, helm helpers + release/namespace `dynapps`, `/etc/dynapps`, `dynapps-gw@`, `DYNAPPS_*`, plugin ids, bridge names, URLs; Atlassian site -> dynapps.atlassian.net; loadtest users @loadtest.dynapps.systemprompt.io).
- CLAUDE.md: rule 5 rewritten; `services.sources`/kit prose removed from "Sources, planes, sync"; production tenant (sp-e43527c1cbb5, e43527c1cbb5.systemprompt.io, dynapps.systemprompt.io) added under "Operating on production"; `dynapps-commons` hooks owner + who-am-i example.
- Deleted (git rm): docs/india-marketplace.md, docs/sfnext-marketplace.md, docs/kits-on-another-instance.md. docs/README.md index entry removed.
- docs/integrations/centralized-mcp.md kept (generic connector doc); India/Google/Astound-org wording neutralised.
- docs/CONFIGURED-CONNECTORS.md: Google Workspace (Agent Search) section removed.
- SECURITY.md: kit-bundle egress item removed; AD FS -> Microsoft Entra ID (SAML); Google connector dropped.
- README.md: sfnext kit paragraph removed. deploy/self-host/SECRETS.md: `ghcr_pull_token` row removed.
- AD FS SSO prose -> Microsoft Entra ID (SAML) in self-host README/SINGLE-NODE; `/documentation/adfs-sso-setup` slug kept.
- Salesforce ECA description + decision doc: Astound US/UK org names -> neutral source/installing org.
- Multi-region examples (India/US/EU) -> Spain/Belgium/Netherlands.

## Skipped / notes
- SECURITY.md still says "this release is 0.63.1 on core 0.63.1" and Chart appVersion 0.63.1 (pre-existing staleness, not brand) — left for the version owner.
- docs/decisions/salesforce-connector.md is a historical decision record; only brand names neutralised, Developer Edition org hostname left.
- Bridge managed-policy keys in docs/install/docker.md now read `Policies\dynapps` / `io.systemprompt.dynapps`; whoever owns bridge code should confirm they match.

No handoffs. No requirements.txt blockers.
