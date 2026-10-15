# Taxonomia de cobertura: modulo -> funcionalidad -> flujo

Objetivo: que **cada funcionalidad** de **cada modulo instalado testeable** tenga al menos un
escenario. Dos fuentes de flujos:

1. **Catalogo del flow-tester** (reutilizar): 190+ flujos atomicos (48 secciones), 39 cadenas
   E2E, 28+ apps. Definidos en:
   - el catalogo de `odoo-flow-tester`, via su `flows.py`: `--list` da los flujos por
     modelo/operacion y `--chains` las cadenas E2E con revert (no se abre el catalogo entero)
   - `odoo-flow-tester/references/app-coverage.md` (tabla app -> flujos/cadenas)
2. **Flujos generados** (gaps): para custom/Studio/OCA no catalogado. Ver `flow-generation.md`.

## Niveles de cobertura por modulo

Para un modulo testeable, cubrir al menos:

| Nivel | Que cubre | Fuente |
|-------|-----------|--------|
| **L1 Atomico** | crear/confirmar/cancelar/borrar del modelo principal | `flows.py --list` |
| **L2 Cadena** | flujo E2E que cruza apps (ej. venta->entrega->factura->cobro) | `flows.py --chains` |
| **L3 Custom** | metodos/automatizaciones/campos custom y Studio del modulo | generado |
| **L4 Edge** | parciales, devoluciones, NC, reintentos, casos limite | `flows.py --chains` + generado |

Regla practica: si el modulo esta en `catalog_map.json` con `chains`, usar esas cadenas para
L2 y los `atomic` para L1. Para `custom_gaps`/`unknown`, generar L1/L3 como minimo.

## Cruce con personalizaciones (overlay)

El flow-tester tiene un Analysis Engine que cruza 3 fuentes (Studio inventory + doc + codigo).
Esta skill aporta la **1a fuente enriquecida**: el inventario de modulos + el descubrimiento
custom (odoo-module-scanner). Por cada personalizacion detectada decidir:

- **Campo custom `x_studio_*` computed/required** -> assertion en un flujo existente del modelo
  afectado (no flujo nuevo, se anade `assertions` al flujo del modelo).
- **Metodo de accion custom** (`action_*`, `button_*`) -> step `custom_method` en flujo nuevo.
- **Automatizacion / server action** -> assertion que verifica su efecto tras el step que la
  dispara (`detect_cascade` si crea registros).
- **Modelo custom nuevo** -> flujo L1 completo (create/confirm/.../revert) propio.

## Clasificacion por canal (token-eficiencia) — OBLIGATORIO

Tras asignar funcionalidad+nivel, etiquetar el **canal** de cada validacion segun
`validation-channels.md`:
- `ai` — API-aserable (estado, valores, cascadas). Genera flujo. Token-barato.
- `human` — visual/maquetacion. NO genera flujo; va a `human_visual_checklist.md`. Token-caro IA.
- `split` — informe/vista con datos + aspecto: la parte de datos -> `ai` (flujo), el aspecto ->
  `human` (checklist).

Regla: si validar requiere MIRAR (PDF, captura, navegador) -> `human`. Si es un assert API -> `ai`.

## Salida del Paso 2

Una lista de funcionalidades, cada una con:
```
{ module, app, functionality, level (L1-L4), channel ("ai" | "human" | "split"),
  flow_source ("catalog:<flow_id>" | "generate" | "none"),
  flow_id, status ("reuse" | "to_generate" | "human_only" | "no_coverage"), reason_if_no_coverage }
```
Las de `channel=ai` (y la parte de datos de `split`) alimentan `flows/` + `coverage_matrix.md`.
Las de `channel=human` (y la parte de aspecto de `split`) alimentan `human_visual_checklist.md`.

## Modulos sin cobertura posible (declarar como gap honesto)

- Modulos puramente de UI/web cuyo flujo depende del navegador (parte no-API). Anotar que se
  cubre solo la parte de datos resultante (ej. `website_sale` -> el `sale.order` generado).
- Modulos tecnicos (`testable=false`).
- Modulos sin transacciones (config-only). Anotar "config; sin flujo transaccional".
