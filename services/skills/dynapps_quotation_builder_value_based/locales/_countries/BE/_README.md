# Capa país — Bélgica (BE)  ·  ESQUELETO (inerte hasta Fase 1)

Owner de país: **Bélgica (Dynapps NV)**. Contrato: `../../README.md` §1–§6. Plan: `../../../references/add_be_ch_plan.md`.

Composición objetivo: `nl-BE` = idioma `nl` ⊕ país `BE`; `fr-BE` = idioma `fr` ⊕ país `BE`.
Ficheros de esta capa: `instance.json`, `pricing.json`, `fiscal.json`, `format.currency.json`.

Pendiente para pasar a `live` (gate §4): rellenar cada `<<VERIFY>>` / `verify_from_odoo` contra el Odoo BE,
autorar `fiscal.json` bajo **firma fiscal/legal belga**, y firma de hablante nativo por idioma (nl-BE, fr-BE).
Hechos precargados (confirmar): BTW/TVA 21%, mandato Peppol B2B desde 2026-01-01, `cases_base_url` = dynapps.be/{lang}/cases.
