# CRM Pre-Qualification — Workflow detallado (DEPRECADO)

> ⚠️ Este workflow se conserva SOLO como referencia histórica del motor original
> (pre-consolidación). El motor activo es **`sales-skills:qualification-preflight`**.
> No ejecutes este flujo en producción ni lo programes en cron.

## Prerequisitos

- **odoo-pilot**: Skill de `ai-claude-common` para conexion a Odoo via API
- **Credenciales Odoo**: Variables `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, `ODOO_USER`
- **eInforma API**: Variables `EINFORMA_CLIENT_ID` y `EINFORMA_CLIENT_SECRET` en `.env`
- **Node.js**: Para ensamblado JSON en scripts bash
- **Python 3.8+**: Para scripts de enriquecimiento, extraccion y generacion de informe (`pip install requests`)

---

## Paso 0: Verificacion de Entorno

1. Verificar credenciales Odoo (variables de entorno o `.env`)
2. Verificar credenciales eInforma (`EINFORMA_CLIENT_ID`, `EINFORMA_CLIENT_SECRET`)
3. Localizar odoo-pilot scripts

> **DETENTE** si faltan credenciales. Pedir al usuario.

---

## Paso 1: Detectar Oportunidades Sin Cualificar

EJECUTAR `scripts/fetch_unqualified_opps.sh`:

```bash
source .env 2>/dev/null
export ODOO_URL ODOO_DB ODOO_KEY ODOO_USER

bash scripts/fetch_unqualified_opps.sh \
  --pilot-dir /path/to/odoo-pilot/scripts \
  > opps_unqualified.json
```

El script busca oportunidades con:
- `type = opportunity`
- Etapa "Lead" (primera etapa del pipeline)
- Activas (`active = true`)

**Verificar**: Que se obtuvieron oportunidades (si 0 -> nada que cualificar).

---

## Paso 2: Extraer Datos del Chatter

PARA CADA OPORTUNIDAD, EJECUTAR `scripts/fetch_opp_chatter.sh`:

```bash
bash scripts/fetch_opp_chatter.sh \
  --pilot-dir /path/to/odoo-pilot/scripts \
  --opp-id <ID> \
  > chatter_<ID>.json
```

Extrae `mail.message` con `message_type=email` del chatter de la oportunidad.

Luego EJECUTAR `scripts/extract_from_emails.py`:

```bash
python3 scripts/extract_from_emails.py --input chatter_<ID>.json > extracted_<ID>.json
```

Analiza emails por keywords para extraer: sistema actual, pain points, modulos interes, presupuesto, decisores, timeline.

---

## Paso 3: Enriquecer con eInforma

EJECUTAR `scripts/enrich_company.py`:

```bash
source .env
python3 scripts/enrich_company.py --vat <CIF> > enrichment_<ID>.json

# Modo pruebas (sin API)
python3 scripts/enrich_company.py --vat <CIF> --dry-run > enrichment_<ID>.json
```

OAuth2 flow: Client ID + Client Secret -> Access Token -> GET /companies/{cif}

Retorna: company_name, employees, turnover_eur, cnae_code, cnae_description, province, city, founded_year, legal_form, status, vat.

---

## Paso 4: Busqueda Web de la Empresa

EJECUTAR `scripts/web_search_company.py`:

```bash
python3 scripts/web_search_company.py --company "Test Corporation SL" > web_search_<ID>.json

# Modo pruebas (sin API)
python3 scripts/web_search_company.py --company "Test Corporation SL" --dry-run > web_search_<ID>.json
```

Busca informacion publica usando DuckDuckGo Instant Answer API + Wikipedia ES como fallback. No requiere API key.

Retorna: company_name, domain, summary (max 500 chars), source, found.

---

## Paso 5: Generar Informe de Texto Plano

EJECUTAR `scripts/generate_report.py`:

```bash
python3 scripts/generate_report.py \
  --opp opps_unqualified.json \
  --enrichment enrichment_<ID>.json \
  --extracted extracted_<ID>.json \
  --web-search web_search_<ID>.json \
  > report_<ID>.txt
```

Genera informe de texto plano estructurado con 8 secciones:
1. ACTIVIDAD DEL NEGOCIO
2. DIMENSIONAMIENTO (empleados, facturacion, usuarios)
3. SISTEMA ACTUAL (plataforma, version, partner)
4. PAIN POINTS IDENTIFICADOS
5. MODULOS DE INTERES
6. PRESUPUESTO & TIMELINE
7. MAPA DE DECISORES
8. SCORING DE CUALIFICACION (0-100, Tier A/B/C/D)

Incluye secciones adicionales:
- RESUMEN DE LA EMPRESA (de busqueda web, si hay datos disponibles)
- Tags sugeridos (PYME/MIDMARKET/ENTERPRISE, MIGRACION/PRIMERA-IMPLANTACION, etc.)
- SIGUIENTES PASOS personalizados segun tier y datos disponibles

Tags de confianza: `[AUTO]` (dato de API), `[PARCIAL]` (inferido de emails), `[PENDIENTE]` (falta dato).

**Scoring de cualificacion** (6 criterios, 100 puntos):

| Criterio | Peso | Que mide |
|----------|------|----------|
| Datos empresa | 20 | Enriquecimiento eInforma exitoso |
| Tamano empresa | 20 | >50 empleados o >1M facturacion |
| Pain points | 20 | Al menos 2 detectados |
| Modulos interes | 15 | Al menos 2 modulos |
| Presupuesto/timeline | 15 | Estado definido o comparativa |
| Decisores | 10 | Al menos 1 identificado |

**Tiers**: A (>=75, alta prioridad), B (>=50, media), C (>=30, baja), D (<30, investigar).

---

## Paso 6: Publicar en Chatter

EJECUTAR `scripts/apply_prequalification.sh`:

```bash
bash scripts/apply_prequalification.sh \
  --pilot-dir /path/to/odoo-pilot/scripts \
  --opp-id <ID> \
  --report report_<ID>.txt \
  --enrichment enrichment_<ID>.json \
  --extracted extracted_<ID>.json

# Modo prueba
bash scripts/apply_prequalification.sh --opp-id <ID> --report report_<ID>.txt --dry-run
```

Acciones:
1. Actualiza campos en `crm.lead` y `res.partner` via `update_record.sh`
2. Crea/asigna tags automaticos en `crm.tag` (PYME, MIGRACION, etc.)
3. Publica informe de texto plano como `mail.message` (nota interna) en el chatter

**NOTA**: `expected_revenue` NO se rellena automaticamente. Ese campo es para el valor estimado del proyecto Indaws, no la facturacion del prospecto. La facturacion anual del prospecto (eInforma) se incluye en la nota de enriquecimiento y en el informe del chatter.

### Campos actualizados en `crm.lead`
- `partner_name` (nombre empresa)
- `city` (ciudad)
- `state_id` (provincia -> res.country.state)
- `country_id` (Espana -> res.country)
- `description` (resumen pre-cualificacion)

### Campos actualizados en `res.partner`
- `vat` (CIF con prefijo ES)
- `company_registry` (CIF sin prefijo)
- `is_company` (true)
- `employee` (tiene empleados)
- `city` (ciudad)
- `state_id` (provincia)
- `country_id` (pais)
- `industry_id` (CNAE -> res.partner.industry)
- `comment` (resumen eInforma)

---

## Paso 7: Validacion

- Verificar informe de texto visible en chatter de Odoo
- Verificar campos actualizados en oportunidad y partner
- Verificar que relational IDs (state_id, country_id, industry_id) se resolvieron correctamente

---

## Ejecucion Automatizada (DESACTIVAR)

Scheduled task `crm-pre-qualification-auto` ejecutaba cada 3 horas:

```
Cron: 0 */3 * * *
```

> ⚠️ **Desactivar este cron.** Duplica escrituras con qualification-preflight.

En modo automatizado procesaba TODAS las oportunidades en etapa "Lead" sin detenerse. Orden:
1. `fetch_unqualified_opps.sh` -> lista opps
2. Por cada opp: chatter -> extract -> enrich -> web search -> generate report -> apply
3. Log de resultados

---

## Configuracion

### `config/qualification_fields.json`
Mapeo de campos: informe <-> Odoo fields <-> eInforma response. Keywords para extraccion de emails.

Ver `references/form_fields_spec.md` para documentacion completa de campos y mapeo.

---

## Estructura

```
crm-pre-qualification/
├── SKILL.md
├── config/
│   └── qualification_fields.json
├── scripts/
│   ├── fetch_unqualified_opps.sh
│   ├── fetch_opp_chatter.sh
│   ├── enrich_company.py
│   ├── extract_from_emails.py
│   ├── web_search_company.py
│   ├── generate_report.py
│   └── apply_prequalification.sh
├── references/
│   ├── form_fields_spec.md
│   └── workflow.md
└── tests/
    ├── test_extract.py
    ├── test_generate_report.py
    └── fixtures/
        ├── opp.json
        ├── enrichment.json
        ├── chatter.json
        ├── extracted.json
        └── web_search.json
```

---

## Tests

73 tests unitarios cubren la logica de extraccion, generacion de informes y scoring:

```bash
# Ejecutar todos los tests
cd skills/crm-pre-qualification
python3 -m pytest tests/ -v

# Tests individuales
python3 -m pytest tests/test_extract.py -v          # 25 tests
python3 -m pytest tests/test_generate_report.py -v   # 48 tests
```

---

## Reglas originales

1. **NUNCA** inventes datos en el formulario. Solo datos de Odoo, emails o eInforma.
2. **NUNCA** modifiques la etapa de la oportunidad. Solo la pre-cualificacion es automatica.
3. Campos sin dato se marcan como `[PENDIENTE]` en el informe.
4. Delay entre llamadas API: 0.5s (respetar rate limits eInforma).
5. eInforma usa OAuth2 (Client Credentials), NO Bearer token directo.
6. El informe se publica como texto plano envuelto en `<pre>` (compatible chatter Odoo).
7. Oportunidades procesadas avanzan naturalmente de etapa "Lead" por el comercial.
8. Lookups relacionales (state_id, country_id, industry_id) se validan como numericos.

---

## Success Criteria (histórico)

- [ ] Oportunidades en etapa "Lead" detectadas correctamente
- [ ] Emails del chatter extraidos y analizados
- [ ] Enriquecimiento eInforma ejecutado (o dry-run)
- [ ] Busqueda web de la empresa ejecutada
- [ ] Campos Odoo actualizados (crm.lead + res.partner, 16 campos)
- [ ] Informe de texto plano de 8 secciones publicado en chatter
- [ ] Scoring A/B/C/D calculado con acciones recomendadas
- [ ] Tags automaticos asignados en Odoo (tamano, tipo proyecto)
- [ ] Scheduled task configurado (cada 3h)
- [ ] 73 tests unitarios pasando
