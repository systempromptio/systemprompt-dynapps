# Patrones de Server Actions - Recetas Comunes

## P01: Actualizar campo simple

```python
# Cambiar estado de los registros seleccionados
for rec in records:
    rec.write({'state': 'confirmed'})

# O si es el mismo valor para todos (mas eficiente):
records.write({'state': 'confirmed'})
```

## P02: Actualizar campo basado en condicion

```python
for rec in records:
    if rec.amount_total > 1000:
        rec.write({'priority': '1'})
    else:
        rec.write({'priority': '0'})
```

## P03: Crear registro relacionado

```python
for rec in records:
    env['mail.activity'].create({
        'res_model_id': env['ir.model']._get_id(rec._name),
        'res_id': rec.id,
        'activity_type_id': env.ref('mail.mail_activity_data_todo').id,
        'summary': 'Revisar: %s' % rec.name,
        'user_id': env.user.id,
        'date_deadline': datetime.date.today() + dateutil.relativedelta.relativedelta(days=7),
    })
```

## P04: Operaciones x2many - Agregar lineas

```python
# Agregar una linea nueva a un pedido de venta
for rec in records:
    product = env['product.product'].browse(42)
    rec.write({
        'order_line': [Command.create({
            'product_id': product.id,
            'product_uom_qty': 1,
            'price_unit': product.lst_price,
        })]
    })
```

## P05: Operaciones x2many - Vincular/Desvincular tags

```python
# Agregar tag sin quitar los existentes
tag = env['crm.tag'].search([('name', '=', 'VIP')], limit=1)
if tag:
    for rec in records:
        rec.write({'tag_ids': [Command.link(tag.id)]})

# Reemplazar todos los tags
new_tags = env['crm.tag'].search([('name', 'in', ['VIP', 'Premium'])])
records.write({'tag_ids': [Command.set(new_tags.ids)]})

# Quitar un tag especifico sin borrarlo
records.write({'tag_ids': [Command.unlink(tag.id)]})
```

## P06: Operaciones x2many - Actualizar lineas existentes

```python
# Actualizar precio de todas las lineas de un pedido
for rec in records:
    for line in rec.order_line:
        line.write({'discount': 10.0})
```

## P07: Enviar email con plantilla

```python
template = env.ref('module_name.email_template_xml_id')
for rec in records:
    template.send_mail(rec.id, force_send=True)
```

## P08: Enviar email sin plantilla

```python
for rec in records:
    mail_values = {
        'subject': 'Notificacion: %s' % rec.name,
        'body_html': '<p>El registro %s ha sido procesado.</p>' % rec.name,
        'email_from': env.company.email,
        'email_to': rec.partner_id.email,
    }
    env['mail.mail'].sudo().create(mail_values).send()
```

## P09: Buscar registros con dominio complejo

```python
# Facturas pendientes del partner del registro actual
invoices = env['account.move'].search([
    ('partner_id', '=', record.partner_id.id),
    ('move_type', '=', 'out_invoice'),
    ('payment_state', 'in', ['not_paid', 'partial']),
    ('state', '=', 'posted'),
])
log('Facturas pendientes encontradas: %s' % len(invoices))
```

## P10: Retornar accion de ventana (abrir vista)

```python
# Abrir lista de facturas del partner
action = {
    'type': 'ir.actions.act_window',
    'name': 'Facturas del Cliente',
    'res_model': 'account.move',
    'view_mode': 'list,form',
    'domain': [
        ('partner_id', '=', record.partner_id.id),
        ('move_type', '=', 'out_invoice'),
    ],
    'context': {
        'default_partner_id': record.partner_id.id,
        'default_move_type': 'out_invoice',
    },
}
```

## P11: Retornar accion - Abrir formulario especifico

```python
# Abrir el formulario del partner relacionado
action = {
    'type': 'ir.actions.act_window',
    'name': 'Partner',
    'res_model': 'res.partner',
    'view_mode': 'form',
    'res_id': record.partner_id.id,
    'target': 'current',
}
```

## P12: Retornar accion - Abrir wizard

```python
wizard = env['mi.wizard'].create({
    'partner_id': record.partner_id.id,
})
action = {
    'type': 'ir.actions.act_window',
    'name': 'Mi Wizard',
    'res_model': 'mi.wizard',
    'view_mode': 'form',
    'res_id': wizard.id,
    'target': 'new',
}
```

## P13: Validacion con UserError

```python
if not record.partner_id:
    raise UserError("Debe seleccionar un cliente antes de confirmar.")

if record.amount_total <= 0:
    raise UserError("El importe total debe ser mayor que cero.")

# Validacion multiple
errors = []
for rec in records:
    if not rec.partner_id.email:
        errors.append("- %s: sin email" % rec.partner_id.name)
if errors:
    raise UserError("Los siguientes clientes no tienen email:\n%s" % "\n".join(errors))
```

## P14: Trabajo con fechas

```python
today = datetime.date.today()
now = datetime.datetime.now()

# Fecha + 30 dias
deadline = today + datetime.timedelta(days=30)

# Primer dia del mes siguiente
next_month = today + dateutil.relativedelta.relativedelta(months=1, day=1)

# Ultimo dia del mes actual
last_day = today + dateutil.relativedelta.relativedelta(months=1, day=1) - datetime.timedelta(days=1)

# Comparar fechas
if record.date_order and record.date_order < today:
    log('Pedido vencido: %s' % record.name)

# Convertir a timezone
user_tz = timezone(env.user.tz or 'UTC')
now_utc = datetime.datetime.now(datetime.timezone.utc)
now_local = now_utc.astimezone(user_tz)
```

## P15: Trabajo con importes y float_compare

```python
# Comparar importes monetarios (precision 2 decimales)
if float_compare(record.amount_total, 0.0, precision_digits=2) <= 0:
    raise UserError("El importe debe ser positivo.")

# Verificar si dos importes son iguales
if float_compare(record.amount_total, record.amount_due, precision_digits=2) == 0:
    log('Importe coincide exactamente')
```

## P16: Copiar registro

```python
for rec in records:
    new_rec = rec.copy({'name': '%s (copia)' % rec.name})
    log('Creada copia: %s (ID: %s)' % (new_rec.name, new_rec.id))
```

## P17: Sudo para operaciones con permisos elevados

```python
# Leer configuracion que el usuario no tiene acceso
param = env['ir.config_parameter'].sudo().get_param('my_module.setting_key')

# Crear registro en modelo restringido
env['ir.cron'].sudo().create({
    'name': 'Cron generado: %s' % record.name,
    'model_id': env['ir.model']._get_id('sale.order'),
    'state': 'code',
    'code': 'model.search([]).action_confirm()',
    'interval_type': 'days',
    'interval_number': 1,
})
```

## P18: Acceso a configuracion (ir.config_parameter)

```python
api_key = env['ir.config_parameter'].sudo().get_param('my_module.api_key', default='')
if not api_key:
    raise UserError("Configure la API Key en Ajustes > Tecnico > Parametros del sistema")
```

## P19: Generar informe PDF

```python
# Generar informe y adjuntarlo
report = env.ref('account.account_invoices')
for rec in records:
    pdf_content, content_type = env['ir.actions.report']._render_qweb_pdf(report, rec.ids)
    attachment = env['ir.attachment'].create({
        'name': '%s.pdf' % rec.name,
        'type': 'binary',
        'datas': b64encode(pdf_content),
        'res_model': rec._name,
        'res_id': rec.id,
        'mimetype': 'application/pdf',
    })
    log('PDF generado: %s' % attachment.name)
```

## P20: Llamar metodo de negocio existente

```python
# Confirmar pedidos de venta
records.action_confirm()

# Validar facturas
records.action_post()

# Cancelar
records.button_cancel()
```

## P21: Crear nota en chatter

```python
for rec in records:
    rec.message_post(
        body='<p>Accion ejecutada por %s</p>' % env.user.name,
        message_type='comment',
        subtype_xmlid='mail.mt_note',
    )
```

## P22: Trabajo con binarios/adjuntos

```python
# Leer adjunto
attachment = env['ir.attachment'].search([
    ('res_model', '=', record._name),
    ('res_id', '=', record.id),
], limit=1)

if attachment:
    content = b64decode(attachment.datas)
    log('Adjunto encontrado: %s (%s bytes)' % (attachment.name, len(content)))
```

## P23: Accion planificada (cron) - Sin record

```python
# En acciones planificadas, record y records estan vacios
# Trabajar directamente con model o env
overdue = model.search([
    ('date_deadline', '<', datetime.date.today()),
    ('state', '!=', 'done'),
])
log('Registros vencidos: %s' % len(overdue))
for rec in overdue:
    rec.write({'state': 'overdue'})
    rec.message_post(body='<p>Marcado como vencido automaticamente.</p>')
```

## P24: Multi-write optimizado

```python
# MALO: N queries (1 por registro)
for rec in records:
    rec.write({'state': 'done', 'date_done': datetime.date.today()})

# BUENO: 1 query (si todos tienen el mismo valor)
records.write({'state': 'done', 'date_done': datetime.date.today()})

# MIXTO: agrupar cuando sea posible
to_confirm = records.filtered(lambda r: r.amount_total > 0)
to_reject = records - to_confirm
to_confirm.write({'state': 'confirmed'})
to_reject.write({'state': 'rejected'})
```

## P25: Retornar accion URL (abrir enlace externo)

```python
action = {
    'type': 'ir.actions.act_url',
    'url': 'https://www.google.com/maps?q=%s' % record.partner_id.contact_address,
    'target': 'new',
}
```

## P26: Retornar accion - Descargar informe

```python
action = env.ref('sale.action_report_saleorder').report_action(records)
```

## P27: Trabajo con campos Selection

```python
# Obtener valor legible de un campo selection
label = dict(record._fields['state'].selection).get(record.state, '')
log('Estado actual: %s' % label)
```

## P28: Acceso a datos de otra compania

```python
# Leer datos de otra compania
other_company = env['res.company'].browse(2)
products = env['product.template'].with_company(other_company).search([
    ('type', '=', 'consu'),
])
```

## P29: Ejecutar accion en contexto diferente

```python
# Forzar idioma
record_es = record.with_context(lang='es_ES')
name_es = record_es.name

# Forzar que no envie email al crear
new_order = env['sale.order'].with_context(mail_create_nosubscribe=True).create({
    'partner_id': record.partner_id.id,
})
```

## P30: Patron seguro para acciones en boton de formulario

```python
# Siempre verificar que record existe y tiene datos
if not record:
    raise UserError("No hay registro seleccionado.")

if not record.exists():
    raise UserError("El registro ya no existe.")

# Logica de negocio
record.write({'processed': True, 'processed_date': datetime.date.today()})
record.message_post(body='<p>Procesado correctamente.</p>')
```

---

## PATRONES CON indaws_base (requiere indaws_base instalado)

Los siguientes patrones usan `wrap_module`, `import_module` y `open`,
que solo estan disponibles cuando `indaws_base` esta instalado.

## P31: Expresiones regulares con wrap_module

```python
re = wrap_module('re', ['findall', 'search', 'sub', 'match', 'split', 'compile'])

# Extraer numeros de una referencia
numbers = re.findall(r'\d+', record.name)
if numbers:
    record.write({'x_studio_ref_number': int(numbers[0])})

# Limpiar HTML basico
clean_text = re.sub(r'<[^>]+>', '', record.description or '')

# Validar formato email
if record.email and not re.match(r'^[\w.+-]+@[\w-]+\.[\w.-]+$', record.email):
    raise UserError("Email con formato invalido: %s" % record.email)

# Extraer NIF/CIF
vat_match = re.search(r'[A-Z]\d{7}[A-Z0-9]', record.vat or '')
if vat_match:
    log('VAT encontrado: %s' % vat_match.group())
```

## P32: Llamadas HTTP a APIs externas

```python
requests = import_module('requests', ['get', 'post', 'put', 'delete'])

# GET simple
api_key = env['ir.config_parameter'].sudo().get_param('my_module.api_key')
response = requests.get(
    'https://api.example.com/v1/customers',
    headers={'Authorization': 'Bearer %s' % api_key},
    timeout=30,
)
if response.status_code == 200:
    data = json.loads(response.text)
    log('Clientes recibidos: %s' % len(data.get('results', [])))
else:
    raise UserError("Error API: %s - %s" % (response.status_code, response.text))

# POST con datos
response = requests.post(
    'https://api.example.com/v1/orders',
    json={'reference': record.name, 'amount': record.amount_total},
    headers={'Authorization': 'Bearer %s' % api_key},
    timeout=30,
)
```

## P33: Leer CSV del servidor

```python
csv = wrap_module('csv', ['reader', 'DictReader'])
io = import_module('io', ['StringIO'])

# Leer CSV desde un campo binario (ej: adjunto)
attachment = env['ir.attachment'].search([
    ('res_model', '=', record._name),
    ('res_id', '=', record.id),
    ('name', 'like', '%.csv'),
], limit=1)

if attachment:
    content = b64decode(attachment.datas).decode('utf-8')
    reader = csv.DictReader(io.StringIO(content), delimiter=';')
    for row in reader:
        log('Fila: %s' % json.dumps(row))
```

## P34: Escribir CSV temporal y adjuntar

```python
csv = wrap_module('csv', ['writer'])
io = import_module('io', ['StringIO'])

output = io.StringIO()
writer = csv.writer(output, delimiter=';')
writer.writerow(['Referencia', 'Cliente', 'Importe'])

for rec in records:
    writer.writerow([rec.name, rec.partner_id.name, rec.amount_total])

content = output.getvalue().encode('utf-8')
attachment = env['ir.attachment'].create({
    'name': 'export_%s.csv' % datetime.date.today().isoformat(),
    'type': 'binary',
    'datas': b64encode(content),
    'mimetype': 'text/csv',
})

action = {
    'type': 'ir.actions.act_url',
    'url': '/web/content/%s?download=true' % attachment.id,
    'target': 'new',
}
```

## P35: Generar hash/checksum

```python
hashlib = import_module('hashlib', ['md5', 'sha256'])

# Generar checksum de un fichero adjunto
if record.x_studio_file:
    content = b64decode(record.x_studio_file)
    checksum = hashlib.sha256(content).hexdigest()
    record.write({'x_studio_checksum': checksum})
    log('Checksum: %s' % checksum)
```

## P36: Parsear XML

```python
ET = import_module('xml.etree.ElementTree', ['fromstring', 'tostring', 'Element'])

# Parsear respuesta XML de un servicio
xml_content = b64decode(record.x_studio_xml_response).decode('utf-8')
root = ET.fromstring(xml_content)

# Extraer datos
status = root.find('.//Status')
if status is not None:
    record.write({'x_studio_status': status.text})
```

## P37: Leer/escribir ficheros en /tmp

```python
# Escribir fichero temporal
export_path = '/tmp/odoo_export_%s.json' % record.id
with open(export_path, 'w') as f:
    data = {
        'id': record.id,
        'name': record.name,
        'date': str(datetime.date.today()),
        'lines': [{'product': l.product_id.name, 'qty': l.product_uom_qty}
                   for l in record.order_line],
    }
    f.write(json.dumps(data, indent=2))

log('Exportado a: %s' % export_path)

# Leer fichero
with open('/tmp/import_config.json', 'r') as f:
    config = json.loads(f.read())
```

## P38: URL encoding para APIs

```python
urllib_parse = import_module('urllib.parse', ['urlencode', 'quote'])

# Construir URL con parametros
params = urllib_parse.urlencode({
    'q': record.name,
    'format': 'json',
    'limit': 10,
})
url = 'https://api.example.com/search?%s' % params

# Escapar caracteres especiales
safe_name = urllib_parse.quote(record.name)
```

## P39: Generar UUID unico

```python
uuid = import_module('uuid', ['uuid4'])

# Asignar referencia unica
for rec in records:
    if not rec.x_studio_uuid:
        rec.write({'x_studio_uuid': str(uuid.uuid4())})
```

## P40: Operaciones matematicas avanzadas

```python
math = import_module('math', ['ceil', 'floor', 'sqrt'])

# Redondear cantidad al palet completo (12 unidades)
for rec in records:
    units_per_palet = 12
    palets = math.ceil(rec.product_uom_qty / units_per_palet)
    rec.write({'x_studio_palets': palets})
```

---

## PATRONES ESPECIFICOS DE ODOO 19

Los siguientes patrones usan triggers o capacidades exclusivas de v19.
El codigo Python dentro de la accion es compatible con v18, pero la definicion
XML usa triggers/campos que solo existen en v19.

## P41: Trigger on_archive / on_unarchive (v19)

```xml
<!-- XML: base.automation con trigger v19 -->
<record id="rule_on_archive_notify" model="base.automation">
    <field name="name">Notificar al archivar</field>
    <field name="model_id" ref="sale.model_sale_order"/>
    <field name="trigger">on_archive</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
for rec in records:
    rec.message_post(
        body='<p>Pedido archivado por %s el %s</p>' % (
            user.name,
            datetime.datetime.now().strftime('%d/%m/%Y %H:%M')
        ),
        subtype_xmlid='mail.mt_note',
    )
    # Notificar al comercial
    if rec.user_id and rec.user_id != user:
        rec.activity_schedule(
            'mail.mail_activity_data_todo',
            summary='Pedido %s archivado' % rec.name,
            user_id=rec.user_id.id,
        )
]]></field>
</record>
```

## P42: Trigger on_message_received (v19)

```xml
<!-- Priorizar pedido cuando cliente responde por email -->
<record id="rule_on_message_prioritize" model="base.automation">
    <field name="name">Priorizar al recibir mensaje</field>
    <field name="model_id" ref="sale.model_sale_order"/>
    <field name="trigger">on_message_received</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
for rec in records:
    if rec.state in ('draft', 'sent'):
        rec.write({'priority': '1'})
        log('Pedido %s priorizado por mensaje recibido' % rec.name)
]]></field>
</record>
```

## P43: Webhook trigger con payload (v19)

```xml
<!-- Recibir datos de sistema externo via webhook -->
<record id="rule_webhook_external" model="base.automation">
    <field name="name">Webhook sistema externo</field>
    <field name="model_id" ref="sale.model_sale_order"/>
    <field name="trigger">on_webhook</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
# payload es un dict con los datos del webhook HTTP
if not payload:
    log('Webhook sin payload', level='warning')
else:
    reference = payload.get('order_reference')
    status = payload.get('status')

    if reference:
        order = model.search([('name', '=', reference)], limit=1)
        if order:
            if status == 'shipped':
                order.write({'x_studio_shipping_status': 'shipped'})
                order.message_post(
                    body='<p>Envio confirmado via webhook: %s</p>' % json.dumps(payload)
                )
            elif status == 'delivered':
                order.write({'x_studio_shipping_status': 'delivered'})
            log('Webhook procesado para %s: %s' % (reference, status))
        else:
            log('Pedido no encontrado: %s' % reference, level='warning')
]]></field>
</record>
```

## P44: Server action con group_ids (v19)

```xml
<!-- v19: usar group_ids en vez de groups_id -->
<record id="action_server_admin_only" model="ir.actions.server">
    <field name="name">Solo Administradores</field>
    <field name="model_id" ref="model_sale_order"/>
    <field name="binding_model_id" ref="model_sale_order"/>
    <field name="state">code</field>
    <field name="group_ids" eval="[(4, ref('base.group_system'))]"/>
    <field name="code"><![CDATA[
# Esta accion solo es visible para administradores
for rec in records:
    rec.sudo().write({'state': 'cancel'})
    rec.message_post(body='<p>Cancelado por administrador.</p>')
]]></field>
</record>
```

## P45: Multi-action con parent_id (v19)

```xml
<!-- v19: child_ids ahora es O2M con parent_id -->
<record id="action_parent_process" model="ir.actions.server">
    <field name="name">Procesar Completo</field>
    <field name="model_id" ref="model_sale_order"/>
    <field name="binding_model_id" ref="model_sale_order"/>
    <field name="state">multi</field>
</record>

<!-- Las acciones hijas declaran su parent -->
<record id="action_child_validate" model="ir.actions.server">
    <field name="name">Paso 1: Validar</field>
    <field name="parent_id" ref="action_parent_process"/>
    <field name="model_id" ref="model_sale_order"/>
    <field name="state">code</field>
    <field name="sequence">10</field>
    <field name="code"><![CDATA[
if not records.mapped('partner_id.email'):
    raise UserError("Todos los clientes deben tener email.")
]]></field>
</record>

<record id="action_child_confirm" model="ir.actions.server">
    <field name="name">Paso 2: Confirmar</field>
    <field name="parent_id" ref="action_parent_process"/>
    <field name="model_id" ref="model_sale_order"/>
    <field name="state">code</field>
    <field name="sequence">20</field>
    <field name="code"><![CDATA[
records.action_confirm()
]]></field>
</record>
```

## P46: on_time_created / on_time_updated (v19)

```xml
<!-- Escalar pedidos sin confirmar 48h despues de creacion -->
<record id="rule_escalate_stale" model="base.automation">
    <field name="name">Escalar pedidos sin confirmar (48h)</field>
    <field name="model_id" ref="sale.model_sale_order"/>
    <field name="trigger">on_time_created</field>
    <field name="trg_date_range">48</field>
    <field name="trg_date_range_type">hours</field>
    <field name="filter_domain">[('state', '=', 'draft')]</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
for rec in records:
    rec.write({'priority': '1'})
    if rec.user_id:
        rec.activity_schedule(
            'mail.mail_activity_data_todo',
            summary='Pedido sin confirmar hace 48h: %s' % rec.name,
            user_id=rec.user_id.id,
        )
    log('Escalado: %s (creado hace 48h sin confirmar)' % rec.name)
]]></field>
</record>
```
