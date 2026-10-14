# project-gantt — gotchas

Los dieciséis originales más los tres que salieron de ejecutar la skill el 2026-09-03. Viven aquí
porque son diagnóstico: se leen cuando algo no cuadra, no en cada activación.

## Los tres de la revisión de v1.3.0

**A. `LC_ALL` no es cosmética: sin ella el gantt emitía CSS inválido.** *(El generador clásico se retiró en v2.0.0; la lección vale para cualquier generador que use `printf` con decimales.)* `gen_gantt_report.sh` no
exportaba locale, así que `printf` usaba el del sistema. En `es_ES.UTF-8` eso produce **coma
decimal**: `left:1,57%`, que no es CSS válido. Medido sobre el proyecto real: **123 posiciones**
—etiquetas de mes, marcas de semana, posiciones de barra— todas inertes, y el encabezado de la
línea temporal colapsado al origen. Arreglado con `export LC_ALL=C.UTF-8`.

**B. Los nombres se recortaban más cortos de lo pedido.** En Debian/Ubuntu `awk` es **mawk**, sin
soporte multibyte: `substr(nm,1,40)` cuenta BYTES. Un nombre con `·` (2 bytes) y `—` (3 bytes) —o
sea, casi todos los de este parque— pierde unos 5 caracteres: `[DEV] 1/7 · Contenedores — el
estado` en vez de `...el estado no `. Medido: **29 nombres afectados** en el proyecto real. Y en
el caso peor el corte parte un carácter y deja el HTML en UTF-8 inválido (medido el 2026-09-02 en
`dev-panel`, byte 444275). Se prefiere `gawk` y, si no está, `clip()` cuenta caracteres — **con una TABLA de anchos, nunca con regex de rangos `\xNN`**: gawk compila el programa entero aunque esa rama no se ejecute, y bajo `C.UTF-8` esos rangos no compilan («Invalid collation character»), así que el generador moría en toda máquina con gawk. Medido el 2026-09-03 en gawk 5.3.0.

**C. El dato viene HTML-escapado; los sitios que lo pintan como TEXTO lo enseñaban literal.**
`esc()` mete entidades en los datos JS, que es lo correcto para `innerHTML` (gotcha 10). Las
vistas nuevas de v1.3.0 usan `textContent` / `createTextNode` / `.title`, donde la entidad se ve
tal cual: un item llamado `Defaults proveedor > usuario` salía como
`Defaults proveedor &gt; usuario` **en el tablero que se le enseña al cliente**. `dtxt()` deshace
la entidad justo en ese borde; decodificar hacia texto no puede inyectar marcado. Si añades una
vista, la regla es: `innerHTML` → dato tal cual; texto o `title` → `dtxt()`.

## Los dieciséis originales

1. `planned_date_begin`/`date_deadline` are **datetime** → cut to date (10 chars).
2. `user_ids` (m2m) returns **ids only** → resolve names with a separate `res.users/read`.
3. `state` v19: `01_in_progress`, `03_approved`, `1_done`, `1_canceled`… `1_done`=completed,
   `1_canceled` is struck through and doesn't count.
4. (retired with the classic gantt: dependency arrows) — kept as a numbering tombstone.
5. Odoo has no baseline: the baseline is OURS (first snapshot). Never overwrite it on refresh.
6. awk: pass /dev/null as input for BEGIN/END-only programs; never reuse arrays as scalars.
7. (retired with the classic gantt: its axis-label clipping) — the board's own axis gotchas
   live in references/board-spec.md.
8. If you change tareas.csv columns, update EVERY `$N` reader in the scripts: when the hours
   columns were dropped, the tag extraction kept reading `$15` and silently emptied
   etiquetas.csv (all tasks fell into "Sin área"). Fixed in v1.2.0 — the tags live in `$13`.
9. Board geometry (TODAY line per row, chip strip, unified track canvas, zoom that compensates
   width) is documented in [references/board-spec.md](references/board-spec.md) — read it BEFORE
   touching the board template's CSS.
10. Everything Odoo writes reaches the browser through `innerHTML`, so `esc()` MUST escape
   `&<>` on top of the JS string: a task named `</script>` closed the script block and killed
   the whole board. And the generated file is a REAL document — `<!DOCTYPE html>` (no quirks
   mode) and `<meta charset="utf-8">` (or the accents break on the client's machine).
11. A task with NO checklist has no measurable progress: it is grey ("Sin checklist"), never
   red. Scoring it 0% painted correct tasks as deviated. Counted once as an amber finding.
12. **No-hours policy covers the EMBEDDED DATA too**: `TSF` (execution light) carries only
   `[color, last_date]` — never emit timesheet hours into the HTML (view-source must stay clean
   for client publishing); the hours column of partes.csv stays in the local CSV.
13. Odoo returns the stage's SOURCE-language name while the web UI shows the translation
   ("To Merge" vs "Pasar a producción") → the pipeline bucket regexes map BOTH; when a stage
   seems missing, compare `date_last_stage_update` with what the user sees before blaming data.
14. "Technical task" = bracket name prefix `[X]` OR the Odoo tag "Desarrollo" (`const TDEV`,
   emitted by gen from a SEPARATE tag scan — the main-tag loop has a `break` that would skip it).
15. Chart palettes are **dataviz-validated on both themes** (brand hues darkened into the
   lightness band; `--lm` is a deliberate neutral for "Pendiente" that fails the chroma floor on
   purpose; pure carmine next to tangerine is indistinguishable, ΔE 10.7 → the stalled red is
   #D6336C). Do not "fix" chart colors by eye — rerun the validator from the dataviz skill.
16. The Pipeline toolbar (search + grouping pills) lives OUTSIDE `#pipebody` and only the body
   re-renders per keystroke — rebuilding the input on `oninput` loses keyboard focus.
17. **The Odoo 19 migration renamed every task stage to an English catalog** (New, Analysis,
   Implementation, Test, Acceptance, Release, Done); with the old regexes every technical task
   fell into En proceso. `tbk9` now maps both catalogs (table "Stage → bucket" in `board-spec.md`).
   **The dev-panel skill implements the same table in awk (`bucket()`)**: change one, change the
   other — the first version of this fix left Analysis and the Spanish names out of this one.
