# Patrones de Extensión del Portal

## Tabla de Contenidos

- [Patrón General de Extensión](#patrón-general-de-extensión)
- [Home Counter](#home-counter)
- [List Page](#list-page)
- [Detail Page](#detail-page)
- [Breadcrumbs](#breadcrumbs)
- [Grouping Avanzado](#grouping-avanzado-proyecto-como-referencia)
- [Cadena de Herencia de Controllers](#cadena-de-herencia-de-controllers)
- [Enterprise Patterns](#enterprise-patterns)

---

## Patrón General de Extensión

> **De dónde sale esto.** No es doctrina: es la forma que tienen los módulos del propio Odoo 18.
> El ejemplo canónico y más corto es `odoo18/addons/sale/controllers/portal.py:17-29`, `CustomerPortal._prepare_home_portal_values`, y los
> patrones de abajo están calcados de ahí, de `account/controllers/portal.py` y de
> `project/controllers/portal.py`. Cuando dudes de un patrón, abre ese fichero antes que este.
>
> ⚠️ **`has_access('read')` es de v18.** `odoo18/odoo/models.py:4446`, `BaseModel.has_access` la define; en **v17 no
> existe** — allí el equivalente es `check_access_rights('read', raise_exception=False)`. En v19
> sigue (`odoo19/odoo/orm/models.py:4117`, `BaseModel.has_access`).

Cada módulo que extiende el portal sigue 5 pasos:

1. **Modelo**: `_inherit = ['portal.mixin']` + `_compute_access_url()`
2. **Seguridad**: Record rules para `base.group_portal` filtradas por partner
3. **Controller**: Heredar `CustomerPortal`, override `_prepare_home_portal_values()`
4. **Templates**: Heredar `portal.portal_my_home` + `portal.portal_breadcrumbs`
5. **Rutas**: List page (`auth="user"`) + Detail page (`auth="public"`)

---

## Home Counter

Añadir una card al dashboard `/my/home`.

### Modelo

```python
class MyModel(models.Model):
    _name = 'my.model'
    _inherit = ['portal.mixin', 'mail.thread']
    _description = 'My Model'

    partner_id = fields.Many2one('res.partner', required=True)
    name = fields.Char(required=True)

    def _compute_access_url(self):
        for record in self:
            record.access_url = f'/my/things/{record.id}'
```

### Controller

Calcado de `odoo18/addons/sale/controllers/portal.py:17-29`, `CustomerPortal._prepare_home_portal_values`, que hace exactamente esto con
`quotation_count` y `order_count`:

```python
from odoo.addons.portal.controllers.portal import CustomerPortal

class MyPortal(CustomerPortal):

    def _prepare_home_portal_values(self, counters):
        values = super()._prepare_home_portal_values(counters)
        if 'thing_count' in counters:
            partner = request.env.user.partner_id
            MyModel = request.env['my.model']
            values['thing_count'] = MyModel.search_count(
                self._prepare_things_domain(partner)
            ) if MyModel.has_access('read') else 0
        return values

    def _prepare_things_domain(self, partner):
        return [
            ('partner_id', 'child_of', [partner.commercial_partner_id.id]),
        ]
```

### Template

```xml
<template id="portal_my_home_thing"
          name="Portal My Home: My Things"
          inherit_id="portal.portal_my_home"
          priority="40">
    <xpath expr="//div[hasclass('o_portal_docs')]" position="before">
        <t t-set="portal_client_category_enable" t-value="True"/>
    </xpath>
    <div id="portal_client_category" position="inside">
        <t t-call="portal.portal_docs_entry">
            <t t-set="icon" t-value="'/my_module/static/src/img/thing.svg'"/>
            <t t-set="title">My Things</t>
            <t t-set="url" t-value="'/my/things'"/>
            <t t-set="placeholder_count" t-value="'thing_count'"/>
            <t t-set="text">View and manage your things</t>
        </t>
    </div>
</template>
```

El `placeholder_count` debe coincidir con la key del dict en `_prepare_home_portal_values`.
El JS `PortalHomeCounters` (`odoo18/addons/portal/static/src/js/portal.js:55`, `PortalHomeCounters`) hace RPC a
`/my/counters` (`:92`) y actualiza el badge. En v18 es un `publicWidget.Widget`, no un
componente OWL.

---

## List Page

Página con tabla paginada, filtros y ordenación.

### Controller completo

```python
from odoo.addons.portal.controllers.portal import CustomerPortal, pager as portal_pager
from odoo.http import request

class MyPortal(CustomerPortal):

    _items_per_page = 20  # el default es 80: odoo18/addons/portal/controllers/portal.py:133

    # misma forma que odoo18/addons/sale/controllers/portal.py:110
    @http.route(
        ['/my/things', '/my/things/page/<int:page>'],
        type='http', auth='user', website=True
    )
    def portal_my_things(self, page=1, date_begin=None, date_end=None,
                         sortby=None, filterby=None, search=None,
                         search_in='all', **kwargs):
        values = self._prepare_portal_layout_values()
        partner = request.env.user.partner_id
        MyModel = request.env['my.model']

        # --- Domain base ---
        domain = self._prepare_things_domain(partner)

        # --- Searchbar sortings ---
        searchbar_sortings = {
            'date': {'label': _('Newest'), 'order': 'create_date desc'},
            'name': {'label': _('Name'), 'order': 'name asc'},
            'state': {'label': _('Status'), 'order': 'state'},
        }
        if not sortby or sortby not in searchbar_sortings:
            sortby = 'date'
        order = searchbar_sortings[sortby]['order']

        # --- Searchbar filters ---
        searchbar_filters = {
            'all': {'label': _('All'), 'domain': []},
            'open': {'label': _('Open'), 'domain': [('state', '=', 'open')]},
            'done': {'label': _('Done'), 'domain': [('state', '=', 'done')]},
        }
        if not filterby or filterby not in searchbar_filters:
            filterby = 'all'
        domain += searchbar_filters[filterby]['domain']

        # --- Searchbar inputs ---
        searchbar_inputs = {
            'all': {'label': _('All'), 'input': 'all'},
            'name': {'label': _('Name'), 'input': 'name'},
        }

        # --- Search ---
        if search and search_in:
            if search_in == 'name':
                domain += [('name', 'ilike', search)]
            else:  # 'all'
                domain += ['|', ('name', 'ilike', search),
                           ('description', 'ilike', search)]

        # --- Date filter ---
        if date_begin and date_end:
            domain += [('create_date', '>', date_begin),
                       ('create_date', '<=', date_end)]

        # --- Pager ---
        thing_count = MyModel.search_count(domain)
        pager_values = portal_pager(
            url='/my/things',
            total=thing_count,
            page=page,
            step=self._items_per_page,
            url_args={
                'date_begin': date_begin, 'date_end': date_end,
                'sortby': sortby, 'filterby': filterby,
                'search': search, 'search_in': search_in,
            },
        )

        # --- Fetch records ---
        things = MyModel.search(
            domain, order=order,
            limit=self._items_per_page,
            offset=pager_values['offset'],
        )

        # --- Session history (para record pager en detail page) ---
        request.session['my_things_history'] = things.ids[:100]

        values.update({
            'things': things.sudo(),
            'page_name': 'my_things',
            'pager': pager_values,
            'default_url': '/my/things',
            'searchbar_sortings': searchbar_sortings,
            'sortby': sortby,
            'searchbar_filters': searchbar_filters,
            'filterby': filterby,
            'searchbar_inputs': searchbar_inputs,
            'search_in': search_in,
            'search': search,
        })
        return request.render('my_module.portal_my_things', values)
```

### Template list page

```xml
<template id="portal_my_things" name="My Things">
    <t t-call="portal.portal_layout">
        <t t-set="breadcrumbs_searchbar" t-value="True"/>

        <t t-call="portal.portal_searchbar">
            <t t-set="title">My Things</t>
        </t>

        <t t-call="portal.portal_table">
            <thead>
                <tr class="active">
                    <th>Reference</th>
                    <th>Name</th>
                    <th class="text-end">Date</th>
                    <th class="text-center">Status</th>
                </tr>
            </thead>
            <tbody>
                <t t-foreach="things" t-as="thing">
                    <tr>
                        <td>
                            <a t-att-href="thing.get_portal_url()">
                                <t t-out="thing.name"/>
                            </a>
                        </td>
                        <td><span t-field="thing.display_name"/></td>
                        <td class="text-end">
                            <span t-field="thing.create_date" t-options='{"widget": "date"}'/>
                        </td>
                        <td class="text-center">
                            <span class="badge rounded-pill text-bg-info">
                                <t t-out="thing.state"/>
                            </span>
                        </td>
                    </tr>
                </t>
            </tbody>
        </t>
    </t>
</template>
```

`portal.portal_table` ya incluye el pager automáticamente si `pager` está en los valores.

---

## Detail Page

Vista individual de un registro con sidebar y chatter.

### Controller

```python
@http.route(
    ['/my/things/<int:thing_id>'],
    type='http', auth='public', website=True
)
def portal_my_thing(self, thing_id, access_token=None, report_type=None,
                    download=False, **kwargs):
    try:
        thing_sudo = self._document_check_access(
            'my.model', thing_id, access_token=access_token
        )
    except (AccessError, MissingError):
        return request.redirect('/my')

    # Reporte PDF/HTML
    if report_type in ('html', 'pdf', 'text'):
        return self._show_report(
            model=thing_sudo,
            report_type=report_type,
            report_ref='my_module.action_report_thing',
            download=download,
        )

    values = {
        'thing': thing_sudo,
        'page_name': 'my_thing',
        'report_type': 'html',
    }

    # Inyecta chatter tokens, record pager, mensajes
    values = self._get_page_view_values(
        thing_sudo, access_token, values,
        'my_things_history', False, **kwargs
    )

    return request.render('my_module.portal_thing_page', values)
```

### Template detail page

```xml
<template id="portal_thing_page" name="My Thing">
    <t t-call="portal.portal_layout">
        <t t-set="o_portal_fullwidth_alert" groups="my_module.group_user">
            <t t-call="portal.portal_back_in_edit_mode">
                <t t-set="backend_url"
                   t-value="'/odoo/my-things/%s' % thing.id"/>
            </t>
        </t>

        <div class="row mt16 o_portal_sidebar">
            <!-- ===== SIDEBAR (desktop: col-lg-4, mobile: offcanvas) ===== -->
            <t t-call="portal.portal_record_sidebar">
                <t t-set="classes" t-value="'col-lg-4 col-xxl-3 d-print-none'"/>
                <t t-set="title">
                    <h2 class="mb-0" t-field="thing.name"/>
                </t>
                <t t-set="entries">
                    <div class="d-flex flex-column gap-4 mt-3">
                        <!-- Botón principal -->
                        <a class="btn btn-primary btn-block mb-1"
                           t-att-href="thing.get_portal_url(report_type='pdf', download=True)">
                            <i class="fa fa-download"/> Download PDF
                        </a>

                        <!-- Estado -->
                        <div class="d-flex align-items-center flex-wrap gap-1">
                            <strong>Status:</strong>
                            <span class="badge rounded-pill text-bg-info"
                                  t-out="thing.state"/>
                        </div>

                        <!-- Fecha con delay label -->
                        <div t-if="thing.date_deadline">
                            <strong>Deadline:</strong>
                            <span class="o_portal_sidebar_timeago ml4"
                                  t-att-datetime="thing.date_deadline"/>
                        </div>

                        <!-- Contacto -->
                        <t t-if="thing.user_id">
                            <div class="d-flex flex-column gap-3">
                                <strong>Your Contact</strong>
                                <t t-call="portal.portal_my_contact">
                                    <t t-set="_contactAvatar"
                                       t-value="image_data_uri(thing.user_id.avatar_128)"/>
                                    <t t-set="_contactName"
                                       t-value="thing.user_id.name"/>
                                    <t t-set="_contactEmail"
                                       t-value="thing.user_id.email"/>
                                    <t t-set="_contactPhone"
                                       t-value="thing.user_id.phone"/>
                                </t>
                            </div>
                        </t>
                    </div>
                </t>
            </t>

            <!-- ===== CONTENIDO PRINCIPAL ===== -->
            <div id="thing_content" class="col-12 col-lg-8 mb-5">
                <!-- Cabecera -->
                <div class="d-flex justify-content-between align-items-center mb-3">
                    <h3 t-field="thing.display_name"/>
                </div>

                <!-- Info cards -->
                <div class="row mb-4">
                    <div class="col-6">
                        <strong>Date:</strong>
                        <span t-field="thing.create_date" t-options='{"widget": "date"}'/>
                    </div>
                    <div class="col-6 text-end">
                        <strong>Reference:</strong>
                        <span t-field="thing.name"/>
                    </div>
                </div>

                <!-- Tabla de líneas (si aplica) -->
                <table class="table table-sm" t-if="thing.line_ids">
                    <thead>
                        <tr>
                            <th>Description</th>
                            <th class="text-end">Quantity</th>
                            <th class="text-end">Amount</th>
                        </tr>
                    </thead>
                    <tbody>
                        <t t-foreach="thing.line_ids" t-as="line">
                            <tr>
                                <td t-field="line.name"/>
                                <td class="text-end" t-field="line.quantity"/>
                                <td class="text-end" t-field="line.amount"
                                    t-options='{"widget": "monetary", "display_currency": thing.currency_id}'/>
                            </tr>
                        </t>
                    </tbody>
                    <tfoot>
                        <tr>
                            <td colspan="2" class="text-end"><strong>Total</strong></td>
                            <td class="text-end">
                                <strong t-field="thing.amount_total"
                                        t-options='{"widget": "monetary", "display_currency": thing.currency_id}'/>
                            </td>
                        </tr>
                    </tfoot>
                </table>

                <!-- Chatter -->
                <div id="thing_communication" class="mt-4">
                    <h4><strong>History</strong></h4>
                    <t t-call="portal.message_thread">
                        <t t-set="object" t-value="thing"/>
                    </t>
                </div>
            </div>
        </div>
    </t>
</template>
```

---

## Breadcrumbs

Cada módulo extiende `portal.portal_breadcrumbs` para añadir su sección.

```xml
<template id="portal_my_home_menu_thing"
          name="Portal Breadcrumbs: My Things"
          inherit_id="portal.portal_breadcrumbs"
          priority="40">
    <xpath expr="//ol[hasclass('o_portal_submenu')]" position="inside">
        <li t-if="page_name == 'my_things' or page_name == 'my_thing'"
            t-attf-class="breadcrumb-item #{'active' if page_name == 'my_things' else ''}">
            <a t-if="page_name != 'my_things'" href="/my/things">My Things</a>
            <t t-else="">My Things</t>
        </li>
        <li t-if="page_name == 'my_thing'" class="breadcrumb-item active">
            <t t-out="thing.name"/>
        </li>
    </xpath>
</template>
```

El `page_name` lo setea el controller en los valores del template.

---

## Grouping Avanzado (proyecto como referencia)

El módulo `project` muestra el patrón más complejo de agrupación:

```python
# En el controller, después de search():
if groupby != 'none':
    grouped_tasks = [
        Task_sudo.concat(*g)
        for k, g in groupbyelem(tasks, itemgetter(groupby))
    ]
else:
    grouped_tasks = [tasks] if tasks else []
```

En el template, iterar sobre grupos:

```xml
<t t-foreach="grouped_tasks" t-as="tasks">
    <t t-if="groupby != 'none'">
        <tr class="table-light">
            <th colspan="99">
                <span t-out="tasks[0][groupby].display_name or _('Undefined')"/>
                <span class="badge rounded-pill text-bg-secondary ms-2"
                      t-out="len(tasks)"/>
            </th>
        </tr>
    </t>
    <t t-foreach="tasks" t-as="task">
        <tr><!-- row normal --></tr>
    </t>
</t>
```

---

## Cadena de Herencia de Controllers

Los controllers heredan en cadena — cada módulo extiende al anterior:

```
portal.CustomerPortal (base)
  ├─ payment.PaymentPortal (añade payment routes)
  │    └─ sale.CustomerPortal (extiende PaymentPortal, no la base)
  ├─ account.PortalAccount (extiende base)
  ├─ purchase.CustomerPortal (extiende base)
  └─ project.ProjectCustomerPortal (extiende base)
```

Si tu módulo necesita pagos, hereda de `PaymentPortal` en lugar de `CustomerPortal`.

```python
# Sin pagos
from odoo.addons.portal.controllers.portal import CustomerPortal

# Con pagos
from odoo.addons.payment.controllers.portal import PaymentPortal
class MyPortal(PaymentPortal):
    ...
```

---

## Enterprise Patterns

Los módulos Enterprise siguen exactamente los mismos patrones que Community, con adiciones:

### Modales Bootstrap para acciones

```xml
<div class="modal fade" id="modalClose" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">Close Ticket</h5>
                <button class="btn-close" data-bs-dismiss="modal"/>
            </div>
            <div class="modal-body">
                <select class="form-select" name="close_reason">
                    <t t-foreach="close_reasons" t-as="reason">
                        <option t-att-value="reason.id" t-out="reason.name"/>
                    </t>
                </select>
            </div>
            <div class="modal-footer">
                <button class="btn btn-primary" type="submit">Confirm</button>
            </div>
        </div>
    </div>
</div>
```

### Subscription — Sidebar custom

```xml
<t t-call="portal.portal_record_sidebar">
    <t t-set="entries">
        <!-- Next billing info -->
        <div class="d-flex align-items-center gap-1">
            <strong>Next invoice:</strong>
            <span t-field="order.next_invoice_date"/>
        </div>
        <!-- Plan change button -->
        <a class="btn btn-secondary" data-bs-toggle="modal"
           data-bs-target="#changePlan">
            Change Plan
        </a>
        <!-- Payment token management -->
        <a t-att-href="'/my/subscriptions/%s/assign_token' % order.id">
            Manage Payment
        </a>
    </t>
</t>
```

### Helpdesk — Filtros por equipo/asignado

```python
searchbar_groupby = {
    'none': {'label': _('None')},
    'stage': {'label': _('Stage')},
    'team': {'label': _('Team')},
    'assigned': {'label': _('Assigned to')},
    'create_date': {'label': _('Create Date')},
}
```

El patrón es siempre el mismo: `searchbar_*` dict → template usa `portal.portal_searchbar`.

---

## Patrones para módulos extensibles

Cuando construyes un módulo portal base que otros módulos heredarán (ej: `indaws_portal_picking` base + `eurolotes_portal_picking` extensión), hay que pensar en extensibilidad desde el principio.

### Extraer searchbar dicts a métodos

Si dejas los searchbar dicts inline en la ruta, ningún módulo hijo puede añadir opciones sin duplicar toda la ruta:

```python
# ❌ MAL: inline dict — no extensible
@http.route(['/my/things', '/my/things/page/<int:page>'], ...)
def portal_my_things(self, ...):
    searchbar_groupby = {'none': {...}, 'type': {...}}  # hijo no puede tocar esto

# ✅ BIEN: método separado — hijo hace super() + añade
def _get_thing_searchbar_groupby(self):
    return {'none': {...}, 'type': {...}}

def _get_thing_groupby_map(self):
    """Return {groupby_key: field_name} for ORM groupby."""
    return {'type': 'picking_type_id', 'status': 'state'}

@http.route(['/my/things', '/my/things/page/<int:page>'], ...)
def portal_my_things(self, ...):
    searchbar_groupby = self._get_thing_searchbar_groupby()
    groupby_map = self._get_thing_groupby_map()
```

El módulo hijo solo necesita:
```python
def _get_thing_searchbar_groupby(self):
    res = super()._get_thing_searchbar_groupby()
    res['recipient'] = {'input': 'recipient', 'label': _('Recipient')}
    return res

def _get_thing_groupby_map(self):
    res = super()._get_thing_groupby_map()
    res['recipient'] = 'partner_id'
    return res
```

Aplica lo mismo a `_get_thing_searchbar_sortings()`, `_get_thing_searchbar_filters()`, y el domain base (`_get_thing_partner_ids()`).

### _get_page_view_values() es obligatorio en detail pages

Nunca construyas el dict de values manualmente en una detail page. Siempre usa `_get_page_view_values()`:

```python
# ❌ MAL: values manuales — rompe record pager, token, chatter
values = {'thing': thing_sudo, 'token': access_token}
return request.render('template', values)

# ✅ BIEN: _get_page_view_values inyecta todo automáticamente
values = self._get_page_view_values(
    thing_sudo, access_token,
    {'thing': thing_sudo, 'report_type': 'html'},
    session_history='my_things_history',
    no_breadcrumbs=False,
)
```

Sin esto se rompen: record pager (prev/next), propagación del token al chatter, historial de sesión.

### has_access('read') en counters

Siempre verifica permisos antes de `search_count` en `_prepare_home_portal_values`. Sin esto, una desinstalación parcial o cambio de permisos causa error 500 en `/my/home`:

```python
# ❌ MAL: explota si el modelo no es accesible
values['thing_count'] = request.env['my.model'].search_count(domain)

# ✅ BIEN: graceful degradation
MyModel = request.env['my.model']
if MyModel.has_access('read'):
    values['thing_count'] = MyModel.search_count(domain)
else:
    values['thing_count'] = 0
```
