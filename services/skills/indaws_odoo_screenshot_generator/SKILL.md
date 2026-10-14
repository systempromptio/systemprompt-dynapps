---
name: indaws-odoo-screenshot-generator
version: 2.0.3
effort: low
model: sonnet
description: >-
  Genera capturas de pantalla automáticas de vistas Odoo usando Playwright. Acepta documentos que
  describen qué capturas generar (documentación de módulos, especificaciones, flujos de formación),
  detecta placeholders [SCREENSHOT:id], valida datos de prueba, captura con resaltado de campos y
  actualiza documentos. Batch con sesión compartida, reintentos, crop, DPR y validación post-
  captura. Usar cuando haya que generar capturas de Odoo o rellenar placeholders [SCREENSHOT:id] en
  documentación o formación.
allowed-tools: [Read, Write, Bash, Glob, Grep]
---

# Odoo Screenshot Generator v2.0

Capturas automáticas de vistas Odoo via Playwright headless.

## Workflow

Full details: `references/workflow-details.md`

### Paso 0 — Recopilación [STOP]
Preguntar: documento de entrada (ruta), credenciales Odoo (URL, DB, user, key), configuración (resolución, zoom, idioma). **Esperar respuesta.**

### Paso 1 — Análisis del documento
Leer documento → detectar placeholders `[SCREENSHOT:identifier]` → extraer modelos y vistas a capturar. Ver `references/capture-planning.md`.

### Paso 2 — Validación de datos [STOP si vacío]
Conectar con odoo-pilot → verificar que existen registros en cada modelo. Si vacío → avisar y parar.

### Paso 3 — Preparar Playwright
Verificar instalación, crear directorio `screenshots/{module_name}/`.

### Paso 4 — Obtener URLs
Buscar action IDs con odoo-pilot → construir URLs de vistas (list, form, kanban).

### Paso 5 — Capturar
Ejecutar `scripts/capture.js` por cada captura. Ver `references/capture-script.md` para parámetros.

### Paso 6 — Documentar + actualizar placeholders
Generar `README_screenshots.md`. Si hay placeholders → reemplazar en documento original.

### Paso 7 — Presentar resultados
Tabla resumen: archivo, modelo, vista, descripción.

## Capacidades de capture.js

- **Batch mode** — un solo browser/login para múltiples capturas (`--config captures.json`)
- **Cookie reuse** — guarda/restaura sesión entre ejecuciones (`--cookie-file`)
- **Reintentos con backoff** — reintentos automáticos en fallos de selector o carga
- **DPR configurable** — captura HiDPI con `--device-scale-factor 2`
- **Crop selector** — `--crop-selector ".o_content"` para capturar solo el área relevante
- **Validación post-capture** — detecta página de login, diálogos de error, PNGs vacíos
- **Validación de sesión** — detecta sesión expirada y re-autentica automáticamente

## GOTCHAS

1. **Capturas NUNCA vacías** — validación automática post-captura: tamaño mínimo, PNG header, detección de login page y error dialogs
2. **SIEMPRE resaltar campos** — usar `--highlight` con selectores CSS de campos nuevos/modificados del módulo
3. **No duplicar capturas** entre placeholders — si dos apuntan a la misma vista, buscar ángulo diferente
4. **Contraseña, no API key** — `capture.js` hace login via web, necesita password real
5. **Zoom 125% por defecto** — mejora legibilidad de textos en capturas
6. **Esperar carga completa** — usar `--wait-selector ".o_content"` + `--wait-time 2000`
7. **Batch mode para múltiples capturas** — evita 15 logins para 15 capturas. Usar `--config` siempre que haya >1 captura
8. **DPR 2x para UIs densas** — kanban con muchas columnas o list views anchas se benefician de `--device-scale-factor 2`

## Rules

- **NUNCA** continuar sin datos (salvo insistencia del usuario)
- **SIEMPRE** validar datos antes de capturar (Paso 2)
- **SIEMPRE** documentar capturas generadas
- **SIEMPRE** resaltar campos relevantes con rectángulo rojo

## Scripts

| Script | Qué hace |
|--------|----------|
| `scripts/capture.js` | Playwright: login → navigate → wait → highlight → capture PNG. Single + batch mode, reintentos, crop, DPR, cookie reuse |

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | Validar datos y obtener action IDs | Bash inline (scripts) |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-module-documenter | common | Screenshots for module documentation |
| odoo-flow-tester | common | Screenshots for flow documentation |
| indaws-core-documenter | consultant | Screenshots for Indaws Core packages |
| training-planner | consultant | Screenshots for training materials |
| project-validation-drafting | consultant | Screenshots for validation documents |

See `references/shared/odoo-connection.md` for connection pattern used in Paso 2.

## References

| File | Content |
|------|---------|
| `references/workflow-details.md` | Full workflow with sub-items |
| `references/capture-planning.md` | Document analysis, placeholder detection, capture list |
| `references/capture-script.md` | capture.js parameters, examples, interactions |
| `references/playwright_odoo_guide.md` | Playwright + Odoo patterns |
