# Studio Engine — Como Funciona por Dentro

Conocimiento critico del motor Studio que afecta directamente a la migracion.

## 1. Tres Capas de Customizacion

Studio opera en 3 capas independientes. Una migracion debe cubrir LAS TRES:

```
Capa 1: CAMPOS (ir.model + ir.model.fields)
  - Modelos runtime via studio_model_create() con state='manual'
  - Campos x_studio_* registrados en ir.model.data con studio=True
  - studio.mixin auto-genera los xml_id

Capa 2: VISTAS (ir.ui.view + XPath)
  - Studio views heredan de base: priority=99 (SIEMPRE ULTIMO)
  - XPath almacenado en vista separada: "Odoo Studio: {name} customization"
  - xml_id empieza con 'studio_customization'
  - En produccion: XPaths rotos se saltan SILENCIOSAMENTE (sin error)

Capa 3: ACCESO + AUTOMATIZACIONES
  - ir.model.access auto-generado: system=all, user=no_unlink
  - ir.rule multi-company auto-generado si use_company
  - base.automation envuelve server actions con triggers
  - studio.approval.rule para workflows de aprobacion
```

## 2. Creacion de Modelos: 13 Opciones Built-in

Al crear un modelo via Studio, estas opciones generan campos y configuracion automatica:

| Opcion | Campo Generado | Efecto |
|--------|---------------|--------|
| `use_mail` | hereda mail.thread + mail.activity.mixin | Chatter, actividades |
| `use_active` | `x_active` (Boolean, default=True) | Archivado |
| `use_sequence` | `x_studio_sequence` (Integer) | Drag-drop ordering |
| `use_responsible` | `x_studio_user_id` (M2O res.users) | Asignacion |
| `use_partner` | `x_studio_partner_id` + phone/email related | Cliente |
| `use_company` | `x_studio_company_id` + record rules | Multi-company |
| `use_notes` | `x_studio_notes` (Html) | Descripcion |
| `use_date` | `x_studio_date` (Date) | Fecha unica |
| `use_double_dates` | `x_studio_date_start/stop` (Datetime) | Rango fechas |
| `use_value` | `x_studio_value` (Monetary) + currency_id | Importe |
| `use_image` | `x_studio_image` (Binary) | Imagen |
| `use_stages` | modelo secundario + `x_studio_stage_id` | Workflow kanban |
| `use_tags` | modelo secundario + `x_studio_tag_ids` (M2M) | Etiquetas |

**Implicacion migracion:** Al migrar un modelo nuevo trivial, estas opciones lo cubren sin codigo (mail.thread + stages + tags = 3 checkboxes en Studio). Para logica no trivial, mantener como modulo gobernado (criterio en `odoo-classification-guide`): reducir codigo no es un fin en si mismo.

## 3. Vistas Auto-generadas

Studio genera hasta 8 vistas automaticamente segun las opciones del modelo:

| Vista | Condicion | Se genera si... |
|-------|-----------|----------------|
| List | Siempre | Siempre |
| Form | Siempre | Siempre (header + sheet + chatter) |
| Search | Siempre | Siempre (filtros + group by) |
| Calendar | `use_date` | Campo fecha existe |
| Gantt | `use_double_dates` | Rango fecha existe |
| Kanban | `use_stages` | Stage existe |
| Map | `use_partner` | Partner existe |
| Pivot + Graph | `use_value` | Monetary existe |

**Implicacion migracion:** Si el modulo origen tiene form+list+search custom, evaluar si las auto-generadas cubren el caso. Si lo trivial se cubre con vistas nativas, mejor; pero no sacrifiques logica no trivial por 'menos XPath': el criterio es codigo gobernado (`odoo-classification-guide`).

## 4. XPath Stack — la prioridad se CALCULA, 99 es solo el caso base

```
Vista base (primary, priority=16)
  └── Extension de modulo (inherit, priority=32)
      └── Studio customization (inherit, priority=max(hijas)*10, minimo 99)
```

**Verificado en fuente** (`web_studio/controllers/main.py:745-748`, `WebStudioController._create_studio_view`):

```python
priority = max(view.inherit_children_ids.mapped("priority"), default=0) * 10
default_prio = view._fields["priority"].default(view)
if priority <= default_prio:
    priority = 99
```

O sea: Studio **no pone 99 siempre**. Toma la mayor prioridad de las vistas que ya heredan de esa
base y la multiplica por diez; 99 es el valor de reserva cuando eso no supera el default. Con una
extension de modulo en 32, la vista Studio sale con **320**, no con 99. Y los informes usan otro
camino distinto: `priority = 9999999` (`web_studio/controllers/report.py:388`, `_get_and_write_studio_view`).

**Por que importa al migrar:** el mecanismo garantiza "ultimo *en el momento de crearla*", no
"ultimo para siempre". Si despues instalas un modulo cuya vista herede con una prioridad mayor, esa
gana. Buscar `priority` en las vistas de Studio y comparar es una comprobacion de treinta segundos
que evita un "mi personalizacion ha dejado de verse" sin causa aparente.

**Critico para migracion:**
- Las vistas Studio se aplican despues de las que existian **cuando se crearon**
- El orden no se elige a mano, pero tampoco es inmutable
- Si un modulo actualiza la vista base y el XPath Studio ya no matchea → **se salta silenciosamente** en produccion (sin error, sin warning)
- En modo Studio: el error SI se muestra al usuario

### 4b. Cómo Studio genera la herencia xpath (motor reutilizable, EE-only)

Cuando editas visualmente y Studio guarda, NO reescribe la vista entera: emite una **herencia xpath
mínima** que representa el diff estructural. Los helpers viven en **EE `web_studio/models/ir_ui_view.py`**:

- `normalize()` (`:631`) — normaliza el arch antes de comparar.
- `_node_to_xpath(node)` (`:1040`) — construye el XPath que ancla un nodo (por `name=` único cuando
  existe; si no, por posición).
- `_identify_node(node)` (`:1074`) — identifica el nodo equivalente en el arch base.

Junto a ellos, el **web editor** usa `web_editor.save()` / `replace_arch_section` para edición
**in-place** (patrón COW de website: se clona la vista al primer edit). Reutilizar `_node_to_xpath` es
la vía correcta para generar una herencia quirúrgica en un script de migración/diff en vez de copiar
la vista. **Son EE-only**: `hasattr(env['ir.ui.view'], '_node_to_xpath')` antes de invocar (gotcha 10b).
El branding por-nodo (`data-oe-id`/`data-oe-xpath`) que estos helpers explotan solo existe si la vista
se combinó con `inherit_branding=True`.

## 5. Automation Engine — Triggers y Limites

### Triggers Disponibles (14+)
| Categoria | Triggers |
|-----------|---------|
| Record | on_create, on_write, on_create_or_write, on_unlink, on_archive, on_unarchive |
| Campo | on_stage_set, on_user_set, on_tag_set, on_state_set, on_priority_set |
| Tiempo | on_time, on_time_created, on_time_updated |
| UI | on_change (form, SINCRONO — bloquea UI) |
| Mail | on_message_received, on_message_sent (solo mail.thread) |
| Externo | on_webhook (UUID unico, POST) |

### Safe-eval: Lo Que PUEDES y NO Puedes Hacer

```python
# DISPONIBLE en server actions:
env, model, record, records    # ORM
time, datetime, dateutil       # Fechas
UserError, ValidationError     # Excepciones
Command                        # API O2M/M2M
log                            # Logging

# NO DISPONIBLE:
import          # NINGUNO — ni requests, ni json, ni os
exec, eval      # Prohibidos
__builtins__    # Bloqueado
hasattr, getattr, setattr  # Bloqueados
```

**Implicacion migracion:** Si el codigo Python del modulo usa `import requests`, `import json`, `subprocess`, etc. → NO migrable a Server Action. Necesita quedarse como codigo o usar webhook action para llamadas externas.

## 6. Requisitos de Exportacion (ir.model.data)

Para que Studio exporte un elemento, DEBEN cumplirse los 3:

| Campo | Valor OBLIGATORIO | Si falta... |
|-------|-------------------|-------------|
| `module` | `studio_customization` | No se encuentra en export |
| `studio` | `True` | **NO se exporta** |
| `noupdate` | `False` | NO se exporta |

**Regla:** Despues de cada creacion via script, SIEMPRE verificar exportabilidad. Usar `studio.verify_all_exportable()` + `studio.fix_all_exportable()` antes de exportar.

## 7. Performance — Lo Que No Te Dicen

| Operacion | Impacto |
|-----------|---------|
| Crear modelo nuevo | Registry rebuild en todos los workers (100-500ms pausa) |
| Campo M2O sin indice | Full table scan en busquedas (Studio NO auto-indexa) |
| Muchos XPath en una vista | Compilacion secuencial, cada xpath anade latencia |
| Computed field store=False | Recalculado en cada lectura (N+1 si en lista) |
| Properties field en filtro | JSONB sin indice GIN = full scan |

**Implicacion migracion:** Al crear campos via script, anadir indices manualmente para M2O frecuentes:
```sql
CREATE INDEX IF NOT EXISTS idx_my_model_x_studio_partner
ON x_my_model (x_studio_partner_id) WHERE x_studio_partner_id IS NOT NULL;
```

## 8. Supervision de Aprobaciones

```
studio.approval.rule (definicion del paso)
  ├── method (que accion requiere aprobacion)
  ├── notification_order (paso 1-9, secuencial)
  └── approver_ids → studio.approval.rule.approver
      ├── user_id O group_id
      └── date_to (delegacion temporal)

studio.approval.entry (estado activo por registro)
  ├── rule_id + res_id
  └── approved / rejected / pending
```

**Requisito:** El modelo DEBE heredar mail.thread (`use_mail`). Sin chatter, las aprobaciones no pueden notificar.

## 9. Datos que Sobreviven Upgrades

| Que | Sobrevive? | Como |
|-----|-----------|------|
| ir.model (state='manual') | Si | Definicion en BD |
| ir.model.fields | Si | Columnas en tabla |
| ir.ui.view (studio_customization) | Si* | *Si el XPath sigue valido |
| base.automation | Si | Triggers en BD |
| Datos en tablas custom | Si | Datos puros SQL |
| Codigo Python en compute/SA | Si pero puede ROMPER | Si API cambia (v19: Domain class) |

## 10. Migracion Studio → Codigo (Camino Inverso)

Si una customizacion Studio crece demasiado, se puede exportar a modulo:

1. `studio.export.wizard` → genera zip con XML de todo
2. Instalar como modulo normal
3. Las customizaciones pasan a ser code-based
4. Version-controlable, testeable, reviewable

**Ojo:** El export NO incluye datos (registros). Solo estructura (modelos, campos, vistas, automatizaciones).
