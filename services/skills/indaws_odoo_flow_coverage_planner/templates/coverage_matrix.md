# Matriz de cobertura funcional — {{instance_name}}

- Fecha: {{date}}
- Version Odoo: {{target_version}}
- Alcance: {{scope}}  ·  Modo de entrega: {{delivery_mode}}
- Modulos instalados: {{installed}}  ·  Testeables: {{testable_total}}

## Resumen

| Metrica | Valor |
|---------|-------|
| Cobertura AUTOMATIZABLE IA (validaciones `ai` con flujo) | {{coverage_pct_ai}} % |
| Validaciones canal IA (flujo) | {{n_ai}} |
| Validaciones canal HUMANO (checklist visual) | {{n_human}} |
| Validaciones `split` (datos IA + aspecto humano) | {{n_split}} |
| Modulos con flujo de catalogo | {{covered}} |
| Flujos custom generados (gaps) | {{custom_gaps}} |
| Modulos sin clasificar (OCA/terceros) | {{unknown}} |
| Modulos no testeables (tecnicos/maestros) | {{not_testable}} |

## Cobertura por modulo (canal IA — automatizado)

| Modulo | App | Funcionalidad | Nivel | Canal | Flujo | Fuente | Estado | Resultado |
|--------|-----|---------------|-------|-------|-------|--------|--------|-----------|
| sale_management | Ventas | Venta E2E (cobro) | L2 | ai | CHAIN_SALE_TO_CASH | catalog | reuse | {{PASS/FAIL/—}} |
| sale_management | Ventas | Crear/confirmar pedido | L1 | ai | (atomic sale_order) | catalog | reuse | — |
| bdk_sales | Ventas | Informe pedido: DATOS (totales, secciones) | L3 | split | CHAIN_BDKSALES_PICKING_SECTIONS | generate | to_generate | {{result}} |
| {{module}} | {{app}} | {{functionality}} | {{level}} | {{ai/split}} | {{flow_id}} | {{catalog/generate}} | {{reuse/to_generate}} | {{result}} |

> Canal: `ai` = validacion API (flujo) · `split` = datos por flujo + aspecto en checklist humano.
> Estado: `reuse` = flujo de catalogo · `to_generate`/`generated` = flujo custom en `flows/`.
> Resultado: se rellena en el Paso 5 con la salida del flow-tester (PASS/FAIL/SKIP + evidencia).

## Derivado a HUMANO (canal visual — ver `human_visual_checklist.md`)

| Modulo | Funcionalidad (aspecto) | Por que humano |
|--------|-------------------------|----------------|
| bdk_sales | Informe pedido: ALINEACION columna "Pos." | maquetacion PDF (visual) |
| bdk_website_sale | Efecto visual cambio de variante | render navegador |
| {{module}} | {{aspecto}} | {{visual/maquetacion/UX}} |

## Gaps custom (flujos generados)

| Modulo | flow_id generado | Funcionalidad custom | Archivo |
|--------|------------------|----------------------|---------|
| {{module}} | {{flow_id}} | {{desc}} | flows/{{flow_id}}.yaml |

## Modulos sin cobertura (declaracion honesta)

| Modulo | Motivo |
|--------|--------|
| {{module}} | {{tecnico / solo-UI / config-only / OCA pendiente de revisar}} |
