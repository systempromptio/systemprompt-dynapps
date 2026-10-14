# Cambios en Server Actions: v18 vs v19

## Resumen Ejecutivo

> **El codigo Python que escribes DENTRO de una server action es casi identico entre v18 y v19.**
> Las diferencias estan en: como DEFINES la accion (XML) y cambios en nombres de campos del modelo `ir.actions.server`.
>
> **Lo REALMENTE nuevo en v19** (verificado contra fuente): estado `object_copy`, campo `html_value`,
> `evaluation_type='sequence'`, excepcion `ServerActionWithWarningsError`, modelo `ir.actions.server.history`,
> renombrado `groups_id`->`group_ids` + `child_ids` M2M->O2M con `parent_id`, y eliminacion de `DEFAULT_PYTHON_CODE`.
> NO son novedad v19 (ya en v18): los triggers archive/unarchive/message/webhook/time_*, y los campos de object_write
> `update_m2m_operation`, `update_boolean_value` y el traversal de `update_path`.

---

## 1. Cambios en safe_eval (interno - NO afecta al codigo del usuario)

| Aspecto | v18 | v19 |
|---------|-----|-----|
| Firma | `safe_eval(expr, globals_dict, locals_dict, mode, nocopy, ...)` | `safe_eval(expr, /, context=None, *, mode, filename)` |
| Parametro context | `globals_dict` + `locals_dict` separados | Un solo `context` dict |
| `nocopy` | Existia | **Eliminado** (siempre muta el context) |
| Builtins | 25 funciones | **Identicos** - mismos 25 |
| Opcodes bloqueados | 7 opcodes | **Identicos** - mismos 7 |
| Atributos inseguros | 16 atributos | **Identicos** - mismos 16 |
| Modulos wrapped | time, datetime, dateutil, json, pytz | **Identicos** |

**Conclusion**: El sandbox es el mismo. Lo que puedes/no puedes hacer en codigo es identico.

---

## 2. Cambios en el Eval Context (variables disponibles)

| Variable | v18 | v19 | Impacto en codigo |
|----------|-----|-----|-------------------|
| `env` | Si | Si | Ninguno |
| `model` | Si | Si | Ninguno |
| `record` | Si | Si | Ninguno |
| `records` | Si | Si | Ninguno |
| `uid` | `self._uid` | `self.env.uid` | **Ninguno** (valor identico) |
| `user` | Si | Si | Ninguno |
| `time/datetime/dateutil` | Si | Si | Ninguno |
| `timezone` | Si | Si | Ninguno |
| `float_compare` | Si | Si | Ninguno |
| `b64encode/b64decode` | Si | Si | Ninguno |
| `Command` | Si | Si | Ninguno |
| `UserError` | Si | Si | Ninguno |
| `log()` | Si | Si | Ninguno |
| `_logger` | Si | Si | Ninguno |
| `json` | Via base_automation | Via base_automation | Ninguno |
| `payload` | Via base_automation | Via base_automation | Ninguno |

**Conclusion**: Las variables disponibles dentro del codigo son identicas.

---

## 3. Cambios en la Definicion XML de Server Actions

### 3.1 Campo `groups_id` renombrado a `group_ids`

```xml
<!-- v18 -->
<field name="groups_id" eval="[(4, ref('base.group_system'))]"/>

<!-- v19 -->
<field name="group_ids" eval="[(4, ref('base.group_system'))]"/>
```

### 3.2 Campo `child_ids` cambia de M2M a O2M

```xml
<!-- v18 - Many2many via tabla rel_server_actions -->
<field name="child_ids" eval="[(6, 0, [ref('action_1'), ref('action_2')])]"/>

<!-- v19 - One2many con parent_id -->
<!-- Las child actions ahora tienen parent_id apuntando a la accion padre -->
<record id="action_child_1" model="ir.actions.server">
    <field name="parent_id" ref="action_parent"/>
    ...
</record>
```

### 3.3 Nuevo estado `object_copy`

```xml
<!-- v19 - Nuevo: duplicar registro -->
<record id="action_copy" model="ir.actions.server">
    <field name="name">Duplicar Pedido</field>
    <field name="model_id" ref="model_sale_order"/>
    <field name="state">object_copy</field>
    <field name="resource_ref">sale.order,</field>
    <!-- Opcional: vincular copia via relacion -->
</record>
```

### 3.4 Campo `name` ahora es computed

En v19, `name` es `compute='_compute_name', store=True, readonly=False`.
Se auto-genera basado en el `state` si no se especifica manualmente.

### 3.5 DEFAULT_PYTHON_CODE eliminado

En v18, el campo `code` tenia un valor por defecto con el template de ayuda.
En v19, el campo `code` no tiene valor por defecto.

---

## 4. Triggers en base.automation (NO son novedad v19)

> Verificado contra `base_automation/models/base_automation.py:127-143`, `BaseAutomation.trigger` (v18): TODOS estos triggers ya existen en v18.
> NO son nuevos en v19. El cuadro se mantiene como referencia de triggers disponibles.

### Triggers disponibles (v18 y v19)

| Trigger | Descripcion |
|---------|-------------|
| `on_create` | Al crear registro |
| `on_write` | Al modificar registro |
| `on_create_or_write` | Al crear o modificar |
| `on_unlink` | Al eliminar registro |
| `on_change` | Al cambiar campo (UI) |
| `on_time` | Condicion temporal |
| `on_stage_set` | Al entrar en etapa |
| `on_tag_set` | Al agregar tag |
| `on_state_set` | Al cambiar a estado |
| `on_priority_set` | Al cambiar prioridad |
| `on_user_set` | Al asignar usuario |

### Triggers tambien presentes en v18 (NO nuevos en v19)

| Trigger | Descripcion | record/records |
|---------|-------------|----------------|
| `on_archive` | Al archivar registro | Registro archivado |
| `on_unarchive` | Al desarchivar registro | Registro desarchivado |
| `on_message_received` | Al recibir mensaje (mail) | Registro del hilo |
| `on_message_sent` | Al enviar mensaje (mail) | Registro del hilo |
| `on_time_created` | Temporal desde fecha creacion | Registros que cumplen filtro |
| `on_time_updated` | Temporal desde fecha modificacion | Registros que cumplen filtro |
| `on_webhook` | Al recibir webhook HTTP | Registro identificado por payload |

### Ejemplo base.automation v19 con nuevos triggers

```xml
<!-- Trigger al archivar -->
<record id="rule_on_archive" model="base.automation">
    <field name="name">Al archivar pedido</field>
    <field name="model_id" ref="sale.model_sale_order"/>
    <field name="trigger">on_archive</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
for rec in records:
    rec.message_post(body='<p>Pedido archivado por %s</p>' % user.name)
]]></field>
</record>

<!-- Trigger al recibir mensaje -->
<record id="rule_on_message" model="base.automation">
    <field name="name">Al recibir email en pedido</field>
    <field name="model_id" ref="sale.model_sale_order"/>
    <field name="trigger">on_message_received</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
for rec in records:
    if rec.state == 'draft':
        rec.write({'priority': '1'})
        log('Pedido %s priorizado por mensaje recibido' % rec.name)
]]></field>
</record>

<!-- Webhook trigger -->
<record id="rule_webhook" model="base.automation">
    <field name="name">Webhook externo</field>
    <field name="model_id" ref="sale.model_sale_order"/>
    <field name="trigger">on_webhook</field>
    <field name="state">code</field>
    <field name="code"><![CDATA[
# payload contiene los datos del webhook
if payload:
    order_ref = payload.get('reference')
    if order_ref:
        order = model.search([('name', '=', order_ref)], limit=1)
        if order:
            order.write({'note': json.dumps(payload)})
]]></field>
</record>
```

---

## 5. Sistema de Validacion y Warnings (NUEVO en v19)

v19 introduce un sistema de validacion que IMPIDE ejecutar server actions con problemas:

### Exception nueva: `ServerActionWithWarningsError`

```python
class ServerActionWithWarningsError(UserError):
    """ Raised when a server action that has warnings is run. """
    pass
```

### Validaciones que bloquean ejecucion

| Validacion | Descripcion |
|-----------|-------------|
| Modelo hijo incompatible | Child actions en multi-mode con modelo diferente al padre |
| Campos con grupo en webhook | Webhook que intenta enviar campos con restriccion de grupo |
| Recursion en multi-actions | Cadena circular de acciones hijas |
| Path de campo invalido | Campo relacional que no existe o no es navegable |
| Campo no-relacional en path | Intentar navegar a traves de un campo que no es relacion |

### Impacto en desarrollo

Si creas acciones via XML y la validacion detecta un problema, la accion NO se ejecutara
y lanzara `ServerActionWithWarningsError`. Verificar en Settings > Technical > Server Actions
que el campo `warning` esta vacio.

---

## 6. Campos de object_write (mayoria YA en v18)

> Verificado contra `ir_actions.py` v18: `update_m2m_operation`, `update_boolean_value` y el
> traversal de `update_path` YA EXISTEN en v18. NO son novedad v19. Solo es realmente nuevo en
> v19 el `evaluation_type='sequence'` (6.5) y `html_value` (6.2).

### 6.1 Operaciones Many2many (ya en v18)

```xml
<!-- update_m2m_operation: ya existe en v18 (ir_actions.py:592-597) -->
<record id="action_add_tags" model="ir.actions.server">
    <field name="state">object_write</field>
    <field name="update_path">tag_ids</field>
    <field name="update_m2m_operation">add</field>
    <field name="selection_value" ref="crm.tag_vip"/>
</record>
```

Operaciones: `set`, `add`, `remove`, `clear`

### 6.2 HTML value (realmente nuevo en v19)

```xml
<!-- v19 - Nuevo campo html_value -->
<record id="action_set_comment" model="ir.actions.server">
    <field name="state">object_write</field>
    <field name="update_path">comment</field>
    <field name="value_field_to_show">html_value</field>
    <field name="html_value"><![CDATA[<p>Procesado automaticamente</p>]]></field>
</record>
```

### 6.3 Field path traversal (ya en v18)

```xml
<!-- Navegar relaciones en update_path: ya existe en v18 (ir_actions.py:589, :692-734) -->
<record id="action_update_partner_country" model="ir.actions.server">
    <field name="state">object_write</field>
    <field name="update_path">partner_id.country_id</field>
    <field name="evaluation_type">value</field>
    <field name="resource_ref">res.country,73</field>
</record>
```

### 6.4 Boolean updates (ya en v18)

```xml
<!-- update_boolean_value: ya existe en v18 (ir_actions.py:598) -->
<record id="action_toggle_active" model="ir.actions.server">
    <field name="state">object_write</field>
    <field name="update_path">active</field>
    <field name="update_boolean_value">false</field>
</record>
```

### 6.5 Sequence (realmente nuevo en v19)

```xml
<!-- v19 - evaluation_type='sequence': generar secuencia -->
<record id="action_set_ref" model="ir.actions.server">
    <field name="state">object_write</field>
    <field name="update_path">ref</field>
    <field name="evaluation_type">sequence</field>
    <field name="sequence_id" ref="ir_sequence_custom"/>
</record>
```

---

## 7. Code History (NUEVO en v19)

### Modelo `ir.actions.server.history`

v19 trackea automaticamente los cambios en el campo `code` de las server actions.
Permite ver el historial de versiones del codigo.

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `server_action_id` | Many2one | Accion de servidor padre |
| `code` | Text | Codigo de esta version |
| `create_date` | Datetime | Cuando se guardo |
| `create_uid` | Many2one | Quien lo guardo |

### Impacto en desarrollo

- El historial se limpia automaticamente (garbage collection)
- El campo `show_code_history` indica si hay historial disponible
- Metodo `history_wizard_action()` abre el wizard de historial

---

## 8. Tabla Resumen de Diferencias

| Aspecto | v18 | v19 | Afecta al codigo Python? |
|---------|-----|-----|--------------------------|
| Eval context variables | 15+ vars | 15+ vars (identicas) | **NO** |
| Builtins safe_eval | 25 | 25 (identicos) | **NO** |
| Opcodes bloqueados | 7 | 7 (identicos) | **NO** |
| `groups_id` en XML | `groups_id` | `group_ids` | NO (solo XML) |
| `child_ids` en XML | M2M | O2M con parent_id | NO (solo XML) |
| Estado `object_copy` | No existe | **Nuevo v19** | NO (sin codigo) |
| Triggers base.automation | 18 (incl. archive/message/webhook/time_*) | 18 (identicos) | NO (solo definicion) |
| Validacion/Warnings (`ServerActionWithWarningsError`) | No existe | **Nuevo v19** | NO (pero bloquea ejecucion) |
| Code History (`ir.actions.server.history`) | No existe | **Nuevo v19** | NO |
| DEFAULT_PYTHON_CODE | Template en campo | **Eliminado v19** | NO |
| M2M operations (`update_m2m_operation`) | Ya existe | Ya existe | NO (sin codigo) |
| HTML value (`html_value`) | No | **Nuevo v19** | NO (sin codigo) |
| Field path traversal (`update_path`) | Ya existe | Ya existe | NO (sin codigo) |
| Boolean update (`update_boolean_value`) | Ya existe | Ya existe | NO (sin codigo) |
| Sequence evaluation (`evaluation_type='sequence'`) | No | **Nuevo v19** | NO (sin codigo) |

---

## 9. Checklist: Migrar Server Action de v18 a v19

### En el codigo Python (campo `code`)
- [ ] **Nada que cambiar** - El codigo es compatible tal cual

### En la definicion XML
- [ ] Renombrar `groups_id` a `group_ids`
- [ ] Si usa `child_ids` M2M: migrar a O2M con `parent_id`
- [ ] Verificar que no hay warnings en la accion (nuevo sistema validacion)
- [ ] (Opcional) Aprovechar nuevos triggers si aplican

### En base.automation XML
- [ ] Renombrar `groups_id` a `group_ids` en record rules referenciadas
- [ ] (Opcional) Usar nuevos triggers: `on_archive`, `on_unarchive`, `on_message_received`, etc.
- [ ] (Opcional) Configurar webhooks con `on_webhook`
