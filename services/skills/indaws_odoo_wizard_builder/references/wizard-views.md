# Vista del wizard y campos temporales

## Vista form con footer

La vista de un wizard es un `<form>` cuyos botones van en `<footer>`. Patrón real
(`account/wizard/account_move_reversal_view.xml:8`, `view_account_move_reversal`):

```xml
<record id="view_account_move_reversal" model="ir.ui.view">
    <field name="name">account.move.reversal.form</field>
    <field name="model">account.move.reversal</field>
    <field name="arch" type="xml">
        <form string="Reverse Journal Entry">
            <group>
                <field name="reason"/>
                <field name="journal_id"/>
                <field name="date"/>
            </group>
            <footer>
                <button string="Reverse" name="refund_moves" type="object" class="btn-primary" data-hotkey="q"/>
                <button string="Reverse and Create Invoice" name="modify_moves" type="object"
                        class="btn-secondary" invisible="move_type == 'entry'"/>
                <button string="Discard" class="btn-secondary" special="cancel" data-hotkey="x"/>
            </footer>
        </form>
    </field>
</record>
```

Claves:
- Botones de acción: `name="metodo"` + `type="object"` → llama al método del wizard.
- Botón cancelar/cerrar: `special="cancel"` (NO necesita método; cierra el modal).
- `class="btn-primary"` para el botón principal, `btn-secondary` el resto.
- `data-hotkey` da atajos de teclado (UX).
- `invisible="..."` evalúa expresiones sobre los campos del propio wizard.

La acción debe abrir esta vista con `target="new"` (ver `context-and-defaults.md`).

## Visibilidad por state (multi-paso patrón A)

```xml
<form>
    <group invisible="state != 'step1'">
        <field name="source_ids"/>
    </group>
    <group invisible="state != 'step2'">
        <field name="confirm_note"/>
    </group>
    <field name="state" invisible="1"/>
    <footer>
        <button string="Next" name="action_next" type="object"
                class="btn-primary" invisible="state != 'step1'"/>
        <button string="Apply" name="action_apply" type="object"
                class="btn-primary" invisible="state != 'step2'"/>
        <button string="Cancel" special="cancel" class="btn-secondary"/>
    </footer>
</form>
```

El campo `state` se incluye con `invisible="1"` para que esté disponible en el
contexto de evaluación de los `invisible` de los demás campos/botones.

## Campos en el wizard

Tipos habituales y consideraciones (delega la definición fina a `odoo-orm-modeler`):

- **Many2one a un modelo NORMAL**: correcto y habitual (ej. `partner_id`,
  `journal_id`). El wizard apunta a registros reales.
- **Many2many / One2many temporales**: para listas de líneas que el usuario
  edita en el popup. El m2m a modelos reales (`move_ids` en el ejemplo) es la
  forma típica de arrastrar la selección de origen.
- **Binary / adjuntos**: para subir/descargar ficheros (import/export). Usa
  `fields.Binary` + `fields.Char` para el nombre del fichero.
- **Reference**: cuando el wizard debe apuntar a un registro de modelo variable.
- **PROHIBIDO**: un `Many2one` desde un modelo NORMAL hacia este transient (FK a
  tabla que el vacuum vacía). Al revés (transient → normal) sí.

> Un One2many del wizard hacia otro TransientModel "línea" funciona para editar
> líneas en el popup, pero ambos son efímeros: no los persistas.
