---
name: indaws-demo-ventas-presupuesto
description: >
  A partir de una oportunidad del CRM (crm.lead), genera dos artefactos:
  (1) un sale.order relleno en Odoo con lineas de fases del proyecto
  (auditoria, diseño, instalacion, puesta en marcha) calculadas sobre el
  expected_revenue, y (2) un PDF de propuesta comercial profesional con
  branding Indaws, portada, resumen ejecutivo personalizado por el cliente,
  descripcion tecnica por fase, planificacion, inversion y condiciones.
  Usar cuando el usuario pida: generar presupuesto, preparar propuesta
  comercial, rellenar sale.order desde una oportunidad, PDF propuesta cliente.
version: 1.0.2
model: sonnet
effort: medium
---

# Demo · Ventas presupuesto

Generacion de presupuesto de proyecto de ingenieria a partir de una oportunidad CRM: sale.order en Odoo + PDF comercial con marca Indaws.

## Prerequisitos

- **odoo-pilot** (common): lectura del crm.lead + creacion del sale.order + subida de adjunto
- **pdf-renderer** (common): render HTML → PDF via Playwright
- **dynapps-brand** (common): tokens de marca para el PDF (colores, tipografia, logos)
- **Credenciales Odoo** en env: `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY`
- **Python 3.8+** + **Node.js** (para Playwright)
- `config/config.json` con fases default, IVA y datos de la empresa

## Workflow

| Paso | Accion | Output |
|------|--------|--------|
| 1 | Autenticar en Odoo (`auth.sh`). Leer `config/config.json`. | -- |
| 2 | Ejecutar `python scripts/main.py --lead-id <ID>` (fase **prepare**). El script lee el `crm.lead` (partner, contacto, descripcion enriquecida, chatter), calcula el desglose por fases sobre el `expected_revenue`, genera el HTML con placeholders `{{LLM_EXECUTIVE_SUMMARY}}`, `{{LLM_CONTEXT}}`, `{{LLM_PHASE_N_DESCRIPTION}}`, `{{LLM_TIMELINE}}` y guarda el JSON de contexto en `output/<lead>_context.json`. | HTML + context.json |
| 3 | Leer `output/<lead>_context.json`. Redactar contenido personalizado: resumen ejecutivo (3-4 parrafos) + contexto del cliente (2-3 parrafos) + descripcion por fase (2-3 frases por fase, cruzando pain points detectados con los entregables de la fase) + planificacion (narrativa de los hitos). Escribir el contenido en `output/<lead>_assessments.json` con el formato `{"executive_summary": "...", "context": "...", "phases": [{"code":"F1","description":"..."},...], "timeline": "..."}`. | assessments.json |
| 4 | Ejecutar `python scripts/main.py --lead-id <ID> --finalize`. El script aplica los assessments al HTML, renderiza a PDF via `pdf-renderer`, crea el `sale.order` en Odoo con las 4 lineas de fase (nota interna y descripcion libre, sin product_id), sube el PDF como adjunto (`ir.attachment`) vinculado al sale.order, y devuelve los IDs al usuario. | sale.order + PDF |
| 5 | Presentar al usuario: numero del sale.order, ruta del PDF local, link a Odoo para revisar. | -- |

## Como debe ser el contenido generado (paso 3)

### Resumen ejecutivo (`executive_summary`)
3-4 parrafos que respondan:
1. **Quien es el cliente y que situacion tiene** (basado en description del lead)
2. **Que propone Lumina y por que encaja** (oportunidad + linea de servicio)
3. **Que resultado espera obtener el cliente** (retorno, mejora operativa, KPIs)

### Contexto (`context`)
2-3 parrafos que expliquen el entorno del cliente (sector, tamaño, presencia) y los desafios concretos detectados (pain points). Usar datos concretos (facturacion, empleados, internacional) del lead enriquecido. No repetir el resumen ejecutivo.

### Descripcion por fase (`phases`)
Para cada fase (F1-F4), 2-3 frases que conecten los entregables de la fase con la realidad del cliente. Ej: en F3 (Instalacion) mencionar el tipo concreto de equipo que se instala en este proyecto, no descripcion generica.

### Planificacion (`timeline`)
Un parrafo narrativo con los hitos principales, fechas relativas ("semana 1-2", "mes 2", etc.) y puntos de decision del cliente.

## RULES

1. **NUNCA** uses product_id en las lineas del sale.order — son lineas libres con descripcion HTML
2. **NUNCA** inventes datos del cliente — si falta info, el contexto se escribe mas generico pero nunca se fabrica
3. **NUNCA** modifiques el `crm.lead` (la skill 03 no enriquece, usa lo que esta)
4. **SIEMPRE** el total del sale.order = `expected_revenue` del lead (sin margen adicional)
5. **SIEMPRE** presenta el sale.order al usuario ANTES de subir el PDF definitivo
6. **SIEMPRE** branding Indaws en el PDF: Montserrat + colores del `brand-tokens.json` + zero emojis
7. El HTML template vive en `templates/proposal.html` — modificarlo solo si la estructura cambia, no por contenido

## GOTCHAS

1. Si el lead no tiene `expected_revenue`, el script falla con error claro — debe corregirse el lead antes
2. Si el lead no tiene `partner_id`, el sale.order se crea con `partner_name` pero sin partner real — warning visible
3. El render PDF necesita Playwright. Si falla, verificar `pdf-renderer/scripts/node_modules/` instalado
4. El PDF se guarda primero en `output/` local y despues se sube a Odoo como `ir.attachment` — si la subida falla, el PDF local queda disponible
5. Los porcentajes de fase suman 100; si se tocan en config.json, validar que sigan sumando 100
6. En la demo de Odoo Club, el cliente final es Lumina Ingenieria con su oportunidad real — el branding del PDF es Indaws (partner que presenta la demo)

## Delegates to

| Skill | When |
|-------|------|
| **common:odoo-pilot** | lectura de `crm.lead`/`res.partner`, creacion de `sale.order`/`sale.order.line`, subida de `ir.attachment` |
| **common:pdf-renderer** | render HTML → PDF (Playwright/Chromium) |
| **common:dynapps-brand** | tokens de marca (colores, logo, tipografia) — se leen de `assets/brand-tokens.json` |

## References

| File | Content |
|------|---------|
| `config/config.json` | Fases default (% reparto), IVA, empresa, condiciones de pago |
| `templates/proposal.html` | Template HTML del PDF con placeholders |
