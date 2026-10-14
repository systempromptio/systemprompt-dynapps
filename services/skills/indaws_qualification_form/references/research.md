# Investigación pre-form (Paso 0.5) — detalle

Se ejecuta **después** de validar la oportunidad (Paso 0) y **antes** de generar el HTML (Paso 1).

**Política**: CONSERVADOR. Solo pre-rellenar campos con fuente verificable y citable. **NUNCA inventar**. Relevancia > cantidad.

**Filosofía**: parar lo antes posible. 1 sola call dirigida a la fuente canónica adecuada vale más que 4 calls genéricas. Jerárquico y parable.

---

## Tier 0 — Datos del lead (gratis, instantáneo)

`orm_search_read` sobre `res.partner` con `partner_id`: `name, website, street, city, country_id, vat, description, comment`. También del `crm.lead`: `description`, `email_from`.

Extraer:
- `domain` = `partner.website` (si existe) o dominio del `email_from` **ya reparado** (descartar genéricos: gmail, hotmail, outlook, yahoo, icloud, protonmail; **y el relay de leads web `dynapps.be`/`indaws.es`** — si el `email_from` sigue siendo del relay, la reparación previa (qualification-form Paso 0.4 / qualification-preflight §2.0) no se ejecutó: NO investigues ese dominio). Si no hay nada → `domain = null`.
- `cif` = `partner.vat` (si existe). Limpiar prefijo "ES" si lo tiene.
- `descripcion_cliente` = `lead.description` (HTML stripped) + `partner.comment` si existe.

**Si la descripción del cliente ya cubre actividad + (facturación O empleados)** → construir `suggestions` solo desde Tier 0 + lead, **STOP** y saltar a "Construir suggestions".

---

## Tier 1 — una sola call dirigida (1-5s, ~1k tokens)

Elegir la call **más eficiente** según lo que tenemos:

**Caso A — Tenemos CIF**:
```
WebFetch(
  url=f"https://www.einforma.com/buscar-empresa-por-cif/{cif}",
  prompt="Extract as compact JSON: {razon_social, facturacion_ultimo_anio_en_M_euros, num_empleados, cnae_sector, administradores_lista}. If a field is not available, omit it. NO commentary, only JSON."
)
```

⚠️ **eInforma URL directa por CIF da 404 con frecuencia** (usa slugs, no el patrón `/buscar-empresa-por-cif/{CIF}` siempre). Si Caso A devuelve 404 o falla → ejecutar Caso B + Caso C **en paralelo** como fallback automático.

**Caso B — Tenemos dominio pero no CIF (o fallback del A)**:
```
WebFetch(
  url=f"https://{domain}",
  prompt="Extract as compact JSON: {sector_breve, actividades_de_estas_5_categorias_solo_las_que_aplican: ['Distribución','Fabricación','Proyectos','Construcción','Instalación'], tamano_aproximado_si_se_menciona, tecnologias_o_ERP_mencionados}. NO commentary, only JSON."
)
```

**Caso C — Solo tenemos nombre (o fallback del A)**:
```
WebSearch(
  query=f'"{nombre_empresa}" facturacion empleados site:einforma.com OR site:axesor.es OR site:iberinform.es OR site:navarracapital.es'
)
```
Especialmente potente: el snippet del SERP de eInforma/Iberinform/Axesor suele incluir directamente facturación, empleados, CIF y sector — sin un segundo WebFetch.

| Tienes | 1ª call | Si falla |
|---|---|---|
| CIF + dominio | Caso A (eInforma por CIF) | B + C en paralelo |
| Solo dominio | Caso B (homepage) | C |
| Solo nombre | Caso C (search) | — |

---

## Stop check (tras Tier 1)

Evaluar si tenemos: ✅ Actividad (≥1 categoría) · ✅ Facturación · ✅ Empleados.

- Los 3 → **SKIP Tier 2**, pasar a Tier 3.
- Falta alguno → Tier 2 (máx 1 call).

`sistema_actual` queda **fuera** del stop condition. Se incluye SI sale en passing en Tier 1/2, pero NO se busca activamente (difícil de encontrar; el SDR lo rellena durante la llamada).

---

## Tier 2 — complemento targeted (solo si gap, máx 1 call)

Elegir SOLO 1 de:
- **Falta sector** → WebFetch homepage del cliente si no se hizo en Tier 1 (prompt del Caso B).
- **Falta facturación** → `WebSearch f'"{empresa}" facturacion millones site:expansion.com OR site:eleconomista.es OR site:cincodias.elpais.com OR site:navarracapital.es OR site:alimarket.es'`
- **Falta empleados** → `WebFetch f'https://www.linkedin.com/company/{slug_empresa}'` con prompt "Extract employee range and headquarters".

NO más de 1 call en Tier 2. Si quedan 2 gaps, priorizar el más crítico (facturación > empleados > sector).

---

## Tier 3 — scout-odoo (siempre, info propietaria)

```
Skill(skill="scout-odoo", args=f"Busca '{nombre_empresa}' en la base de datos publica de clientes Odoo. Si la encuentras, devuelve: partner actual, fecha de aparicion en directorio, y si es 'switcher' (cambio de partner). Si no esta, responde solo 'no encontrada'.")
```

⚠️ **scout-odoo v2.1.0 accede EXCLUSIVAMENTE vía el connector MCP del scraper** (`odoo-scraper-mcp` / connector "Odoo References Scraper"), tool `scraper_run_query`. Ya NO existe CLI (`query.py`, `--reference`, `sql --inline`, `sql --file`): solo invoca el skill en lenguaje natural y él elige la receta `customer-detail` y lanza el SELECT por el MCP. Si las tools `scraper_*` no están conectadas, scout-odoo lo avisa — no bloquea esta cualificación.

Procesar la respuesta:
- **"no encontrada"** → resultado VÁLIDO, no error. El cliente no está en el scraper público → prospecto fresco o el partner actual no lo declaró. No añadir nada a notas.
- **Match encontrado** → añadir 1-2 líneas a `notas_sistema`:
  - `"⚠️ Cliente actual de [Partner X] segun directorio publico Odoo (fecha aparicion: YYYY-MM-DD)."`
  - `"📊 Switcher detectado: migro de [Y] a [X] el [fecha]."` (si aplica)
- Info propietaria muy valiosa — diferenciador competitivo de Indaws.

Si scout-odoo falla por error técnico (timeout, traceback), continuar sin bloquear — no añadir nada.

---

## Construir `suggestions` final

Consolidar Tier 0 + 1 + 2 + 3 en el objeto `suggestions` y avanzar al Paso 1. Si no se encontró dominio o ninguna fuente devolvió datos útiles → `suggestions = {}` y el form mostrará el aviso amarillo "Sin investigación pública disponible".

### Mapeo a campos del form (CONSERVADOR)

| Campo | Pre-rellenar si | Skip si |
|---|---|---|
| `actividades` (multi) | Homepage o about mencionan explícitamente una de las 5 categorías (Distribución/Fabricación/Proyectos/Construcción/Instalación) | Sector ambiguo o no encaja |
| `notas_actividad` | Siempre rellenar con 1-2 frases describiendo sector + fuente | — |
| `facturacion` | Encuentras cifra en SERP/eInforma/news con año | No hay cifra clara |
| `empleados` | LinkedIn da rango ("51-200") o fuente da número | No hay dato |
| `usuarios` | **NUNCA** pre-rellenar (judgment del SDR) | Siempre skip |
| `notas_dim` | Resumen breve de fuentes encontradas | — |
| `sistema` | El sitio o un job posting mencionan explícitamente un ERP ("trabajamos con SAP") | Inferencia o suposición |
| `version`, `edicion`, `partner_actual`, `erp_nombre` | Si `sistema` está pre-rellenado **Y** la fuente da el detalle | Skip si dudas |
| `notas_sistema` | Si sistema rellenado, breve nota con fuente | — |
| `pain_points` | **NUNCA** (especulativo) | Siempre skip |
| `modulos` | **NUNCA** (derivado, no directo) | Siempre skip |
| `presupuesto` | **NUNCA** (no hay fuente pública) | Siempre skip |
| `timeline` | Solo si el lead lo dice explícitamente en `description` (ej. "antes de Q3") | Skip si web/SERP |
| `decisores` | **SÍ** desde datos del propio lead (`contact_name`, `email_from`, `phone`) — el cliente nos los dio al rellenar el formulario web. NUNCA desde búsqueda web especulativa | Skip si no hay `contact_name` |
| `notas_pain`, `notas_modulos`, `notas_presupuesto` | Si el lead `description` o el chatter contienen contexto literal relevante (ej. cliente pide "modulo PRESTO") | Skip si solo es especulación |

### Forma del objeto

```json
{
  "actividades": ["Distribución", "Fabricación"],
  "notas_actividad": "Fabricante de componentes industriales con presencia en 5 países (fuente: homepage).",
  "facturacion": { "value": "12M€", "source": "axesor.es 2023" },
  "empleados": { "value": "85", "source": "LinkedIn" },
  "notas_dim": "Datos de eInforma 2023 y LinkedIn público.",
  "sistema": { "value": "ERP tercero", "source": "Oferta de empleo en InfoJobs" },
  "erp_nombre": { "value": "SAP", "source": "Oferta de empleo en InfoJobs" },
  "notas_sistema": "Mencionan experiencia con SAP en sus ofertas de empleo (verificar).",
  "decisores": { "value": "Juan García (CEO) — juan@empresa.com · 600123456", "source": "Contacto principal del lead" },
  "notas_modulos": "Cliente pide explicitamente modulo de gestion de proyectos tipo PRESTO (fuente: lead description).",
  "notas_presupuesto": "Contacto principal: Juan Garcia (CEO) — juan@empresa.com. Verificar si involucra CTO/CFO."
}
```

Los campos NO encontrados simplemente NO aparecen en el objeto (el form los deja vacíos sin badge). Pasar al Paso 1 como `{{suggestions_json}}` vía `JSON.stringify(suggestions)`.

### Campos que el template sabe aplicar (`applySuggestions()` en `qualification-form.html`)

- Multi-select: `actividades` (lista de strings).
- Radios: `sistema`, `edicion` (objetos `{value, source}`).
- Text inputs: `facturacion`, `empleados`, `decisores`, `timeline`, `version`, `partner_actual`, `erp_nombre` (todos `{value, source}`).
- Textareas: `notas_actividad`, `notas_dim`, `notas_sistema`, `notas_pain`, `notas_modulos`, `notas_presupuesto` (strings planos).
- Si añades un campo nuevo, asegúrate de que aparece en `valueFields`, `noteFields` o un handler específico del template, o no se aplicará.

---

## FUENTES CANÓNICAS

Orden de prioridad. **Usar primero las de arriba** (más estructuradas, mayor probabilidad de éxito):

| # | Fuente | Da | Cómo acceder | Tier |
|---|---|---|---|---|
| 1 | **eInforma** | CIF, facturación oficial, empleados, CNAE, administradores | `WebFetch https://www.einforma.com/buscar-empresa-por-cif/{CIF}` | 1 (si hay CIF) |
| 2 | **Axesor** | Datos corporativos similares a eInforma | `WebFetch https://www.axesor.es/Informes-Empresas/{slug}` | 1 alt |
| 3 | **Iberinform** | Datos corporativos + financieros | `WebFetch https://www.iberinform.es/empresa/{id}` | 1 alt |
| 4 | **Homepage del cliente** | Sector, propuesta valor, logos-clientes, tecnologías | `WebFetch https://{domain}` con prompt JSON-corto | 1 (si no hay CIF) |
| 5 | **Navarra Capital, Expansión, Cinco Días, El Economista** | Facturación reciente, crecimiento, noticias | `WebSearch [empresa] facturacion millones site:navarracapital.es OR site:expansion.com OR site:cincodias.elpais.com OR site:eleconomista.es` | 2 |
| 6 | **Alimarket** | Datos sectoriales (alimentación, distribución…) | `WebSearch [empresa] site:alimarket.es` | 2 |
| 7 | **LinkedIn company** | Empleados (rango), sede, sector | `WebFetch https://www.linkedin.com/company/{slug}` | 2 |
| 8 | **InfoJobs / LinkedIn jobs** | ERP en uso (anuncios mencionan SAP/Navision…) | `WebSearch [empresa] site:infojobs.net OR site:linkedin.com/jobs` | 2 (solo para sistema) |
| 9 | **scout-odoo** (skill propietario) | Cliente Odoo actual, partner actual, switcher history | `Skill('scout-odoo', query)` | 3 (siempre) |

**NO usar**:
- `WebSearch` genérico sin `site:` filter — mucho ruido, alto coste en tokens.
- Prensa generalista (Marca, El País sección general) — no son fuentes empresariales.

**Prompts cortos siempre**: pedir JSON compacto, sin commentary. Ej: `"Extract as compact JSON: {revenue_M_eur, employees, sector}. NO commentary, only JSON."`
