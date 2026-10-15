# capture.js v2.0 — Playwright Capture Script

## Modo Single

```bash
node ./scripts/capture.js \
  --url "https://demo.odoo.com/web#action=123..." \
  --output "screenshots/module_name/01_form.png" \
  --user "admin" \
  --password "password" \
  --db "demo-julio"
```

## Modo Batch (recomendado para múltiples capturas)

Crear un archivo `captures.json`:
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
  },
  {
    "url": "https://demo.odoo.com/web#id=1&action=123&model=crm.prospection&view_type=form",
    "output": "03_smart_button.png",
    "clickSelector": ".oe_stat_button:has-text('Total Pending')",
    "waitAfterClick": 3000
  }
]
```

Ejecutar:
```bash
node ./scripts/capture.js \
  --config captures.json \
  --output-dir screenshots/module_name/ \
  --user "admin" \
  --password "password" \
  --db "demo-julio"
```

Beneficio: un solo login, un solo browser, todas las capturas en secuencia.

## Parámetros Globales

```bash
  --url "..."                       # URL de la vista (single mode)
  --output "..."                    # Ruta de salida (single mode)
  --config "captures.json"          # Lista de capturas (batch mode)
  --output-dir "..."                # Directorio de salida (batch mode)
  --odoo-url "https://demo.odoo.com"  # URL base Odoo
  --user "admin"                    # Usuario
  --password "..."                  # Contraseña (NO API key)
  --db "demo-julio"                 # Base de datos
```

## Parámetros de Captura

```bash
  --viewport "1920x1080"            # Resolución (default: 1920x1080)
  --zoom "1.25"                     # Zoom navegador (default: 1.25)
  --device-scale-factor "2"         # DPR para HiDPI (default: 1)
  --wait-selector ".o_content"      # Selector a esperar (default: .o_content)
  --wait-time "2000"                # Espera tras carga ms (default: 2000)
  --full-page "false"               # Página completa o viewport (default: false)
  --hide-debug "true"               # Ocultar debug elements (default: true)
  --locale "es-ES"                  # Idioma navegador (default: es-ES)
  --retries "2"                     # Reintentos en fallo (default: 2)
  --crop-selector ".o_content"      # Capturar solo este elemento
  --cookie-file "cookies.json"      # Guardar/restaurar cookies
```

## Parámetros de Interacción

```bash
  --click-selector ".oe_stat_button"  # Clic antes de capturar
  --wait-after-click "3000"           # Espera tras clic ms (default: 3000)
  --highlight "[name=field1]"         # Selectores CSS a resaltar (rectángulo rojo)
```

## Ejemplo con Crop

```bash
node ./scripts/capture.js \
  --url "https://demo.odoo.com/web#id=1&action=123&model=crm.prospection&view_type=form" \
  --output "screenshots/03_form_cropped.png" \
  --crop-selector ".o_content" \
  --device-scale-factor "2" \
  --user "admin" --password "password" --db "demo-julio"
```

## Ejemplo con Cookie Reuse

Primera ejecución guarda cookies:
```bash
node ./scripts/capture.js \
  --url "https://demo.odoo.com/web#action=123..." \
  --output "screenshot.png" \
  --cookie-file "/tmp/odoo_session.json" \
  --user "admin" --password "password" --db "demo-julio"
```

Siguientes ejecuciones restauran sesión (evita re-login):
```bash
node ./scripts/capture.js \
  --url "https://demo.odoo.com/web#action=456..." \
  --output "screenshot2.png" \
  --cookie-file "/tmp/odoo_session.json" \
  --user "admin" --password "password" --db "demo-julio"
```

## Validaciones Automáticas (v2)

1. **PNG header** — verifica que el archivo es un PNG válido
2. **Tamaño mínimo** — rechaza archivos < 5 KB (probablemente vacíos)
3. **Detección de login page** — si la sesión expiró, re-autentica automáticamente
4. **Detección de error dialogs** — avisa si hay `.o_error_dialog` visible
5. **Reintentos con backoff** — en fallo de selector, reintenta con espera progresiva

## Lo que hace el script

1. Lanza Chromium headless (o restaura sesión desde cookies)
2. Navega a /web/login → autentica
3. Navega a URL de la vista
4. Espera carga (selector visible + tiempo)
5. [OPCIONAL] Clic en elemento + espera
6. Oculta debug bar, notificaciones, tooltips
7. Aplica zoom CSS (125% default)
8. Resalta campos con `outline: 3px solid #e74c3c` (si `--highlight`)
9. Captura PNG (viewport, full-page, o crop a selector)
10. Valida resultado (PNG header, tamaño, estado Odoo)
11. En batch mode: continúa con siguiente captura usando misma sesión
