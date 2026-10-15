# Guía de Playwright para Odoo

Esta guía documenta selectores CSS comunes, estrategias de espera y mejores prácticas para capturar screenshots de Odoo con Playwright.

---

## Selectores CSS Comunes en Odoo

### Estructura Principal

```css
/* Contenedor principal de la aplicación */
.o_web_client

/* Área de contenido */
.o_content

/* Contenedor de acción actual */
.o_action_manager .o_action

/* Formulario */
.o_form_view

/* Lista/Tabla */
.o_list_view

/* Kanban */
.o_kanban_view
```

### Elementos de Vista

```css
/* Botones de acción (Crear, Editar, Guardar, etc.) */
.o_cp_buttons button

/* Smart buttons */
.o_form_statusbar .oe_stat_button

/* Statusbar (estados) */
.o_form_statusbar .o_statusbar_status

/* Chatter (mensajería) */
.o_chatter

/* Notebook tabs */
.o_notebook .nav-link

/* Campos de formulario */
.o_field_widget

/* Tabla de líneas (one2many) */
.o_field_one2many .o_list_table
```

### Elementos de Navegación

```css
/* Menú principal */
.o_menu_brand

/* Apps menu */
.o_apps

/* Menú de usuario */
.o_user_menu

/* Breadcrumbs */
.o_breadcrumb
```

### Elementos a Ocultar

```css
/* Barra de debug */
.o_debug_manager

/* Notificaciones/Toasts */
.o_notification
.o_notification_manager

/* Tooltips */
.o_tooltip
.tooltip

/* Loading spinners */
.o_loading
.fa-spinner
```

---

## Estrategias de Espera

### 1. Esperar a que Cargue la Vista

```javascript
// Esperar contenido principal
await page.waitForSelector('.o_content', { timeout: 15000 });

// Esperar vista específica
await page.waitForSelector('.o_form_view', { timeout: 10000 });
await page.waitForSelector('.o_list_view', { timeout: 10000 });
await page.waitForSelector('.o_kanban_view', { timeout: 10000 });
```

### 2. Esperar a que Termine la Carga de Datos

```javascript
// Esperar a que desaparezca el spinner
await page.waitForSelector('.o_loading', { state: 'hidden', timeout: 10000 });

// Esperar red idle
await page.waitForLoadState('networkidle');
```

### 3. Esperar Tiempo Fijo (Para Animaciones)

```javascript
// Esperar animaciones CSS
await page.waitForTimeout(2000); // 2 segundos
```

### 4. Esperar Elemento Específico

```javascript
// Esperar smart button con datos
await page.waitForSelector('.oe_stat_button .o_stat_value:not(:empty)', {
  timeout: 5000
});

// Esperar tabla con filas
await page.waitForSelector('.o_list_table tbody tr', { timeout: 5000 });
```

---

## Flujo de Login

### Método 1: Login con Formulario (Recomendado)

```javascript
// Navegar a login
await page.goto(`${odooUrl}/web/login?db=${database}`);

// Rellenar formulario
await page.fill('input[name="login"]', username);
await page.fill('input[name="password"]', password);
await page.click('button[type="submit"]');

// Esperar navegación
await page.waitForNavigation({ timeout: 10000 });

// Verificar login exitoso
const isLoggedIn = await page.$('.o_web_client');
if (!isLoggedIn) {
  throw new Error('Login failed');
}
```

### Método 2: Login con Cookies (Más Rápido)

Si ya tienes una sesión válida:

```javascript
// Guardar cookies después del primer login
const cookies = await context.cookies();
fs.writeFileSync('cookies.json', JSON.stringify(cookies));

// En siguientes ejecuciones, restaurar cookies
const savedCookies = JSON.parse(fs.readFileSync('cookies.json'));
await context.addCookies(savedCookies);
```

---

## Construcción de URLs de Odoo

### URL de Lista

```
{odoo_url}/web#action={action_id}&model={model}&view_type=list
```

**Ejemplo**:
```
https://demo.odoo.com/web#action=123&model=crm.prospection&view_type=list
```

### URL de Formulario

```
{odoo_url}/web#id={record_id}&action={action_id}&model={model}&view_type=form
```

**Ejemplo**:
```
https://demo.odoo.com/web#id=1&action=123&model=crm.prospection&view_type=form
```

### URL de Kanban

```
{odoo_url}/web#action={action_id}&model={model}&view_type=kanban
```

### Obtener Action ID

Usa odoo-pilot para buscar el action:

```bash
./scripts/search_records.sh ir.actions.act_window \
  '[["res_model","=","crm.prospection"]]' \
  '["id","name"]' 1
```

Resultado:
```json
[
  {"id": 123, "name": "Prospecciones"}
]
```

---

## Mejores Prácticas

### 1. Viewport Apropiado

```javascript
// Desktop estándar
{ width: 1920, height: 1080 }

// Laptop
{ width: 1366, height: 768 }

// Tablet
{ width: 1024, height: 768 }
```

### 2. Zoom para Legibilidad

Aplicar zoom CSS al 125% mejora notablemente la legibilidad de los textos en las capturas:

```javascript
// Aplicar zoom 125% (valor por defecto en capture.js)
await page.evaluate(() => {
  document.body.style.zoom = '1.25';
});
await page.waitForTimeout(500);
```

**Nota**: Se aplica después de que la página ha cargado completamente y antes de la captura. El valor por defecto es `1.25` (125%). Usar `1` para desactivar el zoom.

### 3. Ocultar Elementos de Desarrollo

```javascript
await page.evaluate(() => {
  // Debug manager
  document.querySelector('.o_debug_manager')?.remove();

  // Notificaciones
  document.querySelectorAll('.o_notification').forEach(n => n.remove());

  // Tooltips
  document.querySelectorAll('.o_tooltip, .tooltip').forEach(t => t.remove());
});
```

### 3. Scroll al Elemento Antes de Capturar

```javascript
// Scroll a smart button específico
await page.locator('.oe_stat_button').first().scrollIntoViewIfNeeded();

// Scroll al final de la página
await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
```

### 4. Capturar Solo un Elemento

```javascript
// Capturar solo el formulario, sin menú
const formElement = await page.$('.o_form_view');
await formElement.screenshot({ path: 'form.png' });

// Capturar solo la tabla
const tableElement = await page.$('.o_list_table');
await tableElement.screenshot({ path: 'table.png' });
```

### 5. Captura en Diferentes Idiomas

```javascript
const context = await browser.newContext({
  locale: 'es-ES',  // Español
  // locale: 'en-US',  // Inglés
  // locale: 'fr-FR',  // Francés
});
```

---

## Troubleshooting

### Problema: Página No Carga

```javascript
// Solución: Aumentar timeout y esperar networkidle
await page.goto(url, {
  waitUntil: 'networkidle',
  timeout: 30000
});
```

### Problema: Elemento No Encontrado

```javascript
// Solución: Esperar con estado visible
await page.waitForSelector('.o_form_view', {
  state: 'visible',
  timeout: 15000
});
```

### Problema: Screenshot Muestra Loading

```javascript
// Solución: Esperar a que desaparezca
await page.waitForSelector('.o_loading', {
  state: 'hidden',
  timeout: 10000
});
```

### Problema: Login Falla

```javascript
// Debug: Capturar pantalla de error
try {
  await page.click('button[type="submit"]');
  await page.waitForNavigation();
} catch (error) {
  await page.screenshot({ path: 'login_error.png' });
  throw error;
}
```

---

## Ejemplo Completo

```javascript
const { chromium } = require('playwright');

async function captureOdooView() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 },
    locale: 'es-ES'
  });
  const page = await context.newPage();

  try {
    // Login
    await page.goto('https://demo.odoo.com/web/login?db=demo');
    await page.fill('input[name="login"]', 'admin');
    await page.fill('input[name="password"]', 'admin');
    await page.click('button[type="submit"]');
    await page.waitForNavigation();

    // Navegar a vista
    const url = 'https://demo.odoo.com/web#action=123&model=crm.prospection&view_type=form&id=1';
    await page.goto(url, { waitUntil: 'networkidle' });

    // Esperar contenido
    await page.waitForSelector('.o_form_view');
    await page.waitForTimeout(2000);

    // Limpiar
    await page.evaluate(() => {
      document.querySelector('.o_debug_manager')?.remove();
    });

    // Capturar
    await page.screenshot({ path: 'form_view.png' });

    console.log('✅ Screenshot guardado');

  } finally {
    await browser.close();
  }
}

captureOdooView();
```

---

## Referencias

- **Playwright Docs**: https://playwright.dev/docs/api/class-page
- **Odoo Web Client**: Inspecciona elementos en el navegador para ver clases CSS
- **Odoo Framework**: https://www.odoo.com/documentation/18.0/developer/
