# Requirements — Dynapps Migrator

Antes de arrancar el modo autónomo, la skill **imprime el checklist del escenario
resuelto** y pide **una sola confirmación**. El objetivo es que el usuario sepa de
antemano qué necesita para llegar al hito **"Entorno local listo"** (ver
`autonomous-mode.md`).

Distinguir dos clases:
- **[BLOQUEANTE]** — sin esto no se puede construir/arrancar el entorno local. Si falta → STOP.
- **[INFO]** — necesario para CI/MR/go-live, que están **fuera del hito local**. Se lista
  para transparencia, pero **no bloquea** la corrida autónoma.

## Decisiones requeridas antes de arrancar

El pre-flight, además de los requerimientos técnicos, debe **recabar dos decisiones** (del
estudio previo del consultor si existe; si no, preguntarlas):

- **[DECISIÓN] Portar o no, por módulo.** Para cada módulo custom/OCA/terceros: **portar**
  (KEEP → se migra el código) o **no portar** (TO_DROP/NEUTRALIZE → stub para desinstalar). Sin
  esta decisión no se puede fijar el orden topológico ni saber qué entra al entorno migrado. Si
  el estudio previo la cubre, se auto-avanza; los módulos sin decisión clara caen a **STOP ④**.
  Presentar por lotes (todos los OCA juntos, todos los custom juntos) para minimizar prompts.
- **[DECISIÓN] Política de paridad 1:1 (default).** Ante features que cambian en la v-destino,
  el default es **mantener el comportamiento del cliente 1:1** (ver la regla en `SKILL.md` y
  `autonomous-mode.md`). El pre-flight lo declara explícitamente; desviarse es decisión de negocio.

## Común (ambos escenarios)

- **[BLOQUEANTE]** Docker (contenedor PostgreSQL en 5433 para el iterate loop).
- **[BLOQUEANTE]** `buildout` / `python-clone`, `python3`, `psql`, `pg_restore`, `git`.
- **[BLOQUEANTE]** `indaws-migrate` (CLI del migration kit) en PATH.
- **[BLOQUEANTE]** Dump de producción accesible (origen del upgrade).
- **[BLOQUEANTE]** **Suscripción enterprise dentro de la BD** — si falta, el Upgrade
  Service falla sin error claro (gotcha conocido).
- **[BLOQUEANTE]** Acceso a `dynapps-apps/*` (el `dyn-build.yaml` los descarga para
  construir el buildout; sin acceso, el build no levanta).

### Opcional — validación en navegador (BLOQUE 4)

- **[INFO]** **Playwright MCP** para la Capa 2b de `odoo-migration-parity`. **Nunca bloquea**:
  la Capa 2b es posterior al hito local y se omite si el MCP no está.
  ```bash
  claude mcp add playwright -- npx @playwright/mcp@latest \
    --headless --isolated --viewport-size 1440x900 \
    --allowed-origins "localhost:8069;127.0.0.1:8069" \
    --output-dir ~/.dynapps/migrations/_browser
  ```
  Requiere Node ≥18 y **reiniciar el proceso `claude`** para que aparezcan las herramientas
  `browser_*`. `--allowed-origins` no es cosmética: el entorno local es un clon de producción
  e impide que el navegador alcance un dominio del cliente.

## `onpremise_gitlab`

- **[BLOQUEANTE]** Sesión Vault/OIDC activa (STOP ① si no la hay: `! vault-login`).
- **[BLOQUEANTE]** Acceso SSH al servidor de BD de producción del cliente (para lanzar el
  Upgrade Service; el host exacto lo resuelve `dynapps-db-upgrade-runner`).
- **[BLOQUEANTE]** Acceso de lectura al repo `gitlab.dynapps.be:customers/<v>/<cliente>/odoo`.
- **[INFO]** Usuario **no External** en GitLab y acceso a `dynapps-apps/dyn-edi/*`:
  necesarios para que **el pipeline CI** pase (0 jobs si External). El **hito local NO
  depende del CI** → no bloquea; se resolverá al fase de MRs (posterior).
- **[INFO]** `develop` protegida (rebase por Maintainer): solo relevante al pushear (posterior).

## `odoosh_github`

- **[BLOQUEANTE]** Acceso al repo GitHub (`DynAppsNV/<cliente>` y/o `indawsodoo/odoo-migkit-<cliente>`).
- **[BLOQUEANTE]** `gh` autenticado.
- **[BLOQUEANTE]** Proyecto odoo.sh accesible (el upgrade es **nativo**, siempre `test`).
- **[INFO]** El go-live en odoo.sh es un cutover por commit (`odoosh_cutover_commit.md`),
  posterior al hito local.

## Formato del pre-flight (lo que ve el usuario)

```
Migración <cliente> v<origen>→v<destino> · escenario: <scenario>
Objetivo autónomo: entorno local levantado con código y BD migrados, sin errores,
REPRODUCIBLE por terceros (solo con el repo migrado + la BD del Upgrade Service).

Requerimientos [BLOQUEANTE]:
  [ ] Docker activo
  [ ] indaws-migrate en PATH
  [ ] Dump de producción: <ruta o "pendiente">
  [ ] Suscripción enterprise en la BD
  [ ] <específicos del escenario…>

Decisiones a confirmar:
  [ ] Portar/no-portar por módulo (del estudio previo, o te pregunto por lotes)
  [ ] Política de paridad 1:1 por defecto (mantener comportamiento del cliente)

Errores comunes ya conocidos (de Dynbrain, a evitar de antemano):
  - <N fichas del catálogo aplicables a este escenario/versiones>

Informativos (para MRs/go-live posteriores, no bloquean):
  - <External/dyn-edi | cutover odoo.sh…>

Me detendré solo en: vault-login, decisión test/prod del Upgrade Service,
gate de neutralización en rojo, y decisiones de módulo/rebase sin patrón.

¿Confirmas para arrancar? [Sí / faltan requisitos]
```

El bloque de **errores comunes** sale de consultar Dynbrain en el startup (ver
`dynbrain-feedback-loop.md`). Si Dynbrain no responde, se indica y se sigue: es prevención,
no un gate.

Los `[BLOQUEANTE]` que se puedan comprobar por comando se verifican automáticamente
antes de imprimir el checklist (marcados ✓/✗); los que no (accesos remotos) se listan
para que el usuario los confirme.
