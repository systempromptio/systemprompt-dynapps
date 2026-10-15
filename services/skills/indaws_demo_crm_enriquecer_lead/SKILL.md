---
name: indaws-demo-crm-enriquecer-lead
description: >
  Enriquecimiento y cualificacion de leads del CRM de Odoo. En modo interactivo
  (1 lead) lee el lead completo incluyendo description previa y chatter, consulta
  empresite.eleconomista.es (rango de facturacion, empleados, CIF, sede),
  investiga en web la presencia internacional, oficinas y contexto relevante,
  puntua contra ICP, asigna comercial por provincia, crea la actividad siguiente
  y consolida toda la informacion en la descripcion del lead. En modo batch/cron
  procesa multiples leads via registros mercantiles (eInforma/AXESOR).
  Usar cuando el usuario pida: enriquecer un lead, buscar info de una empresa,
  cualificar pipeline, triaje de leads, evaluar calidad de leads.
version: 2.0.1
model: sonnet
effort: medium
---

# Demo · CRM enriquecer lead

Enriquecimiento conversacional de un lead de Odoo con datos de mercado (empresite.eleconomista.es + busqueda web), scoring ICP y actualizacion de la descripcion del lead.

## Modos de operacion

- **Interactivo** (default en Claude Code): procesa UN lead concreto. Lee chatter + description previa, enriquece via WebFetch/WebSearch, consolida toda la informacion en `crm.lead.description` y crea la actividad siguiente. Es el modo de la demo.
- **Batch / Cron**: procesa N leads sin cualificar usando API de registros mercantiles (eInforma/AXESOR). Ver `references/workflow_detail.md`.

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

## Workflow interactivo (1 lead) — modo demo

| Paso | Accion | Herramienta |
|------|--------|-------------|
| 1 | Identificar el lead objetivo. Si el usuario no da id, buscar "el ultimo lead sin cualificar" o preguntar. | `odoo-pilot: search_records.sh` sobre `crm.lead` |
| 2 | Leer ficha completa del lead: `name, partner_name, contact_name, email_from, phone, website, function, street, city, state_id, country_id, vat, description, user_id, team_id, stage_id, expected_revenue, create_date`. Presentar al usuario lo que ya sabemos. | `odoo-pilot: search_records.sh` con lista de fields |
| 3 | Leer el chatter: mensajes asociados al lead en `mail.message` con `model='crm.lead'` y `res_id=<lead_id>`. Resumir interacciones previas (quien escribio, de que, cuando). | `odoo-pilot: search_records.sh` sobre `mail.message` con campos `body, date, author_id, subject` |
| 4 | **Empresite** · buscar la empresa en `empresite.eleconomista.es`. Extraer: rango de facturacion, numero de empleados, CIF, domicilio social, sector CNAE, fecha de constitucion, situacion. | `WebFetch` a `https://empresite.eleconomista.es/{nombre-empresa}.html` o buscador |
| 5 | **Busqueda web libre** · investigar: presencia internacional, oficinas fisicas (sede + delegaciones), tamaño real (LinkedIn, web corporativa), tech stack, competidores, noticias recientes relevantes (fusiones, financiacion, nuevos proyectos). | `WebSearch` + `WebFetch` a la web corporativa |
| 6 | **Clay MCP** (opcional, si disponible) · enriquecer contactos y datos de la empresa. | Clay MCP tools |
| 7 | Scoring ICP (0-100) con datos enriquecidos. **NUNCA cualificar manualmente.** Calcular siempre con `scripts/qualify_leads.py`, pasando un JSON con los datos recolectados. | `scripts/qualify_leads.py` |
| 8 | Presentar resumen al usuario: score, veredicto (qualified/review/discard), comercial sugerido (por provincia), siguiente paso propuesto. **DETENTE y espera aprobacion.** | -- |
| 9 | Aplicar en Odoo (tras aprobacion): <br>• Escribir `description` consolidada (ver plantilla abajo) <br>• Asignar `user_id` segun provincia (`config/province_assignment.json`) <br>• Mover `stage_id` si score >= 60 <br>• Crear `mail.activity` con el siguiente paso (llamada, email, reunion) | `odoo-pilot: update_record.sh` + `create_record.sh` |

**Scoring**: empleados 30% + facturacion 30% + sector 20% + revenue 20%. Umbrales: qualify >=60, review 35-59, discard <35. Ver `references/icp_criteria.md`.

## Workflow batch / cron (N leads)

> Detalle completo, plantillas y configuracion de cron: `references/workflow_detail.md`

| Paso | Accion | Script |
|------|--------|--------|
| 0 | Recopilar info: credenciales, API key eInforma/AXESOR, provincias. **DETENTE y espera respuesta.** | -- |
| 1 | Extraer leads sin cualificar de Odoo (ultimos N dias) | `scripts/fetch_new_leads.sh` |
| 2 | Enriquecer con registro mercantil espanol (eInforma/AXESOR) | `scripts/enrich_leads.py` |
| 3 | Scoring ICP automatico (0-100) | `scripts/qualify_leads.py` |
| 3.5 | Presentar resumen al usuario. **DETENTE y espera aprobacion** (excepto modo cron). | -- |
| 4 | Aplicar en Odoo: asignar, mover etapa, crear actividades | `scripts/apply_triage.sh` |

## Plantilla de descripcion consolidada (HTML)

`crm.lead.description` es HTML. Escribir con esta estructura (mantener lo que ya habia al inicio o al final, nunca destruir contenido previo sin confirmacion):

```html
<h3>Ficha enriquecida · {fecha_iso}</h3>
<p>
  <b>Razon social:</b> {razon_social}<br/>
  <b>CIF/NIF:</b> {vat}<br/>
  <b>Sector (CNAE):</b> {sector_cnae}<br/>
  <b>Facturacion (rango):</b> {rango_facturacion}<br/>
  <b>Empleados:</b> {empleados}<br/>
  <b>Sede social:</b> {sede}<br/>
  <b>Otras oficinas / delegaciones:</b> {oficinas}<br/>
  <b>Presencia internacional:</b> {internacional}<br/>
  <b>Fecha de constitucion:</b> {constitucion}<br/>
  <b>Web:</b> <a href="{web}">{web}</a>
</p>

<h3>Score ICP: {score}/100 · {qualification}</h3>
<p>{justificacion_breve}</p>

<h3>Contexto previo (chatter)</h3>
<p>{resumen_chatter_o_NA}</p>

<h3>Notas adicionales</h3>
<p>{noticias_relevantes_o_NA}</p>

<h3>Fuentes consultadas</h3>
<ul>
  <li><a href="{empresite_url}">Empresite — El Economista</a></li>
  <li><a href="{web_corporativa}">Web corporativa</a></li>
  <li>{otras_fuentes}</li>
</ul>
```

Si la descripcion previa tiene informacion util (ej. notas del comercial, contexto del form de web), **preservarla** anteponiendola o separandola con un `<hr/>`.

---

## RULES

1. **SIEMPRE** leer `description` y chatter del lead ANTES de enriquecer (evita duplicar info y capta contexto del comercial)
2. **NUNCA** cualifiques leads manualmente -- siempre con `qualify_leads.py`
3. **NUNCA** apliques cambios en Odoo sin aprobacion del usuario (excepto modo cron)
4. **NUNCA** inventes datos -- si empresite u otra fuente no encuentra la empresa, dejarlo como "no localizado" en la ficha
5. **NUNCA** sobrescribas `description` destruyendo contenido previo util -- preservalo con separador `<hr/>`
6. Lead sin CIF/NIF valido va a revision manual, no a descartado
7. `crm.lead.description` es HTML -- escribir con tags, no texto plano con `\n`
8. Modo `--dry-run` disponible en scripts batch para pruebas
9. Delay entre llamadas API / WebFetch: 0.5s (respetar rate limits)
10. Si empresite pide captcha o devuelve pagina vacia, probar buscador `https://empresite.eleconomista.es/Actividad/EMPRESAS-{NOMBRE}/` o buscar en web con `site:empresite.eleconomista.es "{nombre}"`

---

## GOTCHAS

1. Leads sin partner (`res.partner`) se enriquecen por nombre de empresa (campo `partner_name`); no ir a `res.partner.read`
2. Empresite filtra por nombre exacto; si la empresa tiene nombre generico (ej. "Soluciones SA"), usar CIF si esta disponible o buscar por web en lugar de scraping directo
3. Clay MCP y eInforma/AXESOR son opcionales -- el modo interactivo ya funciona solo con WebFetch/WebSearch
4. `config/province_assignment.json` debe estar actualizado antes de ejecutar
5. eInforma/AXESOR no encuentran autonomos, empresas nuevas ni extranjeras
6. CIF con prefijo "ES" se normaliza automaticamente (ES + B12345678 -> B12345678)
7. En modo cron, ni Clay ni WebFetch estan disponibles (requieren Claude interactivo) -- usar solo eInforma/AXESOR
8. Si 0 leads extraidos (modo batch), no hay nada que triagear -- informar y parar
9. El campo `description` de `crm.lead` **es HTML**, no texto plano. Escribir tags `<p>`, `<br/>`, `<h3>`, `<a href>`
10. El chatter son registros de `mail.message` con `model='crm.lead'` y `res_id={lead_id}` -- ordenar por `date desc`

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
