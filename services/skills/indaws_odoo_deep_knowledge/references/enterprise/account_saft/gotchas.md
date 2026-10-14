<!-- verified: v18.0 @ 2026-06 (source: odoo_enterprise/18/account_saft + l10n_no/lt/ro/at/pl_saft + account_saft_import) -->
# account_saft — Gotchas (prefijo SAFTG)

Comportamientos no obvios del módulo base `account_saft` y su interacción con las localizaciones `l10n_*_saft`. Paths relativos a `odoo_enterprise/18/`.

---

## SAFTG1 — España NO usa SAF-T (no existe `l10n_es_saft`)

**Síntoma**: alguien propone "exportar SAF-T" a un cliente español, o busca el botón SAF-T en una instancia ES y no lo encuentra.

`account_saft` NO añade ningún botón por sí mismo: lo añade la localización `l10n_<país>_saft`, y **no existe ninguna localización SAF-T para España** (verificado: `ls account_saft l10n_*_saft` no incluye ES). España cumple con AEAT vía **SII** (tiempo real, SOAP) y **VeriFactu/TicketBAI**, no vía fichero SAF-T. Instalar `account_saft` en una company ES no hace nada útil: el botón solo aparece si `company.account_fiscal_country_id.code` coincide con un país soportado (`l10n_no_saft/models/account_general_ledger.py:16`, `GeneralLedgerCustomHandler._custom_options_initializer`). Para ES, dirigir al cliente a los módulos SII/VeriFactu.

---

## SAFTG2 — El módulo base no genera nada por sí solo

**Síntoma**: instalas `account_saft`, no aparece botón ni export en el Libro Mayor.

Es un módulo BASE intencionadamente incompleto (`account_saft/__manifest__.py:11`: *"meant to be used with localization specific modules"*). No declara ningún `account.report`, no añade botones, y las variables `xmlns`/`file_version`/`accounting_basis` se rellenan como `''`/`'undefined'` (`account_saft/models/account_general_ledger.py:421-424`, `GeneralLedgerCustomHandler._saft_prepare_report_initial_values`). Sin un `l10n_*_saft` instalado y la company con el `account_fiscal_country_id` correcto, no hay salida.

---

## SAFTG3 — SAF-T reutiliza el report del Libro Mayor, no tiene report propio

**Síntoma**: buscas un menú/acción "SAF-T Report" y no existe.

No hay registro `account.report` para SAF-T. El handler hace `_inherit = 'account.general.ledger.report.handler'` (`account_saft/models/account_general_ledger.py:11`, `class GeneralLedgerCustomHandler`) y el botón SAF-T se inyecta en el report **General Ledger** existente vía `_custom_options_initializer`. Se accede desde Contabilidad → Informes → Libro Mayor → botón "SAF-T". `data/saft_report.xml` son `<template>` QWeb, NO un `account.report`.

---

## SAFTG4 — Solo un column group permitido

**Síntoma**: `UserError: "SAF-T is only compatible with one column group."` al exportar.

`_saft_prepare_report_values` valida `len(options["column_groups"]) > 1` y aborta (`account_saft/models/account_general_ledger.py:412-413`, `GeneralLedgerCustomHandler.format_date`). Ocurre si activas comparación de periodos, multi-company en columnas, o cualquier opción que genere >1 column group en el report. Quita la comparación antes de exportar.

---

## SAFTG5 — Las queries son SQL crudo, no ORM (record rules NO se aplican igual)

**Síntoma**: aparecen en el XML asientos que esperabas filtrados, o el rendimiento difiere de la vista del report.

Los 4 fills usan `self._cr.execute(SQL(...))` con SELECT directos sobre `account_move_line` (`account_saft/models/account_general_ledger.py:169`, `GeneralLedgerCustomHandler._saft_fill_report_general_ledger_entries`, `:235`). El filtrado de seguridad/periodo viene de `report._get_report_query(options, 'strict_range')` (`from_clause`+`where_clause`), no de un `search` ORM. Si un módulo añade record rules vía ORM esperando que apliquen, NO se reflejan automáticamente: hay que tocar la query. Además se procesa en batches de `10**4` filas (`dictfetchmany(10**4)`), no todo de golpe.

---

## SAFTG6 — `xmlns`/`file_version`/`accounting_basis` los pone la localización, no el base

**Síntoma**: el XML sale con `<AuditFile xmlns="">` y versión vacía; la administración lo rechaza.

El base deja estas variables en `''`/`'undefined'` (`account_saft/models/account_general_ledger.py:421-424`, `GeneralLedgerCustomHandler._saft_prepare_report_initial_values`). Cada localización las sobreescribe en su `_l10n_<país>_prepare_saft_report_values` (NO en `_custom_options_initializer`). Ej. NO: `urn:StandardAuditFile-Taxation-Financial:NO` / `1.30` / `A` (`l10n_no_saft/models/account_general_ledger.py:66-68`, `GeneralLedgerCustomHandler._l10n_no_prepare_saft_report_values`); LT: `https://www.vmi.lt/cms/saf-t` / `2.01` / `K` (`l10n_lt_saft/models/account_general_ledger.py:41-43`, `GeneralLedgerCustomHandler._l10n_lt_saft_prepare_report_values`). Si haces un bridge país nuevo y olvidas el override, el fichero es inválido.

---

## SAFTG7 — `accounting_basis` (`TaxAccountingBasis`) NO significa lo mismo en cada país

**Síntoma**: la administración rechaza por base contable incorrecta, o se asume "devengo/caja" cuando el campo significa otra cosa.

El elemento `<TaxAccountingBasis>` es un código país-específico cuyo **significado cambia radicalmente**:
- **LT**: devengo vs caja. `K` = accrual (cuando se registra), `P` = cash (cuando se cobra) (`l10n_lt_saft/models/account_general_ledger.py:43`, `GeneralLedgerCustomHandler._l10n_lt_saft_prepare_report_values`). LT usa `K` fijo.
- **NO**: valor literal `'A'` (`l10n_no_saft/models/account_general_ledger.py:68`, `GeneralLedgerCustomHandler._l10n_no_prepare_saft_report_values`) — NO es "accrual", es el valor que pide el XSD noruego.
- **RO**: NO es devengo/caja en absoluto. Es el **tipo de plan de cuentas / regulación contable** de la empresa: `A` (CoA general de empresas), `I`, `IFRS`, `BANK`, `INSURANCE`, `IFN`, `NORMA39/36/14`... configurable en `res.company.l10n_ro_saft_tax_accounting_basis` (selección de 9 valores, `l10n_ro_saft/models/res_company.py:10-24`, `Company.l10n_ro_saft_tax_accounting_basis`). Obligatorio o el export RO bloquea.
- **AT**: el método de evaluación de beneficio `§ 4(1) EStG` / `§ 5 EStG` derivado de `company.l10n_at_profit_assessment_method` (`l10n_at_saft/models/account_general_ledger.py:94-97`, `GeneralLedgerCustomHandler._l10n_at_prepare_saft_report_values`).

No asumas un valor fijo ni que "basis" = devengo/caja: depende del esquema XSD nacional.

---

## SAFTG8 — Norway exige el código de impuesto estándar dentro del NOMBRE del impuesto

**Síntoma** (NO): `UserError: "Please change your tax descriptions to include their Norwegian standard tax code..."`.

`l10n_no_saft` extrae el "standard tax code" parseando el primer número que encuentra en la descripción del impuesto vía `re.split(r'\W+', ...)` (`l10n_no_saft/models/account_general_ledger.py:46-54`, `GeneralLedgerCustomHandler._l10n_no_prepare_saft_report_values`). Si no hay número → UserError. Hay que renombrar descripciones tipo `'33 Utgående mva lav sats 12%'`. Es un hack histórico (no hay campo dedicado; ver el `TODO` en `:45`).

**SOLO v18**: en v19 esto se sustituye por un campo dedicado `account.tax.l10n_no_standard_code` (ya no se parsea la descripción). Ver `v19_changes.md`.

---

## SAFTG9 — Norway: warning de upgrade si falta `l10n_no_saft.balance_account`

**Síntoma** (NO): `RedirectWarning` "The version for the SAF-T file has been updated. Please upgrade its module (l10n_no_saft)".

Al actualizar a SAF-T 1.30 se añadieron templates y el data record `balance_account`. El export comprueba `self.env.ref('l10n_no_saft.balance_account', raise_if_not_found=False)` y si no existe, fuerza un RedirectWarning a Apps para actualizar el módulo (`l10n_no_saft/models/account_general_ledger.py:30-40`, `GeneralLedgerCustomHandler._l10n_no_prepare_saft_report_values`). Pasa cuando se actualizó código pero no se actualizó el módulo en la DB.

---

## SAFTG10 — Partners sin Zip/City bloquean o se omiten (salvo `saft_allow_empty_address`)

**Síntoma**: error "Some partners are missing at least one address (Zip/City)" o partners ausentes del XML.

`_track_address` solo registra una dirección si `partner.zip and partner.city`, o si `options.get('saft_allow_empty_address')` está activo y no es la company (`account_saft/models/account_general_ledger.py:356-360`, `_track_address`, nested en `_saft_fill_report_partner_ledger_values`). Si un partner queda sin direcciones → se acumula en `missing_partner_zip_city` (`account_saft/models/account_general_ledger.py:394-398`, `GeneralLedgerCustomHandler._saft_fill_report_partner_ledger_values`). Noruega activa `saft_allow_empty_address` cuando `country_code == 'NO'` (`l10n_no_saft/models/account_general_ledger.py:27`, `GeneralLedgerCustomHandler._l10n_no_prepare_saft_report_values`) para tolerar partners sin dirección; otros países NO, así que ahí es bloqueante.

---

## SAFTG11 — La company DEBE tener al menos un contacto hijo

**Síntoma**: error bloqueante `missing_company_contact` "Please define one or more Contacts belonging to your company."

Si `partner == company.partner_id` y es `is_company` pero no tiene hijos `type='contact'` activos y no-company → error `level='danger'` que bloquea el export (`account_saft/models/account_general_ledger.py:368-375`, `GeneralLedgerCustomHandler._saft_fill_report_partner_ledger_values`). A diferencia de otros partners empresa (que toman solo el hijo de menor ID), la company necesita contactos explícitos como ContactPersons del fichero.

---

## SAFTG12 — `CustomerID`/`SupplierID` usan el ID interno de Odoo, no un código fiscal

**Síntoma**: el integrador externo o la administración espera un identificador estable y recibe el `res.partner.id` de Odoo.

En la plantilla, `CustomerID`/`SupplierID` se renderizan con `partner.id` / `move_vals['partner_id']` (`account_saft/data/saft_report.xml:158`, `saft_template` y `account_saft/data/saft_report.xml:178`, `saft_template` y `account_saft/data/saft_report.xml:236-237`, `saft_template` y `account_saft/data/saft_report.xml:246-247`, `saft_template`), es decir el ID numérico de base de datos. Si restauras un backup en otra DB o migras, esos IDs cambian. Las localizaciones que necesitan un identificador fiscal real (ej. RO) sobreescriben la plantilla para usar el VAT/registration number.

---

## SAFTG13 — `partner_detail_map['type']` (singular) es legacy; usar `types` (lista)

**Síntoma**: un partner que es cliente Y proveedor sale mal clasificado en una línea concreta.

El código mantiene dos campos: `type` (singular, `'customer'`/`'supplier'`) marcado como *"TODO: remove in master"* (`account_saft/models/account_general_ledger.py:284`, `GeneralLedgerCustomHandler._saft_fill_report_partner_ledger_values`, y `account_saft/models/account_general_ledger.py:326-328`) y `types` (lista, puede contener ambos). La plantilla usa `types` en la cabecera de transacción (`account_saft/data/saft_report.xml:236-237`, `saft_template`) pero `type` singular a nivel de línea (`:246-247`). Un partner mixto puede emitir `CustomerID` a nivel transacción y `SupplierID` a nivel línea de forma inconsistente si la lógica singular eligió mal (`customer_rank >= supplier_rank`).

---

## SAFTG14 — Líneas con debe=0 Y haber=0 igualmente emiten un importe según move_type

**Síntoma**: aparecen `<DebitAmount><Amount>0.00</Amount>` o `<CreditAmount>` en líneas a cero.

La plantilla fuerza emitir un Debit o Credit a 0.00 cuando ambos son cero, eligiendo cuál según `move_type` (`account_saft/data/saft_report.xml:77-93`, `line_debit_credit_amount`): `in_invoice`/`out_refund` → DebitAmount; `out_invoice`/`in_refund`/`entry` → CreditAmount. Es intencionado (algunos XSD exigen siempre uno de los dos), pero sorprende al revisar el XML.

---

## SAFTG15 — `rate` (tipo de cambio) se calcula como abs(amount_currency)/abs(balance), 1.0 si balance=0

**Síntoma**: ExchangeRate raro (o 1.0) en líneas multidivisa con balance contable cero.

`line_vals['rate'] = abs(amount_currency) / abs(balance) if balance else 1.0` (`account_saft/models/account_general_ledger.py:179`, `GeneralLedgerCustomHandler._saft_fill_report_general_ledger_entries`). Si una línea tiene `amount_currency` pero `balance == 0` (ajustes de cambio, redondeos), el rate cae a `1.0` aunque haya importe en divisa. Se renderiza con 8 decimales (`account_saft/data/saft_report.xml:66`, `line_debit_credit_amount`).

---

## SAFTG16 — RO/AT/PL añaden campos y modelos propios — no es "solo XML"

**Síntoma**: instalas `l10n_ro_saft` y aparecen campos nuevos en impuestos, facturas y activos.

A diferencia de NO/LT (casi solo plantilla), las localizaciones ricas añaden modelos y campos: `l10n_ro_saft` crea el modelo `l10n_ro_saft.tax.type`, añade `l10n_ro_saft_tax_type_id`/`l10n_ro_saft_tax_code` a `account.tax`, `l10n_ro_is_self_invoice` a `account.move`, categorías de activos, y `post_init_hook='_update_saft_fields_on_taxes'` (`l10n_ro_saft/__manifest__.py`, `l10n_ro_saft/models/account_tax.py:10-16`, `class AccountTax`). El export RO valida que TODOS los impuestos tengan tax_type y tax_code o bloquea (`account_saft/models/account_general_ledger.py:341-354`, `GeneralLedgerCustomHandler._saft_fill_report_partner_ledger_values`). AT carga el plan de cuentas oficial desde CSV (`l10n_at_saft.account.csv`). PL exporta en formato **JPK** (templates propios).

---

## SAFTG17 — El detalle de impuestos descarta líneas donde el signo de cuota ≠ signo de base

**Síntoma**: faltan importes de impuesto en el bloque `TaxInformation` de ciertas líneas (típicamente reverse-charge / inversión del sujeto pasivo o ajustes), aunque la línea existe en el Libro Mayor.

`_saft_fill_report_tax_details_values` filtra la query con `WHERE SIGN(tax_detail.tax_amount) = SIGN(tax_detail.base_amount)` (`account_saft/models/account_general_ledger.py:251`, `GeneralLedgerCustomHandler._saft_fill_report_tax_details_values`). Solo agrupa el detalle de impuesto cuando la cuota y la base tienen **el mismo signo**. Las líneas donde difieren (algunos casos de IVA de inversión, impuestos negativos sobre bases positivas, ajustes) NO se añaden a `tax_detail_vals_list` y por tanto no emiten su `TaxInformation`. Es intencionado para evitar dobles conteos con `_get_query_tax_details`, pero sorprende al cuadrar el XML contra los libros.

---

## SAFTG18 — `tax_base_amount` y `amount` salen de la query de tax details, NO del campo `tax_base_amount` de la línea

**Síntoma**: la base imponible que aparece en el XML no coincide con `account_move_line.tax_base_amount` de la línea de impuesto.

El base+cuota por impuesto se calcula con `account.move.line._get_query_tax_details(from, where)` (`account_saft/models/account_general_ledger.py:232`, `GeneralLedgerCustomHandler._saft_fill_report_tax_details_values`), que reparte base y cuota por par (base_line, tax) sumando `SUM(tax_detail.base_amount)` y `SUM(tax_detail.tax_amount)` agrupados por `base_line_id, currency_id, tax.id` (`:236-253`). NO usa el `tax_base_amount` almacenado en la línea (que sí se lee en el fill de asientos, `:25` doc). En facturas con varios impuestos sobre la misma base, o impuestos en cascada/grupos, el reparto de `_get_query_tax_details` puede diferir del `tax_base_amount` simple. Para cuadrar usa el detalle de tax details, no el campo directo.

---

## SAFTG19 — El nombre de la línea se sintetiza desde el producto si falta, y el `sign` por move puede sorprender

**Síntoma**: descripciones de línea tipo `[CODE] Nombre producto` que no escribiste, o signos de debe/haber inesperados en notas de crédito.

Dos comportamientos del fill de asientos (`_saft_fill_report_general_ledger_entries`): (1) si `account_move_line.name` está vacío, la descripción se construye desde el producto como `[default_code] product_name` (`account_saft/models/account_general_ledger.py:208-209`, `GeneralLedgerCustomHandler._saft_fill_report_general_ledger_entries`). (2) El `sign` que decide DebitAmount vs CreditAmount a nivel move es `-1 if move_type in get_inbound_types(include_receipts=True) else 1` (`:173,195`), combinado con SAFTG14 (líneas a cero emiten un lado según `move_type`). En refunds/notas de crédito esto invierte el lado esperado respecto a una factura normal. Revisar contra el esquema XSD del país.

---

## SAFTG20 — El `sign` del bloque `TaxInformation` se decide por línea (debe/haber), no por move

**Síntoma**: el `TaxAmount` de una línea sale con signo invertido respecto al que esperas al cuadrar contra el move.

A diferencia del `sign` a nivel transacción (SAFTG19, por `move_type`), el `TaxInformation` usa OTRO sign calculado en la plantilla: `sign = -1 if line_vals['credit'] else 1` (`account_saft/data/saft_report.xml:252`, `saft_template`). Es decir, depende de si la **línea base** tiene importe en el haber, no del tipo de movimiento. El mismo move puede emitir `TaxAmount` con signos distintos por línea. La cuota emitida es `format_float(sign * tax_vals['amount'])` (`:101`).

---

## SAFTG21 — `FirstName` siempre se emite como literal `"NotUsed"`

**Síntoma**: todos los ContactPerson del XML tienen `<FirstName>NotUsed</FirstName>`; el integrador externo pregunta por qué.

La plantilla hardcodea `<FirstName t-out="'NotUsed'"/>` (`account_saft/data/saft_report.xml:28`, `addresses_contacts`) y mete el nombre completo del contacto en `<LastName>` (cortado a 35 chars). Odoo no separa nombre/apellido en `res.partner`, así que es intencionado. El propio `account_saft_import` lo sabe: al reimportar ignora `FirstName` si vale `'NotUsed'` (`account_saft_import/wizard/import_wizard.py:209`, `SaftImportWizard._prepare_partner_data`). Si un esquema nacional exige FirstName real → hay que sobreescribir la plantilla.

---

## SAFTG22 — `CustomerID`/`SupplierID` a nivel línea se decide con `type` (singular legacy), a nivel transacción con `types`

**Síntoma** (refuerza SAFTG13): un partner cliente+proveedor emite `CustomerID` en la cabecera de la transacción y `SupplierID` (o nada) en la línea, o viceversa.

Confirmado en plantilla v18: cabecera `Transaction` usa `'customer' in partner_vals['types']` / `'supplier' in partner_vals['types']` y puede emitir AMBOS IDs (`account_saft/data/saft_report.xml:236-237`, `saft_template`); la `Line` usa `partner_vals['type'] == 'customer'` (singular, excluyente, `:246-247`). El `type` singular se fija con `customer_rank >= supplier_rank` (`account_saft/models/account_general_ledger.py:327`, `GeneralLedgerCustomHandler._saft_fill_report_partner_ledger_values`). Para un partner mixto, los dos niveles pueden ser incoherentes entre sí.

---

## SAFTG23 — RO: NO es "un botón SAF-T", son DOS exports con secciones distintas (D.406 mensual vs activos)

**Síntoma** (RO): aparecen dos botones SAF-T en el Libro Mayor y no sabes cuál usar; o el XML de activos no lleva asientos.

`l10n_ro_saft` añade DOS botones: `l10n_ro_export_saft_to_xml_monthly` ("D406 Declaration") y `l10n_ro_export_saft_to_xml_assets` ("D406 Asset Declaration") (`l10n_ro_saft/models/account_general_ledger.py:23-36`, `GeneralLedgerCustomHandler`). Cada uno fija `l10n_ro_saft_required_sections` (un dict de booleanos) que activa/desactiva secciones del XML: el mensual lleva GL entries, customers, suppliers, taxes, products, invoices, payments; el de activos solo lleva accounts + assets + asset_transactions (`:48-77`). RO incluso sobreescribe `_saft_fill_report_general_ledger_entries`/`_saft_fill_report_tax_details_values` para **saltar** esos fills si la sección no se pide (`:898-919`). v19 añade un tercero: `l10n_ro_saft_stock` (D.406 de stocks).

---

## SAFTG24 — RO: el export valida MUCHO más que NO/LT (taxes, products, partners, company) y bloquea

**Síntoma** (RO): el export falla con errores sobre CUI inválido, productos sin referencia interna, impuestos sin tax_type/tax_code, falta tax accounting basis, etc.

RO corre `_l10n_ro_saft_check_report_values` con 5 bloques de validación (`l10n_ro_saft/models/account_general_ledger.py:94-99`, `GeneralLedgerCustomHandler._l10n_ro_saft_check_report_values`):
- **Header**: tax accounting basis obligatorio, bank account, teléfono, VAT o CUI válido (valida CUI con `stdnum.ro.cui`).
- **Partners**: nombre (danger), city/country (warning), prefijo VAT = país, CUI/VAT válido para empresas, VIES.
- **Taxes**: `l10n_ro_saft_tax_type_id` + `l10n_ro_saft_tax_code` obligatorios (SAFTG16).
- **Products**: `default_code` obligatorio y único (**danger**); Intrastat code si el módulo está instalado.
- **Assets**: categoría de activo obligatoria; y si la company no está en RON → `UserError` directo (`:680-681`).

A diferencia de NO/LT (que casi solo validan dirección/contacto), RO puede bloquear por decenas de motivos de datos maestros. El `RegistrationNumber` de cada partner se construye con un prefijo de tipo (00/01/02/03/04/05/06...) según residencia y tipo (`:286-331`).

---

## SAFTG25 — RO: las líneas con `balance == 0` se ELIMINAN del XML (al contrario que el base)

**Síntoma** (RO): líneas que ves en el base no aparecen en el XML rumano; el conteo de líneas difiere.

`_l10n_ro_saft_clean_move_vals_list` (`l10n_ro_saft/models/account_general_ledger.py:888-896`, `GeneralLedgerCustomHandler`) filtra `line_vals_list` quitando toda línea con `not vals['balance']`, y luego quita moves que se quedan sin líneas. Esto **contradice SAFTG14** (el base fuerza emitir Debit/Credit a 0.00): en RO esas líneas a cero se descartan por completo. Comportamiento país-específico; no extrapolar el SAFTG14 a RO.

---

## SAFTG26 — AT: cada cuenta debe mapear a EXACTAMENTE un `l10n_at_saft.account` vía un tag numérico

**Síntoma** (AT): error "Some accounts can not be mapped to an account from the chart of accounts given in the SAF-T specification".

AT no usa el código contable directamente: lee los `account.tag_ids` del account, filtra los que son numéricos (`tag.name.isnumeric()`), y exige **exactamente uno** (`l10n_at_saft/models/account_general_ledger.py:41-49`, `GeneralLedgerCustomHandler._l10n_at_prepare_saft_report_values`). Con ese número busca en el modelo `l10n_at_saft.account` (CoA virtual oficial cargado desde `l10n_at_saft.account.csv`). Si la cuenta tiene 0 tags numéricos, >1, o el número no existe en la tabla virtual → se acumula en `accounts_with_mapping_problem` y bloquea. Además AT rechaza impuestos no-`percent` (`level='danger'`, `:85-92`) y exige `l10n_at_profit_assessment_method` + `l10n_at_oenace_code` en la company.

---

## SAFTG27 — LT: `_saft_get_account_type` revienta con KeyError ante un `account_type` no mapeado

**Síntoma** (LT): `KeyError` al exportar si alguien añadió un `account_type` nuevo o usa uno fuera del dict.

El override LT mapea con `return account_type_dict[account_type]` (acceso directo, sin `.get()`, `l10n_lt_saft/models/account_general_ledger.py:157`, `GeneralLedgerCustomHandler._saft_get_account_type`). El dict cubre los `account_type` estándar de Odoo, pero si aparece uno no contemplado (módulo custom, versión nueva) → `KeyError` no capturado, no un error de usuario limpio. RO en cambio tiene fallback a `Bifunctional` (`:605-606`) y NO devuelve `"GL"` siempre.

---

## SAFTG28 — Import: el wizard solo entiende el namespace genérico `urn:StandardAuditFile-Taxation-Financial`

**Síntoma**: importas un SAF-T noruego/austriaco/rumano/lituano con `account_saft_import` y falla o no encuentra nodos.

`_get_cleaned_namespace` (`account_saft_import/wizard/import_wizard.py:39-51`, `SaftImportWizard._get_cleaned_namespace`) busca en el `nsmap` una entrada que **empiece por** `urn:StandardAuditFile-Taxation-Financial` y la renombra a `saft`; si no la encuentra → `KeyError`/`None`. Es decir, el importador asume el esquema OCDE genérico (el que usa NO entre otros), NO los namespaces propios (`https://www.vmi.lt/...` de LT, `mfp:anaf:dgti:d406...` de RO, `urn:OECD:...AT_1.01` de AT). Importar exports nacionales con namespace distinto requiere un bridge `l10n_<país>_saft_import` que ajuste el parsing. Los bridges DK/LT/RO existentes solo sobreescriben `_get_account_types()`, no el namespace.

---

## SAFTG29 — Import: dedup por claves "naturales" → reimportar NO duplica, pero cambios de nombre SÍ

**Síntoma**: reimportas el mismo fichero y no pasa nada (bien); o renombras un partner/cuenta en el XML y se crea un duplicado.

El import deduplica por claves de negocio, no por ID SAF-T:
- Cuentas: por `code` contra el CoA existente (`:85`).
- Impuestos: por `(description, amount_type, amount)` (`:276-282`).
- Partners: por `(name, vat)` (`:182,196`).
- Moves: por xml_id `{company}_move_{journal}_{name}` (`:375-376`).

Reimportar idéntico es idempotente. Pero cualquier cambio en esas claves (renombrar cuenta, cambiar VAT, cambiar amount de impuesto) crea un registro nuevo en vez de actualizar. Los importados quedan con `ir.model.data` (xml_id `l10n_<cc>_saft_import.{company}_...`), por eso el re-match funciona entre ejecuciones.

---

## SAFTG30 — Import: fija `sequence.mixin.constraint_start_date` global para tragar fechas históricas

**Síntoma**: tras importar SAF-T, asientos nuevos en otros journals se quejan de huecos/fechas de secuencia; o el parámetro queda alterado.

`action_import` (`:470-477`) hace `set_param('sequence.mixin.constraint_start_date', <fecha del move más antiguo>)` ANTES de cargar, para que el constraint de secuencias de Odoo no rechace asientos con fechas pasadas que no casan con el año de su nombre. Es un parámetro **global de la company/sistema** que queda modificado tras el import (no se restaura). Si luego ves comportamiento raro en numeración de asientos, revisa este `ir.config_parameter`. La carga usa `skip_invoice_sync=True` para no duplicar líneas de impuesto.

---

## SAFTG31 — Import opcional de saldos de apertura crea un asiento de AJUSTE, no replica los OpeningBalance

**Síntoma**: marcas "Import account opening balances" y aparece un asiento "SAF-T opening balance move" con importes que no son los del XML.

`_prepare_opening_balance_move` (`:102-167`) NO vuelca los `OpeningDebit/CreditBalance` tal cual: calcula el saldo real ya existente en Odoo a la `SelectionStartDate` (SQL sobre `account_move_line`, excluyendo cancelados) y crea un asiento solo con la **diferencia** por cuenta (`balance_XML - balance_Odoo`), fechado HOY, en un journal `general`, usando `amount_currency` en la moneda por defecto del fichero. Si los saldos ya cuadran, no crea nada. Es un mecanismo de reconciliación de apertura, no una réplica literal.
