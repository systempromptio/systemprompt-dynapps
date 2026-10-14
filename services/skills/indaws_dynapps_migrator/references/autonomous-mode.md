# Autonomous Mode — Dynapps Migrator

El modo autónomo corre BLOQUE 1 → 2 → 3 **sin parar**, salvo en gates irreducibles,
hasta entregar el hito **"Entorno local listo"**. Todo lo posterior (MRs, validación
funcional, go-live) queda fuera de la corrida autónoma.

## Hito: "Entorno local listo" (criterio de aceptación)

La corrida autónoma termina cuando se cumplen TODOS estos puntos:

- **BD:** upgraded por el Upgrade Service → restaurada → **neutralizada + verificada**
  (gate duro, `dynapps-db-upgrade-runner/references/neutralization.md`) → `-u all` **limpio**
  en el contenedor Docker (5433).
- **Código:** módulos custom portados en orden topológico; flags `installable:False`
  limpiados; scripts de migración presentes (incl. `end-reactivate_views.py`); los módulos
  cargan con `-u` **sin ERROR/WARNING** en el log (estilo `checklog-odoo`).
- **Arrancando:** `odoo-bin` levanta contra la BD migrada; smoke RPC verde; se entrega al
  usuario la URL `http://localhost:8069` (admin/admin).
- **Reproducible por terceros (gate):** el entorno debe poder levantarlo **otra persona** con
  solo (1) el **código migrado en GitLab/GitHub** y (2) la **BD recién migrada por el Upgrade
  Service** — sin nada local extra. Comprobar antes de cerrar el hito:
  - Todo lo necesario está **commiteado** al repo (scripts `migrations/<version>/`, deps del
    buildout en `<prefijo>_base`, config); `git status` limpio salvo lo intencional.
  - **No hay parches locales ni pasos manuales no versionados** que el entorno necesite para arrancar.
  - La BD entregable es la **salida del Upgrade Service** (+ neutralización reproducible por script),
    no una BD retocada a mano de forma irrepetible.
  - **Verificación** (recomendada): clonar el repo en un dir limpio + restaurar el dump migrado en
    una BD nueva + `odoo-bin -u all` limpio, sin tocar nada de la máquina origen.
- **Explícitamente FUERA:** ningún `git push`, ninguna MR, ningún go-live.

En este punto la skill **se detiene y devuelve el control** con el entorno usable.

## Default de decisiones: paridad de comportamiento 1:1

Ante cualquier diferencia introducida por la v-destino (feature cambiada/movida/eliminada por
Odoo), el **default es preservar el comportamiento del cliente 1:1** respecto a la v-origen. No se
adopta el nuevo estándar de Odoo por serlo. Aplica a: decisiones por módulo (STOP ④), TODOs de port
(STOP ⑥) y clasificación de `odoo-migration-parity` (una diferencia = regresión a corregir por
defecto, no "cambio de versión aceptado"). Desviarse del 1:1 = **decisión de negocio explícita**,
nunca asumida; si aparece y no está resuelta en el estudio previo → es uno de los STOP que se mantienen.

## STOPs que se MANTIENEN (irreducibles)

| STOP | Cuándo | Por qué no se puede automatizar |
|---|---|---|
| ① vault-login | Máquina lista, sin sesión Vault | OIDC por navegador |
| Upgrade Service | Falta acceso SSH/cred, o decisión **test/producción** | Acceso humano / decisión de riesgo (odoo.sh siempre `test` → sin STOP) |
| Neutralización ROJA | Cualquier check de neutralización falla | Seguridad: dump de producción, nunca se salta |
| ④ módulo ambiguo | El estudio del consultor no resuelve un módulo | Decisión de negocio (KEEP/STUDIO/NEUTRALIZE/TO_DROP) |
| ⑤ rebase sin ruleset | Conflicto de rebase que el ruleset no cubre | Decisión técnica |
| ⑥ TODO sin patrón | TODOs no resueltos tras el port | Decisión técnica |

## Lo que AUTO-AVANZA (antes STOP o manual)

- **② creación de buildout** — la skill la conduce; no para a preguntar (repo/escenario ya resueltos).
- **③ estudio previo** — si se aportó al inicio o se auto-detecta, no bloquea; solo cae a
  STOP ④ por módulo concreto sin decisión.
- **IDE config (SmartGit/PyCharm, STEPS 4–7 de `dynapps-local-env-setup`)** — **se SALTAN**
  en modo autónomo/headless: no hacen falta para arrancar `odoo-bin`. (Se dejan para cuando
  un dev quiera depurar.)
- **⑦ MR / ⑧ validación funcional / ⑨ go-live** — posteriores al hito → fuera de alcance.

## Esperas largas → background + watcher + paralelismo

El detalle completo (niveles 1–3, convergencia, caveats) está en `workflow.md §Paralelización`.
En modo autónomo se aplica siempre:

- **Upgrade Service** (minutos a horas): lanzar en background con vigilante; **en paralelo**
  avanzar TODO el track de código — incl. **el porting de los módulos** (Nivel 1), no solo
  buildout/rebase/análisis. Convergir al llegar el ZIP.
- **Módulos independientes** del mismo nivel topológico → **fan-out** con workers (Nivel 2,
  opt-in). Serializar el `-u` sobre la BD compartida, o BD scratch por worker.
- **Verificación en dos capas** (Nivel 3): install-sobre-limpia adelantado durante la espera;
  iterate-sobre-migrada en la convergencia.
- **`-u all` del iterate loop:** ejecutar y vigilar; parsear el log al terminar.
- No hacer espera pasiva: comprobar estado periódicamente.

**Serie inevitable**: neutralización e iterate `-u all` final (una sola BD, gate de seguridad).

## Checkpointing (reanudar ante crash)

Reutilizar `resume-mode.md`: tras cada fase completada, commit de checkpoint en el workdir
y línea en `~/.dynapps/migrations/<cliente>/session.log`. Una corrida autónoma interrumpida
se reanuda desde el último checkpoint sin rehacer.

## Bucle de aprendizaje (Dynbrain)

La corrida autónoma **lee** el catálogo de errores comunes en el pre-flight y en la
convergencia, y **escribe** en él cuando detecta un fallo del mecanismo que cumple el filtro
de admisión. Ninguna de las dos vías pide permiso ni detiene la corrida; si Dynbrain no está
disponible se registra y se sigue. Detalle en `dynbrain-feedback-loop.md`.

## Contrato de la corrida autónoma

0. Consultar el catálogo de errores comunes (Dynbrain) y llevarlo como checklist preventivo.
1. Resolver `scenario` (`scenarios.md`) + imprimir requerimientos (`requirements.md`) + 1 confirmación.
2. Correr BLOQUE 1 → 2 → 3 auto-avanzando, parando solo en los gates de arriba.
3. Al cumplir el criterio de aceptación → imprimir resumen del hito + URL local + parar.
4. Nunca escribir en GitLab/GitHub antes del hito.
