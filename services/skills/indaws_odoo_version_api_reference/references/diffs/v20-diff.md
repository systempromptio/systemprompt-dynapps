<!-- PARTIALLY UNVERIFIED: v18/v19 facts verified vs $ODOO_SRC; v20 API reported, NOT verified vs source (no v20 checkout locally) @ 2026-07 -->
# v20 Diff — Changes from v19 to v20 (REPORTING ENGINE only)

> ⚠ **Alcance y fiabilidad.** No hay checkout de Odoo 20 en local, así que este archivo **NO está
> verificado contra fuente** salvo la parte negativa (lo que NO existe en v18/v19, sí verificado).
> Trata los nombres de API v20 como **reportados, a confirmar** cuando haya fuente v20. Todo lo demás
> de `v18-base/` sigue siendo la referencia canónica.

## Motor de PDF: Paper Muncher (reportado)

**Verificado (grep en v18 y v19): NO existe** ningún módulo `base_report_paper_muncher`, ningún
`report_type = 'qweb-pdf-paper-muncher'`, ni métodos `_run_pdf_engine` / `_get_pdf_engine` /
`get_pdf_engine_state`. **v18 y v19 son wkhtmltopdf-only** (motor WebKit ~2012: sin flex/grid/CSS
moderno; ver `references/gotchas.md` y `odoo-qweb-report-builder`).

**v20 — REPORTADO, sin verificar contra fuente:**
- Odoo introduciría en v20 un **motor de PDF propio, "Paper Muncher"**, que sustituye/complementa a
  wkhtmltopdf y **soporta CSS moderno** (flexbox, grid, etc.).
- Sería **enchufable** vía el campo `report_type` (nuevo valor reportado
  `'qweb-pdf-paper-muncher'`) y un juego de métodos de motor reportados: `_run_pdf_engine`,
  `_get_pdf_engine`, `get_pdf_engine_state`. **Estos nombres NO están confirmados contra código v20.**

### Implicación práctica (esto SÍ es accionable hoy)
La emisión de informes que quiera **relajar los límites de wkhtmltopdf** (usar CSS moderno) debe ser
**motor-aware**: comprobar qué motor renderiza antes de asumir soporte de flex/grid. En v18/v19 el motor
es wkhtml y el CSS moderno **rompe** el PDF — no lo uses ahí. No escribas plantillas que dependan de
Paper Muncher hasta poder confirmar la versión objetivo (v20) y su API contra fuente.

## Todo lo demás
Sin datos verificados de otros cambios v19→v20. Para cualquier otra API, usa `v18-base/` + `v19-diff.md`
y **verifica contra fuente v20** cuando esté disponible antes de emitir código específico de v20.
