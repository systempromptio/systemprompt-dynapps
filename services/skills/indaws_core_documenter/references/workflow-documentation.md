# Indaws Core Documenter -- User Documentation Workflow (Steps 6-9)

## Paso 6: Documentacion de Usuario Final

### 6a) Recopilacion de credenciales demo

PREGUNTAR al usuario: URL, Database, Usuario, **Contrasena** (no API key -- Playwright necesita contrasena real).
Si no tiene instancia demo, DETENTE.

### 6b) Preparacion del entorno demo

Para cada modulo del paquete:

1. **Verificar instalacion** via odoo-pilot:
   ```
   search ir.module.module [["name","=","{nombre_modulo}"],["state","=","installed"]]
   ```
2. Si NO instalado: instalar via `button_immediate_install` y verificar.
3. **Verificar datos de prueba**: buscar registros en modelos que el modulo extiende. Si no hay datos:
   - Configurar modulo (activar features)
   - Crear datos realistas (nunca "Test 1", "Ejemplo A")

**REGLA**: Screenshots NUNCA deben mostrar vistas vacias.
**REGLA**: TODOS los modulos del paquete deben tener al menos 1 screenshot (excepto glue code sin UI visible).

### 6c) Generacion de documentacion

Para cada modulo:
1. Tipo: `end_user`. Idioma: espanol. Placeholders: si.
2. Generar Markdown: Descripcion General, Funcionalidades Principales, Guia de Uso, FAQ.
3. Guardar como `output/{nombre_modulo}_documentation.md`.

**Titulo e Indice obligatorios**: `# {Nombre Comercial} ({modulo_1}, {modulo_2}, ...)` + tabla de contenidos. Si varios modulos, combinar en un unico `documentation.md`.

**NOTA**: El titulo Markdown (con nombres tecnicos) y el H1 del HTML de KB son DIFERENTES. En KB: `[CORE] {Nombre Comercial}` sin nombres tecnicos.

**Nombres de campos y menus en espanol**: Obtener de archivos `.po` del modulo (`i18n/es.po` o `i18n/es_ES.po`): `msgstr` = espanol. Si no hay `.po`, usar `string=` de vistas XML. NUNCA dejar nombres en ingles.

**Tildes y acentos obligatorios**: configuracion, informacion, seleccion, automaticamente, tambien, segun, especifico, mas, ademas, saldra, modulo, aplicacion, descripcion.

### 6d) Generacion de screenshots

Para cada placeholder `[SCREENSHOT:id]`:
1. Determinar URL Odoo correspondiente
2. Obtener action ID via odoo-pilot
3. Construir URL: `{odoo_url}/odoo/{model}/{record_id}` o `{odoo_url}/odoo/action-{action_id}`
4. Capturar con `capture.js` (viewport 1920x1080, idioma es_ES)
5. Guardar en `output/screenshots/{nombre_modulo}/`
6. Actualizar Markdown: `![Desc](./screenshots/{nombre_modulo}/{seq}_{id}.png)`

**Playwright tips**:
- Login: usar `page.waitForTimeout(5000)` despues del submit. NUNCA `page.waitForURL(...)`.
- Vista incorrecta: clic en `button.o_switch_view.o_list` (o `.o_kanban`, `.o_form`).
- Pestana incorrecta: clic en `a.nav-link:has-text("{nombre}")`.
- Ajustes: clic en menu lateral `text={nombre_seccion}`.
- Esperas: `page.waitForTimeout(2000)` despues de cada clic que cambie la vista.

## Paso 7: Validacion de la Documentacion (OBLIGATORIO)

DETENTE y presenta resumen: modulos documentados con secciones y screenshots, preview, lista screenshots.
Espera: Si / Observaciones.

## Paso 8: Publicacion en Knowledge Base

### 8a) Conversion Markdown -> HTML

**METODO**: El LLM prepara JSON. Ejecuta `./scripts/build_knowledge_html.py` con `lang: "es"`. NO generar HTML ad-hoc.

Estructura obligatoria del HTML:
1. `<h1>` con `[CORE] {Nombre Paquete}` -- SIN estilos inline, SIN nombres tecnicos
2. Subtitulo: `<p><i>Documentacion de usuario -- Paquete de Indaws Core (ES)</i></p>`
3. Indice de modulos: tabla en div gris `#f8f8f8` con cabecera violeta `#875A7B`. OBLIGATORIO incluso para 1 modulo.
4. Separador 40px entre indice y primer modulo
5. Por cada modulo: `<h2>` nombre COMERCIAL, badge nombre tecnico, parrafo introductorio, contenido
6. Separador violeta entre modulos

**PROHIBICIONES**: No wrapper div global, no estilos inline en h1/h2, no nombres tecnicos en h1, no `<ol>` con links como TOC, no `<hr>`.

Conversion: `##` -> `<h3>`, `###` -> `<h4>`, imagenes en base64.

### 8a.2) Validacion obligatoria antes de publicar

**GATE**: Ejecutar `./scripts/validate_output.py`. Si `"valid": false`, corregir antes de continuar.

### 8b) Escritura en Knowledge Base

1. Leer `x_studio_documentacion_id` del producto.
   - Si tiene valor: actualizar `body` del articulo existente.
   - Si vacio: crear `knowledge.article`: `name` = nombre comercial (sin nombres tecnicos), `parent_id` = 2618 ("INDAWS CORE"), `icon` = paquete, `internal_permission` = "write".
   - Vincular: `x_studio_documentacion_id = new_article_id`.
2. Solo actualizar `body`. NUNCA modificar `name`, `parent_id`, `icon` de existentes.

### 8c) Documentacion en Ingles (EN)

#### 8c.1) Traduccion
Para cada modulo:
1. Leer archivos `.po`: `msgid` = ingles, `msgstr` = espanol.
2. Extraer nombres reales en ingles de campos, menus, botones, selecciones.
3. Escribir documentacion en ingles usando nombres del `.po` (NO traduccion literal).
4. Guardar como `knowledge_article_{product_id}_en.html`.

#### 8c.2) Conversion a HTML
Mismo proceso que 8a pero con `lang: "en"`. `build_knowledge_html.py` genera automaticamente textos de cabecera en ingles.

#### 8c.3) Publicacion
1. Leer `x_studio_documentation_en_id`.
   - Si tiene valor: actualizar `body`.
   - Si vacio: crear con `parent_id` = 2623 ("DYNAPPS CORE").
   - Vincular: `x_studio_documentation_en_id`.
2. Escribir con `context={'lang': 'en_US'}`.

## Paso 9: Confirmacion Final

Verificar articulos ES + EN. Informar:
- IDs articulos, modulos documentados, screenshots totales
- URLs: `https://www.indaws.es/knowledge/article/{article_id}`
