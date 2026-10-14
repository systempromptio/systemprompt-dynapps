# Lecciones aprendidas del flujo de migración Bélgica

Catálogo **genérico** (patrones, no incidencias concretas de un cliente) que
justifica las reglas de `conventions.md`. Cada lección enlaza su **ID de regla**.
Al descubrir una lección nueva en un ticket, añadirla aquí **y** a memoria
(Fase 5 de la skill) para que no se pierda entre sesiones.

## Código y estilo

- **[LANG-EN]** El código de estos repos lo mantiene también el equipo Bélgica:
  cualquier comentario o string en español es deuda inmediata. Regla dura: **todo
  en inglés**, sin excepción, incluso en scripts de un solo uso.
- **[STYLE]** Un módulo con poco comentario no quiere de repente comentarios
  densos, y viceversa. Imitar la densidad y el tono del fichero que se toca.
- **[NO-HARDCODE]** Un fix atado a un cliente concreto reaparece en el siguiente.
  Resolver de forma genérica y parametrizable desde el principio.

## Anti-regresión (la lección más cara)

- **[ANTI-REGRESSION]** El mayor coste histórico no son bugs nuevos, sino
  **correcciones que tumban otra funcionalidad de forma indirecta**: un xpath que
  reactiva mal, una vista que se desactiva, un campo que desaparece del form. Antes
  de entregar, comparar origen↔migrado con `odoo-migration-parity` y revisar
  explícitamente vistas/campos/menús/datos que el cambio roza.
- El upgrade puede **desactivar vistas custom** o **vaciar campos required** sin
  error visible en el `-u`: los gates B1/B2/B3 del post-restore existen para eso.
  No fiarse de que "instala sin error" = "no hay regresión".

## Datos

- **[DATA-PREUPLOAD-POSTMIG]** Un fix de datos hecho **solo** con pre-upload SQL
  **no llega a la BD de TEST ya migrada**: el reporter lo devuelve ("still there").
  Entregar **siempre el par**: el pre-upload (para futuras subidas) **y** una
  post-migration que haga lo mismo sobre la BD ya migrada.
- **[NEUTRALIZE]** Los conectores no siempre neutralizan por `active`: algunos
  (EDI, banca, transportistas) usan flags no estándar (`is_active`, `prod_environment`).
  Una copia "neutralizada" a medias puede enviar EDI o correo **reales**. Revisar
  los gaps antes de arrancar sobre una copia de producción.

## Git / GitLab (Bélgica)

- **[GIT-BRANCH]** Ramas `feature/<TICKET>` (`feature/ZNU-744`), no nombres
  descriptivos.
- **[GIT-NOFORCE]** En una rama con MR abierta **nunca** se reescribe historia:
  los fallos de pipeline —incluidos los **warnings**— se corrigen con **commits
  nuevos encima**.
- **[GIT-NOCOAUTHOR]** Los commits van **sin** línea `Co-Authored-By`.
- **[MR-STATUS]** Una MR marcada **"closed"** puede estar **mergeada por rebase**.
  Comprobar el **contenido en la rama base**, no el estado de la MR, antes de
  concluir que algo no se integró.
- **[MR-REVIEWER]** Asignar el **revisor por defecto del repo** (no asumir uno).

## CI

- **[CI-GREEN]** El gate es pipeline **verde**, y `checklog-odoo` marca **WARNINGs**
  además de errores. Un warning sin tratar bloquea igual.
- **[CI-EXTERNAL]** Si el pipeline sale con **0 jobs** o casca al clonar por
  accesos (usuario **External**, sin acceso a componentes CI internos), no es un
  fallo de código: es **petición a infra**, y mientras tanto el gate pasa a **verde
  en local** (pre-commit + tests). Documentarlo, no forzar el CI.
- **[VERIFY-LOCAL]** No todo el CI se puede reproducir en local (p.ej. actions
  privadas del org que no clonan sin credenciales). Correr lo que sí (tests del kit,
  `-u`/install, parity) y **documentar** lo que no.
