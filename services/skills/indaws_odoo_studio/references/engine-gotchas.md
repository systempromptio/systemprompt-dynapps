# Studio Engine — Lo Que Necesitas Saber

Conocimiento interno de Studio que evita errores al crear campos, modelos y vistas.

## 1. Que Puedes y No Puedes Personalizar

### SI puedes
- Anadir campos a CUALQUIER modelo con state='base' (modelos estandar de Odoo)
- Crear modelos nuevos (state='manual') con hasta 13 opciones built-in
- Heredar vistas con XPath
- Crear automated actions con 14+ tipos de trigger
- Crear workflows de aprobacion (si el modelo tiene chatter)

### NO puedes
- Anadir campos a SQL views (modelos report/analysis) — error directo
- Anadir chatter (mail.thread) a modelos base existentes — solo a modelos custom
- Override de metodos Python (create, write, unlink) — solo automations
- Campos computed via UI visual (si via API/script)
- Modificar campo `related` una vez creado — hay que borrar y recrear

## 2. Opciones al Crear Modelo (13 Built-in)

Al crear un modelo nuevo, estas opciones generan campos + configuracion automatica:

| Opcion | Que genera | Efecto |
|--------|-----------|--------|
| `use_mail` | hereda mail.thread | Chatter, actividades, aprobaciones |
| `use_active` | `x_active` (Boolean) | Archivado (OJO: no x_studio_active) |
| `use_sequence` | `x_studio_sequence` | Drag-drop en lista |
| `use_responsible` | `x_studio_user_id` | Asignacion a usuario |
| `use_partner` | `x_studio_partner_id` + phone/email | Relacion cliente |
| `use_company` | `x_studio_company_id` + record rules | Multi-company |
| `use_notes` | `x_studio_notes` (Html) | Descripcion |
| `use_date` | `x_studio_date` | Fecha + vista Calendar |
| `use_double_dates` | `x_studio_date_start/stop` | Rango + vista Gantt |
| `use_value` | `x_studio_value` (Monetary) | Importe + vistas Pivot/Graph |
| `use_image` | `x_studio_image` | Imagen del registro |
| `use_stages` | modelo secundario + stage_id | Kanban con workflow |
| `use_tags` | modelo secundario + tag_ids | Categorias color |

**Consejo:** Usar estas opciones genera vistas automaticas (form, list, search, calendar, gantt, kanban, map, pivot). Menos XPath manual = menos mantenimiento.

## 3. Gotchas que Evitan Errores

### Campos

**G1. SQL views rechazan campos** — Modelos de reporting (analysis, report) basados en SQL views NO aceptan campos custom. Error: `table_kind() != 'Regular'`. Antes de crear un campo, verificar que el modelo es una tabla real.

**G2. Binary crea 2 campos** — Al crear campo Binary `x_studio_doc`, Odoo tambien crea `x_studio_doc_filename` (Char). Contar 2 campos por binary.

**G3. Selection: eliminar opcion = borrar datos** — Si eliminas un valor de un campo Selection, TODOS los registros con ese valor pasan a vacio (False). Sin confirmacion en todos los casos. Verificar cuantos registros usan el valor ANTES de eliminarlo.

**G4. M2M al mismo target: colision de tabla** — 2 campos Many2many apuntando al mismo modelo crean tablas `_rel` y `_rel_1`. El orden de creacion importa. Si recreas en otro orden, colision.

**G5. x_active, NO x_studio_active** — El ORM solo reconoce `active` o `x_active` para el mecanismo de archivado. Si nombras el campo `x_studio_active`, el archivado NO funciona.

**G6. Monetary necesita currency_id ANTES** — El campo currency_id debe existir en el modelo ANTES de crear un campo Monetary. Si no existe, crearlos en orden: primero currency M2O, luego Monetary.

### Vistas

**G7. XPath rotos se saltan sin error** — Si actualizas un modulo y la vista base cambia, los XPath de Studio pueden dejar de matchear. La vista renderiza pero la personalizacion **desaparece silenciosamente**. Verificar visualmente despues de upgrades.

**G8. Studio views siempre ultimas (priority=99)** — Las personalizaciones Studio se aplican DESPUES de todas las extensiones de modulos. No hay forma de cambiar este orden.

**G9. Tab/pagina nueva: se "congela" al editar** — Al crear o editar una pagina por primera vez, Studio materializa la vista (guarda el arch completo). Cambios futuros en la auto-generacion no afectan.

### Exportacion

**G10. 3 campos OBLIGATORIOS en ir.model.data** — Para que Studio exporte un elemento:
- `module = 'studio_customization'` (exacto)
- `studio = True` (CRITICO — sin esto, invisible para export)
- `noupdate = False` (si True, no exporta)

**G11. Botones con ID numerico no portables** — `<button name="738" type="action"/>` solo funciona en la BD donde se creo. Usar XML ID siempre.

### Performance

**G12. Studio NO crea indices automaticamente** — Campos M2O creados via Studio tienen `index=False`. En tablas grandes, buscar por ese campo = full table scan. Para M2O frecuentes en filtros, crear indice SQL manualmente.

**G13. Properties sin indice GIN** — Campos Properties (JSON) sin indice = scan completo en busquedas. Para filtros frecuentes sobre properties, mejor usar campos reales.

## 4. Requisitos de Exportacion

```
Antes de exportar customizaciones Studio, verificar:

1. TODOS los elementos tienen studio=True en ir.model.data
2. TODOS tienen noupdate=False
3. TODOS tienen module='studio_customization'
4. Botones de vistas usan XML ID, no ID numerico
5. Accesos (ir.model.access) tienen grupo asignado (global no exporta)
```

## 5. Automated Actions — Limites

### Triggers disponibles
- Record: on_create, on_write, on_unlink, on_archive, on_unarchive
- Campo: on_stage_set, on_user_set, on_tag_set, on_state_set, on_priority_set
- Tiempo: on_time, on_time_created, on_time_updated
- UI: on_change (**sincrono — bloquea UI**, evitar para logica pesada)
- Mail: on_message_received, on_message_sent (solo modelos con chatter)
- Externo: on_webhook

### Safe-eval: sin import
```python
# Disponible: env, model, record, records, time, datetime, dateutil, log, UserError, Command
# NO disponible: import, hasattr, getattr, setattr, exec, eval, __builtins__
```

Si necesitas llamar una API externa → usar accion tipo "webhook" (POST a URL), no codigo Python.
