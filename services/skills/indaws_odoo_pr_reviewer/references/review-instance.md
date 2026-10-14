# Instancia efímera de review (Step 1)

Montar un entorno **aislado y desechable** para probar el PR, sin tocar nada de lo que tengas en
marcha. Ciclo cerrado: **worktree → instancia → probar → destruir**.

---

## 0. Por qué NO se hace checkout en el árbol montado

La tentación es `git checkout pr-42` en el repo que ya monta tu docker. No se hace, y no es una
cuestión de estilo:

- `${PROJECTS_ROOT:-/opt/PROYECTS}/indawsodoo/18` lo montan **13 proyectos del host** (remedido
  2026-08-17; **20 proyectos** montan algún major del repo). Un checkout ahí les cambia el
  código **a todos a la vez**, y a los que estén levantados en caliente (es un bind mount).
- Los repos compartidos suelen estar **en una rama en curso**, no en la estable. Medido en el parque
  el 2026-08-17: `indawsodoo/17` → `17-perf-import-phase3`, `indawsodoo/18` →
  `feat/portal-picking-delivery-state`, `AUTOSUR/addons-autosur` → `feature/autosur_repair`,
  `EUROLOTES18/desarrollos` → `fix/prosersa-invoice-report-albaran-groups`. Hacer checkout encima
  **te machaca tu propio trabajo sin avisar**. (`indawsodoo/19` estuvo en
  `feature/indaws-uom-refactor` y hoy está en `19.0` — **compruébalo, no lo asumas en ninguna de las
  dos direcciones**.)

Por eso todo el aislamiento se hace con `git worktree`: comparte el `.git`, no re-clona, tarda
segundos y se borra sin dejar rastro.

## 1. Localizar el repo del PR

**PR de Core** — siempre el mismo sitio:

```
$PROJECTS_ROOT/indawsodoo/<MAJOR>     (repo indawsodoo/indaws-odoo, rama <MAJOR>.0)
```

**PR de cliente** — no hay convención: cada proyecto monta su custom en un sitio distinto
(`./addons/custom`, `<proj>/addons`, `./desarrollos`, o módulos sueltos). Se **deriva del compose**
del proyecto:

```bash
docker compose -f <proyecto>/docker-compose.yml config \
  | awk '/^ *source: \//{s=$2} /^ *target: /{if(s!=""){print s" -> "$2; s=""}}'
```

y de cada `source` se saca su repo con `git -C <source> rev-parse --show-toplevel`.

> **Si el montaje NO es un repo git → abortar el review con mensaje claro.** En el parque, **16 de 22
> montajes de custom no están versionados en su sitio**. Si existe un PR es que hay repo en alguna
> parte; hay que localizarlo antes de seguir, no improvisar un directorio.

## 2. Traer el PR y crear el worktree

Se revisa **la rama del PR tal cual la entregó el dev** (no el merge result):

```bash
cd <repo>
git fetch origin pull/<n>/head:pr-<n>          # GitHub (indawsodoo/*). GitLab: merge-requests/<n>/head
git worktree add .worktrees/pr-<n> pr-<n>
git -C .worktrees/pr-<n> rev-parse --short HEAD   # anotar el SHA para el veredicto
```

El clon compartido **no se toca**: sigue en su rama y los proyectos que lo montan siguen igual.

## 3. Pinnear los demás repos montados (reproducibilidad)

El repo del PR no es el único que se monta. Si Core está en una rama en curso, **tu trabajo a medias
participa en el review**: el PR puede fallar por tu código o pasar gracias a él, y el veredicto diría
"verde en Docker limpio" siendo falso.

Por eso los repos que **no** se revisan se montan también por worktree, en su **rama estable**:

```bash
git -C ${PROJECTS_ROOT:-/opt/PROYECTS}/indawsodoo/18 worktree add --detach .worktrees/stable-18.0 18.0
```

`--detach` es obligatorio: git rechaza un segundo worktree de una rama que ya tiene checkouteada el
clon principal (`fatal: '18.0' is already used by worktree at ...`). Con `--detach` se ancla al mismo
commit sin ocupar la rama.

Los montajes de solo lectura que no son repos (Odoo CE, EE, OCA fuera del árbol de proyectos) se comparten tal cual:
no cambian y no hace falta duplicarlos.

**Anotar en el veredicto la referencia exacta de cada repo montado**: `Core 18.0@c72eafe`,
`PR pr-42@a1b2c3d`. Sin eso el review no es reproducible.

## 4. Crear la instancia efímera

Ubicación: **junto al repo que se revisa**.

```
${PROJECTS_ROOT:-/opt/PROYECTS}/indawsodoo/.reviews/pr-<n>/      ← PR de Core
<proyecto>/.reviews/pr-<n>/                    ← PR de cliente
```

Misma infra que un proyecto normal (`docker-deployment` asigna slot libre, puertos, subred,
volúmenes y nombres propios → cero colisión con lo tuyo), con **los mounts de los repos desviados a
sus worktrees**:

```bash
new.sh INDAWS_PR<n> 18.0 --path ${PROJECTS_ROOT:-/opt/PROYECTS}/indawsodoo/.reviews/pr-<n> \
  --mount ${PROJECTS_ROOT:-/opt/PROYECTS}/indawsodoo/18/.worktrees/pr-<n>:/addons/indaws
```

**Cliente: el desvío puede ser de varios mounts.** Algunos proyectos montan **módulos sueltos** de
dentro del repo (visto en AUTOSUR: cuatro mounts distintos a `addons-autosur/<modulo>`). El remapeo
respeta la subruta de cada uno: `addons-autosur/autosur_repair` → `<worktree>/autosur_repair`.

## 5. Datos

La estrategia **se decide en el Step 0 caso por caso y se deja escrita en el veredicto**. No hay
regla automática todavía: se elige a conciencia y se registra, para poder calibrar con el tiempo
(mismo criterio que el ledger de `manual-review-triggers.md`).

| Estrategia | Cuándo | Qué cuesta |
|---|---|---|
| BD limpia + módulos del PR | funcionalidad nueva, sin dependencia de datos reales | no ves migraciones ni efectos sobre datos existentes |
| Dump del cliente (neutralizado) | el comportamiento depende de sus datos, config o Studio | minutos de restore; en Core hay que **elegir** qué cliente |
| Clonar una instancia viva | el cliente ya está montado y con sus datos | clonas tu estado de desarrollo → **el gate de "instala en limpio" deja de probar nada**; hay que decirlo en el veredicto |

Con dump de cliente: **neutralizado siempre** (lo hace `restore.sh` por defecto). Nunca
`--no-neutralize` en un review.

## 6. Destruir — todo o nada

Al cerrar el review, en este orden:

```bash
destroy.sh INDAWS_PR<n> --yes    # down -v + carpeta + LIBERA EL SLOT
git -C <repo> worktree remove .worktrees/pr-<n>
git -C <repo> branch -D pr-<n>
```

Si se queda a medias tienes un proyecto zombi en el registro de slots y un worktree colgando. Los
worktrees de rama estable (paso 3) pueden quedarse: son baratos y se reutilizan entre reviews.

---

## Core vs cliente, resumido

| | PR de Core | PR de cliente |
|---|---|---|
| Dónde está el repo | siempre `indawsodoo/<MAJOR>` | hay que derivarlo del compose |
| ¿Es repo? | siempre | **16 de 22 montajes no lo son** → abortar |
| Forma del desvío | un mount → un worktree | N mounts, remapeo por subruta |
| Datos | hay que elegir contra qué cliente | el suyo; clonar sale a cuenta |
| Radio | 13 proyectos, hasta 136 módulos | 1 proyecto + sus deps a Core |
| Riesgo propio | `auto_install`: 75 de 160 módulos | mounts de Core en rama en curso |
