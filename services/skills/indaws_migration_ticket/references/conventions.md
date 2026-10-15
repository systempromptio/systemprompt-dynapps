# Convenciones del flujo de migración (Dynapps Bélgica) — REGISTRO ÚNICO

> **Fuente de verdad única.** Cada regla tiene un **ID estable**. La skill
> `migration-ticket` (su checklist), el bloque de CLAUDE.md y los hooks de
> `settings.json` **referencian estos IDs** — nunca reescriben la regla. Así no
> hay desconexión: se cambia aquí y todo lo demás sigue apuntando al mismo sitio.
>
> Alcance: **solo el flujo de migración Bélgica** (GitLab `dynapps.be`, buildouts,
> el kit, las skills `dynapps-*`/`odoo-database-upgrader`/`odoo-migration-parity`).
> **NO** aplica al flujo España (`ticket-solver`, `git-manager`), ni al revés.

## Cómo se aplica cada regla (columna Enforcement)

- **hook** — forzado por `settings.json` en local, en cada sesión y tras reiniciar (no se olvida).
- **CI** — forzado por el pipeline de GitLab del repo del cliente.
- **skill** — el checklist de `migration-ticket` lo comprueba/recuerda al resolver el ticket.
- **memory** — lección persistente que se auto-recuerda entre sesiones.

Lo robusto es la combinación: `hook`/`CI` garantizan, `skill`/`memory` guían.

## Registro

| ID | Regla | Por qué | Enforcement |
|---|---|---|---|
| **LANG-EN** | Todo comentario y string de código en **inglés**. Nunca español en el código. | Repos compartidos con el equipo Bélgica; convención dura del proyecto. | hook + skill + memory |
| **STYLE** | Imitar el estilo y la **densidad de comentarios** del código circundante; no sobre-comentar. | Coherencia con el módulo; el review Bélgica lo espera. | skill |
| **NO-HARDCODE** | Nada hardcodeado por cliente; solución genérica y parametrizable. | Portabilidad; filosofía de módulo. | skill |
| **GIT-BRANCH** | Rama `feature/<TICKET>` (p.ej. `feature/ZNU-744`), no nombres descriptivos. | Convención de ramas Bélgica. | skill + memory |
| **GIT-NOFORCE** | **Nunca `push --force`** en una rama con MR abierta. Los fallos (incl. warnings) se corrigen con **commits nuevos encima**. | No reescribir historia bajo review; no romper la MR. | hook + skill + memory |
| **GIT-NOCOAUTHOR** | Commits **sin** `Co-Authored-By` (ni Claude ni ninguno automático). | Convención dura del usuario/equipo. | hook + skill + memory |
| **CI-GREEN** | Una MR **no se mergea sin pipeline verde**. `checklog-odoo` flaggea **WARNINGs**, no solo errores: tratarlos. | Gate de calidad Bélgica. | CI + skill + memory |
| **CI-EXTERNAL** | Si el pipeline no corre por accesos (usuario External → 0 jobs, sin acceso a componentes CI) el gate pasa a **verde en local** (pre-commit + tests). Es petición a infra, no fix de código. | Bloqueo conocido de CI que no debe frenar la entrega. | skill + memory |
| **VERIFY-LOCAL** | Antes de subir, **verificar en local todo lo verificable**: tests del kit, `-u`/install del módulo, `odoo-migration-parity`. Documentar lo que **no** se puede correr en local (p.ej. actions privadas que no clonan sin credenciales). | Detectar fallos antes del pipeline; el CI local no siempre es reproducible. | skill |
| **ANTI-REGRESSION** | Un fix **no debe hacer perder funcionalidad de forma indirecta**. Antes de entregar: correr `odoo-migration-parity` y revisar vistas/campos/menús/datos afectados; usar los gates B1/B2/B3 del post-restore. | La causa nº1 de tickets: correcciones que tumban otra cosa. | skill + memory |
| **DATA-PREUPLOAD-POSTMIG** | Un fix de **datos** con pre-upload SQL necesita **post-migration acompañante**: el pre-upload no llega a la BD de TEST ya migrada y el reporter lo devuelve ("still there"). | Patrón repetido; entregar siempre el par. | skill + memory |
| **NEUTRALIZE** | Al trabajar sobre copias de producción, revisar **gaps de neutralización** (conectores con flag no estándar como EDI `is_active`, banca, transportistas) antes de arrancar. | Una copia mal neutralizada envía EDI/correo reales. | skill + memory |
| **MR-REVIEWER** | Asignar el **revisor por defecto del repo** (no asumir). | Que la MR llegue a quien revisa. | skill + memory |
| **MR-STATUS** | Una MR **"closed"** puede estar **mergeada por rebase**: comprobar el **contenido en la rama base**, no el estado de la MR. | Evita rehacer trabajo ya integrado. | skill + memory |

## Definition of Done (resumen accionable)

Un ticket de migración está **hecho** cuando: código y comentarios en inglés
(LANG-EN, STYLE) y sin hardcode (NO-HARDCODE) · verificado en local lo verificable
(VERIFY-LOCAL) · sin regresión comprobada (ANTI-REGRESSION) · par pre/post-migration
si toca datos (DATA-PREUPLOAD-POSTMIG) · rama `feature/<TICKET>` con MR, revisor
asignado, **sin co-author, sin force-push**, **pipeline verde** o verde-en-local
justificado (GIT-BRANCH, GIT-NOFORCE, GIT-NOCOAUTHOR, MR-REVIEWER, CI-GREEN,
CI-EXTERNAL) · lección nueva llevada a memoria si la hubo.
