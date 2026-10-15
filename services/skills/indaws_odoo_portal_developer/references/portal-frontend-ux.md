# Portal Frontend UX — Responsive Design PC + Móvil

## Tabla de Contenidos

- [Sistema de Grid y Layout](#sistema-de-grid-y-layout)
- [Breakpoints y Clases Responsive](#breakpoints-y-clases-responsive)
- [Sidebar Responsive](#sidebar-responsive-desktop--offcanvas-mobile)
- [Tablas Responsive](#tablas-responsive)
- [Searchbar Mobile](#searchbar-mobile)
- [Cards del Dashboard](#cards-del-dashboard)
- [SCSS Portal — Variables y Customización](#scss-portal--variables-y-customización)
- [Patrones de UX Recomendados](#patrones-de-ux-recomendados)
- [Accesibilidad](#accesibilidad)
- [Anti-patrones](#anti-patrones)

---

## Sistema de Grid y Layout

El portal usa Bootstrap 5 con un layout de 2 columnas en desktop que colapsa en mobile.

### Layout principal (portal_layout)

`odoo18/addons/portal/views/portal_templates.xml:162`, `portal_layout`. El reparto del diagrama no es una convención: el área de contenido es
`o_portal_content col-12 col-lg-8` (`:185`) y la columna lateral
`d-none d-lg-flex justify-content-end col-lg-4` (`:188`) — de ahí salen los 992px, que es
donde Bootstrap corta `lg`.

```
Desktop (≥992px / lg):
┌─────────────────────────────────────────────────┐
│ Header: Logo | Nav | User Dropdown              │
├─────────────────────────────────────────────────┤
│ Breadcrumbs                    | Record Pager   │
├──────────────────────────┬──────────────────────┤
│ Content (col-lg-8)       │ Sidebar (col-lg-4)   │
│                          │ - Status             │
│ - Info cards             │ - Actions            │
│ - Table/details          │ - Contact            │
│ - Chatter                │                      │
├──────────────────────────┴──────────────────────┤
│ Footer                                          │
└─────────────────────────────────────────────────┘

Mobile (<992px):
┌─────────────────────┐
│ Header + Hamburger  │
├─────────────────────┤
│ Breadcrumbs         │
├─────────────────────┤
│ Content (col-12)    │
│ - Info cards        │
│ - Table (scroll-x)  │
│ - Chatter           │
├─────────────────────┤
│ Footer              │
└─────────────────────┘
│ ← Offcanvas Sidebar │  (swipe/button)
```

### Clases CSS del layout

Todas verificadas el 2026-08-15 contra `odoo18/addons/portal/static/src/scss/portal.scss` y
`odoo18/addons/portal/views/portal_templates.xml`: las diez existen en la fuente de v18. Son
clases de Odoo, no inventadas para esta guía — si las usas, hereda, no las redefinas.

| Clase | Propósito |
|-------|-----------|
| `.o_portal` | Container principal (aplicado a `#wrapwrap`) |
| `.o_portal_wrap` | Inner wrapper (`id="wrap"`) |
| `.o_portal_content` | Área de contenido principal |
| `.o_portal_sidebar_content` | Info lateral (flex-grow-1, pe-lg-4) |
| `.o_portal_my_home` | Container del dashboard |
| `.o_portal_docs` | Grid de cards (row g-2) |
| `.o_portal_category` | Grupo de cards por categoría |
| `.o_portal_index_card` | Card individual |
| `.o_portal_navbar` | Barra de filtros |
| `.o_portal_my_doc_table` | Tabla de documentos |

---

## Breakpoints y Clases Responsive

El portal usa los breakpoints estándar de Bootstrap 5:

| Breakpoint | Mín | Uso portal |
|-----------|-----|------------|
| `xs` | 0 | Mobile vertical |
| `sm` | 576px | Mobile horizontal |
| `md` | 768px | Tablet |
| `lg` | 992px | **Desktop — punto clave del portal** |
| `xl` | 1200px | Desktop ancho |
| `xxl` | 1400px | Desktop extra ancho |

**`lg` (992px) es el breakpoint principal del portal**. Es donde:
- El sidebar aparece como columna (desktop) o se oculta (mobile)
- El searchbar se expande (desktop) o colapsa (mobile)
- El layout pasa de 1 columna a 2 columnas

### Clases de visibilidad más usadas

```html
<!-- Solo desktop (≥lg) -->
<div class="d-none d-lg-flex">sidebar content</div>

<!-- Solo mobile (<lg) -->
<div class="d-lg-none">mobile-only content</div>

<!-- Ocultar en mobile -->
<th class="d-none d-md-table-cell">Columna extra</th>

<!-- Tamaño responsive de columnas -->
<div class="col-12 col-md-6 col-lg-4">card</div>
```

### Patrón mobile-first

Siempre diseñar primero para mobile y luego añadir estilos para desktop:

```scss
// Bien: mobile-first
.my_portal_element {
    padding: 0.5rem;              // Mobile
    @include media-breakpoint-up(lg) {
        padding: 1.5rem;          // Desktop
    }
}

// Mal: desktop-first
.my_portal_element {
    padding: 1.5rem;
    @include media-breakpoint-down(lg) {
        padding: 0.5rem;
    }
}
```

---

## Sidebar Responsive (Desktop + Offcanvas Mobile)

El sidebar es el componente más importante para la UX PC/mobile. En desktop es una columna fija.
En mobile se convierte en un offcanvas (panel deslizante).

### Patrón de template

```xml
<div class="row o_portal_sidebar">
    <!-- Sidebar: visible solo en desktop -->
    <t t-call="portal.portal_record_sidebar">
        <t t-set="classes" t-value="'col-lg-4 col-xxl-3 d-print-none'"/>
        <t t-set="title">
            <h2 class="mb-0" t-field="record.amount_total"/>
        </t>
        <t t-set="entries">
            <!-- Botones, estado, contacto -->
        </t>
    </t>

    <!-- Contenido principal -->
    <div class="col-12 col-lg-8">
        <!-- El contenido -->
    </div>
</div>
```

### Offcanvas (generado por portal.side_content)

El patrón de abajo es literalmente el de
`odoo18/addons/portal/views/portal_templates.xml:179`, `wrap` (el botón `data-bs-target`) y `:191`
(el panel `offcanvas offcanvas-start d-lg-none` con `id="accountOffCanvas"`).

```xml
<!-- Botón trigger en mobile (d-lg-none) -->
<button class="btn btn-light d-lg-none"
        data-bs-toggle="offcanvas"
        data-bs-target="#accountOffCanvas">
    <i class="fa fa-bars"/>
</button>

<!-- Offcanvas panel -->
<div class="offcanvas offcanvas-start d-lg-none" id="accountOffCanvas">
    <div class="offcanvas-header">
        <button class="btn-close" data-bs-dismiss="offcanvas"/>
    </div>
    <div class="offcanvas-body">
        <!-- Mismo contenido que el sidebar desktop -->
    </div>
</div>
```

### Buenas prácticas sidebar

- **Título**: monto total, nombre del documento, o status principal
- **Botones de acción**: `btn btn-primary btn-block` para la acción principal
- **Delay labels**: Usar `.o_portal_sidebar_timeago` con atributo `datetime` para "Due in X days"
- **Contacto**: `portal.portal_my_contact` con avatar, nombre, email, teléfono
- **Print**: `PortalSidebar._printIframeContent()` para imprimir el contenido

---

## Tablas Responsive

Las tablas de portal usan `portal.portal_table` que ya incluye:
- `.table-responsive` (scroll horizontal en mobile)
- Paginación automática
- Estilos consistentes

### Ocultar columnas en mobile

```xml
<table class="table table-sm">
    <thead>
        <tr class="active">
            <th>Name</th>
            <th class="text-end">Amount</th>
            <!-- Visible solo en md+ -->
            <th class="d-none d-md-table-cell">Date</th>
            <!-- Visible solo en lg+ -->
            <th class="d-none d-lg-table-cell text-center">Status</th>
        </tr>
    </thead>
    <tbody>
        <tr t-foreach="records" t-as="rec">
            <td>
                <a t-att-href="rec.get_portal_url()">
                    <t t-out="rec.name"/>
                </a>
                <!-- En mobile, mostrar info extra debajo del nombre -->
                <small class="d-block d-md-none text-muted">
                    <span t-field="rec.create_date" t-options='{"widget": "date"}'/>
                    — <span t-field="rec.state"/>
                </small>
            </td>
            <td class="text-end" t-field="rec.amount_total"/>
            <td class="d-none d-md-table-cell" t-field="rec.create_date"
                t-options='{"widget": "date"}'/>
            <td class="d-none d-lg-table-cell text-center">
                <span class="badge rounded-pill text-bg-info" t-out="rec.state"/>
            </td>
        </tr>
    </tbody>
</table>
```

El truco clave: en mobile, poner info secundaria como `<small>` debajo de la columna principal,
y ocultarla en desktop con `d-block d-md-none`.

### Alternativa: cards en mobile

Para UX superior en mobile, mostrar cards en vez de tabla:

```xml
<!-- Desktop: tabla -->
<div class="d-none d-md-block">
    <table class="table"><!-- tabla normal --></table>
</div>

<!-- Mobile: cards -->
<div class="d-md-none">
    <t t-foreach="records" t-as="rec">
        <div class="card mb-2">
            <div class="card-body p-3">
                <div class="d-flex justify-content-between">
                    <a t-att-href="rec.get_portal_url()">
                        <strong t-out="rec.name"/>
                    </a>
                    <span class="badge text-bg-info" t-out="rec.state"/>
                </div>
                <small class="text-muted">
                    <span t-field="rec.create_date" t-options='{"widget": "date"}'/>
                </small>
                <div class="text-end mt-1">
                    <strong t-field="rec.amount_total"/>
                </div>
            </div>
        </div>
    </t>
</div>
```

---

## Searchbar Mobile

El searchbar (`portal.portal_searchbar`) usa `navbar-expand-lg`:
- En desktop: todos los filtros visibles en línea
- En mobile: colapsan tras botón hamburger

```xml
<nav class="navbar navbar-expand-lg o_portal_navbar border-bottom">
    <!-- Título (siempre visible) -->
    <span class="navbar-brand">My Things</span>

    <!-- Toggle (solo mobile) -->
    <button class="navbar-toggler" data-bs-toggle="collapse"
            data-bs-target="#o_portal_navbar_content">
        <span class="fa fa-bars"/>
    </button>

    <!-- Filtros (colapsan en mobile) -->
    <div class="collapse navbar-collapse" id="o_portal_navbar_content">
        <!-- Sort, Filter, Group, Search -->
    </div>
</nav>
```

### Variables SCSS del toolbar mobile

```scss
$o-portal-mobile-toolbar: true;
$o-portal-mobile-toolbar-border: $border-color;
$o-portal-mobile-toolbar-bg: $gray-200;
```

---

## Cards del Dashboard

Las cards de `/my/home` usan el grid system:

```xml
<div class="o_portal_docs row g-2">
    <!-- Cards standard: 2 por fila en md+ -->
    <div class="o_portal_index_card col-md-6 order-2">
        <!-- portal_docs_entry content -->
    </div>

    <!-- Cards alert: full width -->
    <div class="o_portal_index_card col-12 order-0">
        <!-- portal_docs_entry con alert styling -->
    </div>
</div>
```

Los `order-*` controlan la posición: alerts primero (order-0), docs después (order-2).

---

## SCSS Portal — Variables y Customización

**Archivo**: `portal/static/src/scss/portal.scss` (476 líneas)

### Variables principales

```scss
// Bootstrap overrides en portal
$o-portal-mobile-toolbar: true;
$o-portal-mobile-toolbar-border: $border-color;
$o-portal-mobile-toolbar-bg: $gray-200;
```

### Cómo extender estilos portal

Crear un archivo SCSS en tu módulo y registrarlo en el manifest:

```python
# __manifest__.py
{
    'assets': {
        'web.assets_frontend': [
            'my_module/static/src/scss/portal_custom.scss',
        ],
    },
}
```

```scss
// portal_custom.scss
.o_portal {
    // Tus customizaciones aquí

    .my_custom_section {
        padding: 1rem;
        border-radius: 0.375rem;
        background: var(--bs-light);

        @include media-breakpoint-up(lg) {
            padding: 1.5rem;
        }
    }

    // Customizar sidebar
    .o_portal_sidebar_content {
        .my_status_badge {
            font-size: 0.875rem;
        }
    }
}
```

### Assets bundle

Los assets del portal van en `web.assets_frontend`, NO en `web.assets_backend`:

```python
'assets': {
    'web.assets_frontend': [
        'my_module/static/src/scss/portal.scss',
        'my_module/static/src/js/portal.js',
    ],
},
```

---

## Patrones de UX Recomendados

### 1. Acciones claras y prominentes

```xml
<!-- Bien: botón primario claro -->
<a class="btn btn-primary" href="...">
    <i class="fa fa-check me-1"/> Accept Order
</a>

<!-- Mal: link perdido en el texto -->
<p>Click <a href="...">here</a> to accept.</p>
```

### 2. Status badges consistentes

```xml
<!-- Usar text-bg-* de Bootstrap 5 -->
<span class="badge rounded-pill text-bg-success">Confirmed</span>
<span class="badge rounded-pill text-bg-warning">Pending</span>
<span class="badge rounded-pill text-bg-danger">Overdue</span>
<span class="badge rounded-pill text-bg-info">Draft</span>
<span class="badge rounded-pill text-bg-secondary">Cancelled</span>
```

### 3. Empty states

```xml
<t t-if="not things">
    <div class="text-center py-5">
        <p class="o_view_nocontent_smiling_face">
            No things yet
        </p>
        <p class="text-muted">
            Your things will appear here once created.
        </p>
    </div>
</t>
```

### 4. Loading states para counters

Los counters se cargan async. El template base ya muestra un placeholder animado
mientras carga. No necesitas hacer nada extra.

### 5. Formularios mobile-friendly

```xml
<form class="o_portal_details" action="/my/account" method="post">
    <div class="row">
        <!-- Campos full-width en mobile, mitad en desktop -->
        <div class="col-12 col-lg-6 mb-3">
            <label class="form-label" for="name">Name</label>
            <input type="text" class="form-control" name="name" id="name"
                   t-att-value="partner.name" required="required"/>
        </div>
        <div class="col-12 col-lg-6 mb-3">
            <label class="form-label" for="email">Email</label>
            <input type="email" class="form-control" name="email" id="email"
                   t-att-value="partner.email" required="required"/>
        </div>
    </div>
    <button type="submit" class="btn btn-primary">Save</button>
</form>
```

### 6. Alertas contextuales

```xml
<!-- Mensaje de éxito tras acción -->
<div t-if="message == 'sign_ok'" class="alert alert-success" role="alert">
    <strong>Thank you!</strong> Your document has been signed.
</div>

<!-- Warning sobre estado -->
<div t-if="thing.state == 'overdue'" class="alert alert-warning" role="alert">
    <i class="fa fa-exclamation-triangle me-1"/>
    This item is overdue. Please take action.
</div>
```

---

## Accesibilidad

El portal de Odoo 18 incluye features de accesibilidad que debes mantener:

- **Skip to content**: `.o_skip_to_content` (visually hidden, focusable)
- **ARIA labels**: en navegación, dropdowns, modals
- **`role="button"`, `role="menuitem"`** en elementos interactivos
- **`data-bs-toggle="dropdown"`** para menús accesibles por teclado
- **Form validation**: `.is-invalid` con mensajes de feedback
- **Semantic HTML**: `<nav>`, `<header>`, `<main>`, `<footer>`

Al crear templates, seguir las mismas prácticas:

```xml
<!-- Bien: semántico + aria -->
<nav aria-label="Things navigation">
    <a class="btn" role="button" href="/my/things">Back to list</a>
</nav>

<!-- Mal: div sin semántica -->
<div>
    <a href="/my/things">Back</a>
</div>
```

---

## Tablas responsive en list pages

Cuando una tabla tiene más de 4 columnas, las secundarias deben ocultarse en móvil. Usa las clases `d-none d-*-table-cell` en **tanto `<th>` como `<td>`** de la misma columna:

```xml
<thead>
    <tr>
        <th>Name</th>                                    <!-- siempre visible -->
        <th>Date</th>                                    <!-- siempre visible -->
        <th class="d-none d-md-table-cell">Type</th>    <!-- oculto en móvil -->
        <th class="d-none d-md-table-cell">Responsible</th>
        <th class="d-none d-lg-table-cell text-end">Source</th>  <!-- solo desktop -->
        <th class="text-center">Status</th>              <!-- siempre visible -->
    </tr>
</thead>
<tbody>
    <tr>
        <td>...</td>
        <td>...</td>
        <td class="d-none d-md-table-cell">...</td>      <!-- mismo patrón -->
        <td class="d-none d-md-table-cell">...</td>
        <td class="d-none d-lg-table-cell text-end">...</td>
        <td class="text-center">...</td>
    </tr>
</tbody>
```

| Clase | Visible en... |
|-------|---------------|
| (sin clase) | Siempre |
| `d-none d-md-table-cell` | Tablet + desktop (≥768px) |
| `d-none d-lg-table-cell` | Solo desktop (≥992px) |

Prioridad de columnas visibles en móvil: Name (link al detalle) > Status > Date. El resto se oculta.

Para info cards en detail pages, usar `col-12 col-md-6` en vez de `col-6` fijo:
```xml
<!-- ❌ col-6 fijo: se comprime en móvil -->
<div class="col-6">...</div>

<!-- ✅ stack en móvil, 2 columnas en tablet+ -->
<div class="col-12 col-md-6">...</div>
```

### Vista dual: Cards en móvil + Tabla en desktop (patrón avanzado)

Ocultar columnas con `d-none` funciona para tablas con pocas columnas, pero cuando hay mucha info contextual (carrier, tracking, origen, partner) la tabla en móvil queda pobre incluso con columnas ocultas. La solución es **renderizar dos vistas completamente distintas**:

```xml
<!-- MOBILE: Cards clicables (<md) -->
<div class="d-md-none">
    <t t-foreach="records" t-as="rec">
        <a t-attf-href="/my/things/#{rec.id}" class="text-decoration-none text-reset">
            <div class="card mb-2 shadow-sm">
                <div class="card-body p-3">
                    <div class="d-flex justify-content-between align-items-start mb-1">
                        <strong class="text-primary">
                            <i class="fa fa-truck me-1"/>
                            <t t-esc="rec.name"/>
                        </strong>
                        <!-- Status badge inline -->
                        <span class="badge bg-success">Done</span>
                    </div>
                    <div class="text-muted small mb-1">
                        <t t-esc="rec.origin"/> — <span t-field="rec.partner_id.name"/>
                    </div>
                    <div class="d-flex justify-content-between small text-muted">
                        <span><i class="fa fa-calendar me-1"/>
                            <span t-field="rec.date" t-options='{"widget": "date"}'/>
                        </span>
                        <span t-if="rec.carrier_id">
                            <span t-field="rec.carrier_id.name"/>
                        </span>
                    </div>
                </div>
            </div>
        </a>
    </t>
</div>

<!-- DESKTOP: Tabla clásica (≥md) -->
<div class="d-none d-md-block">
    <t t-call="portal.portal_table">
        <!-- thead + tbody normales -->
    </t>
</div>
```

La card mobile incluye **toda la info relevante** sin scroll horizontal: nombre como link, status badge, origen+partner, fecha, carrier. El usuario toca la card entera para entrar al detalle.

Para el pager en mobile, añadir uno fuera de `portal_table` ya que ese solo se muestra en desktop:
```xml
<div class="d-md-none mt-3">
    <t t-call="portal.pager"/>
</div>
```

### Detail page: Cards de producto en móvil

Para tablas de líneas (move lines, order lines) en la detail page, el mismo patrón aplica. En vez de tabla con columnas Demand/Done, usar cards con badge de status:

```xml
<!-- MOBILE: Cards por producto -->
<div class="d-md-none">
    <t t-foreach="lines" t-as="line">
        <div class="card mb-2">
            <div class="card-body p-3">
                <div class="d-flex justify-content-between align-items-start">
                    <span class="fw-bold" t-field="line.product_id.name"/>
                    <!-- Badge done/total con color -->
                    <t t-if="line.quantity >= line.product_uom_qty">
                        <span class="badge bg-success ms-2">
                            <i class="fa fa-check me-1"/>
                            <span t-field="line.quantity"/> / <span t-field="line.product_uom_qty"/>
                        </span>
                    </t>
                    <t t-elif="line.quantity > 0">
                        <span class="badge bg-warning text-dark ms-2">
                            <span t-field="line.quantity"/> / <span t-field="line.product_uom_qty"/>
                        </span>
                    </t>
                    <t t-else="">
                        <span class="badge bg-light text-muted border ms-2">
                            0 / <span t-field="line.product_uom_qty"/>
                        </span>
                    </t>
                </div>
            </div>
        </div>
    </t>
</div>

<!-- DESKTOP: Tabla clásica -->
<table class="table table-sm d-none d-md-table">...</table>
```

Los 3 colores del badge (verde ✓ completado, amarillo parcial, gris pendiente) son más informativos que una progress bar fina que se corta en columnas estrechas.

---

## Status badges — patrón correcto

Para renderizar badges de estado en templates portal, usar condicionales directos con clases explícitas:

```xml
<!-- ✅ BIEN: directo, eficiente, claro -->
<t t-if="record.state == 'done'">
    <span class="badge bg-success">Done</span>
</t>
<t t-elif="record.state == 'cancel'">
    <span class="badge bg-secondary">Cancelled</span>
</t>
<t t-elif="record.state == 'assigned'">
    <span class="badge bg-primary">Ready</span>
</t>
<t t-else="">
    <span class="badge bg-info" t-esc="record.state"/>
</t>
```

```xml
<!-- ❌ MAL: loop sobre fields_get — ineficiente, ejecuta RPC por fila -->
<t t-set="selections" t-value="request.env['model'].fields_get()['state']['selection']"/>
<t t-foreach="selections" t-as="value">
    <t t-if="value[0] == record.state">
        <span t-esc="value[1]" class="badge bg-info"/>
    </t>
</t>
```

El anti-patrón `fields_get()` causa una llamada RPC adicional y un loop por cada fila de la tabla, degradando el rendimiento significativamente en listas largas.

---

## JavaScript portal — deprecación .include() (Odoo 18.2+)

En Odoo 18.2+ y 19.0, `publicWidget.registry.X.include()` está **deprecado**. Usar `patch()`:

```javascript
// ❌ DEPRECATED en 18.2+/19.0: .include()
publicWidget.registry.PortalHomeCounters.include({
    _getCountersAlwaysDisplayed() {
        return this._super(...arguments).concat(['my_count']);
    },
});

// ✅ CORRECTO: patch()
import { patch } from '@web/core/utils/patch';
import publicWidget from '@web/legacy/js/public/public_widget';

const PortalHomeCounters = publicWidget.registry.PortalHomeCounters;
patch(PortalHomeCounters.prototype, {
    _getCountersAlwaysDisplayed() {
        return super._getCountersAlwaysDisplayed(
            ...arguments
        ).concat(['my_count']);
    },
});
```

Nota: dentro de `patch()`, `super` funciona directamente (no necesita `this._super`).

---

## Anti-patrones

| Anti-patrón | Problema | Solución |
|-------------|----------|----------|
| Layout fijo en px | No escala en pantallas distintas | Usar Bootstrap grid + rem/em |
| `display: none` en mobile | Oculta info que el usuario podría necesitar | Reorganizar, no ocultar. Si ocultas, añade alternativa |
| Tabla con >5 columnas en mobile | Scroll horizontal excesivo | Ocultar columnas secundarias con `d-none d-md-table-cell` |
| `fields_get()` loop para badges | RPC + loop por fila, lento | Condicionales directos `t-if`/`t-elif` con clases explícitas |
| `.include()` en JS (18.2+) | Deprecated, romperá en v19 | Usar `patch()` de `@web/core/utils/patch` |
| Formularios sin `form-label` | Accesibilidad rota, UX confusa | Siempre `<label class="form-label">` |
| Botones pequeños en mobile | Difícil tocar (min 44x44px) | `btn-block` o padding suficiente |
| CSS inline en templates | No mantenible, no responsive | SCSS en archivo separado |
| JavaScript en template | No cacheable, no bundleable | JS en `static/src/js/` + `publicWidget`/`patch` |
| Duplicar template base | Se rompe con actualizaciones | `inherit_id` + `xpath` |
| Ignorar RTL | Portal roto para idiomas RTL | Usar clases Bootstrap (me/ms, start/end) en vez de left/right |
| `col-6` fijo en detail pages | Se comprime en móvil | Usar `col-12 col-md-6` |
| Progress bar en columna estrecha | Se corta, ilegible | Usar badges `done/total` con 3 colores (verde/amarillo/gris) |
| Tabla con mucha info en mobile | Columnas ocultas = info perdida | Vista dual: cards `d-md-none` + tabla `d-none d-md-block` |
