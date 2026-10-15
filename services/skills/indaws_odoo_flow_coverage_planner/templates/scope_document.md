# Documento de alcance de pruebas funcionales — {{instance_name}}

> Generado por `odoo-flow-coverage-planner` para consumo del `odoo-flow-tester` (Paso 0/1).

## 1. Contexto

- Instancia / DB: {{db}}
- Version Odoo: {{target_version}}
- Entorno: {{staging|production}}
- Motivo: {{validacion post-migracion / QA release / auditoria funcional}}
- Modo de ejecucion solicitado: {{A_document | B_verify | C_execute}}

## 2. Alcance

Probar las funcionalidades de los modulos instalados clasificados como testeables
(ver `coverage_matrix.md`). Total testeable: {{testable_total}} modulos.

Incluye:
- Apps de negocio estandar -> cadenas y flujos atomicos del catalogo.
- Personalizaciones (custom/Studio) -> flujos a medida en `flows/`.

Excluye de la AUTOMATIZACION IA (con motivo): modulos tecnicos, maestros (se prueban indirecto),
y **validaciones visuales/maquetacion** (PDF, capturas, UI/UX). Estas ultimas NO se piden al
flow-tester ni las interpreta la IA: van al `human_visual_checklist.md` por eficiencia de tokens.
De los informes/web se automatiza la parte de DATOS; el ASPECTO lo valida un humano.

## 3. Flujos a ejecutar

### 3.1 De catalogo (reutilizados)
{{lista de flow_id de catalog_refs.txt, ej. CHAIN_SALE_TO_CASH, CHAIN_PURCHASE_TO_PAY, ...}}

### 3.2 Custom (en flows/, ya validados con validate_flow.py)
{{lista de flow_id generados con su modulo y funcionalidad}}

## 4. Precondiciones de datos

Los flujos localizan maestros existentes via `preconditions.required_data`. Si la BD no tiene
datos suficientes (clientes, productos, diarios), generar antes con `odoo-test-data-generator`.

Maestros minimos esperados: cliente con `customer_rank>0`, proveedor con `supplier_rank>0`,
producto `sale_ok`/`purchase_ok`, diarios `sale`/`purchase`/`bank`.

## 5. Seguridad y reversion

- Todos los registros llevan prefijo `[TEST-...]`.
- Modo C: Safe Mode (neutraliza integraciones) + Snapshot antes; revert + restauracion despues.
- Modulos fiscales EDI a verificar antes de flujos contables: {{SII/TicketBAI/Verifactu si aplican}}.

## 6. Criterio de exito

Cada funcionalidad de la matriz con su flujo en estado PASS y revert correcto (snapshot
post-revert == pre-ejecucion). Los FAIL se documentan con evidencia.
