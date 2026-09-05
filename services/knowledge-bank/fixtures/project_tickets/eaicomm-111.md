---
key: EAICOMM-111
title: "Storefront | PDP | Render the configurator steps from a template"
type: Story
status: In progress
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Ben Carver
project: EAICOMM
date: 2026-08-03
---

# EAICOMM-111 Storefront | PDP | Render the configurator steps from a template

## Context

First story of the PDP Configurator from "FSD - PDP Configurator": read the template named by c_configuratorTemplate, render the steps and options, and fall back to the flat variation selector when the template is missing or fails to parse. Pricing, constraints and preview are separate stories.

## Acceptance criteria

1. A product with a valid template shows the steps in template order with the options of each step.
2. A product whose template is missing or invalid shows the flat variation selector and logs a warning naming the product ID.
3. The template is validated against the schema with zod at load and a schema failure is treated as invalid.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Template loader in app/data/catalog/configurator.ts; steps rendered by NwtConfigurator under app/components/catalog. Schema in the same folder, shared with the upload validator to be built in EAICOMM-113.

## Comments

- **Finding** (Ben Carver, 2026-08-10): Templates larger than the content asset body limit need splitting; raised with the product data team, tracked as a note on EAICOMM-113.

Status: In progress. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Ben Carver.
