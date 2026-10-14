# Code port patterns (cross-customer)

Catálogo vivo de patrones de port de código que surgen al migrar
módulos Odoo entre versiones. Esto es complemento a `odoo-module-upgrader`
(que aplica las transformaciones) y a `index.md` (gotchas de BD).

**Cuándo se usa**: cuando portas un módulo y te encuentras un error que
no es trivial. Antes de investigar de cero, busca aquí.

**Cómo se actualiza**: cuando arreglas un caso reproducible (afectará
a otro cliente), añade fila con `Symptom / Cause / Fix / First seen`.

---

## v16 → v19 patterns (origen: cliente Pedak)

| # | Symptom | Cause | Fix | First seen |
|---|---|---|---|---|
| 1 | `KeyError: 'tax_totals'` en plantillas de report | XPath `position="replace"` sobre `<t t-call>` elimina los `<t t-set>` hijos que pasan contexto al template llamado | Re-añadir los `<t t-set>` dentro del nuevo `<t t-call>` | Pedak PDK-314 |
| 2 | `KeyError: 'formatted_amount'` / `'formatted_amount_total'` en tax totals | v17+ eliminó strings pre-formateados del dict tax-totals; ahora amounts raw + widget monetary | Sustituir `subtotal['formatted_amount']` por `subtotal['base_amount_currency']` + `t-options='{"widget":"monetary","display_currency":currency}'`. Mismo para `total_amount_currency` | Pedak PDK-314 |
| 3 | `ValueError: Deprecated usage of 'default_res_id', should use 'default_res_ids'` | `mail.compose.message` wizard espera lista en v17+ | `"default_res_ids": self.ids` (era `"default_res_id": self.ids[0]`) | Pedak PDK-319 |
| 4 | `Error: Service user is not available` en componente OWL | `useService("user")` ya no existe; import distinto | `import {user} from "@web/core/user"`; llamar `user.hasGroup(...)` directamente | Pedak PDK-303 |
| 5 | OWL `Only one conditional branching directive is allowed per node` | XPath `position="attributes"` añade `t-if` a nodo que ya tiene `t-elif` | Extender la condición existente; no añadir segunda. `t-elif="(original) and (yours)"` | Pedak PDK-303 |
| 6 | Opción "Apply on" de selección custom desaparece en v19 UI | v19 introdujo `display_applied_on` para el radio en form view; extender solo `applied_on` deja la nueva opción invisible | Extender AMBOS campos con `selection_add`; añadir `_onchange_display_applied_on` para sync; data migration script para filas existentes | Pedak PDK-321 |
| 7 | `TypeError: ProductPricelistItem._compute_price() got an unexpected keyword argument 'section_id'` | v19 añadió `**kwargs` a la signature base; el override OCA es previo | Añadir `**kwargs` a la signature del override y forwardear a `super()` | Pedak PDK-320 |
| 8 | `EmptyNamesError` se renderiza como traceback raw al guardar contacto sin first/last name | `EmptyNamesError` hereda de `ValidationError` pero pone `self.args = (name, value)` (tupla 2 strings); el frontend OWL v19 renderiza la tupla como traceback | Override `_check_firstname_lastname` en módulo custom; lanzar `ValidationError(str)` plano con mensaje company-vs-individual | Pedak PDK-312 |
| 9 | Vista queda inactiva tras `v_N→v_(N+M)` upgrade pese a arch válido | Upgrade Service desactiva vistas cuya arch v_N falló validación v_(N+M); el module update posterior NO las reactiva | Añadir `post-migration.py` al módulo: `UPDATE ir_ui_view SET active=TRUE WHERE id IN (SELECT res_id FROM ir_model_data WHERE module='<name>')`. **Nota**: cubre el caso _genérico_ vía P6 del kit (`reactivate_disabled_views: true` en `post_restore_validator`) | Pedak PDK-314 |
| 10 | `partner_firstname` constraint bloquea onchange address-copy cuando type es custom (ej. `contact_own`) | v19 `onchange_parent_id` nativo solo copia parent address cuando `type='contact'`; no conoce extensiones custom del campo | Override `onchange_parent_id` en módulo custom; llamar `super()`, luego añadir copia address para el type custom | Pedak PDK-308 |
| 11 | Menú heredado sigue visible tras `util.remove_module` SQL-equivalente | `ir.ui.menu` tiene self-FK en `parent_id`; DELETE filas falla por FK; `DELETE FROM ir_model_data` NO cascadea a `ir.ui.menu` | Usar `util.remove_module(cr, name)` en `pre-migrate.py` (cascadea). Si replicas en SQL: CTE recursivo + loop borrando hojas primero (cubierto por validate_menus.phantom_menus en kit) | Pedak local repro |
| 12 | `ir.asset` filas huérfanas tras uninstall módulo, web assets rotos | Assets custom no se auto-limpian al desinstalar el módulo | `DELETE FROM ir_asset WHERE path LIKE '%<module>%'` en post-restore SQL. Genérico cubierto por validate_artifacts del kit | Pedak `date_range` |

---

## Cómo añadir una fila

Cuando arregles un ticket que podría afectar a CUALQUIER cliente
migrando entre versiones similares:

1. Elige el siguiente número.
2. Rellena las 4 columnas. Sé concreto en el fix recipe — pega el
   snippet si ayuda.
3. `First seen` = cliente + ticket. Ancla la fila a historia real.
4. Si el patrón es generalizable como helper de BD, añade nota
   "**cubierto por X del kit**" cuando se implemente.

## Qué NO va aquí

- Quirks específicos del cliente (van en su `particularities.md` /
  memoria del proyecto)
- Quirks del Upgrade Service / infra (van en `index.md` o
  `upgrade-api.md`)
- Patrones de **datos en BD** (van en `index.md` o como helper del
  kit P1-P8+)

## Distinción con `index.md` y el kit

- **Estos patrones**: aplican al **código fuente** del módulo (Python/
  XML/JS) y los resuelve el dev portando el archivo.
- **`index.md`**: gotchas de **datos en BD** del cliente migrado.
- **Helpers P1-P8+ del kit**: implementación ejecutable de los gotchas
  de BD; se aplican automáticamente desde `migration.yaml`.

Algunos patrones cruzan: ej. #9 (vistas desactivadas) se resuelve en el
código con `post-migration.py` pero también lo cubre genéricamente el
helper P6 del kit. El dev tiene 2 opciones (módulo custom vs flag del
kit), elige según contexto.
