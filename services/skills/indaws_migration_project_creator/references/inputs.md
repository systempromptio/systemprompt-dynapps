# Inputs detallados — migration-project-creator

Lista exhaustiva de inputs que el skill pide al usuario, en que orden, con que validaciones y como autocompletar lo que se pueda.

---

## Inputs OBLIGATORIOS (Paso 0)

### 1. `project_id` (int)

ID del proyecto Odoo. Debe existir y estar en etapa "Migraciones".

**Si el usuario no lo sabe**: ofrecer buscar por nombre cliente:
```python
projects = odoo_search_read('project.project',
    [['stage_id.name', '=', 'Migraciones']],
    ['id', 'name', 'partner_id'], limit=20)
```
Mostrar la lista y pedir que elija.

### 2. `date_start` (YYYY-MM-DD)

Fecha de kick-off tecnico (NO de Gate 0 — esa es el dia 0). Normalmente es el lunes despues de que el cliente firma el alcance.

**Validacion**: debe ser fecha futura. Si es pasada, avisar y pedir confirmacion (puede que sea una migracion ya en curso).

### 3. `date_golive` (YYYY-MM-DD)

Fecha objetivo de Go-Live. **DEBE ser posterior a `date_start` con al menos 6 semanas de margen** (Art. 2602 establece minimos por fase).

**Validacion**:
- `date_golive > date_start`
- `(date_golive - date_start).days >= 42` (6 semanas). Si menor, avisar y pedir confirmacion explicita.
- **Si `hosting_type = odoo-sh`:** `date_golive` debe ser **anterior** a la fecha en que la version de ORIGEN sale de
  Odoo.sh (deadline duro; consultar el calendario de ciclo de vida de versiones de Odoo). Si el Go-Live cae
  despues, avisar: el entorno dejaria de poder alojarse en Odoo.sh antes de terminar la migracion.

### 4. `strategy` (selector)

Una de las 3 estrategias:

| Valor | Descripcion | Cuando usar |
|---|---|---|
| `1to1` | Migrar cada modulo custom tal cual | Default si el cliente no quiere refactor. Migracion mas rapida pero mantiene deuda tecnica. |
| `refactoring` | Reducir deuda por modulo, reclasificando con `odoo-classification-guide` (Standard > Core > Studio > Custom). Sin tarea aparte; cada modulo se migra segun su estrategia desde la tarea de modulos. | Cliente acepta la reclasificacion. Migracion mas saludable. |
| `refactoring-no-studio` | Refactoring usando solo Standard + Core (sin Studio) | Cliente no quiere depender de Studio. |

**Como ayudar al usuario a decidir**: si el cliente migra a v18 o v19 y tiene muchos modulos custom (>15), recomendar `refactoring` o `refactoring-no-studio`. Si la migracion es urgente, recomendar `1to1`.

### 5. `hosting_type` (selector)

| Valor | Descripcion |
|---|---|
| `cloud-indaws` | Servidores propios de Indaws (default). Checklists completos de infra. |
| `odoo-sh` | Plataforma Odoo.sh. Checklists adaptados (S1-S7) en lugar de los de servidor propio. |
| `on-premise` | Servidores del cliente. Mismos checklists que cloud-indaws. |

### 6. `analysis_path` (path)

Ruta a un fichero .md o .json con el inventario de modulos custom.

**Formato esperado (.md)**:
```markdown
| Modulo | Estrategia | Notas | Horas | Riesgo | Dependencias |
|---|---|---|---|---|---|
| custom_sale_order | Custom | Solo modelo sale.order | 12h | Medio | sale |
| partner_extended_info | Studio | Campos custom a Studio | 8h | Bajo | base |
```

**Formato esperado (.json)**:
```json
[
  {"module": "custom_sale_order", "strategy": "Custom", "hours": 12, "risk": "Medio", "dependencies": "sale"},
  {"module": "partner_extended_info", "strategy": "Studio", "hours": 8, "risk": "Bajo", "dependencies": "base"}
]
```

**Si el usuario no tiene el inventario**: NO continuar. Pedir que ejecute `odoo-database-analyzer` o `odoo-module-upgrader` para generarlo. El analisis es prerequisito.

**Si el usuario tiene el inventario en Knowledge** (articulo del proyecto): leerlo via `odoo-pilot`, extraerlo a `/tmp/inventario_migracion.md` y usar esa ruta.

---

## Inputs AUTOCOMPLETADOS (Paso 2)

El skill sugiere valores y el usuario confirma o ajusta.

### `project_size` (selector)

Inferencia automatica:

```python
n_modules = len(modules)
total_hours = sum(m['hours'] for m in modules)

if n_modules < 10 and total_hours < 100:
    size = 'small'
elif n_modules > 25 or total_hours > 250:
    size = 'large'
else:
    size = 'medium'
```

Mostrar sugerencia: `"Detectados {n} modulos con {h}h. Sugiero tamaño: {size}. ¿OK?"`

### Horas por tarea (sugerencias)

Tabla base por tamaño (el usuario ajusta cada una si quiere):

| Hora | small | medium | large |
|---|---|---|---|
| `--analysis-hours` | 8 | 12 | 20 |
| `--db-hours` | 20 | 30 | 50 |
| `--testing-hours` | 20 | 30 | 50 |
| `--infra-hours` | 4 | 6 | 10 |
| `--validation-hours` | 6 | 8 | 12 |
| `--golive-hours` | 6 | 8 | 12 |
| `--support-hours` | 16 | 20 | 30 |

**Las horas de `[MIG] Migracion de modulos`** no se piden por flag — son la suma automatica de las horas del inventario (columna "Horas" de cada modulo). Si hay cotizacion, esas horas por modulo ya van en el inventario.

**Factor IA:** las horas sugeridas de la tabla son **base, sin asistencia IA**. Aplicar los
factores antes de fijar `allocated_hours`: delega en la skill `odoo-effort-estimator`. Si el
proyecto viene cotizado, las horas ya lo llevan aplicado y **no se factoriza dos veces**.

**Importante:** la tabla de arriba son SUGERENCIAS por tamaño. Si el proyecto esta **cotizado** y ya conoces las horas reales de cada fase (BD, testing, etc.), usa esas cifras en lugar de las sugeridas — el skill las pasa tal cual al motor y la tarea se crea con esas `allocated_hours`.

---

## Inputs OPCIONALES

### `--extra-flows` (string)

Flujos extra de testing especificos del cliente. Separados por coma:

```
"Calculo de precio por escalado de cantidad,Aprobacion de pedido >5000eur,Sync con TPV externo"
```

Si el usuario no aporta, se usan solo los 10 default.

### `--client-name` (string)

Solo para el HTML descriptivo de la tarea Go-Live. Si no se pasa, se lee del `partner_id` del proyecto.

### `--no-analysis-task` (flag)

NO crear la tarea `[MIG] Analisis y clasificacion`. Util si el analisis ya esta hecho (las horas estan consumidas) y solo queremos crear desde Fase 2 en adelante.

**Pregunta al usuario en Paso 0**: "¿La fase de Analisis ya esta completada? Si si, no se crea la tarea." Si dice si, anadir `--no-analysis-task`.

---

## Inputs NO se piden — se leen automaticamente

| Input | De donde se lee |
|---|---|
| `client_name` | `project.partner_id.name` |
| `from_version` y `to_version` | Si el proyecto tiene campos custom `x_from_version` y `x_to_version`, leerlos. Si no, preguntar. |
| `commercial_partner_id` | `project.partner_id.commercial_partner_id` (fallback al propio partner_id si esta vacio) |
| `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY` | Variables de entorno (skill `onboarding` o config local) |
| Version de Odoo | Si el proyecto tiene `x_studio_odoo_version`, leerlo. Si no, preguntar (default: leer URL → Odoo.sh sabe la version). |

---

## Resumen — Cuestionario mental para el skill

Para una migracion estandar el skill SOLO necesita preguntar 4 cosas si todo lo demas se autocompleta:

1. ID del proyecto
2. Fecha kick-off + Fecha Go-Live
3. Estrategia (1to1 / refactoring / refactoring-no-studio)
4. Ruta al inventario de modulos

El resto: hosting (default cloud-indaws), tamaño (inferido), horas (sugeridas), version Odoo (leida), partner (leido).
