# Sintaxis de records en ficheros de datos

Fuente de verdad: `$ODOO_SRC/odoo18/odoo/tools/convert.py`. Las lineas citadas son de v18.

## Raices de fichero

Un fichero de datos arranca con una de estas raices (convert.py:649,
`DATA_ROOTS = ['odoo', 'data', 'openerp']`):

```xml
<odoo>            <!-- preferida -->
    ...
</odoo>
```

`<openerp>` es legado. `<data>` puede ser raiz o anidarse dentro de `<odoo>` para agrupar
records y aplicar atributos (`noupdate`, `auto_sequence`) a un bloque (convert.py:580-611).

Tags reconocidos en la raiz (`self._tags`, convert.py:635-644): `record`, `delete`, `function`,
`menuitem`, `template`, `asset`, y de nuevo las raices (`odoo`/`data`/`openerp`) para anidar.
Cualquier otro tag se ignora silenciosamente (convert.py:582-584).

## `<record model= id=>`

```xml
<record id="mi_xmlid" model="res.partner">
    <field name="name">Acme</field>
</record>
```

- `model`: nombre tecnico del modelo (`rec.get("model")`, convert.py:325).
- `id`: external id; si no lleva punto, se le antepone el modulo actual
  (`make_xml_id`, convert.py:221-224). Se valida con `_test_xml_id` (max un punto;
  el modulo referido debe estar instalado, convert.py:226-234).
- El record se materializa con `model._load_records([data], update)` (convert.py:444), pasando
  `dict(xml_id, values, noupdate)`.

## Formas de `<field>`

`_tag_record` itera los `<field>` hijos (convert.py:378-433). Formas, por prioridad de atributo:

### 1. `ref="otro.xmlid"`
Resuelve el external id a su id de BD (convert.py:402-410).
```xml
<field name="user_id" ref="base.user_admin"/>
```
- Si el campo es `reference` (no m2o), produce `"model,id"` (convert.py:403-405).
- Si el `ref` no resuelve y `forcecreate` no es True, se omite la creacion con warning
  (convert.py:408-410).

### 2. `search="[domain]"` (+ `model=`, `use=`)
Busca registros y asigna su id (convert.py:387-401). Requiere `model=` (assert,
convert.py:390).
```xml
<field name="partner_id" model="res.partner" search="[('name','=','Acme')]"/>
<field name="tag_ids" model="res.partner.category" search="[]"/>  <!-- m2m: todos -->
```
- Campo m2m: asigna `Command.set` con todos los resultados (convert.py:396-397).
- Otro: toma el primero (`s[0][f_use]`, convert.py:398-401). `use="..."` cambia el campo leido
  (por defecto `id`).

### 3. `eval="expresion"`
`safe_eval` con idref (convert.py:98-105, 177-179). Disponibles: `ref(...)`, `Command`,
`obj()`, `time`, `datetime`, `timedelta`, `relativedelta`, etc.
```xml
<field name="active" eval="True"/>
<field name="date" eval="(DateTime.today() + relativedelta(days=7))"/>
<field name="group_ids" eval="[Command.set([ref('base.group_user')])]"/>
<field name="company_id" eval="False"/>
```

### 4. Contenido inline con `type=`
`_eval_xml` segun `type` (convert.py:141-167, default `char`):

| `type` | Resultado |
|--------|-----------|
| `char` (default) | texto tal cual |
| `int` | `int(texto)`; `None` -> None |
| `float` | `float(texto)` |
| `xml` | serializa los hijos como XML (arch de vistas) |
| `html` | serializa los hijos como HTML |
| `list` | lista de `<value>` hijos |
| `tuple` | tupla de `<value>` hijos |
| `file` | `"modulo,ruta"` (valida existencia) |
| `base64` | SOLO con `file=` (sin file -> ValueError, convert.py:164-165) |

```xml
<field name="padding">3</field>                         <!-- char, casteado por el campo -->
<field name="priority" type="int">8</field>
<field name="domain" type="list">
    <value>('active','=',True)</value>
</field>
<field name="image" type="base64" file="mi_modulo/static/img/logo.png"/>
```

### Casteo por tipo de campo
Tras `_eval_xml`, si el campo existe, convert.py castea (convert.py:413-432):
m2o -> int, integer -> int, float/monetary -> float, boolean string -> bool,
one2many -> procesa `<record>` hijos via inverse_name (sub_records). Aviso si un campo `html` se
declara `type="xml"` (convert.py:430-432).

> **`type="xml"` = arch de vista.** Si tienes que **generar o parchear** ese arch por código
> (migración, script) en vez de escribirlo a mano, manipúlalo con **lxml**:
> `etree.fromstring(view.arch.encode('utf-8'))` (el `arch` es `str` → `encode`), `etree.strip_tags(root, 't')`
> para desenvolver `<t>`, `etree.strip_attributes(node, 'data-oe-id', 'data-oe-xpath')`,
> `parent.replace(viejo, nuevo)`. La herencia se valida al escribir la vista → envuelve cada `write`
> en `with cr.savepoint():`. Receta completa en `odoo-shell-scripting/references/recipes.md §7`.

### `%(xmlid)d` / `%(xmlid)s`
En contenido de texto y en `eval`, los patrones `%(xmlid)d`/`%(xmlid)s` se sustituyen por el id
numerico del external id (`_process`, convert.py:106-122). `%%` -> `%`.
```xml
<field name="domain">[('id','=',%(base.main_company)d)]</field>
```

## `<function model= name=>`
Llama un metodo del modelo durante la carga (convert.py:169-188, 257-261).
```xml
<function model="ir.config_parameter" name="set_param">
    <value>mi_modulo.clave</value>
    <value>valor</value>
</function>
```
- Args via `eval="[...]"` o `<value>` posicionales; kwargs via `<value name="...">`.
- Mezcla el context actual (convert.py:186).
- **Respeta noupdate:** no se ejecuta en update bajo noupdate (`if self.noupdate and
  self.mode != 'init': return`, convert.py:258).

## `<delete model= id=|search=>`
Borra registros (convert.py:236-255).
```xml
<delete model="res.partner" id="mi_modulo.viejo_partner"/>
<delete model="ir.rule" search="[('name','=','obsoleta')]"/>
```
- `id` ausente -> warning y se salta (no falla). `search` fallido -> warning.

## `<menuitem>`
Azucar que crea `ir.ui.menu` (convert.py:263-322). Atributos: `id`, `name`, `parent`,
`action`, `sequence`, `groups` (con `-xmlid` para desvincular), `web_icon`, `active`.
Anida `<menuitem>` hijos (parent automatico). Si hay `action` y no `name`, hereda el nombre de
la accion (convert.py:296-297).

## Diffs v17 -> v18 -> v19

- **v17**: `_eval_xml` usa cadenas if/elif para los tipos en vez de `match/case`; acepta los
  mismos `type`. `base64` se evalua tras leer `data`/`file` (no exige `file=` con el guard
  explicito de v18). El resto (`ref`/`search`/`eval`/`%()`/`<function>`/`<delete>`/`<menuitem>`)
  es identico (v17 convert.py:113-170, 236-).
- **v18**: introduce `match/case` en `_eval_xml`; `case 'base64'` sin `file=` lanza ValueError
  (convert.py:164-165). Tags y semantica de records sin cambios funcionales.
- **v19**: identico a v18 en tags y semantica de field (mismo `_tag_record`, `_tag_root`,
  `DATA_ROOTS`). Cambios de firma internos: `convert_file` deprecia el argumento `kind`
  (warning, convert.py v19:677-682) y se anaden type hints (`ConvertMode`, `IdRef`). No afecta
  al XML que escribes.
