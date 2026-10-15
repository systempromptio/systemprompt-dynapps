# CRM Report — Setup Guide

## Environment Variables (Required)

| Variable | Description | Example |
|---|---|---|
| `ODOO_URL` | Odoo instance URL | `https://www.indaws.es/` |
| `ODOO_DB` | Odoo database name | `master` |
| `ODOO_USERNAME` | Odoo API username | `user@indaws.es` |
| `ODOO_PASSWORD` | Odoo API key | (API key) |
| `EMAIL_SMTP_PASSWORD` | SMTP password for Gmail | (app password) |

## Optional Environment Variables

| Variable | Default | Description |
|---|---|---|
| `EMAIL_SMTP_SERVER` | `smtp.gmail.com` | SMTP server |
| `EMAIL_SMTP_PORT` | `587` | SMTP port |
| `EMAIL_USE_TLS` | `true` | Enable STARTTLS |
| `EMAIL_FROM` | `victor@indaws.es` | Sender address |
| `EMAIL_USERNAME` | `victor@indaws.es` | SMTP login |

## Configuration Reference

Edit `scripts/config.py` to update:

- **SALESPEOPLE_OBJECTIVES**: Monthly targets per salesperson
- **SALESPEOPLE_EMAILS**: Email addresses for personalized reports
- **EXCLUDED_SALESPEOPLE**: Users to exclude from analysis
- **STAGE_*_KEYWORDS**: Stage classification keywords
- **PRODUCT_CATEGORIES**: Product categorization rules
- **Thresholds**: DAYS_WARNING (5), DAYS_CRITICAL (7), PROPOSAL_FOLLOWUP_DAYS (3)

## Brand Compliance

Read exact color hex values from `dynapps-brand/references/brand-system.md`.

- **Font**: Montserrat (no other fonts)
- **Colors**: Blue Lilac, Blue Space, Warm Yellow, Light Sky
- **Zero emojis**: Text labels and CSS badges only
- **Language**: Spanish

## Module Architecture

```
scripts/
  config.py           - Centralized configuration and brand constants
  utils.py            - Shared helpers (parse_date, truncate_text)
  odoo_client.py      - XML-RPC connection and data fetching
  data_processor.py   - KPI engine with 17 focused functions
  report_generator.py - Brand-compliant HTML generation
  email_service.py    - SMTP delivery (team + personalized)
  odoo_publisher.py   - Odoo website publishing + activities
  main.py             - CLI entry point
diagnostics/          - 8 debug utilities (see crm-debug skill)
references/           - KPI documentation + this file
```
