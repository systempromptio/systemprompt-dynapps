# done-W (web site, admin UI, assets)

## Files changed
- services/web/config.yaml, metadata.yaml, config/{theme,homepage,navigation}.yaml: name Dynapps, title
  "Dynapps AI Gateway - governed Claude Code for Dynapps teams", copyright 2026 Dynapps, domain dynapps.es,
  baseUrl https://dynapps.systemprompt.io, @dynapps, tagline "Odoo, shaped around how you work.",
  themeColor #F13C42, palette primary #F13C42 / secondary+accent #951CF3, card gradient purple-red-orange,
  fonts "Readex Pro". Footer links now www.dynapps.es / wearedynapps.com / linkedin.com/company/dynapps.
  Docs sidebar: Salesforce link removed; SSO link -> /documentation/entra-id-sso-setup.
  security.yaml has no brand content (unchanged).
- services/web/templates/{homepage,skills}.html, partials/{header,head-assets}.html: copy, bridge links
  -> /files/downloads/dynapps-bridge-*. head-assets: Google Fonts preconnect + Readex Pro stylesheet,
  inline tokens. footer/scripts partials had no brand text.
- storage/files/css: core/variables.css (primary #F13C42, accent #951CF3, brand gradient, syntax colours),
  core/fonts.css + admin/01-tokens-semantic.css (Readex Pro first, Inter fallback),
  admin/01-tokens-primitives.css (accent-red oklch(0.633 0.216 24.4), accent-purple oklch(0.552 0.277 302.4)).
- services/artifacts/admin-ai-usage/view.html: title, brand line, --accent, font.
- storage/files/admin: login.hbs (Dynapps SSO (Microsoft Entra ID), Entra ID groups), layout, sidebar,
  admin-footer, create-user-panel, users, connect (bridge assets, config dirs dynapps/), bridge-device-link,
  project-report.
- storage/files/js/pages/admin-login.js (SSO messages, Entra ID wording), connectors.js (dynapps.atlassian.net
  placeholder).
- services/config/blog.yaml, services/content/config.yaml.
- storage/files/images/architecture/value-architecture.svg: no brand text; accent #fe190a -> #F13C42 by sed.

## New asset renders (same filenames), via cairosvg + Pillow in a scratch venv
- logo.svg (images + assets/logos): brand/dynapps-wordmark.svg with currentColor -> #1A1A1A.
- logo-white.svg: same with #FFFFFF. icon.svg: gradient mark only, 32x32 viewBox.
- logo.png / logo.webp (1200x630, both dirs): wordmark 840px wide on #FAFAFA.
- assets/logos/logo-dark.png: white wordmark on #1A1A1A. logo-512.png: mark 416px on transparent.
- favicon-32x32.png / favicon-96x96.png: mark rasterised. favicon.ico (both dirs): copy of brand/favicon.ico.

## Checks
check-asset-reachability.py OK, check-admin-template-links.sh OK, check-admin-template-assets.sh OK.
git grep -i astound over the W set: zero.

## Skipped / notes
- web/dist is untracked; left alone.
- No handoff-E.md existed, so no sso=group_overage login message was added.
- storage/files/video/showreel.mp4 (homepage hero video) is likely Astound footage; outside my set, not replaced.
- No CSP in this repo; if core sends a restrictive style-src/font-src, fonts.googleapis.com/gstatic.com
  must be allowed or Readex Pro falls back to Inter.
