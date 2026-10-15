# Indaws Core Documenter -- Commercial Content Workflow (Steps 0-5)

## Paso 0: Identificacion del Paquete (GATE OBLIGATORIO)

NUNCA adivinar, inferir ni buscar en el codigo que modulos pertenecen a un paquete. La UNICA fuente de verdad es `x_studio_indaws_core` del producto en Odoo.

1. Recibir **ID del producto** o **nombre del paquete**.
2. Leer el producto desde Odoo:
   - `name`, `x_studio_indaws_core`, `list_price`, `public_categ_ids`
   - `description_sale`, `description_ecommerce`, `description`, `website_description`
3. Si `x_studio_indaws_core` vacio, PREGUNTAR nombres tecnicos.
4. Parsear nombres de modulos (separados por `\n`).
5. **OUTPUT OBLIGATORIO**: Mostrar lista de modulos al usuario ANTES de continuar.

## Paso 1: Analisis del Codigo Fuente

La lista de modulos DEBE ser exactamente la de `x_studio_indaws_core`. No anadir extras ni omitir ninguno.

Para **cada modulo**:
1. Localizar en `indaws_core/{nombre_modulo}/`. Si NO existe, DETENTE.
2. Leer en orden: `__manifest__.py`, `models/*.py`, `views/*.xml`, `wizards/`, `controllers/`, `security/`, `data/`.
3. Extraer: que es, funcionalidades, modelos afectados, dependencias, UI (menus, botones, campos).

**REGLA**: Lee TODOS los archivos Python y XML. Los modulos Core son pequenos (1-5 archivos Python).

## Paso 2: Generacion de Contenido

**METODO**: El LLM genera datos como JSON. Luego ejecuta `./scripts/build_product_fields.py` para obtener HTML formateado. **NO generar HTML ad-hoc**.

Consulta `./references/content_guidelines.md` para formato, tono y estructura.

### a) description_sale
- Espanol. Texto plano 2-3 lineas. Sin HTML, sin preguntas retoricas.
- Tono profesional, orientado al beneficio. Empieza directamente describiendo.

### a.2) description_ecommerce
- Espanol. HTML en `<p>`. Texto DIFERENTE a description_sale.
- Orientado a venta online. Sin preguntas retoricas.

### b) description (Notas internas)
- Espanol. HTML con `<p>`, `<ul>`, `<li>`, `<b>`, `<i>`.
- Audiencia: comerciales y consultores Odoo.
- Estructura obligatoria: Cliente Ideal + Resumen de Modulos + Argumento de Venta.

### c) website_description
- Espanol. HTML con estructura Odoo website (section > container).
- Audiencia: clientes potenciales.
- Estructura por modulo: nombre comercial numerado, nombre tecnico, parrafo, lista funcionalidades.

## Paso 3: Validacion del Contenido (OBLIGATORIO)

DETENTE y presenta los 4 bloques. Espera: Si / Observaciones.

## Paso 4: Actualizacion en Odoo

**Orden obligatorio de escritura por idioma** (CRITICO):
1. PRIMERO escribir `description_sale` en ingles con `context={'lang': 'en_US'}`
2. DESPUES escribir los 4 campos en espanol con `context={'lang': 'es_ES'}` (esto es lo que queda visible)

```python
# PASO 1: description_sale EN
models.execute_kw(db, uid, key, 'product.template', 'write',
    [[product_id], {'description_sale': en_text}],
    {'context': {'lang': 'en_US'}})

# PASO 2: TODOS los campos en ES (SIEMPRE al final)
models.execute_kw(db, uid, key, 'product.template', 'write',
    [[product_id], {
        'description_sale': es_text,
        'description_ecommerce': es_html,
        'description': es_notes_html,
        'website_description': es_web_html,
    }],
    {'context': {'lang': 'es_ES'}})
```

NUNCA escribir en_US DESPUES de es_ES para description_ecommerce, description o website_description.

Solo actualizar campos de contenido. NUNCA modificar: `name`, `list_price`, `categ_id`, `type`, `service_policy`, `x_studio_indaws_core`, `public_categ_ids`.

## Paso 5: Confirmacion

Leer producto para verificar. Informar con URL: `https://www.indaws.es{website_url}`.
