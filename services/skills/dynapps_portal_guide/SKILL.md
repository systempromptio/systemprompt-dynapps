---
name: dynapps-portal-guide
description: Route Odoo questions and actions to the Dynapps portal servers - DynBrain for how-to and
  internal knowledge, Dynapps Odoo for live records - and explain how to connect each one.
---

# Dynapps portal guide

Use when someone asks about Dynapps' own Odoo data, Dynapps internal knowledge,
or why a `dynapps-odoo` or `dynbrain` tool is missing or refusing.

## Which server

- **DynBrain** (`dynbrain`) — the Dynapps knowledge system. Ask it first for
  how-to answers, internal procedures, past project knowledge and anything that
  is documentation rather than a live record. Quote what it returns and say it
  came from DynBrain.
- **Dynapps Odoo** (`dynapps-odoo`) — live records in Dynapps' Odoo on
  portal.dynapps.es. Use it to read or change actual data. It acts as the
  person's own Odoo user, so Odoo's own access rules decide what it sees.

Prefer a read before any write. Before a create, update or delete, state the
record and the change and wait for the person to confirm; the client will also
ask before each call.

## When a tool is missing or refuses

Both servers are connected per person. If the tools are absent or a call
reports that authorization is required, tell the person to open
**Connectors** in the Dynapps dashboard (`/admin/connectors`), press
**Connect** on *Dynapps Odoo* or *DynBrain*, and sign in with their own
portal.dynapps.es account. One connection serves every client they use. A
connection marked *Reconnect required* needs the same step again.

Do not ask for passwords or API keys, and do not try another account.
