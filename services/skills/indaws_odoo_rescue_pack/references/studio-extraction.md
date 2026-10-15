# Fase 3 — Personalizaciones fuera de módulos (Studio + acciones)

La parte de BD del Rescue Pack **NO restaura Odoo**. El partner anterior casi siempre dejó
lógica que vive **en la base de datos, no en módulos de código**: campos Studio, modelos
custom, acciones de servidor y acciones automatizadas. Eso es invisible auditando solo el
código fuente — hay que sacarlo de la BD vía API en **solo lectura**.

## Por qué importa

- Lo que está en `addons` (Pilar A) es solo media foto. La otra mitad está en BD.
- Sin extraerlo no puedes estimar la migración: Studio y acciones se pierden o rompen al
  saltar de versión, y no están versionados en ningún git.
- Una vez extraído como código, **se puede auditar** (realimenta el Pilar A) y **migrar**.

## Qué se extrae (todo en solo lectura)

| Qué | Modelo | Cómo identificarlo |
|-----|--------|--------------------|
| Campos Studio / manuales | `ir.model.fields` | `state = manual` (incluye `x_studio_*` y `x_*`) |
| Modelos custom | `ir.model` | `state = manual` (modelos `x_*`) |
| Acciones de servidor | `ir.actions.server` | todas; interesa `code`, `state`, `usage` |
| Acciones automatizadas | `base.automation` | trigger, modelo, dominio, SA enlazada |
| Vistas Studio | `ir.ui.view` | `arch_fs = false` (arch en BD, no en disco) |
| Acciones de ventana / menús | `ir.actions.act_window`, `ir.ui.menu` | creados en Studio |
| Crons | `ir.cron` | apuntan a `ir.actions.server`; revelan fuentes automáticas |

### Cómo se accede a la BD (sin Odoo)

Se restaura el dump (zip/sql/dump/gz) y se levanta **solo Postgres** con `docker-deployment`:
`new.sh <proj> <ver>` → `up.sh <proj> db` (solo el servicio `db`) → `restore.sh <proj> <dump>`.
Luego se exportan las variables `PG*` desde el `.env` del proyecto. Detalle en
`access-prerequisites.md`.

`scripts/extract_studio.sh` lanza SELECTs de **solo lectura** sobre las tablas `ir_*` (según
los `targets` de `config.json`) y produce:
- `INVENTORY.md` (conteo + listado por tipo) y `json/<key>.json` (filas crudas), y
- el **código** de acciones de servidor (columna `code` de `ir_act_server`) a ficheros `.py`, y
- las **asignaciones de las acciones `object_write`** (`json/server_action_writes.json`): su
  lógica NO está en `code` sino en `ir_server_object_lines` (campo ← `value`/`evaluation_type`).
  Sin esto, las acciones de campo (frecuentísimas en Studio) salen **vacías** y se infravalora
  el Pilar B. El script ya lo extrae vía JOIN; interpreta el `evaluation_type`
  (`value`=literal, `reference`=id de registro, `equation`=expresión Python).

## Consolidación: el módulo `odoo_studio`

Idea clave: **aglutinar todo lo custom-fuera-de-módulos en un único módulo `odoo_studio`**
(nombre en `config.json → odoo_studio_module_name`). Beneficios:

1. Deja de estar "suelto" en la BD → pasa a estar en git, versionado y revisable.
2. Se vuelve auditable como código (Pilar A) en vez de opaco.
3. Es el artefacto que se lleva a la migración (en vez de re-hacer Studio a mano).

**Su contenido se audita y puntúa 0-10 como cualquier custom** (Fase 2): el código de las server
actions/automatizaciones suele esconder lo peor — IDs hardcodeados (`mail_server_id=10`),
queries frágiles sobre modelos volátiles (`bus.bus`), interceptación de flujos de mail,
dependencias externas no declaradas. No lo migres tal cual sin auditarlo.

Estructura del módulo `odoo_studio` (delegar scaffolding a `odoo-module-creator` + `odoo-data-files`;
estructura de records de Studio según `odoo-studio-migration`):

```
odoo_studio/
├── __manifest__.py          # depends de los módulos que tocan esos modelos
├── models/                  # campos manuales no-Studio que convenga pasar a código real
├── data/
│   ├── ir_model_fields.xml  # campos Studio
│   ├── ir_actions_server.xml
│   ├── base_automation.xml
│   └── ir_ui_view.xml       # vistas Studio
└── security/                # si hay grupos/reglas custom
```

> Decisión por elemento (usar `odoo-classification-guide`): qué se mantiene como Studio data,
> qué conviene reescribir como código nativo, y qué es sustituible por estándar/Indaws Core.

## Criterio "fuera de módulos" (clave para no ahogarse en ruido)

Tablas como `ir_act_server`, `ir_act_window`, `ir_ui_menu`, `ir_cron` o `base_automation`
contienen **cientos** de registros estándar de Odoo y de los módulos custom (que ya están en
código). Lo que el Rescue Pack quiere es solo lo que **vive fuera de módulos** = creado en
runtime/Studio, **sin XMLID de módulo** en `ir_model_data`.

El `where` de esos targets usa:
`NOT EXISTS (SELECT 1 FROM ir_model_data d WHERE d.model='<modelo>' AND d.res_id=<tabla>.id AND d.module <> 'studio_customization')`

Así se descartan los registros de módulos (estándar y custom) y se conservan los de runtime +
los de `studio_customization`. Ejemplo real (farmasoft v16): de **127** acciones de servidor,
solo **4** son runtime/Studio; de **573** menús, **3**. El filtro convierte el ruido en señal.

> **El nombre del módulo va en SINGULAR y ahí se perdía todo.** Hasta el 2026-08-15 estos `where`
> decían `studio_customizations`, en plural. El módulo que crea Studio se llama
> `studio_customization` (`web_studio/models/ir_module_module.py:16-19`, `IrModuleModule.get_studio_module`, donde se busca y se crea
> con ese nombre exacto; y `odoo/addons/base/models/ir_ui_view.py:548`, `View.write`, que comprueba el xmlid con
> `startswith('studio_customization')`). Con el plural, la comparación es siempre cierta para un
> registro de Studio → el `NOT EXISTS` da falso → **se descarta justo lo que se venía a buscar**.
> Medido en `tools/bench-rescue`: de una vista, una acción y un menú marcados por Studio, salían
> **cero**. El plural existe en el árbol de Odoo (66 veces), pero en traducciones y en el xmlid de
> una vista de filtro — nunca como nombre de módulo.

Los campos (`ir_model_fields`) y modelos (`ir_model`) ya se aíslan con `state = 'manual'`, que
es marca propia de Studio/manual; no necesitan el join.

## Reglas

- **Solo Postgres, nunca Odoo.** Restaurar el dump y levantar únicamente el servicio `db`
  (`docker-deployment`). No se arranca el servidor de aplicaciones.
- **Solo lectura.** SELECTs; nunca INSERT/UPDATE/DELETE.
- Si una tabla de `targets` no existe en esa versión, el script la marca `N/A` y sigue
  (las tablas `ir_*` cambian entre majors; p. ej. `base_automation` no siempre tiene `code`).

## Reconstrucción por SQL del custom (Fase 3 ampliada — motor dump-only sin addons)

Cuando **solo hay el dump** y no se puede levantar Odoo (sin filesystem/addons) ni hay API live,
`odoo-reverse-engineering` no aplica y esta reconstrucción por SQL es la **única vía**. Recupera
~toda la capa declarativa; solo el **Python imperativo** (compute/create/write/onchange, controladores)
queda opaco, y `ir_model_fields.compute/store` marca dónde está ese hueco. Todo en solo lectura:

- **`scripts/reconstruct_custom.sh`** — capa declarativa de cada módulo **CUSTOM**: modelos; campos con
  `ttype`/`relation`/`related`/`compute`/`store`/`depends` + selección; vistas `arch_db`; seguridad;
  menús; acciones; crons; informes QWeb; plantillas de correo; código de server actions.
  Salida: `custom_reconstruct/<mod>/FOOTPRINT.md`.
- **`scripts/db_intelligence.sh`** — dimensionado **sin código**: grafo de dependencias, versión/estado,
  complejidad por módulo, **fill-rate** (campo vivo vs muerto), última actividad por modelo, seguridad.
- **`scripts/db_state.sh`** — conteo de tablas principales (→ sec. 2).
- **`scripts/extract_studio.sh`** — Studio/acciones runtime por `ir_*` (→ sec. 3).

`ir_model_data` atribuye cada modelo/campo/vista/menú/acción/grupo/regla a su módulo → "instalado sin
fuente" ≠ caja negra.
