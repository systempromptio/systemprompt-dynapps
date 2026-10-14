---
name: indaws-crm-debug
description: >
  Debug utilities for the CRM report system. Use when the report shows
  unexpected data, metrics seem wrong, or you need to trace a specific
  calculation. Provides 8 diagnostic scripts for proposals, alerts, names,
  Odoo views, metrics, and duplicates.
version: 1.0.2
model: sonnet
effort: low
---

# CRM Debug Utilities

**CONFIDENTIAL - SALES TEAM ONLY**
**Document ID:** SALES-SKILL-003
**Classification:** Critical Sales Know-How

## Overview

A collection of 8 diagnostic scripts for troubleshooting CRM report issues. Each script targets a specific area and produces focused output.

## When to Use This Skill

- Report shows unexpected numbers or missing data
- Salesperson names do not match between Odoo and config
- Proposal alerts are not triggering correctly
- Forecast calculations seem wrong
- Odoo website page is not updating
- Duplicate pages or views detected

## Workflow

Run the specific diagnostic script for the symptom. See diagnostics below.

## Available Diagnostics

### 1. Check Proposals

```bash
python skills/crm-report/diagnostics/check_proposals.py
```

Verifies proposal stage matching against configured keywords. Shows which leads are in proposal stages and whether they should trigger alerts.

### 2. Check Alerts

```bash
python skills/crm-report/diagnostics/check_alerts.py
```

Inspects the Odoo view content to verify that expected report sections are present.

### 3. Check Names

```bash
python skills/crm-report/diagnostics/check_names.py
```

Compares salesperson names between Odoo and config. Detects mismatches that cause leads to be excluded from the report.

### 4. Check Odoo View

```bash
python skills/crm-report/diagnostics/check_odoo_view.py
```

Shows the status of the report website page and view in Odoo. Lists all matching pages and views with their IDs and update dates.

### 5. Debug Metrics

```bash
python skills/crm-report/diagnostics/debug_metrics.py
```

Traces the forecast calculation step by step. Outputs to `debug_forecast_result.txt` for detailed analysis.

### 6. Trace Proposal

```bash
python skills/crm-report/diagnostics/trace_proposal.py
```

Shows the proposal detection logic for each lead. Traces keyword matching to explain why a lead is or is not flagged as a proposal.

### 7. Find Duplicates

```bash
python skills/crm-report/diagnostics/find_duplicates.py
```

Searches for duplicate report pages and views in Odoo. Duplicates can cause the wrong page to be updated.

### 8. Verify Generation

```bash
python skills/crm-report/diagnostics/verify_generation.py
```

Generates a report with mock data to verify the HTML generation works without needing a live Odoo connection.

## Prerequisites

All scripts (except verify_generation.py) require Odoo connection environment variables.
See references/shared/odoo-connection.md for connection pattern.

| Variable | Required |
|---|---|
| `ODOO_URL` | Yes |
| `ODOO_DB` | Yes |
| `ODOO_USERNAME` | Yes |
| `ODOO_PASSWORD` | Yes |

## Rules

- **ALWAYS** run `--test` first to verify Odoo connection before diagnostics
- **NEVER** modify Odoo data — all scripts are read-only
- Run the specific diagnostic for the symptom, not all 8 sequentially

## Gotchas

1. Scripts use `ODOO_USERNAME`/`ODOO_PASSWORD` (XML-RPC), not `ODOO_USER`/`ODOO_KEY` (JSON-RPC) — different from odoo-pilot
2. Name mismatches between Odoo and config.py are the #1 cause of missing data in reports
3. Duplicate Odoo website pages cause the wrong one to be updated — run find_duplicates.py first
4. `verify_generation.py` is the only script that works without Odoo connection (uses mock data)
5. Stage keywords in config.py are case-sensitive — check exact match

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Odoo connection for diagnostics (via XML-RPC in scripts) |

## Consumed by
| Skill | Suite | When |
|-------|-------|------|
| crm-report | sales | When report shows unexpected data |

## Success Criteria

- Script runs without errors
- Output clearly indicates the source of any issue
- Actionable recommendations provided
