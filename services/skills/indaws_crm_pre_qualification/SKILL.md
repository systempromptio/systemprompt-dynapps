---
name: indaws-crm-pre-qualification
description: >
  DEPRECADO v1.1.0 — usar qualification-preflight. Conservado solo como referencia del mapeo
  de campos eInforma y logica de scoring A/B/C/D. NOT para procesar leads → qualification-preflight.
  Triggers: (no invocar) crm-pre-qualification legacy, mapeo campos eInforma,
  logica scoring leads CRM, referencia pre-cualificacion v1,
  campos auto-relleno lead Odoo.
version: 1.1.2
model: sonnet
effort: medium
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep]
---

# CRM Pre-Qualification

> ⚠️ **DEPRECADO (v1.1.0).** El motor único de pre-cualificación SDR es ahora
> **`sales-skills:qualification-preflight`**. **Desactiva el cron de 3h** para no
> duplicar escrituras. Este fichero se conserva como referencia del mapeo de campos
> (`config/qualification_fields.json`, `references/form_fields_spec.md`) y lógica eInforma.

**CONFIDENTIAL - SALES TEAM ONLY**

Pre-cualificacion automatica de oportunidades del CRM de Indaws. Motor original (pre-consolidacion): detecta leads en etapa "Lead", analiza chatter + emails, enriquece con eInforma, calcula scoring A/B/C/D y publica informe en el chatter.

**Diferencia con crm-lead-triage:** lead-triage PUNTUA leads contra ICP (0-100). Este skill PRE-CUALIFICA rellenando formulario comercial estructurado de 7 secciones.

## Workflow

Ver detalle completo en [references/workflow.md](references/workflow.md).

1. **Paso 0 — Verificar entorno**: credenciales Odoo + eInforma + odoo-pilot. DETENTE si faltan.
2. **Paso 1 — Detectar oportunidades**: `scripts/fetch_unqualified_opps.sh` → etapa "Lead", activas.
3. **Paso 2 — Extraer chatter**: `scripts/fetch_opp_chatter.sh` + `scripts/extract_from_emails.py` por oportunidad.
4. **Paso 3 — Enriquecer con eInforma**: `scripts/enrich_company.py --vat <CIF>` (OAuth2). Fallback `--dry-run`.
5. **Paso 4 — Búsqueda web**: `scripts/web_search_company.py` (DuckDuckGo + Wikipedia ES, sin API key).
6. **Paso 5 — Generar informe**: `scripts/generate_report.py` → 8 secciones + scoring A/B/C/D.
7. **Paso 6 — Actualizar Odoo**: `scripts/apply_prequalification.sh` → 16 campos en `crm.lead` + `res.partner`.
8. **Paso 7 — Publicar en chatter**: nota interna con informe + tags + acciones recomendadas.

## RULES

**NEVER**
1. Usar este skill para procesar leads nuevos — usar `qualification-preflight` (el motor unificado).
2. Ejecutar el cron de 3h — desactivarlo para evitar conflicto con qualification-preflight.
3. Escribir campos que qualification-preflight ya gestiona — duplica datos.
4. Invocar sin credenciales eInforma (`EINFORMA_CLIENT_ID`/`SECRET`) — falla silenciosamente.
5. Usar este SKILL.md como referencia de campos: ver `config/qualification_fields.json` y `references/form_fields_spec.md`.

**ALWAYS**
1. Tratar este skill como SOLO LECTURA de referencia — no para producción.
2. Redirigir al usuario a `qualification-preflight` si pide pre-cualificación en vivo.
3. Consultar `references/form_fields_spec.md` para el mapeo completo de los 16 campos Odoo.
4. Verificar credenciales antes de cada script (Odoo + eInforma + odoo-pilot path).
5. Usar `--dry-run` en eInforma si no hay API key — evita errores 404.

## GOTCHAS

1. **eInforma 404 frecuente** — el CIF debe ir sin prefijo `ES`. `ES12345678A` → `12345678A`. Fallback a `--dry-run` si sigue fallando.
2. **Cron activo = doble escritura** — si qualification-preflight corre cada hora Y este cron cada 3h, los mismos campos se escriben desde dos sitios. Desactivar este cron YA.
3. **odoo-pilot path hardcodeado** — `--pilot-dir` requiere ruta absoluta del directorio scripts de odoo-pilot. Si el plugin se mueve, los scripts fallan sin aviso claro.
4. **Scoring A/B/C/D no es el ICP score de lead-triage** — son sistemas distintos. A/B/C/D = calidad de cualificación (datos disponibles); 0-100 de lead-triage = fit con ICP.
5. **`extract_from_emails.py` solo extrae de `message_type=email`** — los mensajes internos del chatter (notas) no se analizan. Pain points escritos como notas → no se capturan.
6. **DuckDuckGo Instant Answer API** devuelve vacío para empresas sin Wikipedia — esperado. El script no falla, devuelve `{"found": false}`.

## Delegates to

| Skill | Suite | Para qué |
|-------|-------|---------|
| `odoo-pilot` | common-skills | Conexión Odoo (auth + search_read + write) |
| `qualification-preflight` | sales-skills | Motor activo — redirigir aquí todo procesamiento en vivo |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|---------|
| (ninguno activo) | — | Deprecado. qualification-preflight es el consumidor del mapeo de campos |

## REFERENCES

- [references/workflow.md](references/workflow.md) — Workflow completo detallado (Paso 0-8) con comandos bash, scripts y ejemplos.
- [references/form_fields_spec.md](references/form_fields_spec.md) — Especificación completa de los 16 campos Odoo (fuente de verdad del mapeo).
- `config/qualification_fields.json` — Configuración JSON del mapeo de campos.
- `references/shared/odoo-connection.md` — Patrones de conexión Odoo (odoo-pilot).
