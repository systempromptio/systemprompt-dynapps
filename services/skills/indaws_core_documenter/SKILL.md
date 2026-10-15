---
name: indaws-core-documenter
description: >-
  Documenta paquetes de Indaws Core analizando código fuente, generando descripciones comerciales,
  documentación de usuario final con screenshots (ES + EN) y publicando en Odoo (producto +
  knowledge base). Usar cuando haya que documentar un paquete de Indaws Core.
version: 3.2.2
effort: high
model: opus
allowed-tools: [Read, Write, Bash, Glob, Grep]
---

# Documentador de Paquetes Indaws Core

Analiza el codigo fuente de modulos de un paquete Indaws Core y genera: (1) contenido comercial y web (4 campos del producto en Odoo), (2) documentacion de usuario final en espanol con screenshots reales en Knowledge Base, (3) documentacion en ingles reutilizando screenshots.

## Field contract (preflight OBLIGATORIO)
Antes de leer Odoo, verifica que existen (ver `references/shared/field-contract-preflight.md`):
- modelo `product.template`: campos `x_studio_indaws_core`, `x_studio_documentacion_id`, `x_studio_documentation_en_id`
Si falta algo → STOP con mensaje accionable (qué falta, dónde se configura en Studio). NO continúes con datos parciales.

## Workflow -- Commercial Content (Steps 0-5)

See `references/workflow-commercial.md` for step-by-step commands.

1. **Identificar paquete** (Paso 0): Leer `x_studio_indaws_core` del producto. GATE: mostrar lista modulos antes de continuar.
2. **MANDATORY SUBAGENT -- Scan modules** (Paso 1): For each module in the package, invoke the skill 'odoo-module-scanner' on the module at `indaws_core/{nombre_modulo}/`. This produces a structured inventory of models, fields, views, security, and controllers per module.
3. **Generar contenido** (Paso 2): 4 campos via `./scripts/build_product_fields.py`. Consultar `references/content_guidelines.md`.
4. **Validar contenido** (Paso 3): STOP obligatorio. Presentar 4 bloques. Esperar aprobacion.
5. **Actualizar Odoo** (Paso 4): Orden critico: description_sale EN primero, luego 4 campos ES.
6. **Confirmar** (Paso 5): Leer producto para verificar. Informar URL.

### Critical Writing Order

description_sale EN (`context={'lang': 'en_US'}`) MUST be written BEFORE the 4 ES fields (`context={'lang': 'es_ES'}`). The ES write must be the LAST write -- it is what the user sees. Reversing this order overwrites Spanish content with English.

## Workflow -- User Documentation (Steps 6-9)

See `references/workflow-documentation.md` for step-by-step commands.

1. **Credenciales demo** (Paso 6a): Pedir URL, DB, usuario, contrasena. STOP si no hay instancia.
2. **Preparar entorno** (Paso 6b): Verificar/instalar modulos, crear datos de prueba.
3. **Generar documentacion** (Paso 6c): Markdown end_user por modulo, combinar en un unico `documentation.md`.
4. **Generar screenshots** (Paso 6d): Playwright, 3-7 por modulo, viewport 1920x1080, es_ES.
5. **Validar documentacion** (Paso 7): STOP obligatorio. Presentar resumen.
6. **Publicar KB ES** (Paso 8a-b): Via `./scripts/build_knowledge_html.py` + `validate_output.py`. Articulo padre: 2618.
7. **Publicar KB EN** (Paso 8c): Traduccion via .po files, mismos screenshots. Articulo padre: 2623.
8. **Confirmar** (Paso 9): Verificar articulos ES+EN. Informar URLs y estadisticas.

See references/shared/odoo-connection.md for connection pattern.

## GOTCHAS

1. **Modulos inferidos del codigo**: NUNCA inferir que modulos pertenecen a un paquete. La UNICA fuente es `x_studio_indaws_core` del producto en Odoo.
2. **Orden de escritura idioma**: Si se escribe en_US DESPUES de es_ES para description_ecommerce/description/website_description, se sobrescribe el espanol. Siempre EN primero, ES al final.
3. **HTML ad-hoc en lugar de scripts**: Los scripts `build_product_fields.py` y `build_knowledge_html.py` garantizan estructura correcta. Generar HTML manualmente introduce errores de estructura recurrentes.
4. **Nombres de UI en ingles en doc ES**: Los nombres de campos/botones/menus DEBEN venir de archivos `.po` (`msgstr`). Nunca dejar nombres en ingles en documentacion espanola.
5. **Screenshots de vistas vacias**: Verificar que hay datos de prueba en la instancia ANTES de capturar. Datos genericos ("Test 1") tambien invalidan los screenshots.
6. **validate_output.py no ejecutado**: GATE obligatorio antes de publicar. Si retorna `"valid": false`, corregir primero.
7. **Nombres tecnicos en H1 del HTML KB**: El H1 solo lleva `[CORE] {Nombre Comercial}`. Los nombres tecnicos van en la tabla de modulos y en los badges, no en el titulo.

### Agent() real — Pattern A "Scan First"

En Paso 1, para CADA modulo del paquete, invocar odoo-module-scanner como subagente:

```
Agent({
  description: "Scan {module_name} for documentation",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Scan the Odoo {odoo_version} module at {indaws_core_path}/{module_name}/. Return a structured inventory for documentation purposes: models (name, _inherit, _description, fields with type, help text, and string label), views (name, model, type — flag user-facing forms/lists), security (groups with display names, ACL summary), controllers (portal routes if any). Also read i18n/es.po and return translated field labels, menu names, and view titles. This inventory will be used to generate commercial and end-user documentation."
})
```

Lanzar en paralelo si hay multiples modulos. El inventario de cada modulo alimenta Paso 2.

**NO es Agent()**: odoo-pilot (Bash inline), odoo-screenshot-generator (Bash inline).

## Delegates to

| Skill | Suite | When | Agent()? |
|-------|-------|------|----------|
| odoo-module-scanner | common | Inventario por modulo (Paso 1) | **Yes — Pattern A** |
| odoo-pilot | common | CRUD product.template + knowledge.article | No |
| odoo-screenshot-generator | common | Screenshots vistas Odoo (Paso 6d) | No |

## Consumed by

Standalone. Invocado cuando se necesita documentar un paquete de Indaws Core.

## References

| File | Content | Load when |
|------|---------|-----------|
| `references/workflow-commercial.md` | Pasos 0-5: identificacion, analisis, contenido, publicacion producto | Steps 0-5 |
| `references/workflow-documentation.md` | Pasos 6-9: documentacion usuario, screenshots, publicacion KB ES+EN | Steps 6-9 |
| `references/content_guidelines.md` | Formato, tono, estructura HTML, glosario tecnico-comercial | Paso 2 |
| `references/knowledge_article_template.md` | Plantilla HTML exacta para articulos Knowledge | Paso 8 |

## Scripts

| Script | When |
|--------|------|
| `scripts/build_product_fields.py` | Paso 2 -- genera 4 campos HTML desde JSON |
| `scripts/build_knowledge_html.py` | Paso 8 -- genera HTML completo para knowledge.article |
| `scripts/validate_output.py` | Paso 8a.2 -- gate obligatorio antes de publicar |
