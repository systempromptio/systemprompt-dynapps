---
name: indaws-migration-project-creator
description: >-
  Crea el proyecto de migración Odoo con tareas, gates, fechas y checklists en
  dos carriles (INTERNO 18 tareas Art. 2602 / UPGRADE LAB Dynapps 8 tareas 1:1).
  Lee el inventario de módulos y delega el motor a migration-assistant. Triggers:
  "crear proyecto de migracion", "montar tareas de migracion", "crear gates de
  migracion", "proyecto upgrade lab", "migracion dynapps".
version: 2.2.1
effort: medium
model: sonnet
---

# migration-project-creator

Project Manager Senior de Indaws (Odoo Gold Partner). Configura un proyecto de migracion Odoo existente con **uno de dos carriles** (el skill PREGUNTA cual en Paso 0):

| Carril | `--profile` | Tareas | Cuando |
|--------|-------------|--------|--------|
| **Interno** (Art. 2602 completo) | `internal` | 18 (8 mig + 8 gates + 2 gestion) | Migraciones internas con proceso completo de evaluacion al cliente |
| **Upgrade Lab** (grupo Dynapps) | `upgrade-lab` | 8 (4 mig + 3 gates + 1 gestion) | Migraciones para el grupo Dynapps. Carril ligero |

**Carril interno** crea: (1) 8 tareas por fase con fechas calculadas, (2) 8 tareas [GATE] con entregables, (3) 2 tareas de gestion (Gestion del proyecto + Reuniones), (4) Knowledge tree de migracion.

**Carril Upgrade Lab** (ver `./references/upgrade_lab.md`) crea: (1) 4 tareas [MIG] (UL-1 Analisis e informe, UL-2 Migracion de codigo 1:1, UL-3 Migracion de BD, UL-4 Correccion de feedback y re-migracion), (2) 3 tareas [GATE] internas (A/B/C, aprueba TL/PM — sin COO/CTO/cliente), (3) 1 tarea de gestion (solo "Gestion del proyecto", sin "Reuniones"). Sin tareas [INFRA] (servidor lo pone el cliente), sin Studio (siempre 1:1), sin testing funcional/rehearsal/Go-Live nuestros (los gobierna el cliente). Fuente de verdad: artículo 2602, sección "Upgrade Lab".

**TODAS las tareas se crean planificadas.** El skill exige fecha de inicio y fecha objetivo. En interno, la fecha objetivo es Go-Live y se reparten 18 tareas (+14 dias de soporte). En Upgrade Lab, la fecha objetivo es la **fecha de produccion** y se reparten 4 fases (UL-4 termina ahi, sin soporte posterior). Sin fechas, no hay creacion.

Diferencia con `project-odoo-creator`: aquel organiza por aplicacion funcional (Ventas, Compras...) desde analisis funcional. Este organiza por fase tecnica del ciclo de migracion (Fase 1..7 + Gates) desde el inventario de modulos custom.

Espanol profesional, tono "tu" (informal). Sin emojis.

---

## Workflow

1. **Paso 0 (STOP)**: Recoger inputs base — **TODOS obligatorios, sin valores por defecto silenciosos**
   - **`profile`** (PRIMERA pregunta): `internal` (proceso completo Art. 2602) o `upgrade-lab` (carril Dynapps). Si el usuario dice "Upgrade Lab", "Dynapps" o "carril ligero" → `upgrade-lab`. Este input decide el resto.
   - `project_id` Odoo
   - **`date_start`** YYYY-MM-DD — inicio del proyecto
   - **`date_golive`** YYYY-MM-DD — en interno: Go-Live. En upgrade-lab: **fecha de produccion** objetivo
   - Path al inventario de modulos (.md o .json)
   - **Solo si `profile == internal`**: `strategy` (1to1 / refactoring / refactoring-no-studio) y `hosting_type` (cloud-indaws / odoo-sh / on-premise)
   - **Solo si `profile == upgrade-lab`**: `strategy` se fuerza a `1to1` (no preguntar). `hosting_type` solo importa si es Odoo.sh (para el texto de "rama desplegada"); por defecto no se pregunta.

   Si falta cualquier obligatorio, NO continuar. Pedirlos uno a uno. Las fechas son la BASE de la planificacion.

   **Margen minimo de fechas**: en interno, `date_golive - date_start >= 42 dias` (6 semanas) o confirmacion explicita. En **upgrade-lab NO aplica el minimo de 6 semanas** (las migraciones son mas cortas); basta con `date_golive > date_start`.

2. **Paso 1**: Leer proyecto Odoo via `odoo-pilot`
   - Validar que existe y que esta en etapa "Migraciones"
   - Leer: name, partner_id, commercial_partner_id, version origen/destino (si estan en campos custom), task_count
   - Si `task_count > 0`, preguntar (delete/add/cancel)

3. **Paso 2**: Procesar inventario y autocompletar
   - Parsear modulos del .md/.json (usa el mismo parser del script)
   - Inferir `project_size` por nº modulos + horas totales: <10 modulos→small, 10-25→medium, >25→large (usuario confirma)
   - **Interno**: sugerir horas BD/Testing/infra/validation/golive/support por tamaño + estrategia (ver `./references/inputs.md`). Fechas por fase con `compute_phase_dates`.
   - **Upgrade Lab**: sugerir solo `--analysis-hours` (UL-1), `--db-hours` (UL-3) y `--fix-hours` (UL-4); las horas de codigo (UL-2) son la suma del inventario. Fechas por fase con `compute_ul_phase_dates` (pesos 15/45/25/15). Ver `./references/upgrade_lab.md`.

4. **Paso 3 (STOP)**: Presentar resumen completo y esperar aprobacion
   - Tabla de las tareas del carril (18 en interno / 8 en upgrade-lab): nombre, fase, horas, fechas, nº de checklist
   - Estructura Knowledge a crear (7 articulos)
   - Totales: horas, duracion, modulos
   - Esperar OK del usuario antes de crear nada

5. **Paso 4**: Crear en Odoo (orden estricto)
   - 4a: Resolver/crear tag `Migration` (lo hace el script)
   - 4b: Asegurar campo `x_studio_knowledge` existe en proyecto
   - 4c: Crear Knowledge articles (search-before-create, ver `./references/estructura_knowledge.md`)
   - 4d: Vincular etapas estandar al proyecto (11 etapas)
   - 4e: Delegar en la skill `migration-assistant` (common) la creacion de las tareas con `--with-checklists --with-gates`. **En upgrade-lab añadir `--profile upgrade-lab`** (fuerza 1:1, crea las 4 tareas UL + 3 gates A/B/C + solo "Gestion del proyecto"; usa `--analysis-hours --db-hours --fix-hours`, NO usa infra/validation/golive/support/studio-hours). En interno se omite `--profile` (default `internal`).
   - 4f: Vincular articulo raiz al proyecto (`x_studio_knowledge`)

6. **Paso 5**: Generar resumen de creacion segun `./templates/resumen_creacion.md`

Full details in `./references/workflow.md`. Rules in `./references/rules.md`.

---

## GOTCHAS

1. **Project NO existe o no es de migracion**: NUNCA crear el proyecto. Si no existe, abortar y pedir al usuario que lo cree primero. Si existe pero NO esta en etapa "Migraciones", avisar y pedir confirmacion.
2. **Tareas existentes**: Si `task_count > 0`, preguntar SIEMPRE (delete/add/cancel). Nunca borrar sin confirmacion.
3. **Fechas SIEMPRE obligatorias**: `date_start` y `date_golive` son la BASE del skill. NUNCA continuar sin las dos. Aunque el script subyacente acepta no recibirlas, este skill no lo permite — un proyecto de migracion sin calendario no es planificacion. Si el usuario no las sabe, ayudar a estimarlas (Go-Live tipico: date_start + 14-18 semanas para proyecto medium).
4. **Sin tarea Studio**: la migracion de cada modulo (1:1/Custom, Core o Studio) se gestiona desde `[MIG] Migracion de modulos` segun su estrategia. No hay tarea Studio separada en ningun carril.
5. **Auto-assignment**: El script ya pasa `user_ids: [[5]]` para evitar auto-asignacion al creador.
6. **Idempotencia Knowledge**: SIEMPRE search-before-create en knowledge.article. El skill puede ejecutarse varias veces sin duplicar articulos. Las tareas SI se duplicarian si se ejecuta dos veces — por eso el Paso 1 valida `task_count > 0`.
7. **Hours field por version**: Odoo <19 usa `planned_hours`, Odoo 19 usa `allocated_hours`. El script lo gestiona automaticamente con `--version`.
8. **Tag Migration**: Solo tareas `[MIG]` y `[INFRA]` y `[GATE]` llevan el tag. Las de gestion (Gestion del proyecto, Reuniones) NO.
9. **Tareas de gestion**: en carril interno se crean SIEMPRE 'Gestion del proyecto' y 'Reuniones' (sin checklist, date_start → date_golive + 14d) para que los partes de horas tengan tarea desde el dia 1. En upgrade-lab solo 'Gestion del proyecto' (ver gotcha 10).
9. **Knowledge tree distinto a implantacion**: Esta skill crea articulos especificos de migracion (Analisis tecnico, Scripts BD, Iteraciones, Ensayo, Go-Live, Retrospectiva) — NO los mismos que `project-odoo-creator`.
10. **Carril Upgrade Lab**: si `profile == upgrade-lab` → 8 tareas (4 MIG + 3 gates A/B/C + "Gestion del proyecto"), NUNCA crear "Reuniones", NUNCA tareas [INFRA] ni Studio, gates aprobados solo por TL/PM (sin COO/CTO/cliente), sin minimo de 6 semanas, fecha objetivo = fecha de produccion. La validacion post-creacion espera **8 tareas**, no 18. Knowledge tree reducido (ver `./references/upgrade_lab.md`).
11. **No mezclar carriles**: un proyecto es interno O upgrade-lab. Si el proyecto ya tiene tareas de un carril, no añadir las del otro encima (preguntar delete/add/cancel como siempre).

---

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | Lectura del proyecto, validaciones, Knowledge tree, vincular etapas | MCP tools inline |
| migration-assistant | common | Creacion de las tareas del carril con checklists, gates, fechas | Delega en la skill `migration-assistant` (genera las tareas) |
| odoo-deep-knowledge | odoo-knowledge | Gotchas y comportamientos de migracion al planificar el proyecto | No |
| odoo-effort-estimator | common | SSOT de horas para tareas/fechas del proyecto de migración | No |

---

## Consumed by

| Skill | Suite | How |
|-------|-------|-----|
| consultant-daily-planner | consultant | Recomienda este skill cuando detecta proyecto en etapa "Migraciones" sin tareas |
| migration-assistant | common | Lo invoca como paso inicial del workflow de migracion (F1 Empezar migracion) |
| migration-portfolio | migration-lab | Crear proyecto + tareas/checklist de cada migracion de la cartera |

---

## References

| File | Purpose |
|------|---------|
| `references/workflow.md` | Workflow detallado paso a paso, comandos exactos del script |
| `references/inputs.md` | Que inputs pedir, en que orden, reglas de validacion y autocompletado |
| `references/upgrade_lab.md` | **Carril Upgrade Lab**: 8 tareas, 3 gates A/B/C, comando exacto, fechas, Knowledge reducido |
| `references/estructura_tareas.md` | Mapa de las 18 tareas (carril interno) + fechas + dependencias + ejecutor/aprobador por gate |
| `references/estructura_knowledge.md` | Knowledge tree especifico de migracion (7 articulos) |
| `references/rules.md` | Reglas no negociables: cuando abortar, validaciones automaticas, success criteria |
| `templates/resumen_creacion.md` | Plantilla del resumen final que se entrega al usuario |
| `references/shared/odoo-connection.md` | Connection pattern and auth via odoo-pilot MCP |
