---
name: indaws-accounting-auditor
version: 1.0.4
effort: high
model: opus
description: >-
  Ejecuta auditoría contable experta sobre Odoo con ~42 comprobaciones automáticas basadas en el PGC
  español. Conecta vía odoo-pilot, extrae datos del período, detecta errores e inconsistencias por
  severidad, presenta dashboard de validación y genera informe de 11 secciones. Solo lectura. Usar
  cuando pidan auditoría contable (PGC español) sobre Odoo. NOT health check mensual de cierre
  para entregar al cliente (PDF + Excel de evidencias) → health-check-contable.
---

# Auditoría Contable — Odoo + PGC Español

## Workflow

1. **Paso 0 — Recopilación**: Pedir credenciales Odoo + rango de fechas + company_id opcional. DETENTE y espera.
2. **Paso 1 — Extracción**: Ejecutar `scripts/fetch_accounting_data.sh <date_from> <date_to>`. Obtiene 10 conjuntos de datos (plan contable, asientos, líneas, diarios, extractos bancarios, company, partners, módulos l10n_es, activos, impuestos).
3. **Paso 2 — Auditoría**: Ejecutar `python3 scripts/audit_accounting.py < /tmp/audit_data.json > /tmp/audit_results.json`. ~42 checks en 10 categorías. Ver `references/audit_rules.md`.
4. **Paso 2.5 — Dashboard**: DETENTE — presentar Score/100 + tabla por categoría + top críticos. Esperar confirmación.
5. **Paso 3 — Informe**: Solo tras validación. Generar `informe_auditoria_{company}_{YYYY}_{MM}.md` con 11 secciones. Ver `references/workflow-detail.md`.

Ver `references/workflow-detail.md` para el detalle completo de cada paso.

> Note: `common:references/shared/` — auth pattern via `auth.sh`, batch reads via `search_records.sh`.

## GOTCHAS

1. **Sin módulo l10n_es instalado** → avisar pero continuar; los checks de IVA/IRPF darán falsos negativos.
2. **Sin asientos en el período** → terminar limpiamente, no ejecutar el script de análisis.
3. **Lock dates por compañía** → leer siempre de `res.company` de la compañía seleccionada; en multi-compañía el default es la del usuario autenticado.
4. **Activos fijos**: el modelo `account.asset` puede no existir si el módulo no está instalado → capturar el error y omitir esa categoría.
5. **Importes en formato español** → usar punto miles y coma decimal en el informe; los datos en Odoo vienen en formato numérico estándar.
6. **El script `audit_accounting.py` lee de stdin** → siempre pipear el JSON, no pasar como argumento.
7. **company_id en búsquedas** → incluirlo en el dominio de cada `search_records.sh`; sin él en multi-compañía se mezclan datos.

## NEVER
- Modificar datos en Odoo. Este skill es solo lectura.
- Inventar datos. Si falta información, indicarlo en el informe.
- Generar el informe sin haber validado el dashboard (Paso 2.5).

## ALWAYS
- Pedir credenciales y rango de fechas antes de empezar (Paso 0).
- Usar odoo-pilot (`auth.sh`) con contexto `{"lang": "es_ES"}`.
- Incluir filtro `company_id` en todas las búsquedas.

## Delegates to

| Skill | Suite | Para qué |
|-------|-------|----------|
| `odoo-pilot` | common | Autenticación y consultas a Odoo |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| — | — | Skill terminal de dirección |

## REFERENCES

- `references/audit_rules.md` — Documentación de las ~42 reglas de auditoría
- `references/pgc_chart.md` — Plan General Contable español (cuentas de referencia)
- `references/workflow-detail.md` — Detalle completo de cada paso del workflow
- `templates/informe_auditoria.md` — Plantilla del informe de 11 secciones
- `scripts/fetch_accounting_data.sh` — Extracción de 10 conjuntos de datos
- `scripts/audit_accounting.py` — ~42 comprobaciones en 10 categorías
