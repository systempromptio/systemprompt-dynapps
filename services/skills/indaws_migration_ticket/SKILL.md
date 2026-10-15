---
name: indaws-migration-ticket
description: >
  Resuelve un ticket del flujo de migración Odoo de Dynapps BÉLGICA (buildouts,
  GitLab dynapps.be, el kit, las skills dynapps-*/odoo-database-upgrader/
  odoo-migration-parity) aplicando TODAS las convenciones y lecciones aprendidas
  del proyecto: comentarios/código en inglés, verificar en local antes de subir,
  no perder funcionalidad de forma indirecta (anti-regresión), rama feature/
  <TICKET> con MR, sin co-author, sin force-push, pipeline verde. Punto de entrada
  único por ticket: se le pasa la info del ticket y guía el cambio de principio a
  fin contra el registro de convenciones. NO es el flujo España (ticket-solver /
  git-manager) — es independiente.

  TRIGGER: "voy a resolver el ticket <ID>", "resuelve ZNU-xxx / <TICKET>",
  "cambio de migración", "migration ticket", "arreglar esto en la migración",
  URL/ID de un ticket de migración.
version: 0.1.0
model: opus
effort: high
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, Agent, Skill]
---

# Migration Ticket

Resuelve un ticket del **flujo de migración Bélgica** cumpliendo el registro de
convenciones y las lecciones aprendidas. Es el **punto de entrada por ticket**:
recibe la info del ticket y conduce el cambio hasta la MR con pipeline verde.

> **Fuente de verdad**: `references/conventions.md` (registro con IDs). Este
> checklist **referencia esos IDs**, no los reescribe — así el skill, el bloque de
> CLAUDE.md y los hooks nunca se desincronizan. Las lecciones y su porqué están en
> `references/lessons-learned.md`. Las capas que persisten entre sesiones (CLAUDE.md
> + hooks + memoria) están en `references/enforcement.md`.
>
> **Alcance**: solo Bélgica. **NO** delega en `git-manager` ni `ticket-solver`
> (flujo España) — sus convenciones no aplican aquí.

## Input mínimo

| Campo | Ejemplo | Obligatorio |
|---|---|---|
| `ticket` | ZNU-744 | Siempre |
| `descripción` | "falta el peso en las líneas de factura v19" | Siempre |
| `repo`/módulo | buildout del cliente / `xx_account` | Siempre |
| versión | v16→v19 (si aplica) | Si es cambio de versión |

Pedir solo lo que falte (AskUserQuestion).

## Workflow (cada fase cita los IDs de `conventions.md`)

**Fase 0 — Intake.** Registrar ticket, descripción, módulo/área afectada. Si es
una MR ya abierta, comprobar su **contenido en la rama base** (`MR-STATUS`): una
MR "closed" puede estar mergeada por rebase.

**Fase 1 — Entender + riesgo de regresión.** Leer el código y la funcionalidad en
juego. Preguntar: *¿este cambio puede hacer perder algo de forma indirecta?*
(`ANTI-REGRESSION`). Si toca vistas/campos/menús/datos migrados, planificar correr
`odoo-migration-parity` y/o los gates B1/B2/B3 del post-restore antes de entregar.

**Fase 2 — Implementar según convenciones.**
- Comentarios y strings **en inglés** (`LANG-EN`); imitar el estilo del código
  circundante (`STYLE`); nada hardcodeado por cliente (`NO-HARDCODE`).
- Si es fix de **datos** que el Upgrade Service ya procesó: entregar el **par
  pre-upload SQL + post-migration** (`DATA-PREUPLOAD-POSTMIG`).
- Si se trabaja sobre copia de producción: revisar **gaps de neutralización**
  (`NEUTRALIZE`).

**Fase 3 — Verificar en local (`VERIFY-LOCAL`).** Correr todo lo verificable
antes de subir: tests del kit, `-u`/install del módulo, `odoo-migration-parity`
sobre origen↔migrado (`ANTI-REGRESSION`). **Documentar** lo que no se puede correr
en local (p.ej. actions de CI privadas). No subir con algo verificable en rojo.

**Fase 4 — Entregar por GitLab.**
- Rama `feature/<TICKET>` (`GIT-BRANCH`).
- Commit **sin `Co-Authored-By`** (`GIT-NOCOAUTHOR`).
- **Nunca force-push** en rama con MR; corregir con commits nuevos encima
  (`GIT-NOFORCE`).
- Crear MR, asignar el **revisor por defecto del repo** (`MR-REVIEWER`).
- **Pipeline verde** antes de mergear; `checklog-odoo` flaggea WARNINGs, tratarlos
  (`CI-GREEN`). Si el pipeline no corre por accesos (usuario External / sin acceso
  a componentes), gate = **verde en local** justificado (`CI-EXTERNAL`).

**Fase 5 — Cierre.** Si apareció una **lección nueva** (un gotcha de versión, un
patrón), llevarla a memoria (y, si procede, a `lessons-learned.md`) para que no se
pierda entre sesiones.

## Gate de salida — Definition of Done

No se entrega hasta cumplir el DoD de `conventions.md` (resumen): inglés + estilo +
sin hardcode · verificado en local · sin regresión · par pre/post si toca datos ·
`feature/<TICKET>` + MR + revisor + sin co-author + sin force-push + pipeline verde
(o verde-en-local justificado) · lección a memoria si la hubo.

## GOTCHAS

1. **No confundir con el flujo España.** Nada de `git-manager`/`ticket-solver`
   aquí: GitLab `dynapps.be`, `glab`, convenciones Bélgica.
2. **El pipeline verde incluye WARNINGs** (checklog-odoo), no solo errores.
3. **Verde-en-local es válido SOLO** cuando el pipeline no corre por accesos, y se
   justifica; no es una excusa general para saltarse el CI.
4. **Anti-regresión no es opcional**: el mayor coste histórico son fixes que
   tumban otra funcionalidad. Correr `odoo-migration-parity` cuando el cambio roza
   vistas/campos/datos.
5. **Las capas que sobreviven a nueva sesión / reinicio** son CLAUDE.md + hooks +
   memoria (ver `references/enforcement.md`); la skill sola necesita disparador.

## References

| Fichero | Contenido |
|---|---|
| `references/conventions.md` | **Registro único** de convenciones con IDs (SSOT) + Definition of Done |
| `references/lessons-learned.md` | Catálogo de lecciones aprendidas (genérico) + porqué de cada regla |
| `references/enforcement.md` | Capas durables: bloque para el CLAUDE.md del repo + hooks de `settings.json`, referenciando los mismos IDs |

## Delegates to (solo flujo Bélgica)

| Skill | Cuándo |
|---|---|
| `odoo-migration-parity` | Fase 1/3: comprobar que el cambio no pierde funcionalidad (anti-regresión) |
| `odoo-database-upgrader` / `dynapps-code-migration` | Si el ticket es del upgrade de BD o del port de código |
| `odoo-module-upgrader` | Si el ticket es portar/arreglar un módulo entre versiones |
| `memory-check` | Fase 5: registrar/recuperar lecciones |
