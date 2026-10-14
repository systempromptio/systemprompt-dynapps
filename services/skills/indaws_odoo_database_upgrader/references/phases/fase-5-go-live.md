# FASE 5 — Go-live

## Pre go-live

### `gate-checker` (Pattern B — antes de invocar `ready`)

Antes de ejecutar `ready`, lanza un sub-agente que cruza `gates.json` +
`playbook.md` + `audit.sqlite` para reportar el estado de los 8 gates:

```python
Agent(model="sonnet", description="Gate readiness check",
      prompt=<ver agent-tree.md §8 — gate-checker>)
```

Devuelve `{gates: [{id, status, blocker}], can_go_live, outstanding_actions}`.
Esto permite presentar al usuario un resumen tabulado SIN haber invocado
`ready` (que escribe estado en disco). Si `can_go_live: false`,
**[STOP] preguntar** cómo proceder con `outstanding_actions` antes de
seguir.

### Ejecución de `ready`

```bash
indaws-migrate ready
```

Evalúa 8 gates (3 auto + 5 ack):

| Gate | Tipo | Validación |
|---|---|---|
| `pipeline_clean` | auto | `run --apply` exit 0, sin warnings |
| `smoke_test_passed` | auto | Registry loaded + 0 CRITICAL |
| `comparison_with_ref` | auto + ack | `compare` sin críticos + firma humana |
| `studio_pending_resolved` | ack | Lista resuelta o aceptada por cliente |
| `pdf_validation` | ack | Cliente firmó samples PDF |
| `rollback_plan` | ack | `docs/rollback.md` existe y revisado |
| `security_review` | ack | ACLs/users/grupos validados |
| `data_freeze_communicated` | ack | Cliente notificado de freeze |

**Sin `ready` verde, NO go-live.**

Leer `references/playbooks/go_live/pre_go_live_gate.md` para el recetario.

## [STOP] Go-live

```bash
indaws-migrate go-live
```

Reevalúa gates (hard abort si alguno cae). Según `target_deployment`:

- **odoo.sh** → push a staging + trigger upgrade nativo (Test Upgrade → Phase 1 → commit trigger Phase 2 → Production)
- **indaws | custom** → `run --apply` con dump de producción + `aim=production` al Upgrade Service (1 uso por contrato)

Sella `playbook.md` con status `finalizada` + genera `docs/post_mortem.md`.

## odoo.sh con submódulos

Caso real Pharmactive v15→v18 (prod monorepo → target submódulos). El go-live técnico lo dispara la plataforma, pero la mecánica obliga a una secuencia precisa.

### Mecánica "Waiting for upgrade code to be merged"

En la pestaña **UPGRADE** de la rama de producción, tras armar el upgrade, odoo.sh deja la rama en modo especial esperando **UN** commit en producción. Ese commit (el siguiente que llegue a `master`) dispara la cadena `backup → upgrade.odoo.com → restore → -u`. El backup automático sale del estado de prod **en el momento del commit**.

### Orden obligatorio: prod_live manual → commit disparador

`prod_live.sql` (fixes pre-backup: neutralize OCA sin destino, desactivar menús que rompen el crawler de upgrade.odoo.com, preservar campos, bootstrap del glue como `installed <vprev>`) se aplica **A MANO en el Shell de producción ANTES** del commit disparador. Así el backup automático ya sale con los fixes aplicados. Secuencia:
1. Aplicar `prod_live.sql` en el Shell de prod (verificar que corrió OK).
2. Disparar el commit que avanza `master` (ver técnica commit-tree abajo).
3. odoo.sh hace backup (ya con fixes) → upgrade → restore → `-u`.

### Técnica commit-tree (estructura divergente)

`master` (monorepo) y la rama target (submódulos) tienen árbol top-level incompatible: un `git merge` da conflictos masivos. Se sustituye el árbol de `master` por el del target validado, preservando la historia de prod como primer padre:
```bash
TREE=$(git rev-parse origin/<target>^{tree})
NEW=$(git commit-tree "$TREE" -p origin/master -p origin/<target> -m "[MIGRATION] master -> v18 (= <target> validado)")
git diff --stat "$NEW" origin/<target>   # DEBE estar VACÍO (árbol idéntico)
[ "$(git rev-parse $NEW^1)" = "$(git rev-parse origin/master)" ] && echo "avance directo OK"
git push origin "$NEW":master            # PUNTO DE NO RETORNO
```
Receta paso a paso con verificaciones: `references/playbooks/go_live/odoosh_cutover_commit.md`.

### Gates pre-cutover (además de los 8 de `ready`)

| Gate | Cómo | Gotcha |
|---|---|---|
| SHA validado == desplegado | `git ls-tree <rama> <ruta-submódulo>` == sha del simulacro local | #28 |
| Estructura prod↔target detectada | comparar top-level (`git ls-tree --name-only`); si divergen → commit-tree, NO merge | #27 |
| OCA todos en versión destino | rama de cada submódulo OCA == `18.0` (all-or-nothing) | #29 |
| SHA exacto validado en staging | el staging de odoo.sh validó el CÓDIGO destino sobre una BD migrada (build arranca, website sin 500, forms clave) — confirmar que es el mismo SHA que se va a desplegar | #28, F |

**Nota (F):** el botón Upgrade de un **staging branch** coge una copia FRESCA de prod y la migra automáticamente → NO hay ventana para meter `prod_live` sobre esa copia. Por eso el ENSAYO del glue/migración es el **simulacro local** (restore dump v_prev → prod_live → upgrade.odoo.com test → restore → -u); el staging de odoo.sh solo sirve para validar el CÓDIGO destino sobre una BD ya migrada.

## Ready checklist final

Leer `references/playbooks/go_live/ready_checklist.md` para el checklist completo que el agente repasa con el usuario.
