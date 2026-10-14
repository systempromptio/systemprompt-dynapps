# Delivery Templates

## A) XML in Odoo Module

### Server Action Record

```xml
<record id="action_server_mi_accion" model="ir.actions.server">
    <field name="name">Mi Accion</field>
    <field name="model_id" ref="model_sale_order"/>
    <field name="binding_model_id" ref="model_sale_order"/>
    <field name="binding_view_types">form</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
# Codigo aqui...
for rec in records:
    rec.write({'state': 'done'})
]]></field>
</record>
```

### XML Field Differences v18 vs v19

| Campo | v18 | v19 | Descripcion |
|-------|-----|-----|-------------|
| `model_id` | Si | Si | Modelo objetivo: `ref="model_sale_order"` |
| `binding_model_id` | Si | Si | Menu "Accion" |
| `binding_view_types` | Si | Si | `form`, `list`, `form,list` |
| `state` | code, object_write, object_create, webhook, multi | + **object_copy** | Tipo de accion |
| `groups_id` | **`groups_id`** | **`group_ids`** | **RENOMBRADO en v19** |
| `child_ids` | M2M | **O2M** (con `parent_id`) | **REESTRUCTURADO en v19** |
| `sequence` | Si | Si | Orden de ejecucion |

### Button in Form (no binding)

```xml
<button name="%(action_server_mi_accion)d"
        string="Mi Accion"
        type="action"
        class="btn-primary"/>
```

## B) Via Studio (UI)

1. Ir a Settings > Technical > Server Actions
2. Crear nueva accion
3. Seleccionar modelo
4. Tipo: "Execute Code"
5. Pegar el codigo validado

## C) Via API (odoo-pilot)

```python
action_id = env['ir.actions.server'].create({
    'name': 'Mi Accion',
    'model_id': env['ir.model']._get_id('sale.order'),
    'state': 'code',
    'code': """
for rec in records:
    rec.write({'state': 'done'})
""",
})
```

## D) Automated Action (base.automation)

### XML Template

```xml
<record id="rule_auto_mi_regla" model="base.automation">
    <field name="name">Mi Regla Automatica</field>
    <field name="model_id" ref="model_sale_order"/>
    <field name="trigger">on_create_or_write</field>
    <field name="trigger_field_ids" eval="[(4, ref('sale.field_sale_order__state'))]"/>
    <field name="filter_domain">[('state', '=', 'sale')]</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
for rec in records:
    rec.write({'priority': '1'})
]]></field>
</record>
```

### Available Triggers (base.automation)

| Trigger | Descripcion | record/records | Version |
|---------|-------------|----------------|---------|
| `on_create` | Al crear registro | Registro creado | v18+ |
| `on_write` | Al modificar registro | Registro modificado | v18+ |
| `on_create_or_write` | Al crear o modificar | Registro afectado | v18+ |
| `on_unlink` | Al eliminar registro | Registro a eliminar | v18+ |
| `on_change` | Al cambiar campo (UI) | Registro del form | v18+ |
| `on_time` | Condicion temporal (cron) | Registros que cumplen filtro | v18+ |
| `on_stage_set` | Al entrar en etapa | Registro movido | v18+ |
| `on_tag_set` | Al agregar tag | Registro etiquetado | v18+ |
| `on_state_set` | Al cambiar a estado | Registro cambiado | v18+ |
| `on_priority_set` | Al cambiar prioridad | Registro cambiado | v18+ |
| `on_user_set` | Al asignar usuario | Registro asignado | v18+ |
| `on_archive` | Al archivar registro | Registro archivado | **v19** |
| `on_unarchive` | Al desarchivar registro | Registro desarchivado | **v19** |
| `on_message_received` | Al recibir mensaje | Registro del hilo | **v19** |
| `on_message_sent` | Al enviar mensaje | Registro del hilo | **v19** |
| `on_time_created` | Temporal desde creacion | Registros filtrados | **v19** |
| `on_time_updated` | Temporal desde modificacion | Registros filtrados | **v19** |
| `on_webhook` | Al recibir webhook HTTP | Registro del payload | **v19** |
