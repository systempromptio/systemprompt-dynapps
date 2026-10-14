---
name: indaws-migration-portfolio
version: 0.1.1
effort: high
model: opus
description: >-
  Gestiona la cartera de migraciones Dynapps que ejecuta Indaws (Fase 2):
  intake, reparto entre devs y seguimiento multi-cliente por gate; delega lo
  técnico sin re-ejecutarlo. Triggers: "cartera de migraciones", "reparto de
  migraciones", "asignar migracion a dev", "estado de las migraciones",
  "migration portfolio", "que migraciones hay pendientes", "alta de migracion
  Dynapps", "intake migracion", "quien lleva la migracion de X".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, Skill, Agent]
---

# Migration Portfolio — Cartera de migraciones Dynapps (Fase 2)

Gestiona la **cartera** de las 16+ migraciones que Indaws hace para Dynapps:
**intake**, **reparto entre devs** y **estado multi-cliente por gate**. NO ejecuta
lo técnico: lo **lanza delegando**. Es la capa de gestión de la Fase 2 (Ejecución).

> Contexto, reglas y flujo del suite en `migration-lab:DESIGN.md`.

Conexión a Odoo SIEMPRE vía `odoo-pilot`: ver `references/shared/odoo-connection.md`.

## Inputs (pregunta solo lo imprescindible)
1. **Acción**: `intake` (alta) · `assign` (reparto) · `status` (estado por gate).
2. Para intake: **link de Drive** + **ZIP de módulos** + **tamaño BD** + cliente.
3. Para assign: lista de devs disponibles (o léela del estado guardado).

## Workflow

1. **Cargar cartera** — Lee el estado persistido (`references/gate-tracking.md`
   §Almacenamiento). Si no existe, créalo desde el listado de migraciones.
2. **Intake** [si action=intake] — Registra la migración: Drive, ZIP, tamaño BD,
   cliente, contacto **Janik**, versión origen/destino, horas estimadas (de la
   estimación previa). Detalle de campos en `references/intake.md`. **No descarga
   ni restaura BD** (no hay dump, solo tamaño).
3. **Reparto** [si action=assign] — Reparte migraciones entre devs balanceando carga
   (horas estimadas) y skills. Algoritmo y formato del Excel/CSV en
   `references/assignment.md`. Marca asignado, fecha objetivo y prioridad.
4. **Lanzar lo técnico** [delega] — NO ejecuta migración. Por cada migración activa:
   - **Crear proyecto/tareas** → skill `migration-project-creator`.
   - **Migración técnica** (módulos + BD) → skill `dynapps-migrator`.
5. **Estado por gate** [FAN-OUT PARALELO] — Por cada migración activa de la cartera,
   lanza **en paralelo** un subagente que lee su estado de gate (los gates son la
   fuente de verdad de `migration-assistant`). Recoge todos los resultados en
   `GATE_MAP` y consolida la vista multi-cliente. Lanza un Agent() por migración:
   - **subagent_type:** `general-purpose`
   - **model:** `sonnet` (lectura mecánica de gate, no requiere opus)
   - **description:** `"read gate status for {cliente}"`
   - **prompt:** `"Invoca la skill 'migration-assistant' para el proyecto de migración del cliente {cliente} (project_id {project_id}). Devuelve SOLO el estado actual como JSON: {gate, status, blockers, next_action}. Read-only: no modifiques nada."`

   Guarda cada salida en `GATE_MAP[cliente]`. No leas los gates inline tú: delega cada
   lectura a su subagente para aislar contexto y paralelizar. Mapeo en `references/gate-tracking.md`.
6. **Vista ejecutiva** [delega] — Para el roll-up de cartera (semáforos, riesgo),
   lanza un subagente: **subagent_type** `general-purpose`, **model** `sonnet`,
   **prompt** `"Invoca la skill 'executive-projects-tracker' con la cartera {GATE_MAP} y devuelve el roll-up (semáforos, riesgo)."`. No re-implementes el dashboard.
7. **Persistir** — Guarda el estado consolidado (`GATE_MAP` + horas estimadas de intake
   + dev asignado por migración). Este mapa es lo que consume `migration-billing-tracker`
   (Fase 3): persístelo en el formato de `references/gate-tracking.md`. Devuelve resumen tabla.

## GOTCHAS
1. **Esta skill GESTIONA, no migra.** La migración técnica es de `dynapps-migrator`;
   la creación de tareas de `migration-project-creator`. Aquí solo se orquesta.
2. **Los gates son fuente de verdad en `migration-assistant`.** No inventes ni
   dupliques el estado de gates: léelo de ahí y consolídalo.
3. **NUNCA enviar correo desde Odoo.** Aunque el usuario diga "avisa a Janik del
   reparto", el aviso es manual y fuera de Odoo.
4. **No hay dump, solo el tamaño de la BD.** No intentes descargar ni restaurar.
5. **Contacto siempre = Janik (Dynapps).** Dynapps es el cliente directo, no el
   cliente final.
6. **Balancea por horas estimadas, no por nº de migraciones.** Una de 150h pesa
   como varias pequeñas.
7. **El proyecto se crea en interno** mientras Dynapps no acepta (ver DESIGN §3.5);
   no asumas que está aprobado al hacer intake.
8. **No re-estimar aquí.** Las horas vienen de `migration-estimator`; esta skill
   solo las usa para repartir y seguir.

## NEVER
- Ejecutar la migración técnica directamente (delega en `dynapps-migrator`).
- Crear el estado de gates por tu cuenta (lo posee `migration-assistant`).
- Enviar email desde Odoo.
- Re-estimar horas (eso es de `migration-estimator`).

## ALWAYS
- Persistir el estado de la cartera tras cada cambio.
- Repartir balanceando por horas estimadas.
- Usar `odoo-pilot` para cualquier lectura/escritura en Odoo.
- Tratar los gates de `migration-assistant` como única fuente de verdad.

## Agent routing
Balanceo y consolidación (principal): **OPUS**. Las lecturas de gate (paso 5) y la
vista ejecutiva (paso 6) se delegan como **subagentes `sonnet` en paralelo** (uno por
migración) — trabajo mecánico de lectura, no requiere opus. Delega lo técnico a las
skills propietarias; no lo re-ejecuta.

## Delegates to
| Skill | Suite | For what |
|-------|-------|----------|
| `migration-project-creator` | consultant | Crear proyecto + tareas/checklist de la migración |
| `dynapps-migrator` | developer | Ejecutar la migración técnica (módulos + BD) |
| `migration-assistant` | common | Gates como fuente de verdad (estado por gate) |
| `executive-projects-tracker` | management | Vista ejecutiva/roll-up de la cartera |
| `migration-estimator` | migration-lab | Origen de las horas estimadas (no se re-estima) |
| `odoo-pilot` | common | Lecturas/escrituras en Odoo (proyectos, contactos) |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| `migration-billing-tracker` | migration-lab | Toma el estado por gate para la vista de cobro (Fase 3) |
| `migration-closeout` | migration-lab | Lee la cartera para el cierre formal (Fase 4) |

## REFERENCES
- `references/intake.md` — Alta de una migración (Drive/ZIP/tamaño BD/Janik)
- `references/assignment.md` — Reparto entre devs + formato del Excel/CSV
- `references/gate-tracking.md` — Estado multi-cliente por gate + almacenamiento
- `references/shared/odoo-connection.md` — Conexión Odoo vía odoo-pilot
- `migration-lab:DESIGN.md` — Diseño del suite y contexto de negocio
