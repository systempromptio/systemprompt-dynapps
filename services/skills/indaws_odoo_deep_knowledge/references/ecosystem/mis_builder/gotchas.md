<!-- verified: v18.0 @ 2026-07 (OCA mis-builder 18.0.1.8.3 — mis_builder; verificado vs $OCA_SRC/v18/mis-builder/mis_builder) -->
# OCA mis_builder — Gotchas (MISG)

Prefijo `MISG`. Anclado a `mis_builder/models/<fichero>.py → método` (v18.0.1.8.3).

## Expresiones / AEP

**MISG1. El modo por defecto es `p` (variación del periodo), no el saldo** — `mis_builder/models/aep.py:203-204`, `AccountingExpressionProcessor._parse_match_object`. `bal[70]`
es idéntico a `balp[70]` = variación del saldo EN el periodo, NO el saldo acumulado. Para saldo
acumulado a fin de periodo usa `bale[70]`; inicial `bali[70]`. Confundirlos es el error nº1 al montar un
balance (sale la variación en vez del saldo).

**MISG2. Selector de cuentas: `%`=comodín (`=like`), coma=lista, `[]`=TODAS, o un dominio** — 
`mis_builder/models/aep.py:176-226`, `AccountingExpressionProcessor._account_codes_to_domain`. `bal[70]` exacto; `bal[7%]` todas las que empiezan por 7; `bal[70,71,72]` lista;
`bal[]` **todas las cuentas** (fácil de disparar sin querer); `bal[('account_type','=','income')]` dominio
sobre `account.account`. El `%` va a `code =like`, sin `%` es `code =` exacto (no substring).

**MISG3. `bali`/`bale` calculan distinto en P&L vs balance según `include_initial_balance`** — 
`mis_builder/models/aep.py:354-375`, `AccountingExpressionProcessor.get_aml_domain_for_dates`. Para cuentas con `include_initial_balance=False` (P&L/resultado), inicial/final suman
**solo desde el inicio del ejercicio fiscal actual**; para cuentas de balance
(`include_initial_balance=True`) suman **desde el principio de los tiempos**. Corolario: `bale[6%]` (gasto)
te da el acumulado del AÑO, no de toda la vida de la cuenta. Si un P&L "no acumula lo que esperas",
es esto, no un bug.

**MISG4. Modo `u` (P&L no distribuido) = resultado de ejercicios ANTERIORES** — `mis_builder/models/aep.py:376-386`, `AccountingExpressionProcessor.get_aml_domain_for_dates`,
`get_unallocated_pl`. Suma desde el principio hasta el inicio del ejercicio del periodo, para cuentas
P&L (`include_initial_balance=False`). Es la línea "resultado de ejercicios previos" de un balance; no
lo uses para cuentas de balance (no tiene sentido ahí).

**MISG5. En modo inicial/unallocated, saldo CERO = AccountingNone (celda en blanco), NO 0** — 
`mis-builder/mis_builder/models/aep.py:476-480`, `AccountingExpressionProcessor.do_queries` y `mis-builder/mis_builder/models/aep.py:535-542`, `replace_expr.f`. Las cuentas con saldo inicial neto 0 se **omiten** de la query (`:476-480`) y un
resultado 0 se convierte en AccountingNone ("no tiene sentido distinguir 0 de sin-datos"). Por eso un
`bali[]` puede salir vacío donde esperabas 0.00. En modo `p`/`e` sí se distingue.

**MISG6. `fld` solo con modo `p` y OBLIGA `.fieldname`; los demás fields NO admiten fieldname** — 
`mis_builder/models/aep.py:252-275`, `AccountingExpressionProcessor.parse_expr`. `fldp.quantity[60%]` suma el campo `quantity` de los apuntes; `fldi...`/`flde...` →
`UserError`. Y `bal.quantity[...]` (bal con fieldname) → `UserError`. El campo debe existir en el modelo
de aml de la fuente o revienta la query (MISG14).

## AccountingNone y el eval sandboxed (las trampas de cálculo)

**MISG7. `AccountingNone` se comporta como 0 al comparar pero se DISUELVE en aritmética — salvo dividir
POR él, que lanza ZeroDivisionError** — `accounting_none.py`. `1 + AccountingNone == 1`,
`AccountingNone * 2 == 0.0`, `str(AccountingNone) == ''`, `AccountingNone == 0` es True. PERO
`2 / AccountingNone` **lanza ZeroDivisionError** (`:146-163`) → el KPI muestra `#DIV/0`. Consecuencia: un
ratio `kpi_a / kpi_b` donde `kpi_b` no tiene datos NO sale en blanco, sale `#DIV/0`. Protege los
denominadores (`kpi_a / (kpi_b or 1)` no sirve porque AccountingNone==0 es True → `or 1` lo salva; mejor
comprobar explícito) — o usa el estilo condicional.

**MISG8. Los errores de expresión NO explotan: se vuelven `#NAME` / `#DIV/0` / `#ERR` (estilo Excel)** — 
`mis_builder/models/mis_safe_eval.py:13-33`, `def mis_safe_eval`. Nombre no definido (KPI mal escrito, variable inexistente) → `#NAME`;
división por cero → `#DIV/0`; cualquier otra excepción → `#ERR`. Silencioso: no hay traceback en el
informe. Si un KPI referencia otro por nombre y sale `#NAME`, revisa el `name` técnico del KPI referido
(no la etiqueta). El eval es `safe_eval` con `_SAFE_OPCODES` — sin imports ni builtins peligrosos.

## Instancia / cálculo

**MISG9. `target_move` = "posted" por defecto — el filtro de estado lo pone la INSTANCIA, no el AEP** — 
`mis_builder/models/mis_report_instance.py:510-514`, `class MisReportInstance` (`[('posted','All Posted Entries'),('all','All Entries')]`, default
`posted`). El AEP (`aep.py`) NO filtra por estado del asiento: suma lo que matchee el dominio. El filtro
posted/all lo añade la instancia vía `_get_target_move_domain` como `additional_move_line_filter`
(`:744-745`). Corolario: un uso directo del AEP, o una fuente aml custom que no aplique ese filtro,
**suma también borradores**. Poner "All Entries" incluye drafts en TODAS las columnas.

**MISG10. La analítica se filtra por `analytic_domain` (texto = dominio literal), a nivel instancia Y
columna, + contexto** — `mis-builder/mis_builder/models/mis_report_instance.py:285`, `MisReportInstancePeriod.analytic_domain` y `mis-builder/mis_builder/models/mis_report_instance.py:376-399`, `MisReportInstancePeriod._get_additional_move_line_filter` y `mis-builder/mis_builder/models/mis_report_instance.py:564`, `MisReportInstance.analytic_domain` y `mis-builder/mis_builder/models/mis_report_instance.py:749-752`, `MisReportInstance._get_filter_domain`. Es un `Text` que se
`ast.literal_eval`-ea; se combina con AND al dominio de cada expresión vía
`_get_additional_move_line_filter`. Además se inyecta `mis_analytic_domain` desde el contexto. El hook
`_get_additional_move_line_filter` está pensado para **heredarse** (filtrar por unidad operativa,
dimensión analítica, etc.).

**MISG11. `accumulation_method` (SUM default / AVG / NONE) reajusta el KPI al periodo con PRO-RATA
TEMPORIS** — `mis_builder/models/mis_report.py:132-143`, `class MisReportKpi`. SUM suma periodos más cortos y **prorratea** los que solapan
parcialmente; AVG promedia ponderado por solape. Trampa clásica: un KPI de **saldo** (balance) con
accumulation SUM en una columna de comparación de distinta duración da números raros — los saldos
suelen querer AVG o NONE; los **flujos** (P&L, ventas) quieren SUM.

**MISG12. `type` (num/pct/str) y `compare_method` (diff/pct/none, default PCT) cambian formato y
comparación** — `mis_builder/models/mis_report.py:112-130`, `class MisReportKpi`. Un KPI `pct` se formatea como % y su columna de comparación por
defecto es porcentual. Si una comparación "sale en %" cuando querías diferencia absoluta, mira
`compare_method` del KPI.

**MISG13. `sumcol`/`cmpcol` NO consultan la BD: combinan otras columnas** — `mis-builder/mis_builder/models/mis_report_instance.py:21-24`, `SRC_ACTUALS` y `mis-builder/mis_builder/models/mis_report_instance.py:234-241`, `MisReportInstancePeriod.source`. `sumcol` suma columnas (via `source_sumcol_ids`); `cmpcol` compara dos (`source_cmpcol_from_id`/
`to_id`). Si una columna sum/cmp sale vacía, el problema está en las columnas fuente, no en una query.

**MISG14. `actuals_alt` corre el AEP contra OTRO modelo de move lines (budget, committed…)** — 
`mis-builder/mis_builder/models/mis_report_instance.py:250-270`, `MisReportInstancePeriod.source_aml_model_id` y `mis-builder/mis_builder/models/mis_report_instance.py:323`, `MisReportInstancePeriod._compute_source_aml_model_id`. La columna usa `source_aml_model_id` en vez de `account.move.line`.
Si una expresión o filtro referencia un campo que NO existe en ese modelo alternativo → 
`UserError "field does not exist in the model"` (`mis_builder/models/aep.py:461-471`, `AccountingExpressionProcessor.e`). Verifica que el modelo alt tenga
`debit`/`credit`/`account_id`/`date` y los campos que uses.

**MISG15. `auto_expand_accounts` explota el KPI en una fila por cuenta (pesado)** — `mis_builder/models/mis_report.py:99`, `class MisReportKpi` +
`no_auto_expand_accounts` a nivel instancia (`mis_builder/models/mis_report_instance.py:546`, `class MisReportInstance`). Usa
`replace_exprs_by_account_id` y solo saca cuentas CON datos (`has_data`, `mis-builder/mis_builder/models/aep.py:604`, `AccountingExpressionProcessor.replace_exprs_by_account_id`). En un plan de
cuentas grande genera muchas filas y queries por cuenta; desactívalo en la instancia si no lo necesitas.

**MISG16. Multi-compañía: sin `currency_id` explícito, todas las compañías deben compartir moneda o
UserError; conversión al rate de `date_to`** — `mis-builder/mis_builder/models/aep.py:146-157`, `AccountingExpressionProcessor.__init__` y `mis-builder/mis_builder/models/aep.py:389-403`, `AccountingExpressionProcessor._get_company_rates`. Un informe cross-company en
monedas mixtas exige fijar la moneda del informe; si no, `UserError "all companies must have the same
currency"`. La conversión usa el tipo de cambio a la fecha fin del periodo.

**MISG17. `smart_end` (default ON) hace DOS queries por saldo final** — `mis-builder/mis_builder/models/aep.py:166-170`, `AccountingExpressionProcessor.__init__` y `mis-builder/mis_builder/models/aep.py:433-436`, `AccountingExpressionProcessor.do_queries`. Para
devolver "en blanco" cuando de verdad no hay datos, el saldo final se calcula como inicial + variación (2
queries). En informes grandes es coste real; `aep.smart_end = False` lo hace de una (pierde el blanco
inteligente). Solo relevante si extiendes el AEP en código.

**MISG18. Un KPI `multi=True` (subKPIs) devuelve un SimpleArray, no un escalar** — `mis-builder/mis_builder/models/mis_report.py:88`, `MisReportKpi.multi` y `mis-builder/mis_builder/models/mis_report.py:670-705`, `MisReport._declare_and_compute_col`. Las expresiones operan elemento a elemento sobre el array de subvalores. Referenciar un KPI
multi como si fuera escalar en otra expresión da resultados inesperados o `#ERR`.
