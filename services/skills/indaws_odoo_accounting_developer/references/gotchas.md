# CONTABILIDAD — GOTCHAS (los que rompen en silencio)

> Citas `fichero::simbolo`: el **simbolo manda sobre el numero de linea** (anclados a snapshot
> v18/v19, las lineas pueden desplazarse al parchear la fuente; el simbolo es el recibo real).
> Verificado contra fuente Odoo **v18 y v19** (2026-07). Rutas relativas a `addons/account`
> salvo que se marque **EE** (`account_reports` / `account_accountant`, Enterprise).

1. **Editar `line_ids` a mano vs. el motor de sync — tus cambios se sobrescriben** — en
   `account.move.create`/`write` toda la logica pasa por `_sync_dynamic_lines` dentro de un
   `ExitStack`, que genera/borra/reescribe las lineas dinamicas (payment_term, rounding,
   discount, tax, epd). Ver `account_move.py::AccountMove._sync_dynamic_lines` (v18 L3258,
   v19 L3747) y `AccountMove.create` (v18 L3394-3411). Las lineas de impuesto y de vencimiento
   SON dynamic lines: si editas a mano una `account.move.line` de tipo `tax`/`payment_term` y en
   el mismo write cambia algo que las marca "dirty", `_sync_dynamic_line` (v18 L3126) recalcula
   `needed_after` y hace `to_write`/`to_delete`/`to_create`, machacando tu edicion. Sintoma:
   escribes el importe de una linea de IVA y al guardar vuelve al valor calculado. Fix: no editar
   lineas dinamicas directamente; usar `tax_totals` (ver #5); o escribir con contexto
   `skip_invoice_sync`/`skip_invoice_line_sync` SOLO si sabes lo que haces (`_disable_recursion`,
   v18 L3259).

2. **`amount_currency` vs `balance`: recalculo silencioso cruzado** — para lineas de factura,
   `account.move.line._sync_invoice` (`account_move_line.py` v18 L1577): si cambia `balance` (o
   `move_type`), la linea NO esta protegida y currency==company_currency, fuerza
   `amount_currency = balance`; y si cambia `amount_currency`/`currency_rate` y `balance` no esta
   protegido, hace `balance = round(amount_currency / currency_rate)` (v18 L1601-1615). Sintoma:
   fijas `balance` a mano en una linea en moneda extranjera y se recalcula solo desde
   `amount_currency/currency_rate` (o al reves). Fix: blindar con `env.protecting([field], line)`,
   o controlar `invoice_currency_rate` en la move (de donde sale `currency_rate`,
   `_compute_currency_rate` v18 L663).

3. **Constraint de signo `balance`/`amount_currency` (trap duro, facil de disparar)** — SQL
   constraint `check_amount_currency_balance_sign`: `balance` y `amount_currency` deben tener el
   MISMO signo (ambos <=0 o ambos >=0). v18: lista legacy en `account_move_line.py`
   `_sql_constraints` (L429-447). v19: migrado a atributo
   `AccountMoveLine._check_amount_currency_balance_sign = models.Constraint(...)` (L467). Sintoma:
   crear una linea con `balance>0` y `amount_currency<0` revienta con `IntegrityError` en el flush.
   **Trap v19-only:** existe un tercer `display_type` nuevo `line_subsection` (aparece en
   `_check_credit_debit` y `_check_amount_currency_balance_sign`, v19 L463-470); codigo que filtre
   `display_type in ('line_section','line_note')` NO cubre `line_subsection` en v19 -> lineas de
   subseccion tratadas como contables por error.

4. **`compute_all` sobrevive pero es un wrapper; `rounding_method=None` cambia totales** —
   `account.tax.compute_all(price_unit, currency=None, quantity=1.0, ..., rounding_method=None)`
   sigue presente (v18 `account_tax.py` L4177, v19 L4960) pero internamente delega en el motor
   nuevo: `_prepare_base_line_for_taxes_computation` + `_add_tax_details_in_base_line` +
   `_add_accounting_data_to_base_line_tax_details` (v18 L4231-4245). Trap: con
   `rounding_method=None` se usa `company.tax_calculation_rounding_method` (verificado en
   `_add_tax_details_in_base_line`, v19 L1764: `rounding_method = rounding_method or
   company.tax_calculation_rounding_method`). Round-per-line vs round-globally da totales
   distintos por el `float_round(raw_base, ...)` de `_get_tax_details` (v18 L1172-1173) y el
   redondeo por linea de `add_tax_amount_to_results` (v18 L1115-1116). Fix: en flujos nuevos usar
   la API de base_lines (`_prepare_base_line_for_taxes_computation` /
   `_add_tax_details_in_base_lines`), no `compute_all`; si usas `compute_all`, pasa
   `rounding_method` explicito. *(El caso numerico concreto round-per-line vs round-globally NO se
   ejecuto en un test; se basa en el `float_round` por linea y el docstring de `_get_tax_details`,
   no en un run.)*

5. **`tax_totals` es Binary computed CON inverse: escribir NO es no-op, pero solo ajusta importes
   de impuesto por delta** — `tax_totals = fields.Binary(compute='_compute_tax_totals',
   inverse='_inverse_tax_totals', exportable=False)` (`account_move.py` v18 L551, v19 L593).
   Contra el mito "escribir no hace nada": `_inverse_tax_totals` (v18 L2246, v19 L2531) recorre
   `subtotals`/`tax_groups` del dict y ajusta `first_tax_line.amount_currency -= delta*sign` SOLO
   por la diferencia de `tax_amount_currency` por grupo (L2262-2273). Traps: (a) solo toca
   importes de impuesto de grupos existentes; cambios de base o de estructura se ignoran en
   silencio; (b) el dict debe tener la forma exacta
   `{'subtotals':[{'tax_groups':[{'id','tax_amount_currency',...}]}]}` o casca; (c) en `create`,
   `tax_totals` se aplica aparte tras el super (v18 L3407-3409) y se excluye de
   `_get_protected_vals` (L3387-3388). Ademas, para asientos NO factura `_compute_tax_totals` pone
   `tax_totals = None` (v18 L1706-1708): acceder a `move.tax_totals[...]` en un asiento contable
   normal explota / da None en plantillas QWeb.

6. **`price_include` ahora es computed desde `price_include_override` + `company_price_include`** —
   `account.tax.price_include` es computed (no editable directo): `_compute_price_include`
   (`account_tax.py` v18 L261-267) => `price_include = (price_include_override=='tax_included') or
   (company_price_include=='tax_included' and not price_include_override)`. Trap: escribir
   `price_include=True` a pelo NO persiste; hay que setear `price_include_override`.
   `company_price_include` es related de `company_id.account_price_include` (v18 L131). En el motor,
   `special_mode`/`prepare_tax_extra_data` (v18 L1137-1152) decide si la base es incluida; con
   impuestos price-included y round_globally la base puede diferir vs round_per_line (advertencia
   explicita en docstring `_get_tax_details`, v18 L1100-1101).

7. **Repartition lines: `refund_repartition_line_ids` obligatorio y espejado, pero `account_id` /
   `tag_ids` NO tienen que coincidir** — constraint `_validate_repartition_lines` (`account_tax.py`
   v18 L507-542): `invoice_repartition_line_ids` y `refund_repartition_line_ids` deben tener MISMO
   numero de lineas, mismo `repartition_type` y mismo `factor_percent` en el mismo orden
   (L521-533), al menos una de tipo 'tax' cada una (L524-526) y factor total +100/-100 (L537-542).
   Olvidar el lado refund -> `ValidationError` RUIDOSO. El trap SILENCIOSO: la constraint NO obliga
   a que `account_id` ni `tag_ids` coincidan entre invoice y refund; si configuras solo el lado
   factura y dejas el refund con cuenta/tags por defecto, los abonos se contabilizan en
   cuenta/tags distintas SIN aviso. Ambas familias se autogeneran si faltan:
   `_compute_invoice_repartition_line_ids` / `_compute_refund_repartition_line_ids` (v18 L433-448).

8. **Motor de informe 'custom' (EE `account_reports`): `has_sublines` default False y forma del
   dict** — el engine custom despacha a `_report_custom_engine_<subformula>` via
   `_get_custom_report_function(formula, 'custom_engine')`
   (`account_reports/models/account_report.py` EE L2594) en
   `_compute_formula_batch_with_engine_custom` (L4352-4360). Para linea no-groupby la funcion debe
   devolver un dict del que se hace `safe_eval(result_value_key, result)` con
   `result_value_key='result'` -> `inject_formula_results` (L3342-3377). **Trap 1 (ruidoso):** si el
   dict no trae la clave `'result'`, `safe_eval('result', dict)` lanza `NameError` NO capturado
   (solo se capturan `ValueError`/`SyntaxError`, L3368/L3375) -> traceback. **Trap 2 (silencioso):**
   `has_sublines` es opcional y default False (`result.get('has_sublines')`, L3377; documentado
   L3338-3339). Si tu engine devuelve datos agrupables pero olvidas `has_sublines=True`, la linea se
   pinta SIN triangulo de desplegar y el usuario nunca ve los sub-resultados. Firma obligatoria:
   `(expressions, options, date_scope, current_groupby, next_groupby, offset=0, limit=None,
   warnings=None)`; el nombre debe empezar por `_report_custom_engine_` o `_get_custom_report_function`
   lanza `UserError` (L2601-2602). *(Verificado en EE v18; EE v19 asumido igual, NO reverificado. El
   despacho `custom` de CE tampoco se abrio.)*

9. **EDI / envio: errores tragados a chatter cuando corre desde cron** —
   `account_move_send.py::AccountMoveSend._generate_and_send_invoices` (v18 L729) calcula
   `allow_raising = not from_cron and not allow_fallback_pdf and allow_raising` (L751). En
   `_hook_if_errors` (L448-460): si `allow_raising` es False, NO hace `raise UserError`, sino
   `move.message_post(body=...)` con el error y deja `sending_data={'error': True}` (L765-766).
   Sintoma: el cron de envio/EDI "termina bien" pero la factura no se envio y el error solo esta en
   el chatter. Fix: revisar `sending_data`/mensajes del chatter, no fiarse del retorno del cron.
   `_format_error_text`/`_format_error_html` (L313-338) formatean el dict
   `{'error_title','errors':[...]}`.

10. **`_post(soft=True)` por defecto: moves con fecha futura quedan en borrador, no posteados** —
    `account.move._post(self, soft=True)` (`account_move.py` v18 L5101, v19 L5524). Con `soft=True`
    (default) los moves con `date > hoy` se filtran a `future_moves`, se les pone
    `auto_post='at_date'` y quedan FUERA de `to_post` (v18 L5224-5231). Sintoma: llamas
    `move._post()` esperando `state=='posted'` y sigue en `'draft'` porque su fecha es futura. Fix:
    `_post(soft=False)` para postear ya. Ademas la asignacion de numero/secuencia ocurre via el
    campo computed `name` (`_compute_name`, v18 L929-945: solo hace `_set_next_sequence()` si
    `state != 'draft'`) disparado por el write `state='posted'` (v18 L5260). **Corolario:** si
    sobreescribes `_post` sin `super()`, NO se escribe `state='posted'`, NO se asigna secuencia, NO
    se reconcilian reversas (`_reconcile_reversed_moves`, L5265), NO se crean analytic lines (L5242)
    ni el hash — todo omitido en silencio.

11. **Lock date: la fecha contable se mueve sola al postear** — en `_post`, por cada move a postear:
    `lock_dates = move._get_violated_lock_dates(move.date, affects_tax_report)` y si hay,
    `move.date = move._get_accounting_date(...)` (`account_move.py` v18 L5235-5239). Sintoma: posteas
    con fecha X dentro de periodo bloqueado y el asiento acaba con fecha distinta (siguiente periodo
    abierto) sin mas aviso que el recalculo. Tambien en `copy_data` (v18 L3328-3330): si la fecha de
    la copia <= `user_fiscal_lock_date`, se fuerza `date = lock_date + 1 dia`. Fix: contar con que
    post/copy puede reasignar `date`.

12. **`tax_totals` excluido de proteccion en create; `is_manually_modified` reseteado** — en `create`
    (`account_move.py` v18 L3407-3410) tras el super se aplica `move.tax_totals = vals['tax_totals']`
    explicitamente y luego `moves.is_manually_modified = False`. `_get_protected_vals` (v18 L3384-3392)
    salta explicitamente `tax_totals` ("updated explicitly after create/write"). Trap: si dependes de
    `is_manually_modified` para logica propia, create lo pone a False siempre; y el valor de
    `tax_totals` que pases en create no se protege como el resto de computed-con-inverse. *(El uso
    downstream de `is_manually_modified` que lo convierta en trap real no se verifico; es la
    observacion del reset.)*

13. **`check_field_access_rights` oculta campos "weirdos"** —
    `account.move.check_field_access_rights` (`account_move.py` v18 L3307-3312) elimina del resultado
    `needed_terms`, `quick_encoding_vals`, `payment_term_details` cuando se piden todos los campos.
    Trap: un read generico de "todos los campos" no los devuelve; si tu codigo los espera en un read
    masivo, faltan en silencio.

14. **`_sanitize_vals` fusiona `invoice_line_ids` dentro de `line_ids`** — `account.move._sanitize_vals`
    (`account_move.py` v18 L3355-3375): si vals trae a la vez `invoice_line_ids` y `line_ids`, fusiona
    los UPDATE de `invoice_line_ids` en `line_ids` y BORRA `invoice_line_ids` del vals (L3374
    `del vals['invoice_line_ids']`). Se llama al inicio de create y write (L3402, L3416). Trap: escribir
    ambos campos a la vez con comandos que se pisan produce un merge no obvio; `invoice_line_ids` es
    solo un subconjunto de `line_ids` (comentario en la def del campo, v18 L339). `Command.SET`/`CLEAR`
    sobre `invoice_line_ids` da assert (L3371).

15. **`account.move.line.partner_id` NO es el partner de la factura: es SIEMPRE el comercial** —
    `account_move_line.py::AccountMoveLine._compute_partner_id` (v18 L487-489, v19 L533-535) hace
    literalmente `line.partner_id = line.move_id.partner_id.commercial_partner_id`. Es decir: con una
    factura emitida a un contacto hijo (dirección de facturación/envío), `move.partner_id` es el HIJO
    y `line.partner_id` es la MATRIZ. Sintoma: una validación que recorre `move.line_ids` y una
    exportación/informe que lee `move.partner_id` **leen contactos distintos** y se contradicen — la
    validación da por bueno un dato que está en la matriz mientras la salida exporta el del hijo (o al
    reves), y ningun test lo ve si los fixtures no tienen jerarquia de contactos. Aplica igual a
    campos `company_dependent` leidos sobre el partner (`property_account_receivable_id`, subcuentas
    externas, condiciones de pago): el `with_company()` no salva la discrepancia, porque el partner ya
    es otro. **Fix:** elegir UNA fuente y usarla en los dos lados — normalmente
    `move.commercial_partner_id` (campo estandar, `account_move.py`), que es lo que ya usan las lineas.
    *(Medido 2026-08-13 en un export contable real: el check de cabecera pasaba y el fichero salia con
    la cuenta generica.)*

---

## Diferencias v18 vs v19 relevantes

- **Arquitectura de sync IDENTICA:** `_sync_dynamic_lines`, `_sync_tax_lines`, `_sync_dynamic_line`,
  `_inverse_tax_totals` iguales en v18 y v19 (verificado por presencia de simbolos: v19
  `account_move.py` L3747 / L3267 / L2531).
- **Constraints de `account.move.line`:** v18 usa `_sql_constraints = [...]` (lista); v19 usa
  `models.Constraint(...)` por atributo (`_check_amount_currency_balance_sign`, `_check_credit_debit`,
  etc.). Nuevo `display_type` `line_subsection` en v19 (ver #3).
- **Widget de conciliacion bancaria (EE `account_accountant/static/src/components/bank_reconciliation`):**
  fuertemente reestructurado v18->v19 (v18: `kanban.js`, `list.js`, `amls_list_view.js`; v19:
  `kanban_renderer.js`, `list_view/`, `search_dialog/`, `bankrec_form_dialog/`,
  `bank_reconciliation_service.js`). Los NOMBRES de registry sobreviven en ambas
  (`bank_rec_widget_kanban`, `bank_rec_list`, `bank_rec_list_many2one_multi_id`) pero las RUTAS de
  modulo cambiaron. Un `patch()` que importe desde
  `@account_accountant/components/bank_reconciliation/kanban` (v18) NO resuelve en v19 (el archivo es
  `kanban_renderer.js`) -> **error de bundle RUIDOSO, no un no-op silencioso** (por eso NO es un gotcha
  de fallo silencioso, solo un aviso de migracion).

---

## No verificado (honestidad — no se dio por bueno)

- **EDI localizacion ES** (`l10n_es_edi_sii` / `_tbai` / `_verifactu` / `_facturae`): los modulos
  existen en fuente pero NO se abrio su codigo; no se puede afirmar como tragan/propagan errores
  concretos TicketBAI/Verifactu/SII mas alla del framework generico `account_move_send` (#9).
- **`edi.document` / `account_edi_document`** (framework EDI legacy pre-v16): en v18 el envio pasa por
  `account_move_send`; NO se confirmo si `account_edi.document` sigue vivo como capa separada.
- **round-per-line vs round-globally**: caso numerico NO ejecutado (ver #4).
- **Motor `custom` en EE v19 y en CE**: verificado solo en EE v18 (ver #8).
- **`reconcile()` / `_reconcile_plan`** (`account_move_line.py` v18 L3207/L2590): solo se localizaron
  los simbolos; NO se verifico como fuente de fallo silencioso.

---

## §medidas — lo que salió al ejecutarlo (banco `dyn_acc`, v18 y v19, 14/14)

Estas dos no se descubren leyendo la fuente: la fuente dice el mecanismo, no la
consecuencia. Banco en `tools/bench-orm/addons/dyn_acc`.

### El redondeo de impuestos cambia la BASE, no solo el total

La gotcha 4 dice que `round_per_line` y `round_globally` «dan totales distintos».
Medido sobre la misma factura —40 líneas de 0,155 al 21%— sale algo más gordo:

| | base imponible | IVA | total |
|---|---|---|---|
| `round_per_line` | **6,40** | 1,20 | **7,60** |
| `round_globally` | **6,20** | 1,30 | **7,50** |

La base difiere en 20 céntimos porque `round_per_line` redondea el **subtotal de
cada línea** antes de sumar (0,155 → 0,16), así que no es un detalle de
presentación del total: es otra base declarada. En un modelo 303 eso se nota.

**Y la trampa al comprobarlo:** una llamada suelta a `compute_all` **NO distingue
los dos métodos**. Con una sola línea no hay nada que acumular y los dos devuelven
lo mismo, así que quien intente verificar la gotcha así concluye —razonablemente,
y mal— que es falsa. La diferencia necesita varias líneas.

### Una línea de impuesto escrita a mano SÍ se guarda

La gotcha 1 dice que `_sync_dynamic_lines` es un `@contextmanager` y que «no hay
vía de escritura que no pase por dentro». El mecanismo es cierto; la lectura
tranquilizadora que invita a hacer, no.

Medido: escribir `balance = -999.0` en la línea de impuesto de una factura en
borrador **se guarda**. Sobrevive al `flush`, se relee de la base tal cual, y ahí
se queda. Lo que pasa por el contextmanager es la **materialización**, no toda
escritura. El valor inventado solo desaparece cuando algo toca una línea
editable: entonces el sync recalcula y lo borra.

O sea que una factura en borrador puede quedarse indefinidamente con un impuesto
que nadie calculó. Si validas asientos de terceros, la comprobación no es «¿pasó
por el sync?» sino recalcular y comparar.
