# Gotchas de Migracion a Studio

Errores reales descubiertos al migrar modulos a Studio. Verificados contra codigo fuente Odoo 18.

---

## Modelos y Campos

### 1. SQL views no se pueden extender
Modelos basados en SQL views (report models, analysis views) rechazan campos Studio. `table_kind() != 'Regular'` → UserError. **Detectar en Fase 1** y excluir del scope de migracion.

### 2. x_active DEBE llamarse x_active, no x_studio_active
El ORM tiene tratamiento especial para el campo `active` (archivado). Solo reconoce `active` o `x_active`. Si lo nombras `x_studio_active`, el archivado NO funciona.

### 3. M2M al mismo modelo = collision de tabla relacion
Crear 2 campos Many2many hacia el mismo target → tablas `x_model_x_target_rel` y `x_model_x_target_rel_1`. El orden de creacion determina cual es _1. Si recreas en otro orden → collision. **Crear siempre en el mismo orden.**

### 4. Binary crea campo filename companero silenciosamente
Crear `x_studio_doc` (Binary) → tambien crea `x_studio_doc_filename` (Char). Ambos necesitan gestion. Contar 2 campos por cada binary en el plan de migracion.

### 5. Eliminar opcion de Selection borra datos
Quitar un valor de un campo Selection → todos los registros con ese valor pasan a False. Sin warning en todos los casos. **Antes de modificar opciones, verificar cuantos registros usan cada valor.**

### 6. Related inmutable — no se puede cambiar despues
El atributo `related` de un campo no se puede modificar via UI una vez creado. Path equivocado = borrar campo y recrear (perdiendo vistas que lo referencian).

### 7. Computed fields: via API si, via drag-drop no
El controller `create_new_field()` acepta `compute` + `depends`. Pero el editor visual de Studio NO expone estos campos. Para crear campos computed hay que usar scripts/API. Documentar esto en la migracion.

---

## Vistas

### 8. XPath rotos se saltan SIN ERROR en produccion
Despues de un upgrade de modulo base, los XPath de Studio pueden dejar de matchear. La vista renderiza pero **la personalizacion desaparece silenciosamente**. Ningun log, ningun warning. **Post-migracion: verificar TODAS las vistas Studio tienen efecto visible.**

### 9. Priority 99 = siempre ultimo, sin excepcion
Las vistas Studio siempre se aplican despues de TODAS las extensiones de modulos. No hay forma de forzar otro orden. Si una extension de modulo necesita "ganar" a Studio → no se puede desde Studio.

### 10. Vista auto-generada se "congela" al primer edit
Al editar una vista auto-generada por primera vez, Studio la materializa (guarda el arch completo). A partir de ahi, cambios futuros en la logica de auto-generacion NO afectan a esta vista. Queda congelada.

### 10b. Los helpers de diff de Studio son EE-only — guardar con `hasattr`
`normalize()`, `_node_to_xpath()`, `_identify_node()` (los que generan la herencia xpath mínima por diff estructural) viven en **EE `web_studio/models/ir_ui_view.py`** (verificado: `normalize` :631, `_node_to_xpath` :1040, `_identify_node` :1074). En una instancia **solo CE no existen**. Si tu código de migración los invoca, guárdalo con `hasattr(self.env['ir.ui.view'], '_node_to_xpath')` antes de llamar, o revienta con AttributeError en clientes sin Enterprise.

---

## Automatizaciones

### 11. on_change es SINCRONO — bloquea la UI
A diferencia de on_write/on_create (post-commit), `on_change` ejecuta durante el evento de formulario. Codigo pesado = navegador congelado. **Nunca migrar logica compleja a on_change. Usar on_write en su lugar.**

### 12. Aprobaciones requieren mail.thread
`studio.approval.rule` necesita el chatter para notificaciones. Si el modelo no hereda mail.thread (opcion `use_mail`), las aprobaciones fallan silenciosamente. **Siempre activar use_mail si el modelo necesita aprobaciones.**

### 13. Webhooks necesitan _model + _id en el payload
El record_getter por defecto lee `payload.get('_model')` y `payload.get('_id')`. Si el sistema externo no envia ambos → la accion no encuentra el registro y se salta sin error.

### 14. Safe-eval NO tiene import — NADA
Sin `import`, sin `hasattr`, sin `getattr`, sin `setattr`, sin `__builtins__`. Si el codigo original del modulo usa CUALQUIER import (requests, json, os, re) → NO migrable a Server Action. Opciones: webhook action para llamadas externas, o mantener como modulo Python.

---

## Exportacion

### 15. studio=False en ir.model.data = NO se exporta
El error mas comun. Si al crear via script no se marca `studio=True` + `noupdate=False` + `module=studio_customization`, el elemento es invisible para el export. **SIEMPRE verificar con `studio.verify_all_exportable()` antes de exportar.**

### 16. Botones con ID numerico no son portables
`<button name="738" type="action"/>` funciona en la BD donde se creo pero falla en otra (el ID 738 no existe). **SIEMPRE usar XML ID**: `<button name="studio_customization.mi_accion" type="action"/>`. Usar `studio.fix_button_action_ids()` post-creacion.

### 17. Export NO incluye datos
`studio.export.wizard` exporta estructura (modelos, campos, vistas, automatizaciones). NO exporta registros (datos). Si hay datos maestros (stages, tags, selections) → deben crearse en data XML del modulo exportado o cargarse aparte.

---

## Performance

### 18. Studio NO auto-indexa campos M2O — y el codigo tampoco
Los campos creados por Studio tienen `index=False`. Cierto, pero **no es una desventaja de Studio**:
medido en v18 (`tools/bench-orm`, `TestIndices`), un `fields.Many2one` escrito en Python **tampoco**
crea indice — solo lo hace con `index=True` explicito. Lo mismo con `Char`.

Asi que esto no entra en la comparativa codigo-vs-Studio: entra en la lista de rendimiento de
cualquiera de los dos. Un M2O usado en busquedas/filtros sin indice = escaneo completo.
**Post-migracion (o post-desarrollo): crear los indices de los M2O criticos.**

### 19. Properties (JSON) sin indice GIN = scan completo
Campos Properties se almacenan como JSONB. Sin indice GIN, buscar por valor de propiedad recorre toda la tabla. Para filtros frecuentes, crear indice GIN o usar campos reales.

### 20. Compute code rompe si la API cambia (v19)
Los campos computed almacenan codigo Python como string. Si migras a v19: `expression.AND()` → `Domain() &`, `threading.current_thread().testing` → `modules.module.current_test`. **Auditar TODOS los campos computed con codigo Python almacenado.**
