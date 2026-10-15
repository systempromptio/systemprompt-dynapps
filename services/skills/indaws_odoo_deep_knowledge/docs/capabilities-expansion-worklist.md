# Worklist — ampliar/profundizar capabilities.md

## Estado verificación Tramos A+B+C (2026-07-19)
- 20 dominios, ~168 filas nuevas verificadas contra fuente v18 (Workflow, 1 agente/dominio, adversarial CE/EE/OCA).
- Resultado: **0 REFUTED**. 13 PARTIAL (imprecisión de origen/ancla, no de capacidad) → **corregidos y re-verificados CONFIRMED**.
- Patrón de error dominante: 7× módulo CE marcado como EE (sale_timesheet_margin, project_timesheet_holidays, hr_timesheet_attendance, sale_expense_margin, hr_recruitment_sms/survey). Corregido.
- `cite_audit --methods` limpio (solo account_consolidation v17-only, ya excluido). Sin commit (KB sin git en esta ruta).
## Estado Tramo E (2026-07-19) — ✅ CERRADO Y VERIFICADO
- Verificación semántica adversarial: Workflow `verify-capabilities-tramo-e` (run wf_10370a80-7df, 15 agentes, contexto fresco, CE/EE/OCA/indaws por RUTA real vs fuente). Resultado: **0 REFUTED**, **5 PARTIAL** (todos imprecisión de ancla/línea, ninguno de capacidad) → corregidos y re-gate OK:
  - SVCAP17: eLearning no usa `survey` para el quiz (es `slide.question` nativo); el puente `website_slides_survey` (CE) es para la **certificación**. Reescrito.
  - SVCAP18: `session_show_leaderboard` se define en `survey/models/survey_survey.py:158`, `class Survey` (no `:325`, que es la asignación del compute).
  - EVTCAP18: `product_id` de la entrada se define en `event_product/models/event_type_ticket.py:20`, `EventTemplateTicket.product_id` (heredado); `event_sale/models/event_ticket.py` solo lo usa. Re-anclado + link a `sale_order_line.py`.
  - SLDCAP17: valor `enroll='payment'` lo añade `website_sale_slides` (CE) vía `selection_add` (`:16-18`), no `website_slides/models/slide_channel.py:379`, `Channel.enroll` (solo público/invite).
  - SLDCAP19: el campo funcional real es `slide_category='certification'` (`selection_add` `:50-52`), no `slide_type` `:51`.
- `cite_audit --methods` global limpio (solo account_consolidation 5, v17-only). `gen_registry`+`gen_index` regenerados (90 dominios). Sin commit (KB sin git).

## (histórico) Estado Tramo E (2026-07-19) — ESCRITO + GATE OK · VERIFICACIÓN SEMÁNTICA PENDIENTE (sesión fresca)
- 15 dominios, ~90 filas nuevas. Gate `cite_audit --methods` global limpio (solo account_consolidation v17-only). Sin commit.
  - 26 delivery 14→21 (DELCAP15-21) · 27 loyalty 14→20 (LOYCAP15-20) · 28 payment 14→20 (PAYCAP15-20)
  - 29 survey 12→18 (SVCAP13-18) · 30 fleet 10→15 (FLCAP11-15) · 31 calendar 10→15 (CALCAP11-15)
  - 32 maintenance 10→15 (MTCAP11-15) · 33 repair 10→15 (REPCAP11-15) · 34 event 14→20 (EVTCAP15-20)
  - 35 slides 14→20 (SLDCAP15-20) · 36 membership 8→13 (MEMCAP9-13) · 37 lunch 0→7 (LUNCHCAP1-7)
  - 38 spreadsheet 0→7 (SPRCAP1-7) · 39 pos_blackbox 0→6 (PBBCAP1-6) · 40 indaws 0→9 (INDCAP1-9)
- Hallazgos CE/EE notables: Google+Outlook calendar sync = CE bidireccional; portal paga varias facturas vencidas juntas (PAYCAP20); spreadsheet motor/dashboards/ODOO.PIVOT/xlsx = CE pero crear-editar/colaborativo/Documents = EE; pos_blackbox = Bélgica-only (ES→VeriFactu/TBAI); NO nativos detectados: waitlist eventos, pool-booking+multas fleet, referral+tiers loyalty, renovación/carnet membership, secuencial-lección slides, repuestos-stock maintenance, portal-reparación cliente.
- **PENDIENTE (sesión fresca)**: verificación semántica adversarial. El chat quedó a 299K → pre-flight 🔴; el usuario eligió sesión fresca.

### HANDOFF verificación Tramo E (hacer en sesión fresca /clear)
1. `cd ~/.claude/skills/odoo-deep-knowledge`
2. Pre-flight: `python3 ~/.claude/scripts/context_preflight.py --agents 15` (debe salir 🟢/🟠 con chat limpio).
3. Lanzar Workflow adversarial 1 agente/dominio (contexto fresco, CE/EE/OCA por RUTA real vs fuente), MISMO patrón que el Tramo D (script `verify-capabilities-tramo-d`, run wf_bb1a2194-378, en su transcript dir). Dominios+prefijos+ficheros = la lista de arriba. Se puede trocear en 3 tandas de 5 si se prefiere.
4. Corregir PARTIAL/REFUTED (patrón de error frecuente: abreviar `_metodo` que el gate lee como símbolo inexistente → usar nombre completo; y CE marcado como EE) y re-gate.

## Estado Tramo D (2026-07-19) — ✅ CERRADO Y VERIFICADO
- 5 dominios, 58 filas (mail 18, partnership 10, edi 10, voip 10, iot 10). Gate `cite_audit --methods` global limpio (solo account_consolidation v17-only). Sin commit.
- Hallazgo CE/EE clave: **Peppol + UBL/CII + TODO el EDI español (SII/TicketBAI/Facturae/VeriFactu) = CE** en v18; Intrastat = EE. VoIP e IoT = todo EE.
- Verificación semántica: Workflow `verify-capabilities-tramo-d` (run wf_bb1a2194-378, 5 agentes, contexto fresco, adversarial). Resultado: **0 REFUTED**, 5 PARTIAL (todos imprecisión de ancla/nombre, ninguno de capacidad) → corregidos y re-gate OK:
  - MAILCAP2: `mail.template.lang` no es modelo (es campo); ancla real `_render_lang` en mail_render_mixin.py:663.
  - MAILCAP11: `_mail_track` vive en mail/models/models.py:137, no en mail_thread.py.
  - EDICAP2: clase CII real `AccountEdiXmlCII` (no `AccountEdiXmlCII_Facturx`).
  - EDICAP8: modelo informe `account.intrastat.report.handler` (no `account.intrastat.report`).
  - PARTCAP6: módulo OCA v18 = `commission_oca` (no `commission`).


> Método: `references/_index/HOW-TO-EXTEND.md` (ciclo question-driven, respuesta binaria contra fuente).
> Se procesa **de uno en uno**: (1) listar preguntas-cliente → (2) responder contra fuente → (3) escribir filas → (4) índices + gate.
> Estado: ☐ pendiente · ◐ preguntas listadas · ✅ escrito+gate.

## Tramo A — máxima demanda (contabilidad + localización ES)
1. ✅ `ecosystem/l10n_es`            [AMPLIAR · 0→24] — localización española (ESCAP1-24, verificado v18, gate OK)
2. ✅ `enterprise/account_accountant` [AMPLIAR · 0→13] — conciliación/ejercicio/lock (AACAP1-13 + frontera CE↔EE, verificado v18, gate OK)
3. ✅ `enterprise/account_reports`   [AMPLIAR · 0→18] — informes financieros/fiscales (ARCAP1-18, motor account.report, verificado v18, gate OK)
4. ✅ `functional/account`           [PROFUNDIZAR · 18→30] — core contable (ACCAP19-30, verificado v18, gate OK)
5. ✅ `enterprise/account_saft`      [AMPLIAR · 0→7] — SAF-T (SAFTCAP1-7; NO aplica a ES, verificado v18, gate OK)
6. ✅ `enterprise/account_loans`     [AMPLIAR · 0→7] — préstamos/amortización (LOANCAP1-7, verificado v18, gate OK)

## Tramo B — RRHH / servicios (apps que el cliente compra)
7.  ✅ `enterprise/hr_payroll`       [PROFUNDIZAR · 10→19] — PYCAP11-19 (ES payroll = NO, verificado v18, gate OK)
8.  ✅ `enterprise/timesheet`        [PROFUNDIZAR · 10→18] — TCAP11-18 (verificado v18, gate OK)
9.  ✅ `enterprise/planning`         [PROFUNDIZAR · 12→18] — PLANCAP13-18 (verificado v18, gate OK)
10. ✅ `enterprise/approvals`        [PROFUNDIZAR · 10→15] — APPRCAP11-15 (verificado v18, gate OK)
11. ✅ `functional/expense`          [PROFUNDIZAR · 14→19] — EXPCAP15-19 (verificado v18, gate OK)
12. ✅ `functional/recruitment`      [PROFUNDIZAR · 14→19] — RECCAP15-19 (verificado v18, gate OK)
13. ✅ `enterprise/hr_appraisal`     [PROFUNDIZAR · 10→15] — APRCAP11-15 (verificado v18, gate OK)

## Tramo C — operaciones / calidad / documentos
14. ✅ `enterprise/quality`          [PROFUNDIZAR · 10→15] — QCAP11-15 (verificado v18, gate OK)
15. ✅ `enterprise/knowledge`        [PROFUNDIZAR · 10→14] — KNCAP11-14 (verificado v18, gate OK)
16. ✅ `enterprise/sign`             [PROFUNDIZAR · 12→16] — SIGNCAP13-16 (firma NO cualificada ES, verificado v18, gate OK)
17. ✅ `enterprise/appointment`      [PROFUNDIZAR · 12→16] — APPTCAP13-16 (verificado v18, gate OK)
18. ✅ `enterprise/rental`           [PROFUNDIZAR · 12→15] — RENTCAP13-15 (verificado v18, gate OK)
19. ✅ `enterprise/stock_barcode`    [AMPLIAR · 0→12] — BARCAP1-12 (verificado v18, gate OK)
20. ✅ `enterprise/mrp_workorder`    [AMPLIAR · 0→12] — WOCAP1-12 (verificado v18, gate OK)

## Tramo D — comunicación / integración
21. ✅ `functional/mail`             [AMPLIAR · 0→18] — MAILCAP1-18 (mail/mail_group/sms CE, whatsapp EE; verificado v18, gate OK) — SIN verif. semántica
22. ✅ `functional/partnership`      [AMPLIAR · 0→10] — PARTCAP1-10 (website_crm_partner_assign CE; commission EE/OCA; v19 partnership diffs; gate OK) — SIN verif. semántica
23. ✅ `functional/edi`              [AMPLIAR · 0→10] — EDICAP1-10 (Peppol/UBL/ES-EDI todo CE, Intrastat EE; verificado v18, gate OK) — SIN verif. semántica
24. ✅ `enterprise/voip`             [AMPLIAR · 0→10] — VOIPCAP1-10 (todo EE; softphone SIP, NO grabación/SMS, requiere PBX; verificado v18, gate OK) — SIN verif. semántica
25. ✅ `enterprise/iot`              [AMPLIAR · 0→10] — IOTCAP1-10 (todo EE; device types scale/printer/scanner/payment/camera, triggers MRP, drivers=desarrollo; verificado v18, gate OK) — SIN verif. semántica

## Tramo E — cola larga (subir de suelo)
26. ✅ `functional/delivery`         [PROFUNDIZAR · 14→21] — DELCAP15-21 (return/COD/customs/incoterms/batch/insurance/lead-time; verificado v18, gate OK) — SIN verif. semántica
27. ✅ `cross-module/loyalty`        [PROFUNDIZAR · 14→20] — LOYCAP15-20 (referral/tiers=NO nativo; portal/barcode/nominativo/reglas-min=CE; verificado v18, gate OK) — SIN verif. semántica
28. ✅ `cross-module/payment`        [PROFUNDIZAR · 14→20] — PAYCAP15-20 (downpayment/reconcile/multicompany/portal-multi=CE; SEPA-DD=EE/OCA; dunning=EE subs; verificado v18, gate OK) — SIN verif. semántica
29. ✅ `functional/survey`           [PROFUNDIZAR · 12→18] — SVCAP13-18 (anon/invite/mass/recruit-elearn/live=CE; NPS post-ticket=EE helpdesk rating no survey; verificado v18, gate OK) — SIN verif. semántica
30. ✅ `functional/fleet`            [PROFUNDIZAR · 10→15] — FLCAP11-15 (pool-booking/multas=NO nativo; states/tags=CE; adquisición=config; verificado v18, gate OK) — SIN verif. semántica
31. ✅ `functional/calendar`         [PROFUNDIZAR · 10→15] — CALCAP11-15 (google+outlook sync CE bidireccional; tz/adjuntos/tipos=CE; disponibilidad=parcial; verificado v18, gate OK) — SIN verif. semántica
32. ✅ `functional/maintenance`      [PROFUNDIZAR · 10→15] — MTCAP11-15 (calendar/stages/priority/recurrencia=CE; repuestos-stock=NO nativo; recurrencia=event-driven no cron; verificado v18, gate OK) — SIN verif. semántica
33. ✅ `functional/repair`           [PROFUNDIZAR · 10→15] — REPCAP11-15 (cotización/report/priority=CE; portal cliente=NO nativo; mano de obra=línea servicio SO; verificado v18, gate OK) — SIN verif. semántica
34. ✅ `functional/event`            [PROFUNDIZAR · 14→20] — EVTCAP15-20 (badge/networking/quiz/coupons/multilang=CE; waitlist=NO nativo; verificado v18, gate OK) — SIN verif. semántica
35. ✅ `functional/slides`           [PROFUNDIZAR · 14→20] — SLDCAP15-20 (descargables/email/enroll/multilang/certif=CE; secuencial-lección=parcial; verificado v18, gate OK) — SIN verif. semántica
36. ✅ `functional/membership`       [PROFUNDIZAR · 8→13] — MEMCAP9-13 (estados=CE; renovación/recordatorio/carnet/recurrente=NO nativo, subs=EE; verificado v18, gate OK) — SIN verif. semántica
37. ✅ `functional/lunch`            [AMPLIAR · 0→7] — LUNCHCAP1-7 (catálogo/self-order/wallet/states/alertas/toppings=CE; nómina=NO nativo; verificado v18, gate OK) — SIN verif. semántica
38. ✅ `functional/spreadsheet`      [AMPLIAR · 0→7] — SPRCAP1-7 (motor/dashboards-ver/ODOO.PIVOT/xlsx=CE; crear-editar/colab/Documents=EE; verificado v18, gate OK) — SIN verif. semántica
39. ✅ `enterprise/pos_blackbox`     [AMPLIAR · 0→6] — PBBCAP1-6 (FDM belga EE; NO aplica ES→VeriFactu/TBAI; firma/inmutabilidad/WORK-IN-OUT; verificado v18, gate OK) — SIN verif. semántica
40. ✅ `ecosystem/indaws`            [AMPLIAR · 0→9] — INDCAP1-9 (rappel/triple-desc/composición/picking-invoice/ETL/API/retención/paytef; verificado v18, gate OK) — SIN verif. semántica

## NO tocar (no aplican al eje capacidad — son debug/gotcha puro)
`cron`, `security`, `discuss_bus`, `reporting`, `analytic`, `queue_job`,
`oca-accounting`, `mis_builder`, `inbuilt`, `enterprise_tools`, `data_tools`.

## Notas
- `enterprise/account_consolidation` = módulo **v17-only** (no v18/v19). Fuente fuera de roots → aparcado.
- `enterprise/account_3way_match` tiene fichero con 0 filas → rellenar cuando toque compras/facturas.
