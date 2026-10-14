# Data Analysis Patterns — Case Study Generator

Extract these facts before writing. Don't skip this step — the quality of the comments depends on it.

## Relationship timeline

- First invoice date = relationship start
- First helpdesk ticket date = proxy for go-live (customers file tickets once actively using the system)
- Sale order milestones: find orders by product name pattern:
  - "Migración ERP" or "Implantación ERP" → major project dates
  - "Enterprise" → Community→Enterprise upgrade date
  - "Docky" / "Chatty" / "IA" → AI product adoption date
  - "Cuota servidor" → moved to Indaws Cloud date
  - "Bolsa de horas" → ongoing development engagement

## Modules implemented

Infer from task names (tasks marked Hecho/Realizado = confirmed). Cross-check with ticket subjects. See `references/module_inference.md` for the full signal→module mapping table.

## Custom developments

Look in ticket subjects and task descriptions for: FACE, FACe, Facturae, MRW, SendCloud, Stripe, WhatsApp, VoIP, barcode, comisiones, Medalteca, límite de crédito, DUA, 4 decimales, and other non-standard keywords.

## Scale figures

Database migration task descriptions often contain comparison tables with record counts (manufacturing orders, invoices, quotations). Parse these for use in the outcome section.

## Team

- `user_id` on partner record = account owner
- Project/task assignees = project manager / consultants
- Contacts at customer: filter `res.partner` with `commercial_partner_id = <id>` to find all named users (useful for identifying second speaker, PM contact, etc.)
