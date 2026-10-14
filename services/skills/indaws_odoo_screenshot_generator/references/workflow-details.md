# Workflow Details — Screenshot Generator

## Requisitos Previos

1. Módulo instalado en la instancia Odoo
2. Datos de prueba importados (odoo-test-data-generator + odoo-importation)
3. Usuario con permisos para acceder a las vistas
4. Playwright instalado (Node.js + Chromium)
5. Para batch mode: archivo `captures.json` generado en Paso 1

## Paso 0: Recopilación de Información [STOP]

Preguntar:
1. **Documento de entrada**: ruta al documento con especificación de capturas
2. **Credenciales Odoo**: URL, DB, usuario, contraseña (NO API key)
3. **Configuración** (opcional): resolución (1920x1080), zoom (125%), idioma (es_ES)

## Paso 1: Análisis del Documento

Detectar placeholders `[SCREENSHOT:identifier]`. Ver `capture-planning.md`.

## Paso 2: Validación de Datos [STOP si vacío]

Conectar con odoo-pilot y verificar registros por modelo:
```bash
./scripts/search_records.sh {model} '[]' '["id","name"]' 5
```

Si no hay datos → avisar y sugerir odoo-test-data-generator + odoo-importation.

## Paso 3: Preparación Playwright

```bash
# Verificar
npx playwright --version
# Instalar si falta
npm install -D playwright && npx playwright install chromium
# Crear directorio
mkdir -p screenshots/{module_name}/
```

## Paso 4: Obtención de URLs

```bash
# Buscar action
./scripts/search_records.sh ir.actions.act_window \
  '[["res_model","=","{model}"]]' '["id","name"]' 1
```

URLs:
- Lista: `{URL}/web#action={id}&model={model}&view_type=list`
- Form: `{URL}/web#id={record_id}&action={id}&model={model}&view_type=form`

## Paso 5: Captura

### Opción A: Batch mode (recomendado para 2+ capturas)

Generar archivo `captures.json` con la lista de capturas:
```json
[
  {
    "url": "https://demo.odoo.com/web#action=123&model=crm.prospection&view_type=list",
    "output": "01_list_crm_prospection.png",
    "waitSelector": ".o_list_view"
  },
  {
    "url": "https://demo.odoo.com/web#id=1&action=123&model=crm.prospection&view_type=form",
    "output": "02_form_crm_prospection.png",
    "highlight": "[name=prospection_count],[name=state]"
  }
]
```

```bash
node ./scripts/capture.js \
  --config captures.json \
  --output-dir screenshots/{module_name}/ \
  --user "admin" --password "***" --db "demo" \
  --retries 2
```

**Ventaja:** Un solo login, un solo browser, reintentos automáticos, resumen al final.

### Opción B: Single mode (1 captura)

```bash
node ./scripts/capture.js \
  --url "{URL}/web#id={record_id}&action={id}&model={model}&view_type=form" \
  --output "screenshots/{module_name}/01_form.png" \
  --user "admin" --password "***" --db "demo" \
  --highlight "[name=field1],[name=field2]"
```

Ver `capture-script.md` para todos los parámetros.

## Paso 6: Documentar + Actualizar Placeholders

Generar `README_screenshots.md` con tabla de capturas.
Si había placeholders → reemplazar `[SCREENSHOT:id]` con `![desc](./screenshots/module/N_id.png)`.

## Paso 7: Presentar Resultados

Tabla resumen + documento actualizado (si aplica).

## Validación Post-captura

- **Vistas list**: verificar `.o_data_row` existe
- **Vistas kanban**: verificar `.o_kanban_record` existe
- **Vistas form**: verificar campos con valores

Si vacía → NO generar, avisar.

## Naming Convention

```
{secuencia}_{view_type}_{model_name}.png
```

Ejemplos: `01_form_crm_prospection.png`, `02_list_crm_prospection.png`
