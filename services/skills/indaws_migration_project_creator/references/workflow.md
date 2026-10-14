# Workflow detallado — migration-project-creator

Skill que configura un proyecto de migracion Odoo creando 18 tareas (8 mig/infra + 8 gates + 2 gestion), checklists y Knowledge tree, delegando la creacion de tareas al motor de `migration-assistant`.

---

## Paso 0 — Recoger inputs (STOP)

Pedir al usuario, en este orden. **TODOS obligatorios — sin valores por defecto silenciosos.**

1. **`project_id`** Odoo (entero). Sin este input, abortar.

2. **`date_start`** YYYY-MM-DD — fecha de **inicio del proyecto** (kick-off tecnico, primer dia de Fase 1).
   - Pregunta literal al usuario: "¿En que fecha arranca el proyecto?"
   - Validacion: fecha valida, no en el pasado lejano (si lo es, confirmar explicitamente).

3. **`date_golive`** YYYY-MM-DD — fecha objetivo de **salida a produccion**.
   - Pregunta literal: "¿Cuando quieres que sea el Go-Live?"
   - Validacion: `date_golive > date_start` y `(date_golive - date_start).days >= 42` (minimo 6 semanas). Si menor, avisar y pedir confirmacion explicita.
   - Sugerencias si el usuario no sabe:
     - small (<10 modulos): 8-10 semanas
     - medium (10-25): 14-18 semanas
     - large (>25): 20-30 semanas

   **Estas 2 fechas son la BASE del skill. Sin ellas no se pueden repartir las 18 tareas en el calendario. NO continuar sin las dos.**

4. **`strategy`** — una de:
   - `1to1` (default si dudas) — migrar cada modulo custom tal cual.
   - `refactoring` — reducir deuda tecnica por modulo, reclasificando con `odoo-classification-guide` (escalera Standard > Core > Studio > Custom). No crea tarea aparte; cada modulo se migra segun su estrategia desde la tarea de modulos.
   - `refactoring-no-studio` — refactoring sin Studio.

5. **`hosting_type`** — `cloud-indaws` / `odoo-sh` / `on-premise`. Afecta los checklists de `[INFRA]`.

6. **Ruta al inventario de modulos** (.md o .json). Formato esperado:

   ```
   | Modulo | Estrategia | Notas | Horas | Riesgo | Dependencias |
   |---|---|---|---|---|---|
   | custom_sale_order | Custom | ... | 12h | Medio | sale |
   ```

   Estrategia por modulo: `Standard` / `Studio` / `Core` / `Custom`.

Validacion: si falta cualquiera de los 6 inputs, NO continuar. Pedirlos uno a uno hasta tenerlos todos.

---

## Paso 1 — Leer proyecto Odoo

Via `odoo-pilot` MCP tools:

```python
project = odoo_read('project.project', [project_id], [
    'name', 'partner_id', 'stage_id', 'task_count',
    'x_studio_knowledge', 'user_id',
])
```

Validaciones:

1. **Existe**: si no, abortar con mensaje claro.
2. **Etapa "Migraciones"**: leer `project.project.stage.name`. Si no es exactamente "Migraciones", avisar y pedir confirmacion al usuario (puede ser que la etapa se llame distinto en su instancia).
3. **`task_count > 0`**: preguntar al usuario:
   - **Anadir** (recomendado si las tareas existentes son parciales) → continuar sin borrar
   - **Borrar y recrear** (limpia y empieza de cero) → pedir confirmacion EXTRA
   - **Cancelar** → abortar

Leer `commercial_partner_id` del partner del proyecto:

```python
partner = odoo_read('res.partner', [project['partner_id'][0]], ['commercial_partner_id', 'name'])
commercial_partner_id = partner['commercial_partner_id'][0] if partner.get('commercial_partner_id') else partner['id']
```

---

## Paso 2 — Procesar inventario y autocompletar

### 2a. Parsear el inventario

El usuario puede entregar el inventario en dos formatos:

- **Markdown**: tabla `| Modulo | Estrategia | Notas | Horas | Riesgo | Dependencias |`
- **JSON**: lista de dicts `[{"module": "...", "strategy": "...", "hours": N, ...}]`

El script `create_migration_tasks.py` ya hace el parsing — solo necesitamos pasarle la ruta con `--analysis`.

Para mostrar el resumen al usuario en el Paso 3, leer el archivo y contar modulos por estrategia (`Standard`, `Studio`, `Core`, `Custom`) y sumar horas.

### 2b. Inferir `project_size`

Sugerencia automatica (el usuario confirma):

| Criterio | Tamaño |
|---|---|
| <10 modulos custom | `small` |
| 10-25 modulos custom | `medium` |
| >25 modulos custom | `large` |

Tamaño BD tambien influye (si el usuario lo sabe): >50GB sube un nivel.

### 2c. Sugerir horas

Ver `./inputs.md` para la tabla completa de sugerencias por tamaño + estrategia. Defaults razonables:

| Hora | small | medium | large |
|---|---|---|---|
| `--analysis-hours` | 8 | 12 | 20 |
| `--db-hours` | 20 | 30 | 50 |
| `--testing-hours` | 20 | 30 | 50 |
| `--infra-hours` | 4 | 6 | 10 |
| `--validation-hours` | 6 | 8 | 12 |
| `--golive-hours` | 6 | 8 | 12 |
| `--support-hours` | 16 | 20 | 30 |

### 2d. Calcular fechas

El script ya tiene `compute_phase_dates(date_start, date_golive, project_size)`. Solo se pasan `--date-start` y `--date-golive` y el script lo hace.

Para mostrar al usuario en Paso 3, ejecutar el script en `--dry-run` y capturar las fechas de fase del stderr.

---

## Paso 3 — Resumen completo (STOP)

Mostrar al usuario:

### Tabla de tareas a crear

```
| # | Tarea | Fase | Horas | Inicio | Fin | Checklist | Tag |
|---|-------|------|-------|--------|-----|-----------|-----|
| 1 | [MIG] Analisis y clasificacion de modulos | 1 | 8h | 2026-06-15 | 2026-06-27 | 0 | Migration |
| 2 | [INFRA] Despliegue de servidor | 1 | 6h | 2026-06-15 | 2026-06-27 | 0 | Migration |
| 3 | [MIG] Migracion de modulos | 2 | 72h | 2026-06-27 | 2026-08-03 | 6 | Migration |
| ... |
| 18 | Reuniones | — | 0h | 2026-06-15 | 2026-10-29 | 0 | (ninguno) |
```

### Estructura Knowledge

```
{Nombre del proyecto}  ← raiz
├── Analisis Tecnico         (inventario de modulos, decisiones de estrategia)
├── Scripts BD               (pre-migrate, post-migrate, log de iteraciones)
├── Ensayo                   (rehearsal, tiempos medidos, plan de rollback)
├── Go-Live                  (planning, comunicaciones, retro post-Go-Live)
├── Retrospectiva            (lecciones aprendidas, mejoras para futuras migraciones)
└── Interno                  (notas internas Indaws, no visibles al cliente)
```

### Totales

- **Horas totales estimadas**: XXh
- **Duracion**: X semanas (date_start → date_golive + 2 sem soporte)
- **Modulos**: N (X Standard / Y Studio / Z Core / W Custom)

### Esperar OK del usuario

Solo continuar si el usuario aprueba explicitamente. Si pide cambios, ajustar los inputs y volver a presentar.

---

## Paso 4 — Crear en Odoo

Orden estricto:

### 4a. Resolver/crear tag Migration

Lo hace el script. No hace falta nada por nuestra parte.

### 4b. Asegurar campo `x_studio_knowledge`

```python
field = odoo_search_read('ir.model.fields', 
    [['model', '=', 'project.project'], ['name', '=', 'x_studio_knowledge']], 
    ['id'], limit=1)
if not field:
    # Avisar al usuario que falta el campo. NO crearlo automaticamente.
    abort("El campo x_studio_knowledge no existe. Pide al admin que lo cree o usa Studio.")
```

### 4c. Crear Knowledge tree

Ver `./estructura_knowledge.md` para detalle. Search-before-create para idempotencia:

```python
# 1. Articulo raiz
root_id = ensure_article(name=project['name'], parent_id=None, icon="🔄", 
                          body=build_root_body(project, dates))
# 2. Hijos (6)
for child in ['Analisis Tecnico', 'Scripts BD', 'Ensayo', 'Go-Live', 'Retrospectiva']:
    ensure_article(name=child, parent_id=root_id, icon=ICON[child])
# 3. Interno (con is_desynchronized=true)
ensure_article(name='Interno', parent_id=root_id, icon="🔒", 
                is_desynchronized=True)
```

### 4d. Vincular etapas estandar al proyecto

11 etapas standard:

```
stage_ids = [10, 574, 737, 1008, 11, 23, 26, 25, 35, 12, 37]
for sid in stage_ids:
    odoo_write('project.task.type', [sid], {'project_ids': [[4, project_id]]})
```

### 4e. Invocar `create_migration_tasks.py`

Localiza el script del skill **migration-assistant** (suite common) en tu entorno y sustituye `{MIGRATION_ASSISTANT}` por esa ruta — NUNCA hardcodees una ruta absoluta de una máquina concreta.

**Carril interno** (`--profile internal`, default):
```bash
python3 {MIGRATION_ASSISTANT}/scripts/create_migration_tasks.py \
  --analysis {analysis_path} \
  --url {ODOO_URL} --db {ODOO_DB} --user {ODOO_USER} --key {ODOO_KEY} \
  --project-id {project_id} --version {odoo_version} \
  --client-name "{client_name}" \
  --from-version {from_v} --to-version {to_v} \
  --profile internal --strategy {strategy} \
  --date-start {date_start} --date-golive {date_golive} \
  --project-size {size} --hosting-type {hosting} \
  --analysis-hours {ah} --db-hours {dh} --testing-hours {th} \
  --infra-hours {ih} --validation-hours {vh} \
  --golive-hours {gh} --support-hours {suph} \
  --with-checklists --with-gates
```

**Carril Upgrade Lab**: ver `./references/upgrade_lab.md` (usa `--profile upgrade-lab` y solo `--analysis-hours --db-hours --fix-hours`; fuerza 1:1, sin infra/Studio/testing).

**NO pasar `--no-management`**: las 2 tareas de gestion (Gestion del proyecto + Reuniones) se crean SIEMPRE por defecto, cubriendo todo el rango date_start → date_golive + 14d.

El stdout es JSON con `[{id, name, type, hours, phase, url}, ...]`. Guardarlo para el resumen.

### 4f. Vincular articulo raiz al proyecto

```python
odoo_write('project.project', [project_id], {'x_studio_knowledge': root_id})
```

### 4g. Compartir con el cliente (portal)

```python
# Buscar si el partner ya es miembro
existing = odoo_search('knowledge.article.member',
    [['article_id', '=', root_id], ['partner_id', '=', commercial_partner_id]])
if not existing:
    odoo_create('knowledge.article.member', {
        'article_id': root_id,
        'partner_id': commercial_partner_id,
        'permission': 'read',
    })
```

---

## Paso 5 — Resumen final

Generar segun `../templates/resumen_creacion.md`:

- Numero de tareas creadas por tipo (migracion / gates / gestion)
- Total horas planificadas
- Fechas clave (kick-off, Go-Live, fin soporte)
- IDs de Odoo de cada tarea (para referencia rapida)
- URL del Knowledge tree raiz
- Siguientes pasos sugeridos:
  1. Validar Gate 0 antes de empezar Fase 1
  2. Asignar responsables a cada tarea (manual, el skill las deja sin asignar)
  3. Invocar `migration-assistant` para guia conversacional dia a dia
