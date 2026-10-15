# Registros de sistema tipicos

Plantillas y campos clave para los modelos que mas se siembran por XML. Confirma signaturas y
campos exactos por version con `odoo-version-api-reference`; aqui van los patrones probados de la
fuente (`$ODOO_SRC/odoo18/odoo/addons/base/data/` y addons).

## ir.cron

Ejemplo real: `base/data/ir_cron_data.xml`.
```xml
<record id="mi_cron" model="ir.cron">
    <field name="name">Mi Modulo: tarea periodica</field>
    <field name="model_id" ref="mi_modulo.model_mi_modelo"/>
    <field name="state">code</field>
    <field name="code">model._mi_metodo(batch_size=50)</field>
    <field name="interval_number">1</field>
    <field name="interval_type">days</field>     <!-- minutes/hours/days/weeks/months -->
    <field name="active" eval="True"/>
    <field name="priority">5</field>
    <!-- opcional: <field name="nextcall" eval="(DateTime.now()+timedelta(hours=1))"/> -->
</record>
```
Campos clave: `model_id` (ref al `model_<modelo>` que genera ir.model.access), `state`
(`code`), `code` (en su contexto `model`, `env`, `log` estan disponibles), `interval_number`,
`interval_type`, `nextcall`, `active`, `priority`. OJO: `numbercall` y `doall` fueron
ELIMINADOS en v18 (los sustituyo la cola `ir.cron.trigger`); solo existen en <=v17. El cron real lo dispara
`ir.cron._process_jobs`; si no ejecuta, revisa `active`, `nextcall` y que el cron base este
habilitado en la instancia.

NO suele ir bajo noupdate (es estructura del modulo). Si el cliente debe poder pausarlo desde
la UI sin que `-u` lo reactive, ponlo bajo `noupdate="1"`.

## ir.sequence

Ejemplo real: `crm_iap_mine/data/ir_sequence_data.xml` (bajo `<data noupdate="1">`).
```xml
<odoo>
    <data noupdate="1">
        <record id="seq_mi_doc" model="ir.sequence">
            <field name="name">Mi Documento</field>
            <field name="code">mi.modelo</field>
            <field name="prefix">MD/%(year)s/</field>
            <field name="padding">4</field>
            <field name="company_id" eval="False"/>   <!-- compartida entre companias -->
        </record>
    </data>
</odoo>
```
**Casi siempre noupdate="1":** el contador (`number_next`) lo modifica el uso en runtime; sin
noupdate un `-u` lo reiniciaria. Campos: `code` (clave logica para `_next`), `prefix`/`suffix`
(con placeholders `%(year)s`, `%(month)s`, `%(day)s`, etc.), `padding`, `company_id`.

## ir.config_parameter

Ejemplo real: `base/data/ir_config_parameter_data.xml` (bajo `<data noupdate="1">`).
```xml
<odoo>
    <data noupdate="1">
        <record id="param_x" model="ir.config_parameter">
            <field name="key">mi_modulo.mi_clave</field>
            <field name="value">10</field>
        </record>
    </data>
</odoo>
```
**Siempre noupdate="1":** son valores que el usuario ajusta. Alternativa imperativa con
`<function>` (solo en init bajo noupdate):
```xml
<function model="ir.config_parameter" name="set_param">
    <value>mi_modulo.mi_clave</value>
    <value>10</value>
</function>
```

## mail.template

Ejemplos: `*/data/mail_template_data.xml`. Suelen ir bajo `noupdate="1"` para no pisar ajustes
del cliente (a veces sin noupdate si el modulo es duenno del texto).
```xml
<record id="email_template_x" model="mail.template">
    <field name="name">Mi Notificacion</field>
    <field name="model_id" ref="mi_modulo.model_mi_modelo"/>
    <field name="subject">Pedido {{ object.name }}</field>
    <field name="email_from">{{ object.company_id.email_formatted }}</field>
    <field name="email_to">{{ object.partner_id.email_formatted }}</field>
    <field name="body_html" type="html">
        <div>Hola <t t-out="object.partner_id.name"/></div>
    </field>
    <field name="lang">{{ object.partner_id.lang }}</field>
    <field name="auto_delete" eval="True"/>
</record>
```
Cuerpo en `type="html"`; placeholders con sintaxis QWeb inline (`{{ }}` / `t-out`).

## ir.actions.* y res.groups / menus

Acciones de ventana, server actions y menus. Para `ir.actions.act_window`:
```xml
<record id="action_mi_modelo" model="ir.actions.act_window">
    <field name="name">Mis Registros</field>
    <field name="res_model">mi.modelo</field>
    <field name="view_mode">list,form</field>
    <field name="domain">[]</field>
    <field name="context">{}</field>
</record>
<menuitem id="menu_mi_modelo" name="Mis Registros"
          parent="base.menu_custom" action="action_mi_modelo" sequence="10"/>
```
Para `res.groups` y reglas de acceso:
```xml
<record id="group_mi_rol" model="res.groups">
    <field name="name">Mi Rol</field>
    <field name="category_id" ref="base.module_category_hidden"/>
    <field name="implied_ids" eval="[Command.link(ref('base.group_user'))]"/>
</record>
```
**Diff v19 res.groups:** `category_id` -> `privilege_id` (nuevo modelo `res.groups.privilege`);
los `_sql_constraints` (lista) pasan a `models.Constraint`.

Para **server actions / automatizaciones**, delega en `odoo-server-action-creator`.

## Diffs v17 -> v18 -> v19

- v18: `view_mode` ya no admite `tree`; usa `list` (el tag de vista paso de `<tree>` a
  `<list>`). En v17 todavia es `tree`. Confirma en `odoo-version-api-reference`.
- v18+: `groups_id`/`implied_ids` en res.groups usan `Command.*`; el patron es estable v17-v19.
- mail.template, ir.sequence, ir.config_parameter: estructura de campos estable
  v17->v19 (cambios menores de campos opcionales, no del patron de siembra).
- ir.cron: NO es estable. `numbercall` y `doall` existen en <=v17 pero fueron ELIMINADOS
  en v18 (sustituidos por la cola `ir.cron.trigger`); no los uses en plantillas v18/v19.
- El loader (`<record>`/`<menuitem>`/`<function>`) trata estos modelos igual que cualquier otro;
  no hay diffs de convert.py especificos para ellos.
