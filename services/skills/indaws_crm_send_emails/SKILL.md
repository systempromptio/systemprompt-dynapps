---
name: indaws-crm-send-emails
description: >
  Send CRM reports via email. Delivers the full team report and personalized
  individual summaries to each salesperson. Use when the user asks to send
  the CRM report, email the team, or deliver weekly summaries.
version: 1.0.2
model: sonnet
effort: low
---

# CRM Email Delivery

**CONFIDENTIAL - SALES TEAM ONLY**
**Document ID:** SALES-SKILL-004
**Classification:** Critical Sales Know-How

## Overview

Handles email delivery of CRM reports through two channels: the full team report sent to all recipients, and personalized individual summaries sent to each salesperson with their specific KPIs and action items.

## When to Use This Skill

- Send the weekly CRM report to the team
- Deliver personalized summaries to salespeople
- Test SMTP configuration before sending
- Troubleshoot email delivery issues

## Workflow

### 1. Test Email Connection

Always test before sending:

```bash
python skills/crm-report/scripts/main.py --test-email
```

Verifies SMTP credentials and connectivity without sending any email.

### 2. Send Team Report

```bash
python skills/crm-report/scripts/main.py --email
```

Generates the report and sends it to all recipients in `EMAIL_RECIPIENTS`.

The email includes:
- Full HTML report inline
- HTML file attachment as backup
- Plain text fallback

### 3. Send Personalized Emails

```bash
python skills/crm-report/scripts/main.py --email-salespeople
```

Sends individual summaries to each salesperson configured in `SALESPEOPLE_EMAILS`. Each email contains:
- Personal greeting
- Individual performance metrics (objective, forecast, achievement)
- Alert counts (zombies, proposals, quality issues)
- Top 5 prioritized action items
- Motivational message based on performance

### 4. Send Everything

```bash
python skills/crm-report/scripts/main.py --email --email-salespeople
```

## Configuration

### Recipients

Edit `skills/crm-report/scripts/config.py`:

```python
EMAIL_RECIPIENTS = [
    "user1@indaws.es",
    "user2@indaws.es",
]

SALESPEOPLE_EMAILS = {
    "Salesperson Name": "email@indaws.es",
}
```

### SMTP Settings

| Variable | Default | Description |
|---|---|---|
| `EMAIL_SMTP_SERVER` | `smtp.gmail.com` | SMTP server |
| `EMAIL_SMTP_PORT` | `587` | SMTP port |
| `EMAIL_USE_TLS` | `true` | Enable STARTTLS |
| `EMAIL_FROM` | `victor@indaws.es` | Sender address |
| `EMAIL_USERNAME` | `victor@indaws.es` | SMTP login |
| `EMAIL_SMTP_PASSWORD` | (required) | SMTP password |

### Gmail App Password

For Gmail, you need an App Password (not your regular password):

1. Go to Google Account > Security > 2-Step Verification
2. At the bottom, select "App passwords"
3. Generate a new app password for "Mail"
4. Use this as `EMAIL_SMTP_PASSWORD`

## Troubleshooting

| Issue | Check |
|---|---|
| Authentication error | Verify EMAIL_USERNAME and EMAIL_SMTP_PASSWORD |
| Connection timeout | Check EMAIL_SMTP_SERVER and EMAIL_SMTP_PORT |
| Email not received | Check spam folder, verify recipient address |
| TLS error | Ensure EMAIL_USE_TLS matches server requirements |

## Rules

- **ALWAYS** test SMTP (`--test-email`) before sending
- **NEVER** send personalized emails without generating the report first
- **NEVER** hardcode email credentials — use environment variables

## Gotchas

1. Gmail requires App Password (not regular password) — 2FA must be enabled first
2. `SALESPEOPLE_EMAILS` keys must match salesperson names in Odoo exactly
3. HTML emails get stripped by some clients — the plain text fallback is critical
4. `--email-salespeople` without `--email` skips the team report — usually you want both
5. Rate limits: Gmail allows ~500 emails/day — not an issue for team size but watch in testing

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| crm-report | sales | Report generation (prerequisite) |

## Consumed by
No other skill consumes this directly.

## References
| File | Content |
|------|---------|
| common:references/shared/ | Shared patterns (connection, batch, fields) |
