# Gotchas de Server Actions - Errores Comunes

## G01: Asignacion directa de atributos (STORE_ATTR)
**Severidad: CRITICA** | **Frecuencia: MUY ALTA**

```python
# MAL - Lanza SIEMPRE excepcion (forbidden opcode)
record.name = 'Nuevo nombre'

# BIEN
record.write({'name': 'Nuevo nombre'})
```

**Por que falla**: `safe_eval` bloquea el opcode `STORE_ATTR` a nivel de bytecode. No es un error de permisos, es una restriccion del sandbox.

**Sintoma**: SIEMPRE excepcion `ValueError: forbidden opcode(s) ... STORE_ATTR` al evaluar. Nunca es silencioso.

---

## G02: Usar import (sintaxis diferente segun entorno)

**Severidad: CRITICA (estandar) / BAJA (con indaws_base)** | **Frecuencia: ALTA**

### Sin indaws_base (Odoo estandar):

```python
# MAL - Error inmediato
import re
import os
import requests
from collections import OrderedDict

# BIEN - Usar lo que ya esta disponible
# datetime, dateutil, timezone, json, time ya estan inyectados
today = datetime.date.today()
data = json.loads('{"key": "value"}')
```

**Workaround para `re`**: Sin alternativa directa. Opciones:
1. Usar `str.replace()`, `str.split()`, `str.startswith()`, `str.endswith()`, `str.find()`
2. Mover la logica a un metodo del modelo y llamarlo desde la server action

**Workaround para `requests`**: Usar webhook o crear un metodo en el modelo.

### Con indaws_base instalado:

```python
# MAL - import sigue bloqueado a nivel de opcode
import re  # ERROR: opcode IMPORT_NAME not allowed

# BIEN - Usar wrap_module o import_module
re = wrap_module('re', ['findall', 'sub', 'match', 'search', 'split', 'compile'])
requests = import_module('requests', ['get', 'post', 'put', 'delete'])
csv = wrap_module('csv', ['reader', 'writer', 'DictReader', 'DictWriter'])

# El statement "import X" sigue prohibido
# La diferencia es que ahora tienes funciones para importar
```

**IMPORTANTE**: Aunque `indaws_base` permite importar, la sintaxis `import X` sigue bloqueada.
Siempre usar `wrap_module()` o `import_module()`.

---

## G03: Usar _() para traducciones

**Severidad: MEDIA** | **Frecuencia: ALTA**

```python
# MAL - NameError: name '_' is not defined
raise UserError(_("No se puede procesar"))

# BIEN
raise UserError("No se puede procesar")
```

**Por que**: La funcion `_()` (alias de `odoo._`) no se inyecta en el eval context. Los textos en server actions no son traducibles via `.po` files.

---

## G04: Usar ValidationError o AccessError

**Severidad: MEDIA** | **Frecuencia: ALTA**

```python
# MAL - NameError
raise ValidationError("Campo obligatorio")
raise AccessError("Sin permisos")

# BIEN - Solo UserError disponible
raise UserError("Campo obligatorio")
raise UserError("Sin permisos")
```

**Por que**: Solo `UserError` se inyecta en el contexto. `ValidationError` y `AccessError` no estan disponibles.

---

## G05: record vacio en acciones planificadas (cron)

**Severidad: ALTA** | **Frecuencia: MEDIA**

```python
# MAL - Error si es cron (record esta vacio)
partner_name = record.partner_id.name  # Error o False

# BIEN - Verificar o usar model/env directamente
if record:
    partner_name = record.partner_id.name
else:
    # Es un cron, buscar registros directamente
    records_to_process = model.search([('state', '=', 'pending')])
```

**Por que**: En acciones planificadas (cron), `record` y `records` son recordsets vacios. No hay `active_id` ni `active_ids` en el contexto.

---

## G06: Olvidar que record puede ser multi-registro

**Severidad: MEDIA** | **Frecuencia: MEDIA**

```python
# MAL - Funciona con 1 registro, falla con multiples
partner = record.partner_id  # Si records tiene 2+, cual partner?

# BIEN - Iterar siempre
for rec in records:
    partner = rec.partner_id
    # ... logica por registro
```

**Nota**: En Odoo 18, `record` en multi-mode es `None`. Usar siempre `records` y iterar.

---

## G07: No retornar action correctamente

**Severidad: MEDIA** | **Frecuencia: MEDIA**

```python
# MAL - return no funciona
return {
    'type': 'ir.actions.act_window',
    'name': 'Test',
    'res_model': 'res.partner',
    'view_mode': 'list',
}

# BIEN - Asignar a variable 'action'
action = {
    'type': 'ir.actions.act_window',
    'name': 'Test',
    'res_model': 'res.partner',
    'view_mode': 'list',
}
```

**Por que**: `safe_eval` ejecuta el codigo en modo `exec`. El retorno se obtiene leyendo la variable `action` del eval_context despues de la ejecucion.

---

## G08: Acceder a atributos __dunder__

**Severidad: CRITICA** | **Frecuencia: BAJA**

```python
# MAL - Bloqueado por safe_eval
record.__class__.__name__
env.__dict__
record._name.__len__  # Esto si funciona, pero __len__ como atributo no

# BIEN
model_name = record._name  # _name si funciona (single underscore)
```

**Por que**: `safe_eval` bloquea cualquier acceso a atributos que contengan `__`. Los atributos de Odoo con single underscore (`_name`, `_fields`, `_table`) SI funcionan.

---

## G09: N+1 queries en loops

**Severidad: MEDIA** | **Frecuencia: ALTA**

```python
# MAL - 1 query por registro + 1 por partner + 1 write
for rec in records:
    if rec.partner_id.country_id.code == 'ES':
        rec.write({'is_spanish': True})

# MEJOR - Prefetch y batch
spanish = records.filtered(
    lambda r: r.partner_id.country_id.code == 'ES'
)
spanish.write({'is_spanish': True})
```

**Nota**: En server actions es especialmente critico porque no hay mecanismo de profiling. Los problemas de rendimiento son dificiles de detectar.

---

## G10: Usar cr.commit() o cr.rollback()

**Severidad: CRITICA** | **Frecuencia: BAJA**

```python
# MAL - Nunca gestionar transacciones manualmente
env.cr.commit()
env.cr.rollback()

# BIEN - Dejar que Odoo gestione la transaccion
# Si necesitas savepoints:
env.cr.execute("SAVEPOINT my_savepoint")
# ... operaciones ...
env.cr.execute("ROLLBACK TO SAVEPOINT my_savepoint")
```

**Por que**: Las server actions se ejecutan dentro de una transaccion gestionada por Odoo. Un `commit()` prematuro puede dejar datos inconsistentes si la accion falla despues.

---

## G11: Olvidar sudo() para modelos restringidos

**Severidad: MEDIA** | **Frecuencia: MEDIA**

```python
# MAL - El usuario puede no tener acceso
param = env['ir.config_parameter'].get_param('key')
cron = env['ir.cron'].create({...})

# BIEN
param = env['ir.config_parameter'].sudo().get_param('key')
cron = env['ir.cron'].sudo().create({...})
```

**Por que**: La server action se ejecuta con los permisos del usuario que la dispara. Si es un usuario de portal o con permisos limitados, las operaciones sobre modelos tecnicos fallan.

---

## G12: Asumir que json esta siempre disponible

**Severidad: BAJA (estandar) / NO APLICA (con indaws_base)** | **Frecuencia: MEDIA**

### Sin indaws_base:
```python
# PUEDE FALLAR si base_automation no esta instalado
data = json.loads(record.raw_data)
```

**Por que**: `json` se inyecta desde el modulo `base_automation` (que viene con `mail`). En una instalacion minima sin `mail`, `json` no estaria disponible.

### Con indaws_base:
```python
# SIEMPRE FUNCIONA - indaws_base inyecta json directamente
data = json.loads(record.raw_data)
```

**Por que**: `indaws_base` inyecta `json` directamente en `_get_eval_context()`, sin depender de `mail` ni `base_automation`.

---

## G13: Definir funciones con def

**Severidad: BAJA** | **Frecuencia: MEDIA**

```python
# FUNCIONA - pero con limitaciones
def calculate_discount(amount, percent):
    return amount * percent / 100

for rec in records:
    discount = calculate_discount(rec.amount_total, 10)
    rec.write({'discount_amount': discount})
```

**Nota**: Las funciones `def` SI funcionan en server actions (el opcode no esta bloqueado). Sin embargo:
- No pueden usar `import` dentro
- No tienen acceso a `self`
- Son utiles para evitar repeticion de logica dentro de la misma server action
- Las lambdas tambien funcionan: `records.filtered(lambda r: r.state == 'draft')`

---

## G14: Asumir que print() funciona

**Severidad: BAJA** | **Frecuencia: MEDIA**

```python
# MAL - print no esta disponible
print("Debug:", record.name)

# BIEN
log("Debug: %s" % record.name)
_logger.info("Debug: %s", record.name)
```

---

## G15: Tuplas clasicas x2many vs Command

**Severidad: MEDIA** | **Frecuencia: MEDIA**

```python
# FUNCIONA pero obsoleto y propenso a errores
record.write({'tag_ids': [(4, tag_id, 0)]})
record.write({'line_ids': [(0, 0, {'name': 'test'})]})

# MEJOR - Mas legible y menos errores
record.write({'tag_ids': [Command.link(tag_id)]})
record.write({'line_ids': [Command.create({'name': 'test'})]})
```

**Por que**: `Command` esta en el eval context explicitamente. Las tuplas funcionan pero son menos legibles y el tercer elemento causa confusion (ej: `(4, id, 0)` vs `(4, id)` vs `(4, id, False)`).

---

## G16: No verificar record.exists() despues de operaciones

**Severidad: BAJA** | **Frecuencia: BAJA**

```python
# MAL - Si el registro fue eliminado por otra accion en cadena
record.write({'state': 'done'})  # MissingError

# BIEN - En acciones encadenadas o multi
for rec in records.exists():
    rec.write({'state': 'done'})
```

---

## G17: Usar self en server action

**Severidad: CRITICA** | **Frecuencia: ALTA**

```python
# MAL - self es la ir.actions.server, NO el registro
self.name  # Esto devuelve el nombre de la SERVER ACTION, no del registro

# BIEN
record.name  # Esto devuelve el nombre del registro
records.mapped('name')  # Nombres de todos los registros
```

**TRAMPA**: `self` tecnicamente existe en el eval context (es la accion de servidor). No da error, pero devuelve datos equivocados. Este es probablemente el error mas confuso.

---

## G18: Strings multilinea en el campo code

**Severidad: BAJA** | **Frecuencia: BAJA**

```python
# CUIDADO con indentacion - el campo code no tiene indentacion base
# Todo el codigo debe empezar en columna 0

# MAL (si copias de un metodo Python)
    for rec in records:
        rec.write({'state': 'done'})

# BIEN
for rec in records:
    rec.write({'state': 'done'})
```

**Por que**: Al copiar codigo de un metodo de modelo, se arrastra la indentacion del metodo. En server actions, el codigo empieza en columna 0.

---

## G19: SQL injection en cr.execute

**Severidad: CRITICA** | **Frecuencia: BAJA**

```python
# MAL - SQL injection
env.cr.execute("SELECT id FROM res_partner WHERE name = '%s'" % name)

# BIEN - Parametros seguros
env.cr.execute("SELECT id FROM res_partner WHERE name = %s", [name])
```

---

## G20: Olvidar que env.context es inmutable (frozendict)

**Severidad: MEDIA** | **Frecuencia: MEDIA**

```python
# MAL - No se puede modificar el contexto directamente
env.context['key'] = 'value'  # frozendict, error

# BIEN - Crear nuevo entorno con contexto
new_env = env(context=dict(env.context, key='value'))
# O usar with_context en un recordset
record.with_context(key='value').some_method()
```

---

## GOTCHAS ESPECIFICOS DE indaws_base

---

## G21: Confundir wrap_module con import_module

**Severidad: MEDIA** | **Frecuencia: ALTA** | **Solo con indaws_base**

```python
# wrap_module: devuelve un wrapper que SOLO expone los metodos listados
re = wrap_module('re', ['findall', 'sub'])
re.match(...)  # ERROR - 'match' no fue listado

# import_module: devuelve el modulo real completo
re = import_module('re', ['findall', 'sub'])
re.match(...)  # FUNCIONA - tiene todo

# REGLA: wrap_module es mas seguro, import_module es mas flexible
```

**Cuando usar cada uno:**
- `wrap_module` -> Cuando quieres control estricto sobre que se expone (recomendado)
- `import_module` -> Cuando necesitas el modulo completo con todas sus clases/funciones

---

## G22: Usar open() en rutas peligrosas

**Severidad: CRITICA** | **Frecuencia: BAJA** | **Solo con indaws_base**

```python
# MAL - Rutas de sistema, ficheros sensibles
content = open('/etc/passwd', 'r').read()
content = open('$ODOO_SRC/odoo.conf', 'r').read()

# BIEN - Solo rutas seguras
with open('/tmp/export_data.csv', 'w') as f:
    f.write('col1,col2\n')

# BIEN - Directorio de datos de Odoo
data_dir = env['ir.config_parameter'].sudo().get_param('ir_attachment.location', '/tmp')
```

**Regla**: Solo usar `open()` en `/tmp/` o directorios de datos controlados. Nunca en rutas de sistema o configuracion.

---

## G23: No cerrar ficheros abiertos con open()

**Severidad: MEDIA** | **Frecuencia: MEDIA** | **Solo con indaws_base**

```python
# MAL - Fichero queda abierto si hay excepcion
f = open('/tmp/data.csv', 'r')
content = f.read()
f.close()  # No se ejecuta si falla la linea anterior

# BIEN - Context manager (with) cierra automaticamente
with open('/tmp/data.csv', 'r') as f:
    content = f.read()
```

---

## G24: wrap_module con metodos inexistentes

**Severidad: BAJA** | **Frecuencia: MEDIA** | **Solo con indaws_base**

```python
# MAL - Listar metodos que no existen en el modulo
re = wrap_module('re', ['findall', 'ejecutar', 'buscar'])  # ejecutar/buscar no existen

# BIEN - Solo metodos reales del modulo
re = wrap_module('re', ['findall', 'search', 'match', 'sub', 'split', 'compile'])
```

**Tip**: Si no estas seguro de los metodos, usa `import_module` que da acceso completo.

---

## G25: Olvidar que import_module necesita fromlist

**Severidad: BAJA** | **Frecuencia: MEDIA** | **Solo con indaws_base**

```python
# El segundo parametro (methods) se pasa como fromlist a __import__
# Para submodulos, esto es CRITICO

# MAL - Sin fromlist, __import__ devuelve el paquete padre
xml = import_module('xml.etree.ElementTree', [])
# xml es el modulo 'xml', NO 'xml.etree.ElementTree'

# BIEN - Con fromlist, devuelve el submodulo correcto
ET = import_module('xml.etree.ElementTree', ['parse', 'fromstring', 'Element'])
# ET es xml.etree.ElementTree correctamente
```

---

## GOTCHAS ESPECIFICOS DE ODOO 19

---

## G26: Usar `groups_id` en vez de `group_ids` (v19)

**Severidad: ALTA** | **Frecuencia: ALTA** | **Solo v19**

```xml
<!-- MAL en v19 - campo renombrado -->
<field name="groups_id" eval="[(4, ref('base.group_system'))]"/>

<!-- BIEN en v19 -->
<field name="group_ids" eval="[(4, ref('base.group_system'))]"/>

<!-- BIEN en v18 (sigue siendo groups_id) -->
<field name="groups_id" eval="[(4, ref('base.group_system'))]"/>
```

**Por que**: v19 renombra `groups_id` a `group_ids` en `ir.actions.server` (y en `ir.rule`).
Este cambio es consistente con el renombrado general de `groups_id` -> `group_ids` en v19.

**Afecta tambien a base.automation**: record rules que referencien grupos.

---

## G27: child_ids como M2M en v19

**Severidad: MEDIA** | **Frecuencia: BAJA** | **Solo v19**

```xml
<!-- MAL en v19 - child_ids ya NO es Many2many -->
<field name="child_ids" eval="[(6, 0, [ref('action_1'), ref('action_2')])]"/>

<!-- BIEN en v19 - Cada child tiene parent_id -->
<record id="action_child_1" model="ir.actions.server">
    <field name="name">Child Action 1</field>
    <field name="parent_id" ref="action_parent"/>
    <field name="model_id" ref="model_sale_order"/>
    <field name="state">code</field>
    <field name="code">...</field>
</record>
```

**Por que**: v19 cambio `child_ids` de Many2many (tabla `rel_server_actions`) a One2many con campo `parent_id`.
El XML que creaba relaciones M2M ya no funciona. Cada accion hija debe declarar su `parent_id`.

---

## G28: Server actions con warnings no se ejecutan (v19)

**Severidad: ALTA** | **Frecuencia: MEDIA** | **Solo v19**

```python
# En v19, si una server action tiene warnings, lanza excepcion al ejecutarse
# ServerActionWithWarningsError (hereda de UserError)

# Causas comunes de warnings:
# - Multi-action con child que tiene modelo diferente al padre
# - Webhook que intenta enviar campos con restriccion de grupo
# - Recursion en cadena de multi-actions
# - update_path con campo que no existe o no es navegable
```

**Verificacion**:
```python
# Comprobar si una accion tiene warnings antes de ejecutarla
action = env['ir.actions.server'].browse(action_id)
if action.warning:
    log('WARNING: %s' % action.warning)
```

**Por que**: v19 introduce un sistema de validacion pre-ejecucion. Verifica en
Settings > Technical > Server Actions que el campo `warning` esta vacio.
Si aparece un warning, la accion NO se ejecutara hasta que se corrija.

---

## G29: `with` prohibido en safe_eval — usar savepoints SQL crudos
**Severidad: ALTA** | **Frecuencia: MEDIA (bucles resilientes)**

El bytecode del `with` (`BEFORE_WITH`/`SETUP_WITH`) NO esta en los opcodes permitidos de `safe_eval`. Cualquier `with env.cr.savepoint():` revienta con `ValueError: forbidden opcode(s)`. Para aislar fallos en un bucle (que un registro corrupto no aborte todo), usar savepoints SQL crudos:

```python
fallidos = []
for rec in registros:
    env.cr.execute("SAVEPOINT sp")
    try:
        rec.some_method()
        env.cr.execute("RELEASE SAVEPOINT sp")
    except Exception as e:                      # Exception SI esta en _BUILTINS
        env.cr.execute("ROLLBACK TO SAVEPOINT sp")
        fallidos.append((rec.id, str(e).split(chr(10))[0]))
```

`Exception` esta permitido (`safe_eval._BUILTINS`); `ValidationError`/`UserError` genericas NO se inyectan salvo `UserError` (si esta en el eval context). Captura siempre con `Exception`.

---

## G30: DRY-RUN en server action via `raise UserError` (rollback de la peticion)
**Severidad: INFO (patron util)** | **Frecuencia: ALTA en saneamientos masivos**

Una server action confirma su transaccion al terminar bien. Para un modo "simulacro" que ejecute todo y NO persista, construye el resumen y lanza `raise UserError(resumen)` al final: el manejador de peticiones de Odoo hace **rollback de toda la transaccion** y muestra el texto en el dialogo. Para el run real, pon un flag `DRY_RUN=False` y, en vez de lanzar, devuelve una notificacion:

```python
if DRY_RUN:
    raise UserError("[SIMULACRO]\n" + resumen)   # revierte + muestra
else:
    action = {'type':'ir.actions.client','tag':'display_notification',
              'params':{'title':'OK','message':resumen,'sticky':True,'type':'success'}}
```

OJO: `log()` abre su PROPIO cursor y commitea aparte — sus entradas en `ir.logging` SI persisten aunque el resto haga rollback.

---

## G31: Timeout HTTP en server actions con bucles largos
**Severidad: ALTA** | **Frecuencia: MEDIA (saneamientos de miles de registros)**

Una server action lanzada desde el boton corre en un worker HTTP con `limit_time_real` (~120s). Un bucle sobre miles de registros (reasignar movimientos, recalcular, etc.) puede cortarse y hacer rollback de todo. Para operaciones masivas, crea en su lugar una **Accion Planificada (ir.cron)** con el mismo codigo y usa "Ejecutar manualmente": el worker de cron usa `limit_time_real_cron`, mucho mas alto. Mismo codigo, sin limite de peticion web.

---

## G32: Validacion estatica del sandbox != seguridad en runtime — envuelve en try/except
**Severidad: ALTA** | **Frecuencia: MEDIA (codigo que evalua expresiones o toca registros variables)**

Que `safe_eval` acepte el bytecode de tu codigo (whitelist de opcodes AST) NO significa que no vaya a
petar en ejecucion. Una expresion perfectamente valida a nivel estatico revienta en runtime:
`1/0` (ZeroDivisionError), `record.line_ids[0]` con 0 lineas (IndexError), `int(record.x_texto)` con
texto no numerico (ValueError), un metodo sobre un recordset vacio, una clave inexistente en un dict.

El sandbox de `safe_eval` bloquea el ACCESO peligroso (imports, dunder, STORE_ATTR — ver G01/G08), no
los errores de logica. Si tu server action evalua una expresion dinamica (o corre sobre registros cuyo
estado no controlas), **envuelve la evaluacion en `try/except`** y decide el fallback (saltar el
registro, dejar el valor vacio, loguear) en vez de dejar que la excepcion tumbe toda la accion (y haga
rollback de lo ya hecho):

```python
for record in records:
    try:
        record.x_resultado = record.x_base / record.x_divisor
    except Exception:
        record.x_resultado = 0.0   # fallback explicito, no rollback global
```
Mismo principio en informes QWeb (`odoo-qweb-report-builder` §11) y al diseniar el modelo de expresiones
(`odoo-security-designer`): la whitelist AST es condicion necesaria, no suficiente.
