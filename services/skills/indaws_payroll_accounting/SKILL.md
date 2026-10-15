---
name: indaws-payroll-accounting
version: 1.0.4
effort: medium
model: opus
description: >-
  Genera el asiento contable de nóminas en Odoo a partir de un Excel de resumen. Clasifica empleados
  por departamento, calcula líneas account.move.line con cuentas 640xxx/642xxx, valida que el
  asiento cuadre, crea las líneas en Odoo y publica nota interna con el desglose. Requiere
  confirmación antes de escribir. Usar cuando haya que generar el asiento contable de nóminas desde
  el Excel.
---

# Asiento Contable de Nóminas

## Workflow

1. **Paso 0 — Recopilación**: Pedir ruta Excel + mes/año + ID de `account.move` en Odoo. DETENTE si falta algún dato.
2. **Paso 1 — Cálculo**: Ejecutar `python3 scripts/process_payroll.py --excel <ruta> --month <mes> --year <año> --move-id <id>`. Si hay errores (empleados sin departamento, departamentos sin mapeo) → mostrar y detener.
3. **Paso 2 — Validación**: Mostrar tabla resumen con todas las líneas + total debe = total haber. DETENTE y esperar confirmación explícita.
4. **Paso 3 — Crear líneas**: Para cada línea: buscar cuenta por código vía odoo-pilot, crear `account.move.line`. Verificar que el asiento cuadra. Ver `references/workflow-detail.md`.
5. **Paso 4 — Nota interna**: Crear `mail.message` directo en el asiento con HTML del desglose por departamento. **No usar `message_post`** — escapa el HTML.

Ver `references/workflow-detail.md` para los comandos exactos y `references/account_mapping.md` para el mapeo de cuentas.

> Note: `common:references/shared/` — auth via `auth.sh`, `search_records.sh`, `create_record.sh`.

## GOTCHAS

1. **`message_post` escapa el HTML** del body y se muestra como texto plano. Crear `mail.message` directamente con `odoo_create` para preservar el HTML.
2. **Empleados sin departamento en Odoo** → el script los lista como error; no continuar hasta resolverlos.
3. **Departamentos sin mapeo de cuentas** → el script los lista como error; añadir el mapeo en `references/account_mapping.md` y actualizar `process_payroll.py`.
4. **Si el asiento ya tiene líneas** → advertir al usuario antes de añadir nuevas. No sobrescribir.
5. **`ODOO_UID` se resuelve automáticamente** vía XML-RPC si no está definido como variable de entorno.
6. **El script hace TODOS los cálculos** — Claude solo ejecuta el script, presenta resultados y llama a odoo-pilot para las escrituras.
7. **La ruta a odoo-pilot** debe apuntar al directorio `scripts/` del skill `odoo-pilot` instalado — no hardcodear rutas absolutas de usuario.

## NEVER
- Continuar con errores de empleados/departamentos sin resolver.
- Crear líneas en Odoo sin confirmación explícita del usuario (Paso 2).
- Usar `message_post` para la nota interna del chatter.

## ALWAYS
- Usar odoo-pilot (`auth.sh`) con contexto `{"lang": "es_ES"}`.
- Verificar que total debe = total haber antes y después de crear las líneas.
- Manejar importes con 2 decimales.

## Delegates to

| Skill | Suite | Para qué |
|-------|-------|----------|
| `odoo-pilot` | common | Autenticación, búsqueda de cuentas, creación de líneas y mail.message |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| — | — | Skill terminal de contabilidad |

## REFERENCES

- `references/account_mapping.md` — Mapeo de departamentos a cuentas contables (640xxx, 642xxx)
- `references/workflow-detail.md` — Comandos exactos para Paso 3 y Paso 4
- `scripts/process_payroll.py` — Cálculo completo de líneas contables desde Excel
