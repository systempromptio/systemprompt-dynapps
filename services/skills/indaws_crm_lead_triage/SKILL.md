---
name: indaws-crm-lead-triage
description: >-
  Cualificación automática de leads del CRM de Odoo contra el ICP de Indaws. Extrae leads sin
  cualificar, enriquece datos vía Clay y registros mercantiles (eInforma/AXESOR), puntúa contra
  criterios ICP (+10 empleados, 1M€ facturación, sector, revenue), asigna comercial por provincia y
  ejecuta acciones en Odoo. Usar cuando pidan triaje de leads, cualificar pipeline, asignar leads
  nuevos o automatizar la cualificación del CRM.
version: 1.1.3
model: sonnet
effort: medium
---

# CRM Lead Triage

Cualificacion automatica de leads contra el ICP de Indaws, con enriquecimiento dual (Clay + registros mercantiles espanoles) y asignacion por territorio.

---

## Prerequisitos

- **odoo-pilot** (common skill) para conexion a Odoo via API
- **Credenciales Odoo**: `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, `ODOO_USER`
- See references/shared/odoo-connection.md for connection pattern.
- **Clay MCP** (recomendado): enriquecimiento de contactos/empresas
- **API registros mercantiles** (recomendado): `EINFORMA_API_KEY` o `AXESOR_API_KEY`
- **Node.js** + **Python 3.8+** (`pip install requests`)
- Config `config/province_assignment.json` rellenada con datos reales

---

## Workflow

> Detalle completo de cada paso, comandos, plantillas y cron: `references/workflow_detail.md`

| Paso | Accion | Script |
|------|--------|--------|
| 0 | Recopilar info: credenciales, Clay MCP, API key, provincias, parametros. **DETENTE y espera respuesta.** | -- |
| 1 | Extraer leads sin cualificar de Odoo (ultimos N dias) | `scripts/fetch_new_leads.sh` |
| 1.5 | Enriquecer con Clay MCP (opcional, si conectado) | Clay MCP tools |
| 2 | Enriquecer con registro mercantil espanol (eInforma/AXESOR) | `scripts/enrich_leads.py` |
| 3 | Scoring ICP automatico (0-100). **NUNCA cualificar manualmente.** | `scripts/qualify_leads.py` |
| 3.5 | Presentar resumen al usuario. **DETENTE y espera aprobacion.** | -- |
| 4 | Aplicar en Odoo: asignar comercial, mover etapa, crear actividades | `scripts/apply_triage.sh` |

**Scoring**: empleados 30% + facturacion 30% + sector 20% + revenue 20%. Umbrales: qualify >=60, review 35-59, discard <35. Ver `references/icp_criteria.md`.

---

## RULES

1. **NUNCA** cualifiques leads manualmente -- siempre `qualify_leads.py`
2. **NUNCA** apliques cambios en Odoo sin aprobacion del usuario (excepto modo cron)
3. **NUNCA** inventes datos -- todos los scores vienen del JSON calculado
4. Lead sin CIF/NIF valido va a revision manual, no a descartado
5. Modo `--dry-run` disponible en enriquecimiento y aplicacion para pruebas
6. Delay entre llamadas API: 0.5s (respetar rate limits)

---

## GOTCHAS

1. Leads sin partner (`res.partner`) no se pueden enriquecer -- van a revision manual
2. Clay MCP es opcional pero mejora significativamente la cobertura de datos
3. `config/province_assignment.json` debe estar actualizado antes de ejecutar
4. eInforma/AXESOR no encuentran autonomos, empresas nuevas ni extranjeras
5. CIF con prefijo "ES" se normaliza automaticamente (ES + B12345678 -> B12345678)
6. En modo cron, Clay no esta disponible (requiere Claude interactivo)
7. Si 0 leads extraidos, no hay nada que triagear -- informar y parar

---

## Delegates to

- **odoo-pilot** (common): todas las operaciones de lectura/escritura en Odoo

## Consumed by

- Ningun otro skill consume este skill directamente

---

## References

| File | Content |
|------|---------|
| `references/icp_criteria.md` | Definicion ICP, algoritmo scoring, umbrales, pesos por criterio |
| `references/enrichment_api.md` | APIs eInforma/AXESOR: endpoints, campos, formato CIF, rate limits |
| `references/workflow_detail.md` | Pasos detallados (0-4), comandos, plantillas, cron, config, estructura archivos, success criteria |
