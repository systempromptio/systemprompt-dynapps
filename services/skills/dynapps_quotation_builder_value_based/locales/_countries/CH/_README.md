# Capa país — Suiza (CH)  ·  ESQUELETO (inerte hasta Fase 1)

Owner de país: **Suiza (entidad CH dedicada)**. Contrato: `../../README.md` §1–§6. Plan: `../../../references/add_be_ch_plan.md`.

Composición objetivo: `fr-CH` = idioma `fr` ⊕ país `CH`; `en-CH` = idioma `en` ⊕ país `CH`.
**VERIFICADO (web):** dynapps.ch es la Suiza francófona (Romandía; oficinas Bulle/Fribourg, Martigny/Valais) y solo sirve **FR + EN**
(`/fr/cases` y `/en/cases` = 200; `/de/cases` y `/it/cases` = 404). **NO hay sitio suizo DE/IT** → de-CH/it-CH no aplican y
**no hace falta crear capas de idioma nuevas**: CH reutiliza las capas `fr` y `en` existentes.
Ficheros de esta capa: `instance.json`, `pricing.json`, `fiscal.json`, `format.currency.json`.

Particularidades CH (confirmar): moneda **CHF** (símbolo delante, miles `'`), **no-UE** (sin OSS/IOSS/Intrastat/reverse-charge UE),
MWST 8,1%, **QR-bill** como método de pago, entidad legal CH dedicada con nº MWST propio, `cases_base_url` = dynapps.ch/{lang}/cases (lang ∈ fr|en).
Motor: relajar `scripts/odoo_discovery.py:688-689`, `_smoke` para CHF. Gate §4 + firma fiscal/legal CH + nativo por idioma antes de `live`.
