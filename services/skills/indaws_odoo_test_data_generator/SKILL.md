---
name: indaws-odoo-test-data-generator
description: >-
  Genera datos de prueba realistas para Odoo en formato CSV. Soporta múltiples modos: demo para
  clientes, testing de módulos custom y escenarios personalizados. Usar cuando se necesiten datos de
  prueba realistas para Odoo.
version: 1.1.2
effort: medium
model: sonnet
---

# Test Data Generator for Odoo

Especialista en Preparacion de Datos de Odoo. Genera datos de prueba realistas, coherentes y contextualizados para instancias de Odoo. Los CSV producidos son compatibles con el metodo `load()` de Odoo para importacion via `odoo-importation`.

Principio fundamental: Los datos deben ser REALISTAS y COHERENTES con el sector. Nunca "Cliente 1", "Producto A".

## Workflow

Three modes of operation. See `references/modes_detail.md` for full input requirements per mode.

### Paso 0: Seleccion de Modo y Contexto

| Mode | Objective | Key input |
|------|-----------|-----------|
| 1 - Demo | Datos realistas del sector del cliente | Sector, version Odoo, apps, alcance (basico/completo/exhaustivo) |
| 2 - Testing | Datos para probar modulo custom | Ruta doc del modulo, version, alcance (funcional/completo/regresion) |
| 3 - Personalizado | Datos para POC o caso especifico | Descripcion escenario, version, modelos, requisitos |

DETENTE y espera respuesta del usuario.

### Paso 1: Analisis y Planificacion

- Mode 1: Consult `references/sector_knowledge.md`, identify models per app, design realistic nomenclature
- Mode 2: Read module documentation, extract custom models/fields, plan test scenarios
- Mode 3: Analyze scenario, identify dependencies and creation order

### Paso 2: Presentacion del Plan (VALIDACION OBLIGATORIA)

Present plan with: models to generate, quantities, nomenclature style, relationships, external ID prefix. DETENTE y espera aprobacion.

### Paso 3: Generacion de CSV

Generate CSV files following rules in `references/csv_format_rules.md`. Output to `data/` folder with README, import script, and cleanup script.

Key rules:
- Separator: comma, Encoding: UTF-8
- First row: API column names
- External IDs: `__demo__` / `__test__` / `__dev__` prefix
- Relational fields use `/id` notation
- Masters before transactions (import order matters)

### Paso 4-5: Scripts and Documentation

Generate `import_data.sh` (import guide), `cleanup_data.sh` (delete by external ID prefix), and `README_data.md`.

### Paso 6: Presentacion Final

Summary: files generated, total records, next steps (review CSVs, import via odoo-importation, validate in Odoo).

## GOTCHAS

1. **Datos genericos** -- NUNCA generar "Cliente 1", "Producto A". Usar nomenclatura realista del sector (ej: "Sofa Modelo Valencia 3 plazas", "Hotel Plaza Mayor")
2. **Relaciones rotas** -- Los productos en pedidos DEBEN existir en el maestro de productos. Los external IDs deben ser coherentes entre archivos
3. **Orden de importacion** -- Maestros SIEMPRE antes que transacciones. Si se importa sale.order.line antes que sale.order, falla
4. **Precios irrealistas** -- Usar rangos de precios reales del sector. Consultar `references/sector_knowledge.md`
5. **Sector no documentado** -- Si el sector no esta en sector_knowledge.md, usar conocimiento general pero AVISAR al usuario
6. **Constraints de Odoo** -- Respetar campos requeridos, formatos validos, rangos permitidos. Un CSV mal formado rompe `load()`
7. **Olvidar cleanup** -- SIEMPRE generar cleanup_data.sh con el prefijo correcto para poder borrar los datos de prueba

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-importation | common | Importacion real de los CSV generados | Recommended to user (not automated) |

## Consumed by

| Skill | Suite | Purpose |
|-------|-------|---------|
| -- | -- | Standalone utility skill |

## References

| File | Content |
|------|---------|
| `references/modes_detail.md` | Full input requirements and planning steps per mode |
| `references/csv_format_rules.md` | CSV format, external IDs, column specs, folder structure |
| `references/sector_knowledge.md` | Sector-specific data (nomenclature, prices, categories) |
| `references/shared/odoo-connection.md` | Connection pattern for importing via odoo-importation |
