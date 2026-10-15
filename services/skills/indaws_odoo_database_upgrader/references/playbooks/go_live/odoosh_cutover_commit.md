# Playbook: Cutover odoo.sh con commit-tree

Disparar el go-live en odoo.sh cuando prod (`master`) y la rama destino tienen
estructura top-level divergente (monorepo vs submódulos). El commit que avanza
`master` es el que dispara `backup → upgrade.odoo.com → restore → -u`.

## Requisitos del contexto
- `target_deployment`: odoo_sh
- `to_version`: cualquiera
- Rama destino (`<target>`, p.ej. `staging18`) validada (build arranca, website
  sin HTTP500, forms clave) sobre BD ya migrada en un staging branch
- Simulacro local del glue/migración completado y con SHA conocido del submódulo
- Pestaña UPGRADE de la rama de producción armada → estado
  **"Waiting for upgrade code to be merged"**

## Aplicabilidad
- Setup de submódulos donde `git merge <target> → master` daría conflictos masivos
- Cualquier caso en que el árbol top-level de prod ≠ árbol del target validado

## Gates pre-cutover (BLOQUEANTES)

Ninguno puede fallar antes de tocar `master`:

| Gate | Cómo | Gotcha |
|---|---|---|
| SHA validado == desplegado | `git ls-tree <target> <ruta-submódulo>` == sha del simulacro local | #28 |
| OCA todos en versión destino | rama de cada submódulo OCA == `18.0` (all-or-nothing) | #29 |
| Estructura divergente confirmada | `git ls-tree --name-only origin/master` ≠ `git ls-tree --name-only origin/<target>` → usar commit-tree, NO merge | #27 |
| `ready` en verde | los 8 gates de `pre_go_live_gate.md` | — |
| prod_live preparado | `prod_live.sql` revisado (neutralize, desactivar menús crawler, preserve, bootstrap glue `installed <vprev>`) | G |

## Flujo

### 1. Verificar el SHA del submódulo en la rama destino
```bash
git fetch origin
git ls-tree origin/<target> <ruta-submódulo>     # p.ej. addons-pharmactive
# El sha mostrado DEBE == el commit validado en el simulacro local (no uno anterior con bug).
```
Si no coincide → bump del submódulo a la rama, push, y reempezar el gate. NO continuar.

### 2. Aplicar prod_live A MANO en el Shell de producción
En el Shell de la rama de **producción** de odoo.sh, ANTES del commit disparador:
```bash
psql $PGDATABASE -f prod_live.sql
# verificar exit 0 y que el bootstrap del glue quedó installed <vprev>:
psql $PGDATABASE -c "SELECT name, state, latest_version FROM ir_module_module WHERE name='<cliente>_migration';"
```
Así el backup automático que dispara el commit ya sale CON los fixes.

### 3. Armar el commit disparador con commit-tree
Sustituye el árbol de `master` por el del target validado, conservando la historia
de prod como primer padre (sin merge, sin conflictos):
```bash
TREE=$(git rev-parse origin/<target>^{tree})
NEW=$(git commit-tree "$TREE" -p origin/master -p origin/<target> \
      -m "[MIGRATION] master -> v18 (= <target> validado)")
```

### 4. Verificar ANTES de hacer push (doble check)
```bash
# (a) árbol idéntico al target → salida VACÍA:
git diff --stat "$NEW" origin/<target>
# (b) avance directo sobre master (odoo.sh lo acepta como "siguiente commit"):
[ "$(git rev-parse $NEW^1)" = "$(git rev-parse origin/master)" ] && echo "avance directo OK"
```
Si (a) no está vacío o (b) no imprime OK → NO hacer push. Revisar.

### 5. [STOP] PUNTO DE NO RETORNO — push
```bash
git push origin "$NEW":master
```
Esto dispara en odoo.sh: `backup (con prod_live) → upgrade.odoo.com → restore → -u`.
A partir de aquí no hay vuelta atrás salvo restaurar el backup previo.

### 6. Post-cutover
- Seguir el build en la pestaña de la rama de producción hasta `-u` completo.
- Tras arrancar: el glue figura `installed <vprev>` + código `<vdest>` → Odoo
  corre sus `pre/post/end-migration.py` (canal upgrade del glue).
- Smoke test: build arranca, website sin HTTP500, forms clave, PDFs sample.

## Helpers a invocar
- `git commit-tree`, `git ls-tree`, `git diff --stat` (no requieren el CLI del kit)
- `prod_live.sql` del workdir del cliente

## Pitfalls
- **Nunca** `git merge <target> → master`: conflictos masivos + árbol mezclado (#27).
- Desplegar el SHA del submódulo equivocado = desplegar la versión buggy del glue (#28).
- Dejar algún OCA en versión origen → el build no arranca en la versión nueva (#29).
- `UPDATE ir_ui_view SET active=true` masivo en prod_live/glue → HTTP500 del frontend.
  Reactivación SIEMPRE curada (#30).
- El staging branch de odoo.sh NO sirve para ensayar `prod_live` (coge copia fresca
  de prod y migra automáticamente, sin ventana). El ensayo es el simulacro local (F).
