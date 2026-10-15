# CRM Lead Triage - Workflow Detail

## Stack de Enriquecimiento

El triaje usa un sistema de enriquecimiento en cascada con dos capas complementarias:

| Capa | Herramienta | Datos | Cuando usar |
|------|-------------|-------|-------------|
| **1. Clay MCP** | Clay (via MCP) | Contactos, email, cargo, empresa, tech stack, funding, senales de compra | Siempre (si conectado) |
| **2. Registro mercantil** | eInforma/AXESOR (via API) | CIF/NIF, empleados, facturacion, sector CNAE, provincia (fuentes oficiales) | Siempre para leads espanoles |

**Por que dos capas:**
- Clay agrega datos de 50+ proveedores internacionales (mejor cobertura de contactos)
- eInforma/AXESOR tienen datos oficiales del Registro Mercantil espanol (mas precisos para empresas espanolas)
- Juntos dan la imagen mas completa posible de cada lead

---

## Paso 0: Recopilacion de Informacion

PREGUNTAR al usuario:

1. **Credenciales de Odoo** (obligatorio):
   - URL, DB, API Key. Si ya tiene variables configuradas, confirmar.

2. **Clay MCP** (recomendado):
   - Verificar si Clay MCP esta conectado (buscar herramientas `clay_*` disponibles)
   - Si no esta conectado: el triaje funciona solo con eInforma (modo basico)

3. **API de enriquecimiento espanol** (recomendado):
   - Proveedor: eInforma (defecto) o AXESOR
   - API Key configurada en variable de entorno

4. **Mapeo de provincias** (obligatorio si primera ejecucion):
   - Verificar que `config/province_assignment.json` tiene datos reales
   - Si no, pedir al usuario la lista de provincias por comercial

5. **Parametros opcionales**:
   - Dias de lookback (defecto: 7)
   - Umbral de cualificacion (defecto: 60)
   - Modo dry-run para pruebas

> **DETENTE** aqui y espera respuesta del usuario.

---

## Paso 1: Extraccion de Leads

EJECUTAR `scripts/fetch_new_leads.sh`:

```bash
export ODOO_URL="<url>" ODOO_DB="<db>" ODOO_KEY="<key>" ODOO_USER="<user>"

bash scripts/fetch_new_leads.sh \
  --pilot-dir /path/to/odoo-pilot/scripts \
  --days 7 \
  > leads_raw.json
```

El script extrae:
1. Leads sin comercial asignado (ultimos N dias)
2. Leads en primera etapa del pipeline
3. Etapas del CRM
4. Datos del partner (res.partner) con CIF/NIF

**Verificar**: Que se obtuvieron leads (si 0 -> no hay nada que triagear).

---

## Paso 1.5: Enriquecimiento Clay (si disponible)

Si Clay MCP esta conectado, PARA CADA LEAD:

1. Buscar el contacto en Clay por nombre de empresa o email:
   - Usar herramienta Clay MCP de busqueda de contactos
   - Parametros: nombre empresa (`partner_name`), email (`email_from`), ciudad

2. Si se encuentra, extraer:
   - Email verificado del contacto
   - Cargo/titulo
   - Tamano de empresa
   - Sector/industria
   - Tech stack (si disponible)
   - Senales de compra (funding, contrataciones, etc.)

3. Guardar datos Clay en el JSON de leads (campo `clay_enrichment`):
   ```json
   {
     "clay_enrichment": {
       "found": true,
       "email_verified": "contacto@empresa.com",
       "title": "CEO",
       "company_size": "50-100",
       "industry": "Manufacturing",
       "tech_stack": ["SAP", "Excel"],
       "signals": ["Recent funding", "Hiring IT"]
     }
   }
   ```

**NOTA**: Este paso es opcional. Si Clay no esta conectado, continuar al Paso 2.

---

## Paso 2: Enriquecimiento Registro Mercantil

EJECUTAR `scripts/enrich_leads.py`:

```bash
# Con API real (eInforma)
export EINFORMA_API_KEY="<api_key>"
python3 scripts/enrich_leads.py --input leads_raw.json > leads_enriched.json

# Modo pruebas (sin API)
python3 scripts/enrich_leads.py --input leads_raw.json --dry-run > leads_enriched.json

# Con AXESOR en vez de eInforma
export AXESOR_API_KEY="<api_key>"
python3 scripts/enrich_leads.py --input leads_raw.json --provider axesor > leads_enriched.json
```

El script:
- Valida CIF/NIF de cada lead
- Consulta API de registros mercantiles
- Obtiene: empleados, facturacion, sector CNAE, provincia
- Leads sin CIF valido se marcan para revision manual

**Verificar**: Cuantos leads se enriquecieron vs no encontrados.

---

## Paso 3: Cualificacion y Scoring (OBLIGATORIO)

EJECUTAR `scripts/qualify_leads.py`:

```bash
python3 scripts/qualify_leads.py \
  --input leads_enriched.json \
  --scoring-config config/scoring_config.json \
  --province-config config/province_assignment.json \
  > triage_results.json
```

> **IMPORTANTE**: NUNCA cualifiques leads manualmente. El script aplica los criterios documentados en `references/icp_criteria.md`.

El script calcula:
- Score 0-100 por lead (empleados 30% + facturacion 30% + sector 20% + revenue 20%)
- Cumplimiento ICP estricto (>=10 emp AND >=1M EUR)
- Accion recomendada: qualify (>=60), review (35-59), discard (<35)
- Asignacion de comercial por provincia

---

## Paso 3.5: Validacion de Resultados

PRESENTAR al usuario un resumen:

```
=== TRIAJE CRM ===
Total leads analizados: {total}
Score medio: {avg}/100

CUALIFICADOS (>= 60): {n} leads
  {lead_1}: {score}/100 -> {comercial} ({provincia})
  {lead_2}: {score}/100 -> {comercial} ({provincia})

REVISION MANUAL (35-59): {n} leads
  {lead_3}: {score}/100 - Motivo: {razon}

DESCARTADOS (< 35): {n} leads
  {lead_4}: {score}/100

ICP Match: {n}/{total} ({pct}%)
Sin datos de empresa: {n} (iran a revision)

Datos Clay disponibles: {n}/{total}
Datos eInforma disponibles: {n}/{total}
```

> **DETENTE** y espera aprobacion del usuario antes de aplicar cambios en Odoo.

---

## Paso 4: Aplicar Acciones en Odoo

Solo tras aprobacion del usuario. EJECUTAR `scripts/apply_triage.sh`:

```bash
# Modo prueba (no cambia nada en Odoo)
bash scripts/apply_triage.sh --input triage_results.json --dry-run

# Ejecucion real
bash scripts/apply_triage.sh \
  --input triage_results.json \
  --pilot-dir /path/to/odoo-pilot/scripts
```

Acciones ejecutadas:
- **Cualificados**: Asignar comercial (`user_id`) + mover a etapa "Cualificado" + prioridad alta
- **Revision**: Crear actividad (`mail.activity`) para el responsable con nota del motivo
- **Descartados**: Sin accion automatica (log informativo)

---

## Ejecucion Automatizada (Cron)

Para ejecucion diaria sin intervencion:

```bash
#!/bin/bash
# cron_triage.sh - Ejecucion automatizada diaria
set -euo pipefail

SKILL_DIR="/path/to/crm-lead-triage"
PILOT_DIR="/path/to/odoo-pilot/scripts"
OUTPUT_DIR="/tmp/triage_$(date +%Y%m%d)"
mkdir -p "$OUTPUT_DIR"

# Paso 1: Extraer
bash "$SKILL_DIR/scripts/fetch_new_leads.sh" --pilot-dir "$PILOT_DIR" --days 1 > "$OUTPUT_DIR/leads.json"

# Paso 2: Enriquecer (eInforma - Clay requiere Claude interactivo)
python3 "$SKILL_DIR/scripts/enrich_leads.py" --input "$OUTPUT_DIR/leads.json" > "$OUTPUT_DIR/enriched.json"

# Paso 3: Cualificar
python3 "$SKILL_DIR/scripts/qualify_leads.py" \
  --input "$OUTPUT_DIR/enriched.json" \
  --scoring-config "$SKILL_DIR/config/scoring_config.json" \
  --province-config "$SKILL_DIR/config/province_assignment.json" \
  > "$OUTPUT_DIR/triage.json"

# Paso 4: Aplicar
bash "$SKILL_DIR/scripts/apply_triage.sh" --input "$OUTPUT_DIR/triage.json" --pilot-dir "$PILOT_DIR"
```

---

## Configuracion

### `config/province_assignment.json`

Mapeo de provincias espanolas a comerciales. **RELLENAR antes de usar.**
Campos por comercial: `name`, `odoo_user_id` (ID numerico en Odoo), `email`, `provinces` (array).

### `config/scoring_config.json`

Umbrales ICP, pesos de scoring, rangos de puntuacion por criterio, proveedor de enriquecimiento.
Ver `references/icp_criteria.md` para detalle de los algoritmos.

---

## Estructura de Archivos

```
crm-lead-triage/
├── SKILL.md
├── config/
│   ├── province_assignment.json    # Mapa provincias -> comerciales (RELLENAR)
│   └── scoring_config.json         # Umbrales ICP y pesos de scoring
├── scripts/
│   ├── fetch_new_leads.sh          # Extraccion de leads via odoo-pilot
│   ├── enrich_leads.py             # Enriquecimiento via registros mercantiles
│   ├── qualify_leads.py            # Motor de scoring ICP (OBLIGATORIO)
│   └── apply_triage.sh             # Aplicar acciones en Odoo
└── references/
    ├── icp_criteria.md             # Definicion ICP y algoritmo de scoring
    ├── enrichment_api.md           # Documentacion APIs de enriquecimiento
    └── workflow_detail.md          # Pasos detallados, cron, config, archivos
```

---

## Success Criteria

- [ ] Leads extraidos correctamente de Odoo (con partner y CIF/NIF)
- [ ] Enriquecimiento Clay ejecutado (si MCP disponible)
- [ ] Enriquecimiento eInforma ejecutado (API o dry-run)
- [ ] Scoring calculado con `qualify_leads.py`
- [ ] Resultados presentados y aprobados por el usuario
- [ ] Leads cualificados: comercial asignado + etapa movida en Odoo
- [ ] Leads revision: actividad creada en Odoo para responsable
- [ ] Config de provincias verificada con datos reales
