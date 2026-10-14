# Mockear lo NO estándar adaptándolo al estándar de Odoo

El cliente pide una **solución que ya se imaginó** (su dibujo); tú resuelves el **problema de fondo**,
que casi siempre tiene un patrón nativo. Reutilizar el constructo nativo evita reinventar lo que Odoo ya hace (hay menos que mantener, y lo mantiene Odoo). Regla mental: *"¿qué problema resuelve esto que ha dibujado?"*
→ el constructo Odoo que lo cubre, aunque **no se parezca** al dibujo.

## La escalera de sustitución (elige el peldaño que cubra la necesidad sin reinventar)
1. **Config** — un campo/vista que ya lo hace.
2. **App estándar** — ya existe un módulo Odoo/OCA (CE o EE).
3. **Widget/vista alternativa** — otra forma nativa de enseñar lo mismo.
4. **Studio** — sólo para el retoque trivial (campo/vista/automatización de un paso).
5. **Desarrollo (módulo gobernado)** — para la lógica no trivial que no cubre lo anterior.

El mockup propone el peldaño que cumple sin reinventar lo estándar. **El criterio Studio-vs-desarrollo lo fija `odoo-classification-guide`** (Studio para lo trivial; lógica no trivial → módulo gobernado — no es "último recurso", es lo correcto cuando hay lógica).

## Catálogo de sustituciones (no-nativo → nativo) — verificado en v18 (CE/EE)
| Pide el cliente | Equivalente nativo | Peldaño |
|---|---|---|
| Dashboard combinado con KPIs | **Spreadsheet dashboard** (CE) · vistas `graph`/`pivot` | app/config |
| Barra de progreso de un flujo | `state`/`stage_id` + `widget="statusbar"` | config |
| Asistente multi-paso a medida | `TransientModel` wizard (notebook/secuencial) | desarrollo |
| Semáforo / indicador de color | `state_selection` · `badge` con decoraciones · color de kanban | config |
| Botón flotante / acción rápida | acción de servidor (menú engranaje) · botón de cabecera | Studio |
| Campo "en vivo" calculado en JS | `compute` en servidor + `onchange` (sensación de vivo) | desarrollo |
| Flujo de aprobación custom | app **Approvals** (EE) · o `mail.activity` + `state` + `base_automation` | app/Studio |
| Recordatorios / avisos custom | **actividades** (`mail.activity`) + **base_automation** (CE) | config |
| Firma en pantalla | `widget="signature"` · app **Sign** (EE) | config/app |
| Chat / notas / histórico | **chatter** (`mail.thread`) / Discuss | config |
| Planificador / calendario | vista `calendar` · `gantt` + app **Planning** (EE) | app (EE) |
| Mapa de registros | vista `map` (EE) | app (EE) |
| Escaneo de códigos | `widget="many2one_barcode"` · app Barcode | config |
| Importar Excel con UI propia | **base_import** nativo (CE) | config |
| Gestión documental / previsualizar | app **Documents** (EE) · adjuntos + `widget="pdf_viewer"` | app |
| Portal externo para cliente | website/**portal** (`portal.mixin`) | app |
| Permisos por rol a medida | **grupos + record rules** | config |
| Base de conocimiento / manual | app **Knowledge** (EE) | app |

## Cómo se mockea la sustitución (herramienta de negociación)
1. **Dibuja el equivalente nativo**, no el dibujo del cliente.
2. **Anótalo como sustitución** en la capa dev: *"Pediste un puesto táctil con cronómetro; lo cubrimos con
   un **kanban de tu cola** (nativo, 0 €, upgrades sin dolor). Pierdes el diseño a medida."*
3. Enseña el nativo y pregunta *"¿te vale?"*. Muchas veces el nativo convence; decídelo por el valor (sección "Cuándo NO forzar el estándar"), no por evitar código.
4. **Si de verdad no hay equivalente** → dibújalo como **bloque DEV/OWL** explícito (widget `owl_custom` en
   el studio) y cuantifícalo. No lo escondas ni lo pintes como nativo.

## Cuándo NO forzar el estándar
Si la UX a medida **es el core del negocio** (p.ej. el puesto del operario en un taller de alto volumen:
3 clics de más × 200 OR/día = horas perdidas), el custom se justifica. La regla no es "todo al estándar a
la fuerza", es **"al estándar salvo que el retorno del custom lo pague"** — y el mockup pone ambos lados
sobre la mesa (nativo barato vs custom óptimo) para decidir con datos, no por capricho.
