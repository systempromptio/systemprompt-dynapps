# Widget de conciliación bancaria (EE) — extender el frontend OWL

> Las citas `file::symbol` son recibos: cuando símbolo y línea discrepan, gana el símbolo (la línea envejece).
> Snapshot v18 y v19 verificado contra fuente 2026-07. **Solo EE**: todo vive en `account_accountant`; en CE (`account`) no existe este widget (equivalente CE no revisado).
>
> **Corrección de premisa (verificada):** esto **NO es un `ir.actions.client`** ni hay registro bajo `registry.category("actions")` para la conciliación (el único `category("actions")` de `account_accountant` v19 es `journal_create_wizard`). Es una **`ir.actions.act_window`** que abre una **vista kanban custom** (`js_class="bank_rec_widget_kanban"`) sobre `account.bank.statement.line`. El "OWL client action" del título original es incorrecto: es un *view js_class*, no una client action. Igualmente **falsa** la nota de un rename de directorio `bank_rec_widget → bank_reconciliation`: ambas versiones viven en `.../components/bank_reconciliation/`; lo que cambia es la organización *dentro* de esa carpeta y la desaparición del modelo Python transitorio en v19.

## 1. Punto de entrada, "acción" y registro (idéntico patrón v18/v19)

- Método Python que la abre: `account.bank.statement::action_open_bank_reconcile_widget` → delega en `account.bank.statement.line::_action_open_bank_reconciliation_widget(extra_domain, default_context, name, kanban_first=True)` (`models/account_bank_statement.py::_action_open_bank_reconciliation_widget`). Devuelve una `ir.actions.act_window` con XML id `account_accountant.action_bank_statement_line_transactions_kanban` (o `..._transactions` si `kanban_first=False`), `domain=[('state','!=','cancel')] + extra_domain`, modelo `account.bank.statement.line`.
- Registro de la vista en JS — **clave idéntica en v18 y v19**:
  `registry.category("views").add("bank_rec_widget_kanban", BankRecKanbanView)`.
- Declaración `<kanban js_class="bank_rec_widget_kanban" ...>` en `views/bank_rec_widget_views.xml` (record `view_bank_statement_line_kanban_bank_rec_widget`) — **mismo nombre de fichero XML y de record en ambas versiones** (`views/bank_rec_widget_views.xml::view_bank_statement_line_kanban_bank_rec_widget`).

Bundle de assets: `web.assets_backend` (`__manifest__.py`) en ambas.

## 2. Componente raíz OWL + servicios

### v18 — todo en un fichero `bank_reconciliation/kanban.js`
- `BankRecKanbanView` = spread de `kanbanView` con `Controller: BankRecKanbanController`, `Renderer: BankRecKanbanRenderer`, `Record: BankRecKanbanRecord` (`static/src/components/bank_reconciliation/kanban.js::BankRecKanbanView`).
- Clase raíz de lógica: `BankRecKanbanController extends KanbanController`, `static template = "account.BankRecoKanbanController"` (`kanban.js::BankRecKanbanController`).
- Servicios: `useService("action")`, `useService("orm")`, `useService("ui")`. **No hay servicio dedicado de conciliación.**
- Para el formulario de edición de la línea instancia un modelo relacional aparte `BankRecRelationalModel` (de `./bank_rec_record`) con `resModel: "bank.rec.widget"` y llama `this.orm.call("bank.rec.widget", ...)`. El estado de UI vive en `this.state` (`useState`) del controller.

### v19 — dividido en varios ficheros + servicio nuevo
- `bank_reconciliation/kanban_renderer.js`: `BankRecKanbanView` (`Controller: BankRecKanbanController`, `Renderer: BankRecKanbanRenderer`), mismo registro `registry.category("views").add("bank_rec_widget_kanban", BankRecKanbanView)`; `BankRecKanbanRenderer extends KanbanRenderer`, `static template = "account_accountant.BankRecKanbanRenderer"` (`static/src/components/bank_reconciliation/kanban_renderer.js::BankRecKanbanView`).
- `bank_reconciliation/kanban_controller.js`: `BankRecKanbanController extends KanbanController`, `static template = "account_accountant.BankRecoKanbanController"`. En `setup()`: `this.orm = useService("orm")` y `this.bankReconciliation = useBankReconciliation()`, exponiendo `this.bankReconciliation.bus` por `useSubEnv` (`kanban_controller.js::BankRecKanbanController`).
- **SERVICIO NUEVO (clave del cambio v19):** `bank_reconciliation/bank_reconciliation_service.js` → `class BankReconciliationService`, registrado `registry.category("services").add("bankReconciliation", bankReconciliationService)`, `dependencies: ["orm", "batchedOrm"]`. Hook exportado `useBankReconciliation()` = `useState(useService("bankReconciliation"))`. Mantiene estado reactivo (chatter, `reconcileModelPerStatementLineId`, `availableReconcileLines`, …) y un `EventBus` (`bank_reconciliation_service.js::BankReconciliationService`, `::useBankReconciliation`).
- Árbol de subcomponentes muy ampliado (carpeta propia c/u): `button/`, `statement_summary/`, `line_to_reconcile/`, `control_action/`, `quick_create/`, `line_info_pop_over/`, `statement_line/`, `bankrec_form_dialog/`, `search_dialog/`, `apply_amount/`, `chatter/`, `file_uploader/`, `button_list/`, `list_view/`. Registros adicionales v19: vistas `bank_rec_list`, `bank_rec_dialog_kanban`, `bank_rec_dialog_list`, `bankrec_edit_line`; fields `bank_rec_list_many2one_multi_id`, `apply_amount_html`.

## 3. Punto de extensión (patrón real, ambos verificados)

El patrón es **`patch(Component.prototype, {...})`** sobre el controller exportado — **no** override de registro ni `t-inherit` para la lógica (el `t-inherit` es solo para la plantilla).

### v18 — import desde `.../bank_reconciliation/kanban`
Ejemplos: `account_accountant_batch_payment`, `account_accountant_fleet`.
```js
import { patch } from "@web/core/utils/patch";
import { BankRecKanbanController } from "@account_accountant/components/bank_reconciliation/kanban";
patch(BankRecKanbanController.prototype, {
    getOne2ManyColumns() { /* ... */ },
    _actionValidate(newState) { /* ... */ },
});
```
(`account_accountant_batch_payment/static/src/components/bank_reconciliation/kanban.js::patch(BankRecKanbanController.prototype)`, `account_accountant_fleet/.../kanban.js::patch(BankRecKanbanController.prototype)`)

### v19 — import desde `.../bank_reconciliation/kanban_controller` (¡el fichero cambió!)
Ejemplos: `account_accountant_batch_payment`, `account_fiscal_categories_fleet`.
```js
import { patch } from "@web/core/utils/patch";
import { BankRecKanbanController } from "@account_accountant/components/bank_reconciliation/kanban_controller";
patch(BankRecKanbanController.prototype, {
    get modelParams() { /* ... */ },
    getBankRecLineInvalidFields(line) { /* ... */ },
});
```
(`account_accountant_batch_payment/static/src/components/bank_reconciliation/kanban_controller.js::patch(BankRecKanbanController.prototype)`, `account_fiscal_categories_fleet/.../kanban.js::patch(BankRecKanbanController.prototype)`)

Además, v19 permite **parchear el servicio nuevo** para estado compartido:
```js
import { BankReconciliationService } from "@account_accountant/components/bank_reconciliation/bank_reconciliation_service";
patch(BankReconciliationService.prototype, {
    setup(env, services) { super.setup(env, services); /* ... */ },
});
```
(`account_accountant_batch_payment/static/src/components/bank_reconciliation/bank_reconciliation_service.js::patch(BankReconciliationService.prototype)`)

**Plantillas QWeb-OWL:** se extienden con `<t t-inherit="..." t-inherit-mode="extension">`. El id **cambia de prefijo**: v18 `account.BankRecoKanbanController` → v19 `account_accountant.BankRecoKanbanController` (`kanban.xml::account.BankRecoKanbanController` / `kanban_renderer.xml::account_accountant.BankRecKanbanRenderer`).

## 4. Modelo Python que lo respalda

### v18 — modelo transitorio en memoria `bank.rec.widget`
- `models/bank_rec_widget.py::BankRecWidget` — `class BankRecWidget(models.Model)` con `_auto = False` y `_table_query = "0"` (nunca se persiste; "Bank reconciliation widget for a single statement line") + `bank.rec.widget.line` (`bank_rec_widget_line.py`).
- Métodos clave: `new()`, `fetch_initial_data()`, `_onchange_todo_command()` (despacha a `_js_action_<name>()`). Acciones JS: `_js_action_mount_st_line`, `_js_action_validate`, `_js_action_to_check`, `_js_action_reset`, `_js_action_set_as_checked`, `_js_action_remove_line`, `_js_action_select_reconcile_model`, `_js_action_add_new_aml`, `_js_action_remove_new_aml`, `_action_validate`. **La lógica de conciliación de una línea vive en este modelo transitorio.**
- `account.bank.statement.line` (v18, `account_bank_statement.py`) aporta: `action_open_bank_reconcile_widget`, `_action_open_bank_reconciliation_widget`, `action_open_recon_st_line`, `action_save_close`, `action_save_new`, `_cron_try_auto_reconcile_statement_lines`, `_retrieve_partner`.

### v19 — `bank.rec.widget` y `bank.rec.widget.line` ELIMINADOS
- No existen `models/bank_rec_widget.py` ni `bank_rec_widget_line.py`; `grep bank.rec.widget` en `.py` solo aparece en manifest y tests.
- Toda la lógica se movió a `account.bank.statement.line` (`models/account_bank_statement.py::AccountBankStatementLine`). Métodos nuevos/clave: `_action_manual_reco_model(reco_model_id)`, `_get_counterpart_aml(...)`, `set_partner_bank_statement_line(partner_id)`, `set_account_bank_statement_line(aml_id, account_id)`, `_set_move_line_to_statement_line_move(...)`, `_add_move_line_to_statement_line_move(...)`, `_invoice_matching_post_process`, `_try_auto_reconcile_statement_lines`, `_cron_try_auto_reconcile_statement_lines`, `_partner_mapping`, `_match_accounts_query` (`account_bank_statement.py::_action_manual_reco_model`, `::set_account_bank_statement_line`).
- Los `_js_action_*` **desaparecen**: el frontend escribe directamente sobre `account.move` / statement line vía RelationalModel + servicio `orm`.

## 5. Delta v18→v19 para portar un patch

| Aspecto | v18 | v19 |
|---|---|---|
| Directorio frontend | `static/src/components/bank_reconciliation/` | **igual** (sin rename) |
| Fichero de import del controller | `.../bank_reconciliation/kanban` (un fichero: Controller+Renderer+Record+View) | `.../bank_reconciliation/kanban_controller` (Controller); Renderer+registro en `kanban_renderer` |
| Clase exportada | `BankRecKanbanController` | **mismo nombre** → los `patch(...prototype)` siguen valiendo salvo la ruta de import |
| Clave registro vista | `bank_rec_widget_kanban` | **sin cambios** |
| IDs plantilla OWL | `account.BankRecoKanbanController`, `account.BankRecKanbanRenderer`, `account.BankRecKanbanRecord` | prefijo `account.` → `account_accountant.` (actualizar todo `t-inherit`) |
| IDs acción / vista XML | `action_bank_statement_line_transactions(_kanban)`, `view_bank_statement_line_kanban_bank_rec_widget`, fichero `views/bank_rec_widget_views.xml` | **sin cambios** |
| `<t t-name="card">` en kanban | inline y lleno | **`<t t-name="card"/>` vacío** (la tarjeta la pinta el componente OWL) |
| Modelo Python destino | `bank.rec.widget` (transitorio) + `_js_action_*` | reescribir contra `account.bank.statement.line`; el transitorio y los `_js_action_*` ya no existen |
| Servicio compartido | no existe | `BankReconciliationService` parcheable (`.../bank_reconciliation/bank_reconciliation_service`) |
| Nuevos puntos de extensión | — | subcomponentes con carpeta propia (button, statement_summary, quick_create, search_dialog, apply_amount, chatter…) y vistas dialog (`bank_rec_dialog_list/_kanban`, `bankrec_edit_line`) |

**Receta de porte, en corto:**
1. Cambia el import: `kanban` → `kanban_controller`.
2. Si tocabas la plantilla, cambia el prefijo `account.` → `account_accountant.` en el `t-inherit`.
3. Reescribe cualquier `this.orm.call("bank.rec.widget", "_js_action_*", ...)` como escritura/llamada sobre `account.bank.statement.line` (métodos nuevos del §4).
4. Si compartías estado, muévelo a un `patch(BankReconciliationService.prototype, ...)`.
5. La clave de registro de vista y los XML id de acción/vista NO se tocan.

## Localización española (l10n_es)
En CE `addons/` (v18): `l10n_es`, `l10n_es_edi_facturae`, `l10n_es_edi_sii`, `l10n_es_edi_tbai`, `l10n_es_edi_verifactu` (+ pos). En OCA `l10n-spain` (v18) hay import de extractos (p. ej. `l10n_es_account_statement_import_n43`). **No verificado** que ningún módulo l10n_es ni de OCA l10n-spain parchee el widget de conciliación bancaria (no se abrió su JS).

## Ejecutado contra una instancia (banco `dyn_acc_ee`, 13/13 en v18 y v19)

Todo lo de arriba estaba verificado **contra la fuente**. Lo que la fuente no puede decir es qué
quedó registrado al instalar, que es lo que decide si un `patch` encuentra dónde engancharse.
Comprobado sobre v18 y v19 con `account_accountant` instalado, y **todo confirmó**:

- la acción es una `ir.actions.act_window` sobre `account.bank.statement.line` — **no** una client
  action;
- la vista kanban lleva `js_class="bank_rec_widget_kanban"` en el `arch` ya cargado;
- los tres XML id (`action_bank_statement_line_transactions`, `..._kanban`,
  `view_bank_statement_line_kanban_bank_rec_widget`) existen **en las dos versiones**;
- `bank.rec.widget` está en el registry de v18 y **no está** en el de v19;
- `<t t-name="card">` va **lleno en v18 y vacío en v19**;
- el prefijo de la plantilla OWL pasa de `account.` a `account_accountant.`.

**Una trampa del camino, por si alguien repite la comprobación:** las plantillas OWL de
`static/src` **no son registros `ir.ui.view`** — viajan en el bundle de assets. Buscarlas en
`ir.ui.view` devuelve cero y parece que no existen; hay que leer el fichero del módulo instalado.

## No verificado (honestidad)
- No se abrió el cuerpo completo de las plantillas XML OWL (`kanban.xml` v18 / `kanban_controller.xml` v19) más allá de `t-name` y `t-inherit`: sin diff interno de la estructura de tarjeta.
- No se verificó si algún módulo l10n_es / OCA l10n-spain parchea concretamente este widget.
- No se leyó `kanban_controller.js` v19 entero (solo `setup` y `modelParams`): el conjunto exacto de métodos `action*` del controller v19 no está confirmado.
- No se verificó end-to-end cómo v19 hace el `validate`/reconcile desde JS (qué métodos ORM concretos invoca sobre `account.bank.statement.line`); solo consta que el transitorio `bank.rec.widget` desapareció y la lógica está en `account_bank_statement.py`.
- No se comparó con CE (`account`): el widget es EE (`account_accountant`); no se revisó equivalente en CE.
