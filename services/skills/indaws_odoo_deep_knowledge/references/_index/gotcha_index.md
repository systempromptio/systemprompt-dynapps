# Índice de gotchas — AUTO-GENERADO por `gen_index.py` — NO EDITAR

> 5583 gotchas en 90 dominios. Lookup completo ID → título → fichero (cobertura 100%, siempre al día). Para búsqueda por síntoma en lenguaje natural usa `symptoms.md` (curado a mano). Regenerar: `scripts/gen_index.py --write`. Gate de sincronía: `scripts/gen_index.py --check`.

## functional/account

| ID | Título | Fichero |
|----|--------|---------|
| ACAS1 | `_auto_create_asset()` requiere `account.move.line.name` set. Línea sin producto ni descripción → Us | integrations.md |
| ACAS2 | `multiple_assets_per_line` con multi-currency: `non_deductible_tax_value / quantity` mal computado e | integrations.md |
| ACAS3 | Asset disposal BLOQUEA si `children_ids` en draft/open o `value_residual > 0`. Revalorizaciones pend | integrations.md |
| ACAS4 | `disposal_date <= lock_date` → UserError. No se puede disposal retroactivo si lock_date posterior. ` | integrations.md |
| ACAS5 | `_constrains_check_asset_state`: posted move + asset draft → ValidationError. `account_asset/account | integrations.md |
| ACAS6 | Reversal de depreciation redistribuye monto a siguiente draft. Si no hay moves posteriores → balance | integrations.md |
| ACAS7 | Currency conversion en disposal usa `company.currency_id` base. Asset en otra moneda + fecha actual  | integrations.md |
| ACCAP1 | ¿Multi-moneda con revaluación de tipo de cambio? | capabilities.md |
| ACCAP10 | ¿Contabilidad analítica (centros de coste, distribución)? | capabilities.md |
| ACCAP11 | ¿Informes financieros (balance, P&L, libro mayor) configurables? | capabilities.md |
| ACCAP12 | ¿Cierre de periodo / lock dates / candado fiscal? | capabilities.md |
| ACCAP13 | ¿Multi-compañía con facturas espejo / consolidación? | capabilities.md |
| ACCAP14 | ¿Cash basis / IVA de caja? | capabilities.md |
| ACCAP15 | ¿Conciliación automática por reglas (reconcile models)? | capabilities.md |
| ACCAP16 | ¿Numeración legal por secuencia/diario? | capabilities.md |
| ACCAP17 | ¿Pagos por lote / fichero SEPA? | capabilities.md |
| ACCAP18 | ¿Audit trail / inalterabilidad de asientos? | capabilities.md |
| ACCAP19 | ¿Notas de crédito/abono con reverso storno (asiento en negativo)? | capabilities.md |
| ACCAP2 | ¿Posiciones fiscales automáticas (impuestos por país/cliente)? | capabilities.md |
| ACCAP20 | ¿Anticipos / pagos a cuenta (down payments) en factura? | capabilities.md |
| ACCAP21 | ¿Descuento por pronto pago (early payment discount)? | capabilities.md |
| ACCAP22 | ¿Redondeo de efectivo (cash rounding)? | capabilities.md |
| ACCAP23 | ¿Fiscalidad de línea (IVA incluido/excluido, cascada, base afectada)? | capabilities.md |
| ACCAP24 | ¿Peppol / factura electrónica estructurada genérica (no localización)? | capabilities.md |
| ACCAP25 | ¿Pago online en factura de cliente (portal + proveedores de pago)? | capabilities.md |
| ACCAP26 | ¿Unidades fiscales / grupo IVA (tax units)? | capabilities.md |
| ACCAP27 | ¿Cuentas contables por defecto (producto/categoría/posición fiscal)? | capabilities.md |
| ACCAP28 | ¿Facturación por lotes y validación/pago en bloque? | capabilities.md |
| ACCAP29 | ¿Diario de caja / gestión de efectivo (cash journal)? | capabilities.md |
| ACCAP3 | ¿Conciliación bancaria (import + matching automático)? | capabilities.md |
| ACCAP30 | ¿Registrar pago parcial / agrupado / desde varias facturas? | capabilities.md |
| ACCAP4 | ¿Sincronización bancaria automática (feeds)? | capabilities.md |
| ACCAP5 | ¿Plazos de pago complejos (30/60/90, fin de mes, fraccionado)? | capabilities.md |
| ACCAP6 | ¿Dunning / reclamación de impagos automatizada? | capabilities.md |
| ACCAP7 | ¿Activos / amortizaciones? | capabilities.md |
| ACCAP8 | ¿Ingresos/gastos diferidos? | capabilities.md |
| ACCAP9 | ¿Presupuestos contables? | capabilities.md |
| ACED1 | `edi_document_ids` creados al post(). Reopen + cambiar payment_term + repost → tax tagging cambia, E | integrations.md |
| ACED2 | `_process_job()` con context `_send_only_when_ready` bloquea validaciones. Error interno → state que | integrations.md |
| ACED3 | `blocking_level='error'` deshabilita `edi_show_cancel_button` pero NO levanta excepción. Move `poste | integrations.md |
| ACED4 | (v19 BREAKING) | integrations.md |
| ACED5 | (v19 BREAKING) | integrations.md |
| ACG1 | Configure Automatic Inventory Valuation | config_guide.md |
| ACG10 | Configure Bank Journal for Reconciliation | config_guide.md |
| ACG11 | Register Payment from Invoice | config_guide.md |
| ACG12 | Configure Cash Basis Taxes | config_guide.md |
| ACG2 | Configure Anglo-Saxon vs Continental | config_guide.md |
| ACG3 | Configure Payment Methods | config_guide.md |
| ACG4 | Configure Payment Terms | config_guide.md |
| ACG5 | Configure Taxes | config_guide.md |
| ACG6 | Configure Fiscal Position | config_guide.md |
| ACG7 | Configure Analytic Plans | config_guide.md |
| ACG8 | Configure Analytic Distribution Models | config_guide.md |
| ACG9 | Configure Lock Dates | config_guide.md |
| ACLO1 | `hard_lock_date` ABSOLUTAMENTE irreversible. `_check_fiscal_lock_dates` siempre pasa `hard=True`. Si | integrations.md |
| ACLO2 | Tax lock date solo afecta moves con `tax_ids ≠ empty`. Líneas tax lazy-computed pueden cambiar si mo | integrations.md |
| ACLO3 | Lock exceptions PER-USER pero `user_fiscal_lock_date` cache global. Dos users con excepciones confli | integrations.md |
| ACLO4 | `bypass_lock_check=BYPASS_LOCK_CHECK` requiere objeto EXACTO `BYPASS_LOCK_CHECK`, no True. Cualquier | integrations.md |
| ACLO5 | Auto-adjust date: múltiples facturas antiguas → todas al mismo día posterior → sequence puede fallar | integrations.md |
| ACLO6 | Exception `end_datetime` se checkea contra `self.env.cr.now()` (UTC). Server timezone diferente → ex | integrations.md |
| ACP1 | Outstanding account OBLIGATORIO. Sin `outstanding_account_id` → `UserError("You can't create a new p | integrations.md |
| ACP2 | `_synchronize_to_moves()` solo actúa si `move.state='draft'`. Cambiar amount/fecha tras posted no si | integrations.md |
| ACP3 | Multiple liquidity lines (custom writeoffs) bloquean cambio de amount con UserError. `account/models | integrations.md |
| ACP4 | `state='paid'` requiere `sum(liquidity.amount_residual)==0 AND account.reconcile=True` (lo verificad | integrations.md |
| ACP5 | `payment_account_id` puede ser `None` si se elige `journal.default_account_id` con account_type adec | integrations.md |
| ACP6 | Partner property `property_inbound_payment_method_line_id` override silencioso. Si tipo no compatibl | integrations.md |
| ACP7 | `_get_payment_method_codes_to_exclude()` puede excluir métodos (SEPA, Batch). Si el único disponible | integrations.md |
| ACP8 | `origin_payment_id` huérfano si payment se cancela post-posted. Move queda posted con ref a payment  | integrations.md |
| ACPR1 | `exchange_move_id` solo si `exchange_diff != 0`. Amount ya revaluado → rounding loss (asume 2 decima | integrations.md |
| ACPR2 | `max_date = MAX(debit_date, credit_date)` usado en aged reports. Factura antigua con payment recient | integrations.md |
| ACPR3 | (v19 NEW) | integrations.md |
| ACPR4 | (v19 CHANGE) | integrations.md |
| ACPR5 | (v19 NEW) | integrations.md |
| ACSII1 | `l10n_es_type='recargo'` requiere SIEMPRE tax principal sujeto/sujeto_isp. Mapping mal → SII error 3 | integrations.md |
| ACSII2 | `l10n_es_edi_is_required` es computed dependiente de `invoice_line_ids.tax_ids`. Cambiar journal de  | integrations.md |
| ACSII3 | `_l10n_es_is_dua()` busca `l10n_es_type='dua'`. DUA tiene desglose DIFERENTE (sin CuotaRepercutida). | integrations.md |
| ACSII4 | Intra-community en UBL/CII (`account_edi_ubl_cii/models/account_edi_xml_ubl_bis3.py → _invoice_const | integrations.md |
| ACT1 | `price_include` es COMPUTED (no field directo). Cambiar `company_id.account_price_include` post-fact | integrations.md |
| ACT10 | Reverse charge taxes tienen `tag_ids` con `use_in_tax_closing=False`. Report basado en `use_in_tax_c | integrations.md |
| ACT11 | (v19 BREAKING) | integrations.md |
| ACT2 | `has_negative_factor` (repartition con factor<0 | integrations.md |
| ACT3 | `_batch_for_taxes_computation` agrupa por `(amount_type, price_include, include_base_amount)`. Mezcl | integrations.md |
| ACT4 | `_propagate_extra_taxes_base()` invierte sign para price-included si `include_base_amount=True`. Mul | integrations.md |
| ACT5 | (reverse charge) | integrations.md |
| ACT6 | (refund reverse) | integrations.md |
| ACT7 | (cash basis) | integrations.md |
| ACT8 | (multi-company) | integrations.md |
| ACT9 | `tax_tag_invert` (campo en `account.move.line`, no en `account.tax`) depende de `move_type`/`is_inbo | integrations.md |
| AG1 | Debit/credit are ALWAYS in company currency | gotchas.md |
| AG10 | Exchange differences create unexpected journal entries | gotchas.md |
| AG11 | account.account.code is company-dependent (JSON) | gotchas.md |
| AG12 | Hashed moves cannot be reset to draft | gotchas.md |
| AG13 | analytic_distribution is a JSON field, not a relation | gotchas.md |
| AG14 | Analytic plans create dynamic database columns | gotchas.md |
| AG15 | Analytic distribution models combine by sequence + domain match (NOT a score) | gotchas.md |
| AG16 | Payment term percentages must sum to 100% | gotchas.md |
| AG17 | Early payment discount creates special EPD lines | gotchas.md |
| AG18 | Multi-currency reconciliation requires configured FX accounts | gotchas.md |
| AG19 | Storno accounting reverses debit/credit convention | gotchas.md |
| AG2 | `reconcile` default is auto-set only for receivable/payable; most other accounts can opt in manually | gotchas.md |
| AG20 | Analytic mandatory plans block invoice posting | gotchas.md |
| AG21 | Cash rounding creates invisible adjustment lines | gotchas.md |
| AG22 | Matching numbers use Union-Find graph algorithm | gotchas.md |
| AG23 | Payment state computed via SQL on partials, not from move | gotchas.md |
| AG24 | Exchange journal REQUIRED for multi-currency reconciliation | gotchas.md |
| AG25 | Bank statement suspense created on post, NOT reconciliation | gotchas.md |
| AG26 | Payment tolerance has TWO mechanisms: percentage vs fixed | gotchas.md |
| AG27 | Lock dates have 5 independent channels, hard_lock overrides ALL | gotchas.md |
| AG28 | Payment outstanding account comes from the payment method line (with company-level default) | gotchas.md |
| AG29 | Cash basis entries batched by (currency, partner, account, tax_set, analytics) | gotchas.md |
| AG3 | Bank statement lines ALWAYS create a suspense line | gotchas.md |
| AG30 | Full reconcile triggers only when ALL connected lines have zero residual | gotchas.md |
| AG31 | Reconciliation plan sorts by maturity THEN splits by currency | gotchas.md |
| AG32 | FX difference uses BOTH company AND document currency residuals | gotchas.md |
| AG33 | Bank statement validity uses LAG() window function on first_line_index | gotchas.md |
| AG34 | Analytic distribution "33,33,33" = 99% — posting fails | gotchas.md |
| AG35 | COGS timing depends on valuation method, NOT on invoice_policy | gotchas.md |
| AG4 | Register Payment from invoice auto-reconciles; standalone does NOT | gotchas.md |
| AG5 | payment_state is computed via SQL, not Python | gotchas.md |
| AG6 | Lock dates have user exceptions but hard_lock doesn't | gotchas.md |
| AG7 | Fiscal position auto-maps taxes AND accounts | gotchas.md |
| AG8 | Tax repartition lines control where tax goes | gotchas.md |
| AG9 | Cash basis taxes create transition entries | gotchas.md |

## functional/blog

| ID | Título | Fichero |
|----|--------|---------|
| BLGCAP1 | ¿Filtrado público por fecha? | capabilities.md |
| BLGCAP2 | ¿Búsqueda full-text integrada? | capabilities.md |
| BLGCAP3 | ¿Agregar tags por blog con frecuencia mínima? | capabilities.md |
| BLGCAP4 | ¿Acceso smart para externos? | capabilities.md |
| BLGCAP5 | ¿Notificar a followers al publicar? | capabilities.md |
| BLGCAP6 | ¿Teaser manual override? | capabilities.md |
| BLGCAP7 | ¿Cover con resize on-the-fly? | capabilities.md |
| BLGCAP8 | ¿Comentarios públicos? | capabilities.md |
| BLGG1 | `published_date` se autorrellena al publicar solo si no estaba. | gotchas.md |
| BLGG10 | `website_published` vs `is_published` divergen por website. | gotchas.md |
| BLGG11 | La meta description SEO sale de `subtitle`, no de `content`. | gotchas.md |
| BLGG12 | website. | integrations.md |
| BLGG14 | social / compartir. | integrations.md |
| BLGG16 | snippets / dynamic filters. | integrations.md |
| BLGG2 | `post_date` ≠ `published_date`. | gotchas.md |
| BLGG3 | `visits` se incrementa en CADA lectura pero deduplicado por sesión. | gotchas.md |
| BLGG4 | Archivar un blog archiva TODOS sus posts. | gotchas.md |
| BLGG5 | Archivar un post lo despublica forzosamente. | gotchas.md |
| BLGG6 | Escribir cualquier tag reescribe TODOS sus posts (`post_ids.write({})`). | gotchas.md |
| BLGG7 | El teaser por defecto trunca a 200 chars de texto plano. | gotchas.md |
| BLGG8 | `copy`/duplicate añade " (copy)" y NO copia visits. | gotchas.md |
| BLGG9 | `og:image` se extrae con slicing frágil del JSON cover. | gotchas.md |

## functional/calendar

| ID | Título | Fichero |
|----|--------|---------|
| CALCAP1 | ¿Recordatorios por email y por notificación popup? | capabilities.md |
| CALCAP10 | ¿Recordatorio SMS nativo en CE? | capabilities.md |
| CALCAP11 | ¿Sincronización bidireccional con Google Calendar y Outlook/Microsoft? | capabilities.md |
| CALCAP12 | ¿Zona horaria por evento? | capabilities.md |
| CALCAP13 | ¿Ver la disponibilidad de otros usuarios (calendarios superpuestos)? | capabilities.md |
| CALCAP14 | ¿Adjuntos en el evento? | capabilities.md |
| CALCAP15 | ¿Tipos/categorías de evento configurables? | capabilities.md |
| CALCAP2 | ¿Recurrencia diaria/semanal/mensual/anual con fin por count/fecha/forever? | capabilities.md |
| CALCAP3 | ¿Mensual "2º lunes" vs "día 15"? | capabilities.md |
| CALCAP4 | ¿Exportar invitación `.ics`? | capabilities.md |
| CALCAP5 | ¿Videollamada integrada (Discuss)? | capabilities.md |
| CALCAP6 | ¿Eventos privados que otros no ven? | capabilities.md |
| CALCAP7 | ¿Aceptar/Rechazar/Tentativo y registrarlo en el chatter? | capabilities.md |
| CALCAP8 | ¿Aplicar la respuesta a toda la serie / futuros? | capabilities.md |
| CALCAP9 | ¿Bloquear TODOS los emails del calendario globalmente? | capabilities.md |
| CALG1 | "This Event" con campos de recurrencia lanza UserError. | gotchas.md |
| CALG10 | DST puede duplicar/desplazar ocurrencias; hay un hack para evitarlo. | gotchas.md |
| CALG11 | `copy()` NO copia asistentes ni su estado. | gotchas.md |
| CALG12 | Borrar el base_event NO borra la serie: se reelige base. | gotchas.md |
| CALG13 | Eventos recurrentes tienen `access_token` distinto por ocurrencia (a propósito). | gotchas.md |
| CALG14 | Campos privados se ENMASCARAN en lectura, no se bloquean. | gotchas.md |
| CALG15 | `mail.activity` ↔ `calendar.event` (bidireccional). | integrations.md |
| CALG16 | CRM / "next activity". | integrations.md |
| CALG17 | `res.partner`. | integrations.md |
| CALG18 | mail (notificaciones). | integrations.md |
| CALG19 | Discuss. | integrations.md |
| CALG2 | `recurrence_update` por defecto es `self_only` y NO se almacena | gotchas.md |
| CALG20 | Auth pública por token (portal/email). | integrations.md |
| CALG3 | Editar `all_events` BORRA y RECREA toda la recurrencia. | gotchas.md |
| CALG4 | Editar la hora de UNA ocurrencia la DESVINCULA de la serie. | gotchas.md |
| CALG5 | `future_events` recorta la serie y reactiva el evento como nueva base. | gotchas.md |
| CALG6 | Cambiar la hora resetea el estado de TODOS los asistentes a needsAction. | gotchas.md |
| CALG7 | Si OTRO usuario cambia la hora, el organizador pasa a needsAction. | gotchas.md |
| CALG8 | La alarma de recurrencia NO se reprograma sola en cron. | gotchas.md |
| CALG9 | Alarmas en el pasado se descartan POR DISEÑO. | gotchas.md |

## functional/crm

| ID | Título | Fichero |
|----|--------|---------|
| CCG1 | Activar fase de Leads (Lead → Opportunity) | config_guide.md |
| CCG10 | Configurar Lead Mining (IAP) | config_guide.md |
| CCG11 | Configurar Enriquecimiento automático de leads (IAP enrich) | config_guide.md |
| CCG12 | Configurar canal de leads desde Livechat / Chatbot | config_guide.md |
| CCG2 | Crear y configurar un Equipo de Ventas | config_guide.md |
| CCG3 | Configurar Etapas del Pipeline | config_guide.md |
| CCG4 | Activar y configurar la Asignación Automática de Leads | config_guide.md |
| CCG5 | Configurar Predictive Lead Scoring (PLS) | config_guide.md |
| CCG6 | Configurar Email Alias para captura de leads | config_guide.md |
| CCG7 | Configurar Motivos de Pérdida (Lost Reasons) | config_guide.md |
| CCG8 | Activar Recurring Revenue (MRR en el pipeline) | config_guide.md |
| CCG9 | Configurar el formulario web "Contáctanos" → lead | config_guide.md |
| CG1 | Lead y Opportunity son el MISMO modelo | gotchas.md |
| CG10 | El sync de dirección es all-or-none | gotchas.md |
| CG11 | El cron de asignación está inactivo por defecto | gotchas.md |
| CG12 | La asignación deduplica durante la allocation | gotchas.md |
| CG13 | El update de revenue desde SO es one-directional y solo sube | gotchas.md |
| CG14 | La conversión preserva won/lost | gotchas.md |
| CG15 | El cron PLS reconstruye desde cero | gotchas.md |
| CG16 | Los comerciales NO pueden borrar leads | gotchas.md |
| CG17 | El recurring revenue está gateado por un grupo oculto | gotchas.md |
| CG18 | La detección de duplicados usa `email_domain_criterion` | gotchas.md |
| CG19 | Currency mismatch salta el update de revenue desde SO en silencio | gotchas.md |
| CG2 | Stage NO es una máquina de estados | gotchas.md |
| CG20 | La cancelación de SO nunca revierte expected_revenue | gotchas.md |
| CG21 | La conversión Helpdesk mueve adjuntos, no los copia | gotchas.md |
| CG22 | La conversión Helpdesk anula subtypes de mensajes no-comment | gotchas.md |
| CG23 | `/lead` vs chatbot create_lead tienen comportamientos distintos | gotchas.md |
| CG24 | La asignación ordena por -probability con id() builtin como tiebreaker | gotchas.md |
| CG25 | La auto-asignación convierte leads en opportunities | gotchas.md |
| CG26 | PLS tiene cero impacto sobre recurring revenue | gotchas.md |
| CG27 | El cron de IAP enrich solo toca leads FRESCOS | gotchas.md |
| CG28 | IAP enrich salta leads bloqueados en silencio (v18) vs reintenta (v19) | gotchas.md |
| CG29 | Lead mining cobra créditos por adelantado por fórmula, capeado | gotchas.md |
| CG3 | Probability dual-tracked | gotchas.md |
| CG30 | medium_id del web form tiene fallback 3-step, NO incondicional | gotchas.md |
| CG31 | El type lead vs opportunity del web form deriva del team, no de la config global | gotchas.md |
| CG32 | Los endpoints de lead del mail plugin usan auth='outlook', no 'user' | gotchas.md |
| CG33 | `_stage_find`/`_compute_stage_id` viven en crm_lead.py, no en crm_stage.py | gotchas.md |
| CG34 | stage.team_id es opcional con ondelete='set null' | gotchas.md |
| CG35 | action_set_won busca etapa won bidireccionalmente | gotchas.md |
| CG36 | Ganar un lead lo desarchiva | gotchas.md |
| CG37 | Restaurar un lead lost limpia el lost_reason y recomputa probability | gotchas.md |
| CG38 | expected_revenue es almacenado, no computado | gotchas.md |
| CG39 | recurring_revenue_monthly divide por number_of_months del plan | gotchas.md |
| CG4 | PLS usa offset 0.1, no Laplace +1 | gotchas.md |
| CG40 | lead.mobile SÍ es editable y persiste, pero NO se propaga al partner | gotchas.md |
| CG41 | La quota diaria es assignment_max/30 redondeado, no assignment_max | gotchas.md |
| CG42 | La allocation de equipos es weighted random, no proporcional exacto | gotchas.md |
| CG43 | La quotation desde opportunity sin partner abre wizard, no falla | gotchas.md |
| CG44 | No existe ningún método "prepare_opportunity" — los defaults de la quotation viven en el contexto de | gotchas.md |
| CG45 | El merge PIERDE datos del registro perdedor en campos escalares; los M2M/O2M genéricos NO se unen, s | gotchas.md |
| CG46 | El merge solo transfiere followers que POSTEARON un mensaje en los últimos 30 días | gotchas.md |
| CG47 | Visibilidad multi-team: un comercial ve leads sin asignar de cualquier team | gotchas.md |
| CG48 | `email_domain_criterion` excluye proveedores genéricos pero la lista es fija | gotchas.md |
| CG5 | Leads won incrementan TODAS las etapas en la tabla de frecuencias | gotchas.md |
| CG6 | El gateway de email crea leads SIN comercial | gotchas.md |
| CG7 | `_message_post_after_hook` bulk-actualiza partner en TODOS los leads coincidentes | gotchas.md |
| CG8 | El merge sigue jerarquía de confianza | gotchas.md |
| CG9 | El sync partner↔lead es asimétrico | gotchas.md |
| CRMCAP1 | ¿Lead scoring / probabilidad predictiva? | capabilities.md |
| CRMCAP10 | ¿Ingreso recurrente (MRR) en oportunidades? | capabilities.md |
| CRMCAP11 | ¿Previsión de ventas (forecast)? | capabilities.md |
| CRMCAP12 | ¿Equipos de venta y objetivos? | capabilities.md |
| CRMCAP13 | ¿Detección de duplicados / merge? | capabilities.md |
| CRMCAP14 | ¿Lead → presupuesto? | capabilities.md |
| CRMCAP15 | ¿Live chat → lead? | capabilities.md |
| CRMCAP16 | ¿Multi-moneda en oportunidades? | capabilities.md |
| CRMCAP17 | ¿Informes/embudo de conversión? | capabilities.md |
| CRMCAP18 | ¿Gamificación / ranking de comerciales? | capabilities.md |
| CRMCAP2 | ¿Asignación automática de leads (round-robin)? | capabilities.md |
| CRMCAP3 | ¿Pipeline con etapas personalizables? | capabilities.md |
| CRMCAP4 | ¿Motivos de pérdida? | capabilities.md |
| CRMCAP5 | ¿Actividades y recordatorios? | capabilities.md |
| CRMCAP6 | ¿Integración de email (alias, seguimiento)? | capabilities.md |
| CRMCAP7 | ¿Enriquecimiento automático de leads? | capabilities.md |
| CRMCAP8 | ¿Minería de leads (lead mining)? | capabilities.md |
| CRMCAP9 | ¿Formularios web → lead? | capabilities.md |

## functional/delivery

| ID | Título | Fichero |
|----|--------|---------|
| DELCAP1 | ¿Métodos de envío configurables? | capabilities.md |
| DELCAP10 | ¿Elegir método de envío en el checkout web? | capabilities.md |
| DELCAP11 | ¿Cancelar un envío ya creado? | capabilities.md |
| DELCAP12 | ¿Multi-paquete / peso y dimensiones? | capabilities.md |
| DELCAP13 | ¿Puntos de recogida (pickup points)? | capabilities.md |
| DELCAP14 | ¿Margen coste vs cobrado? | capabilities.md |
| DELCAP15 | ¿Devoluciones con etiqueta de retorno (return label)? | capabilities.md |
| DELCAP16 | ¿Contrareembolso (cash on delivery)? | capabilities.md |
| DELCAP17 | ¿Documentación de aduana/exportación (HS code, país de origen)? | capabilities.md |
| DELCAP18 | ¿Incoterms en entrega/factura? | capabilities.md |
| DELCAP19 | ¿Agrupar varios albaranes en un envío por lotes (batch)? | capabilities.md |
| DELCAP2 | ¿Tarifas por reglas (peso/precio/cantidad)? | capabilities.md |
| DELCAP20 | ¿Seguro / valor declarado del envío? | capabilities.md |
| DELCAP21 | ¿Fecha de entrega comprometida / lead time? | capabilities.md |
| DELCAP3 | ¿Tarifa fija y envío como línea del pedido? | capabilities.md |
| DELCAP4 | ¿Envío gratis a partir de un importe? | capabilities.md |
| DELCAP5 | ¿Calcular el precio real contra el transportista? | capabilities.md |
| DELCAP6 | ¿Generar la etiqueta de envío vía API? | capabilities.md |
| DELCAP7 | ¿Número y URL de seguimiento? | capabilities.md |
| DELCAP8 | ¿Qué transportistas trae de serie? | capabilities.md |
| DELCAP9 | ¿Restringir un método por país/zona/CP? | capabilities.md |
| DG1 | delivery module has NO stock dependency | gotchas.md |
| DG10 | _get_price_dict is the hook for custom rule variables | gotchas.md |
| DG11 | product.weight=0 gives zero shipping weight, not an error | gotchas.md |
| DG12 | Package splitting uses max_weight, NOT item count | gotchas.md |
| DG13 | weight_bulk vs packages — one or the other or both | gotchas.md |
| DG14 | stock.quant.package.shipping_weight overrides computed weight | gotchas.md |
| DG15 | UoM for package dimensions is provider-specific, NOT from config | gotchas.md |
| DG16 | send_to_shipper propagates tracking to ENTIRE move chain, but only the FIRST time | gotchas.md |
| DG17 | Real cost delivery line: price_unit=0 until delivery | gotchas.md |
| DG18 | Return pickings reset carrier_id | gotchas.md |
| DG19 | _send_confirmation_email error handling is asymmetric — flag is per-batch, not per-position | gotchas.md |
| DG2 | fixed_price is computed from product_id.list_price | gotchas.md |
| DG20 | Different carriers = different pickings (key uses the SO carrier) | gotchas.md |
| DG21 | Delivery line deletion clears carrier_id | gotchas.md |
| DG22 | Cannot remove delivery line ONLY if ALL delivery lines are invoiced | gotchas.md |
| DG23 | Delivery lines excluded from pricelist recomputation | gotchas.md |
| DG24 | is_delivery lines can be deleted from confirmed orders | gotchas.md |
| DG25 | Pickup location creates/reuses partner, archived unless in_store | gotchas.md |
| DG26 | available_carriers only checks _match, NOT rate_shipment | gotchas.md |
| DG27 | Zip prefix matching uses regex with ^ anchor only; prefixes are forced UPPER | gotchas.md |
| DG28 | Carrier availability wizard filters by company | gotchas.md |
| DG29 | carrier.company_id comes from product_id.company_id | gotchas.md |
| DG3 | Margins are NOT applied to fixed price carriers | gotchas.md |
| DG30 | Fixed margin converted before adding to price | gotchas.md |
| DG31 | Provider cancel_shipment may raise NotImplementedError | gotchas.md |
| DG32 | log_xml uses separate cursor to survive rollback | gotchas.md |
| DG33 | _get_delivery_type can be overridden for dynamic types | gotchas.md |
| DG34 | stock.move.weight is computed ONLY for products with weight > 0; column created by raw SQL | gotchas.md |
| DG35 | carrier propagation on new pickings: `vals['carrier_id'] = any(...) and carrier_id` → False when no  | gotchas.md |
| DG36 | set_delivery_line propagates carrier to pickings only when SO state == 'sale' | gotchas.md |
| DG37 | The wizard blocks on rate error for fixed/base_on_rule but only warns for external providers | gotchas.md |
| DG38 | send_to_shipper re-derives cost independently of the rate wizard | gotchas.md |
| DG39 | carrier_tracking_url can hold a JSON list of (label, url) pairs for multi-package | gotchas.md |
| DG4 | delivery_type dispatches via getattr — typo = silent no-op (NOT always an error) | gotchas.md |
| DG40 | _should_generate_commercial_invoice compares warehouse country vs partner country | gotchas.md |
| DG41 | product.category 'Deliveries' cannot be deleted | gotchas.md |
| DG5 | integration_level='rate' forces invoice_policy='estimated' | gotchas.md |
| DG6 | free_over does NOT apply to base_on_rule carriers (rate time) | gotchas.md |
| DG7 | Price rules use safe_eval — first match wins | gotchas.md |
| DG8 | Rule variable 'price' = amount_total minus delivery price_total, in COMPANY currency | gotchas.md |
| DG9 | Weight fallback chain: context order_weight > shipping_weight > computed | gotchas.md |

## functional/edi

| ID | Título | Fichero |
|----|--------|---------|
| EDIG1 | UUID order assumption on batched send | gotchas.md |
| EDIG10 | `_peppol_get_new_documents` BATCH_SIZE=50, auto-retriggers | gotchas.md |
| EDIG11 | Self-billing invoice routing | gotchas.md |
| EDIG12 | IAP error 702 silently retried | gotchas.md |
| EDIG13 | Tax not found on import → 0% VAT, no error | gotchas.md |
| EDIG14 | Partner auto-create requires BOTH name and VAT | gotchas.md |
| EDIG15 | Negative `Invoice` auto-flips to credit note | gotchas.md |
| EDIG16 | UoM mapping silently drops | gotchas.md |
| EDIG17 | Tax category code auto-detected by partner ZIP/country, not the tax | gotchas.md |
| EDIG18 | Fixed taxes (Recupel) silently moved to document level | gotchas.md |
| EDIG19 | XRechnung mandates `BuyerReference`, defaults to "N/A" | gotchas.md |
| EDIG2 | Demo mode silently switches to mock | gotchas.md |
| EDIG20 | `BR-CO-09`: VAT must carry country prefix | gotchas.md |
| EDIG21 | `[UBL-SR-48]` one tax category per line | gotchas.md |
| EDIG22 | PDF without embedded XML still processed | gotchas.md |
| EDIG23 | Schematron URLs hard-coded | gotchas.md |
| EDIG24 | `ubl_cii_xml_file` attachment is detached on draft, regenerated on repost | gotchas.md |
| EDIG25 | Currency: `active=False` accepted | gotchas.md |
| EDIG26 | Attachment hash duplicate detection isn't strict | gotchas.md |
| EDIG27 | `disable_pdf_in_xml` ICP (v19 only) | gotchas.md |
| EDIG28 | Service exclusion in v18 base SQL | gotchas.md |
| EDIG29 | Stock value uses invoice, not stock cost | gotchas.md |
| EDIG3 | Token expiry 24h, NOWAIT lock for duplicated DBs | gotchas.md |
| EDIG30 | Stock moves consulted only for warehouse region | gotchas.md |
| EDIG31 | Brexit cutoff hard-coded 2021-01-01 | gotchas.md |
| EDIG32 | No automatic B2C exclusion | gotchas.md |
| EDIG33 | Default transaction codes all expired | gotchas.md |
| EDIG34 | Triangular trade not handled automatically | gotchas.md |
| EDIG35 | Spanish €400 000 threshold check | gotchas.md |
| EDIG36 | Supplementary unit only if code supports it | gotchas.md |
| EDIG37 | Weight from `product.weight`, zero by default | gotchas.md |
| EDIG38 | Region code in NL Intrastat hidden | gotchas.md |
| EDIG39 | `account_peppol_proxy_state` → `peppol_can_send` | gotchas.md |
| EDIG4 | Database duplication → invalid signature | gotchas.md |
| EDIG40 | `account_peppol_advanced_fields` is DEPRECATED on arrival | gotchas.md |
| EDIG41 | `account_intrastat_services` removed, reports split | gotchas.md |
| EDIG42 | Tax category code now on `account.tax`, not auto-derived only | gotchas.md |
| EDIG43 | Class renames: `Key` → `CertificateKey` | gotchas.md |
| EDIG44 | EDI cancel auto-reverses a posted invoice | gotchas.md |
| EDIG45 | `blocking_level='error'` freezes the doc, no auto-retry | gotchas.md |
| EDIG46 | Batched job aborts the whole group on one bad move | gotchas.md |
| EDIG47 | `FOR UPDATE NOWAIT` → "being sent by another process" | gotchas.md |
| EDIG48 | `edi_state` ignores non-web-service formats | gotchas.md |
| EDIG49 | `edi_content` recomputes (and re-validates) on every read | gotchas.md |
| EDIG5 | `peppol_purchase_journal_id` auto-set silently | gotchas.md |
| EDIG50 | Format only appears on journals after the register hook | gotchas.md |
| EDIG51 | UBL/CII does NOT use `account_edi` at all | gotchas.md |
| EDIG52 | Legacy framework only for government web-service formats (v18) | gotchas.md |
| EDIG53 | `invoice_edi_format` is single-valued (Facturae XOR UBL primary) | gotchas.md |
| EDIG54 | Four parallel EDI states, none synchronised | gotchas.md |
| EDIG55 | SII recargo without main sujeto → AEAT error 3000 | gotchas.md |
| EDIG56 | Send & Print hooks are EXTENDS | gotchas.md |
| EDIG57 | Facturae applies by partner country, silently skipped otherwise | gotchas.md |
| EDIG58 | Factur-X is ALWAYS embedded in the PDF for FR/DE, regardless of chosen format | gotchas.md |
| EDIG59 | ZUGFeRD vs Factur-X filename decided by partner country + EAS | gotchas.md |
| EDIG6 | Verification state is stored, not computed | gotchas.md |
| EDIG60 | Pure-UBL XML gets the PDF injected back as `AdditionalDocumentReference` | gotchas.md |
| EDIG61 | Builder errors don't abort send | gotchas.md |
| EDIG62 | In test mode the PDF post-processing is skipped, a stub `factur-x.xml` is created | gotchas.md |
| EDIG63 | `_need_ubl_cii_xml` only fires for sale docs (or self-invoices) AND when no XML exists yet | gotchas.md |
| EDIG64 | Importer auto-(un)groups bill lines by tax to match the last vendor bill | gotchas.md |
| EDIG65 | `AttachedDocument` wrapper is unwrapped before decoding | gotchas.md |
| EDIG66 | UBL attachment is created as SUPERUSER and the move cache is invalidated | gotchas.md |
| EDIG7 | Belgian inverse EAS auto-swap | gotchas.md |
| EDIG8 | Sending while not activated → wizard, not error | gotchas.md |
| EDIG9 | Cron interval acceleration | gotchas.md |

## functional/event

| ID | Título | Fichero |
|----|--------|---------|
| EVTCAP1 | ¿Evento con varios tipos de entrada y aforo? | capabilities.md |
| EVTCAP10 | ¿Emails automáticos programados (antes/después)? | capabilities.md |
| EVTCAP11 | ¿Preguntas personalizadas en el registro? | capabilities.md |
| EVTCAP12 | ¿Generar lead/oportunidad desde inscripciones? | capabilities.md |
| EVTCAP13 | ¿Vender/registrar entradas en POS? | capabilities.md |
| EVTCAP14 | ¿Plantilla reutilizable y eventos recurrentes? | capabilities.md |
| EVTCAP15 | ¿Lista de espera cuando se agotan las entradas? | capabilities.md |
| EVTCAP16 | ¿Badge/gafete imprimible personalizado? | capabilities.md |
| EVTCAP17 | ¿Networking/comunidad entre asistentes (salas)? | capabilities.md |
| EVTCAP18 | ¿Cupones/promos sobre el precio de las entradas? | capabilities.md |
| EVTCAP19 | ¿Gamificación (quiz con puntos) durante el evento? | capabilities.md |
| EVTCAP2 | ¿Inscripción online desde la web? | capabilities.md |
| EVTCAP20 | ¿Página del evento multi-idioma? | capabilities.md |
| EVTCAP3 | ¿Vender entradas de pago? | capabilities.md |
| EVTCAP4 | ¿Entradas gratis con confirmación automática? | capabilities.md |
| EVTCAP5 | ¿Agenda / ponencias / tracks? | capabilities.md |
| EVTCAP6 | ¿Call for proposals (ponentes proponen charlas)? | capabilities.md |
| EVTCAP7 | ¿Stands de patrocinadores/exhibidores? | capabilities.md |
| EVTCAP8 | ¿Eventos online / salas de vídeo? | capabilities.md |
| EVTCAP9 | ¿Check-in con código de barras / badge? | capabilities.md |
| EVTG1 | El registro nace CONFIRMADO, no en borrador. | gotchas.md |
| EVTG10 | Cambiar `event_type_id` reescribe tickets/mails/preguntas como un onchange. | gotchas.md |
| EVTG11 | Los eventos terminados se mueven solos a la etapa "End" por autovacuum. | gotchas.md |
| EVTG12 | No puedes borrar/cambiar tipo de preguntas o tickets con datos. | gotchas.md |
| EVTG13 | `barcode` es único a nivel BD y autogenerado. | gotchas.md |
| EVTG14 | `date_closed` (asistencia) se graba al pasar a `done` y no se limpia si ya existía. | gotchas.md |
| EVTG15 | La SO de pago deja las registraciones en `draft` solo si confirmas UNA. | integrations.md |
| EVTG16 | El registro web de entradas GRATIS NO pasa por website_event_sale aunque esté instalado. | integrations.md |
| EVTG17 | Bajar la cantidad en el carrito cancela attendees silenciosamente; subirla a mano está | integrations.md |
| EVTG18 | El partner público NO se propaga a la registration. | integrations.md |
| EVTG19 | event_crm puede re-disparar la misma regla; el dedupe es por `existing_leads`. | integrations.md |
| EVTG2 | `auto_confirm` fue ELIMINADO del núcleo v18. | gotchas.md |
| EVTG20 | El booth se "paga" al cobrar la FACTURA, no al confirmar la SO. | integrations.md |
| EVTG21 | Reservar un booth ya cogido CANCELA las otras SO en competencia. | integrations.md |
| EVTG22 | El sponsor se crea solo cuando el booth pasa a `unavailable` vía write. | integrations.md |
| EVTG23 | Con pos_event, cada registration (incluso web) busca todas las pos.session abiertas. | integrations.md |
| EVTG24 | El email de cambio de stage de un track puede salir al crearlo. | integrations.md |
| EVTG3 | `seats_available` se calcula con SQL crudo, no con el ORM. | gotchas.md |
| EVTG4 | `done` (asistió) suma a `seats_used`, NO a `seats_reserved`. | gotchas.md |
| EVTG5 | El aforo del evento y la suma de aforos de tickets pueden NO coincidir, a propósito. | gotchas.md |
| EVTG6 | El `scheduled_date` del mail depende del `interval_type`. | gotchas.md |
| EVTG7 | Un mail `before_event` cuyo evento ya terminó NO se envía. | gotchas.md |
| EVTG8 | El scheduler `after_sub` nunca queda "sent"; se mide contra `seats_taken`. | gotchas.md |
| EVTG9 | Confirmar un registro dispara los schedulers `after_sub` (en caliente o por cron). | gotchas.md |

## functional/expense

| ID | Título | Fichero |
|----|--------|---------|
| EXPCAP1 | ¿Registrar un gasto con producto, importe e impuestos? | capabilities.md |
| EXPCAP10 | ¿Gastos en otra moneda? | capabilities.md |
| EXPCAP11 | ¿Kilometraje / dietas (mileage / per diem)? | capabilities.md |
| EXPCAP12 | ¿Aprobación en dos pasos (manager + contabilidad)? | capabilities.md |
| EXPCAP13 | ¿Adjuntar varios justificantes / verlos en Documents? | capabilities.md |
| EXPCAP14 | ¿Límite de importe por categoría de gasto? | capabilities.md |
| EXPCAP15 | ¿Gastos parcialmente deducibles / % de deducibilidad (disallowed expenses)? | capabilities.md |
| EXPCAP16 | ¿Predicción del producto/categoría desde la descripción? | capabilities.md |
| EXPCAP17 | ¿Detección de gastos duplicados? | capabilities.md |
| EXPCAP18 | ¿Distinguir pagado por empleado vs cuenta/tarjeta de empresa (contabilización distinta)? | capabilities.md |
| EXPCAP19 | ¿Margen al refacturar el gasto al cliente? | capabilities.md |
| EXPCAP2 | ¿Agrupar gastos en un informe y enviarlo a aprobar? | capabilities.md |
| EXPCAP3 | ¿Pagado por el empleado (reembolsable) vs tarjeta de empresa? | capabilities.md |
| EXPCAP4 | ¿Reembolsar al empleado y contabilizar el asiento? | capabilities.md |
| EXPCAP5 | ¿Escanear ticket con OCR? | capabilities.md |
| EXPCAP6 | ¿Crear gasto enviando un email con el ticket? | capabilities.md |
| EXPCAP7 | ¿Refacturar el gasto al cliente en un pedido? | capabilities.md |
| EXPCAP8 | ¿Imputar el gasto a un proyecto / cuenta analítica? | capabilities.md |
| EXPCAP9 | ¿Reembolso vía nómina? | capabilities.md |
| EXPG1 | `state` NO es escribible — ni en línea ni en sheet. | gotchas.md |
| EXPG10 | Un no-manager no puede saltarse la validación. | gotchas.md |
| EXPG11 | Quitar todas las líneas de un sheet no-draft está prohibido | gotchas.md |
| EXPG12 | `payment_state='not_paid'` en company_account NO significa impago. | gotchas.md |
| EXPG13 | Refusar borra los moves DRAFT; si hay alguno posteado, falla. | gotchas.md |
| EXPG14 | `can_be_expensed` es computado, no editable libremente. | gotchas.md |
| EXPG15 | No mezclar payment_mode en un sheet. | integrations.md |
| EXPG16 | El pago de company_account NO se puede editar. | integrations.md |
| EXPG17 | Borrado todo-o-nada de moves/payments. | integrations.md |
| EXPG18 | Cancelar el move desvincula el sheet, NO lo cancela. | integrations.md |
| EXPG19 | Reinvoice no se dispara sin distribución analítica | integrations.md |
| EXPG2 | `own_account` vs `company_account` generan asientos RADICALMENTE distintos. | gotchas.md |
| EXPG20 | Resetear un sheet refacturado pone a 0 las cantidades de la SO. | integrations.md |
| EXPG21 | OCR solo arranca con el ticket como main attachment y en draft sin sheet. | integrations.md |
| EXPG22 | Nómina exige regla `EXPENSES` con cuenta payable. | integrations.md |
| EXPG23 | Quitar un sheet de una nómina ya validada falla. | integrations.md |
| EXPG24 | Los gastos "sample" (demo) nunca crean moves ni se digitalizan. | integrations.md |
| EXPG25 | Cambiar `currency_id` recalcula el total desde la divisa, NO al revés. | gotchas.md |
| EXPG26 | El impuesto se computa DESPUÉS del total y depende de `is_multiple_currency`. | gotchas.md |
| EXPG27 | Editar `tax_ids`/`analytic_distribution`/`account_id` exige `is_editable`, no solo permisos | gotchas.md |
| EXPG28 | Cambiar `employee_id` de una línea reasigna el empleado de TODO el sheet o desvincula la | gotchas.md |
| EXPG29 | La distribución analítica obligatoria se valida en la APROBACIÓN, no al editar. | gotchas.md |
| EXPG3 | Un solo "enviar" puede crear DOS informes. | gotchas.md |
| EXPG30 | No puedes borrar una cuenta analítica usada en CUALQUIER gasto (vía el array de | gotchas.md |
| EXPG31 | Adjuntar documento a un gasto submitted exige write sobre el SHEET, no solo sobre el gasto. | gotchas.md |
| EXPG32 | Mover una línea sincroniza adjuntos por CHECKSUM, no por id. | gotchas.md |
| EXPG33 | `accounting_date` es un related STORED protegido por grupos contables. | gotchas.md |
| EXPG34 | El nombre del asiento se fuerza a `'/'` para neutralizar un `default_name` del contexto. | gotchas.md |
| EXPG35 | `price_unit` solo se recomputa en draft/reported. | gotchas.md |
| EXPG36 | El rate manual no persiste como flag. | gotchas.md |
| EXPG37 | El `state` del sheet deriva de `approval_state` + `payment_state` + moves. | gotchas.md |
| EXPG4 | Los impuestos siempre se calculan como precio-INCLUIDO. | gotchas.md |
| EXPG5 | No puedes reportar una línea con importe 0 ni sin producto. | gotchas.md |
| EXPG6 | `check_amount_not_zero`: no puedes poner total 0 en una línea con sheet. | gotchas.md |
| EXPG7 | Multi-currency: un `currency_rate` manual NO se recalcula. | gotchas.md |
| EXPG8 | El sheet fuerza moneda de compañía si `own_account`. | gotchas.md |
| EXPG9 | No se borran gastos `approved`/`done` ni sheets `post`/`done`. | gotchas.md |

## functional/fleet

| ID | Título | Fichero |
|----|--------|---------|
| FLCAP1 | ¿Vehículos con marca/modelo? | capabilities.md |
| FLCAP10 | ¿Vincular vehículo a empleado (y valor en nómina)? | capabilities.md |
| FLCAP11 | ¿Reserva/booking de vehículos de pool? | capabilities.md |
| FLCAP12 | ¿Gestión de multas de tráfico? | capabilities.md |
| FLCAP13 | ¿Estados del vehículo (nuevo/en uso/de baja)? | capabilities.md |
| FLCAP14 | ¿Distinguir método de adquisición (compra/leasing/renting)? | capabilities.md |
| FLCAP15 | ¿Categorías/segmentos de vehículo y tags? | capabilities.md |
| FLCAP2 | ¿Contratos del vehículo con recordatorio de renovación? | capabilities.md |
| FLCAP3 | ¿Servicios/reparaciones y odómetro? | capabilities.md |
| FLCAP4 | ¿Conductor y su historial? | capabilities.md |
| FLCAP5 | ¿Costes del vehículo en contabilidad? | capabilities.md |
| FLCAP6 | ¿Amortizar el vehículo como activo? | capabilities.md |
| FLCAP7 | ¿Recordatorios de vencimiento? | capabilities.md |
| FLCAP8 | ¿Combustible / consumo? | capabilities.md |
| FLCAP9 | ¿Documentos del vehículo? | capabilities.md |
| FLG1 | Daily cron does NOT auto-renew contracts | gotchas.md |
| FLG10 | Falsy model fields don't propagate | gotchas.md |
| FLG11 | Driver change does NOT close the previous assignation log | gotchas.md |
| FLG12 | Service `purchaser_id` is computed-stored — never re-syncs | gotchas.md |
| FLG13 | `cancelled` services excluded from cost report; other states all included | gotchas.md |
| FLG14 | Yearly recurring cost is fully attributed to ONE month | gotchas.md |
| FLG15 | `future_driver_id` flips partner flags only when state ≠ `waiting_list` | gotchas.md |
| FLG16 | Inheritance of `_inherit` lists matters: `mail.thread` listed first | gotchas.md |
| FLG17 | `fleet_group_user` rule is partner-bound, not employee-bound | gotchas.md |
| FLG18 | `fleet.vehicle.state.name` is UNIQUE GLOBALLY (not per-company); copying a state via studio fails | gotchas.md |
| FLG19 | `delay_alert_contract` ir.config_parameter key is namespaced under `hr_fleet` | gotchas.md |
| FLG2 | `cost_generated` is NEVER posted to accounting | gotchas.md |
| FLG20 | `acquisition_date` defaults to today, but `write_off_date` is free | gotchas.md |
| FLG21 | `_compute_count_all` reads counts bucketed by `active` and indexes by the vehicle's own state | gotchas.md |
| FLG22 | `bill_count` (account_fleet bridge) requires `account.group_account_readonly` | gotchas.md |
| FLG23 | Archiving a vehicle CASCADE-archives its contracts and services — but un-archiving does NOT restore  | gotchas.md |
| FLG24 | `future_driver_id` plan-flag logic reads `self.vehicle_type`, not per-record | gotchas.md |
| FLG25 | `contract_renewal_due_soon` and `contract_renewal_overdue` are mutually exclusive — overdue is NOT " | gotchas.md |
| FLG26 | `_search_get_overdue_contract_reminder` hides a vehicle the moment ANY future/open replacement exist | gotchas.md |
| FLG27 | Contract `_compute_contract_reminder` picks the state of the MAX-expiration non-closed contract; tie | gotchas.md |
| FLG28 | `_compute_contract_reminder` only `@api.depends('log_contracts')` — editing a contract's `expiration | gotchas.md |
| FLG29 | `co2` has `aggregator=None` explicitly; `odometer.value` has `aggregator="max"` | gotchas.md |
| FLG3 | Odometer unit change does NOT convert values | gotchas.md |
| FLG30 | `_clean_vals_internal_user` su-writes `plan_to_change_car/bike` because they are RELATED to res.part | gotchas.md |
| FLG31 | Odometer `_set_odometer` on the VEHICLE never reuses an existing same-day row — it always `create()` | gotchas.md |
| FLG32 | `purchaser_id` on a CONTRACT is a live `related`, but on a SERVICE it is computed-stored | gotchas.md |
| FLG33 | `model_count` on a brand counts only ACTIVE models, but `_read_group` filter uses string `'true'` | gotchas.md |
| FLG34 | Contract state auto-fix ONLY fires when `start_date` or `expiration_date` is in `vals` | gotchas.md |
| FLG35 | Contract `_order = 'state desc, expiration_date'` — drives the non-deterministic tie in FLG27 | gotchas.md |
| FLG36 | Default contract expiration is `start_date + 1 year`, computed at default-time only | gotchas.md |
| FLG37 | Contract `user_id` (Responsible) default reads `active_id` from context — blank when created outside | gotchas.md |
| FLG38 | Renewal reminder cron SKIPS contracts with no `user_id` | gotchas.md |
| FLG39 | `action_accept_driver_change` reassigns ACROSS vehicles — it clears the future driver from every OTH | gotchas.md |
| FLG4 | Odometer can never decrease (UserError) | gotchas.md |
| FLG40 | Odometer list shows newest-by-DATE (`_order='date desc'`) but the vehicle's "current" odometer is hi | gotchas.md |
| FLG41 | EVERY new internal user becomes a Fleet Administrator by default | gotchas.md |
| FLG42 | The user-level record rules differ per model: vehicle is read+write-but-not-create, odometer is full | gotchas.md |
| FLG43 | `manager_id` domain is double-filtered: must be in `fleet_group_manager` AND in `env.companies` | gotchas.md |
| FLG44 | account_fleet auto-creates a service log per vendor-bill line at POST time, guarded against duplicat | gotchas.md |
| FLG45 | Contract `has_open_contract` ignores `expired` — only `state='open' AND expiration_date>=today` coun | gotchas.md |
| FLG5 | `fleet.vehicle.odometer.value` aggregator is `max` | gotchas.md |
| FLG6 | Odometer compute is "MAX value", not "latest date" | gotchas.md |
| FLG7 | Empty-odometer guard on services silently strips on create | gotchas.md |
| FLG8 | Contract default state is `'open'`, not `'futur'` | gotchas.md |
| FLG9 | Reassigning `model_id` overwrites vehicle overrides | gotchas.md |

## functional/forum

| ID | Título | Fichero |
|----|--------|---------|
| FORCAP1 | ¿Limitar a una sola respuesta por pregunta? | capabilities.md |
| FORCAP2 | ¿Validación/moderación de posts nuevos? | capabilities.md |
| FORCAP3 | ¿Notificar a moderadores de posts pendientes? | capabilities.md |
| FORCAP4 | ¿Convertir comentario↔respuesta? | capabilities.md |
| FORCAP5 | ¿Posts relacionados por similitud de tags? | capabilities.md |
| FORCAP6 | ¿Microdata SEO (schema.org QAPage)? | capabilities.md |
| FORCAP7 | ¿Foros privados por grupo? | capabilities.md |
| FORCAP8 | ¿Flag de posts ofensivos por usuarios? | capabilities.md |
| FORG1 | Crear pregunta sin karma → AccessError. | gotchas.md |
| FORG10 | Marcar offensive resta karma y desactiva. | gotchas.md |
| FORG11 | Editar post ajeno requiere `karma_edit_all=300`. | gotchas.md |
| FORG12 | Links/imágenes filtrados por karma. | gotchas.md |
| FORG13 | Comentar exige karma y hereda followers de la pregunta. | gotchas.md |
| FORG14 | Crear tag nuevo sin karma → AccessError. | gotchas.md |
| FORG15 | website. | integrations.md |
| FORG17 | gamification. | integrations.md |
| FORG18 | res.users. | integrations.md |
| FORG2 | Auto-validación solo con karma alto; si no, queda pending. | gotchas.md |
| FORG3 | Responder a pregunta cerrada/borrada → UserError. | gotchas.md |
| FORG4 | No puedes votar tu propio post. | gotchas.md |
| FORG5 | No puedes modificar el voto de otro. | gotchas.md |
| FORG6 | Upvote/downvote con umbrales distintos. | gotchas.md |
| FORG7 | Karma al aceptar es diferencial y reversible. | gotchas.md |
| FORG8 | Cerrar como spam/offensive penaliza karma (×10 si es el primer post). | gotchas.md |
| FORG9 | Reabrir post spam/offensive revierte la penalización. | gotchas.md |

## functional/hr

| ID | Título | Fichero |
|----|--------|---------|
| HRCAP1 | ¿Ficha de empleado + contratos? | capabilities.md |
| HRCAP10 | ¿Firma electrónica de contratos? | capabilities.md |
| HRCAP11 | ¿Portal del empleado (autoservicio)? | capabilities.md |
| HRCAP12 | ¿Multi-compañía / multi-país en empleados? | capabilities.md |
| HRCAP13 | ¿Flota de vehículos asignada a empleados? | capabilities.md |
| HRCAP14 | ¿Planes de incorporación/salida (onboarding/offboarding)? | capabilities.md |
| HRCAP15 | ¿Entradas de trabajo / work entries (para nómina)? | capabilities.md |
| HRCAP16 | ¿Referidos de empleados? | capabilities.md |
| HRCAP17 | ¿Documentos del empleado (contratos, DNI)? | capabilities.md |
| HRCAP18 | ¿Turnos / horarios de trabajo? | capabilities.md |
| HRCAP2 | ¿Ausencias/vacaciones con aprobación y saldos? | capabilities.md |
| HRCAP3 | ¿Gastos con aprobación y reembolso? | capabilities.md |
| HRCAP4 | ¿Control de presencia / fichaje? | capabilities.md |
| HRCAP5 | ¿Reclutamiento (vacantes, candidatos, pipeline)? | capabilities.md |
| HRCAP6 | ¿Evaluaciones / appraisal? | capabilities.md |
| HRCAP7 | ¿Nóminas (payroll)? | capabilities.md |
| HRCAP8 | ¿Organigrama / jerarquía manager? | capabilities.md |
| HRCAP9 | ¿Competencias / skills? | capabilities.md |
| HRG1 | employee.name is stored on resource.resource, not hr.employee | gotchas.md |
| HRG10 | Contract calendar overrides employee calendar | gotchas.md |
| HRG11 | first_contract_date ignores gaps < 4 days | gotchas.md |
| HRG12 | Cron transitions contracts silently | gotchas.md |
| HRG13 | Leave duration is calendar-aware — weekends and holidays auto-excluded | gotchas.md |
| HRG14 | Company/global time off can SPLIT or REFUSE overlapping leaves — two different paths | gotchas.md |
| HRG15 | Timezone affects leave boundaries — calendar TZ, not user TZ | gotchas.md |
| HRG16 | Half-day leaves are hard-coded to 0.5 days | gotchas.md |
| HRG17 | Overlap detection blocks on ALL states except cancel/refuse | gotchas.md |
| HRG18 | Flexible employees get different duration calculation | gotchas.md |
| HRG19 | Accrual frequencies are fixed periods (no "random" option) | gotchas.md |
| HRG2 | Non-HR users are silently redirected to hr.employee.public | gotchas.md |
| HRG20 | Accrual carryover only applies on specific date | gotchas.md |
| HRG21 | Negative leave balance persists across years | gotchas.md |
| HRG22 | Carried-over accrual days can expire separately | gotchas.md |
| HRG23 | Cron auto-cancels future leaves with insufficient accrual | gotchas.md |
| HRG24 | Mandatory days block non-officer users completely | gotchas.md |
| HRG25 | Two distinct attendance crons — auto check-out is NOT a 24h rule | gotchas.md |
| HRG26 | Overtime uses raw PostgreSQL SQL (timezone-aware), not window functions | gotchas.md |
| HRG27 | Attendance manager has separate access from HR manager | gotchas.md |
| HRG28 | Work entry conflict detection uses GiST index (PostgreSQL only) | gotchas.md |
| HRG29 | Leaves create work entries of type 'leave' | gotchas.md |
| HRG3 | Barcode has unique constraint but PIN does not | gotchas.md |
| HRG30 | Work entries not auto-generated — bridge modules required | gotchas.md |
| HRG31 | Department manager change does NOT auto-update existing employee parent_ids | gotchas.md |
| HRG32 | Subordinate computation is recursive but not cached | gotchas.md |
| HRG33 | No circular reference validation on parent_id | gotchas.md |
| HRG34 | Employee is locked to one company | gotchas.md |
| HRG35 | Bank account company must match employee company | gotchas.md |
| HRG36 | Salary rules use safe_eval with limited context | gotchas.md |
| HRG37 | Payslip worked_days auto-computed from work entries | gotchas.md |
| HRG38 | Payslip creates account.move on "Done" | gotchas.md |
| HRG39 | Timesheets ARE account.analytic.line — there is no hr.timesheet model | gotchas.md |
| HRG4 | user_id unique per (user, company), not globally | gotchas.md |
| HRG40 | Timesheet department_id/manager_id copied from employee, not re-synced live | gotchas.md |
| HRG41 | The payslip account.move is created by hr_payroll_account, NOT base hr_payroll | gotchas.md |
| HRG42 | planning.slot.employee_id derives from resource_id, not the other way | gotchas.md |
| HRG43 | Fleet links vehicle to employee via work_contact_id, not user/partner directly | gotchas.md |
| HRG44 | Recruitment hire always goes applicant → hr.candidate → hr.employee (v18+) | gotchas.md |
| HRG45 | Presence state lives in core hr.employee.base, not in hr_presence | gotchas.md |
| HRG5 | Archive does NOT cascade to contracts, leaves, or timesheets | gotchas.md |
| HRG6 | departure_date is purely informational | gotchas.md |
| HRG7 | work_contact_id persists when user_id removed | gotchas.md |
| HRG8 | Avatar falls back to user's avatar silently | gotchas.md |
| HRG9 | Multiple open contracts are allowed | gotchas.md |

## functional/im_livechat

| ID | Título | Fichero |
|----|--------|---------|
| LIVCAP1 | ¿Enrutar al operador por idioma? | capabilities.md |
| LIVCAP10 | ¿Integrar el widget en webs externas? | capabilities.md |
| LIVCAP2 | ¿Reconectar con el operador anterior? | capabilities.md |
| LIVCAP3 | ¿Equilibrar carga entre operadores? | capabilities.md |
| LIVCAP4 | ¿Alias público del operador distinto de su nombre? | capabilities.md |
| LIVCAP5 | ¿El chatbot crea un partner desde el email? | capabilities.md |
| LIVCAP6 | ¿Enviar la transcripción por email? | capabilities.md |
| LIVCAP7 | ¿Enviar al visitante su historial de navegación? | capabilities.md |
| LIVCAP8 | ¿Reiniciar el chatbot en la misma sesión? | capabilities.md |
| LIVCAP9 | ¿Medir satisfacción y duración? | capabilities.md |
| LIVG1 | La regla que gana es la de menor `sequence`, no la "más específica". | gotchas.md |
| LIVG10 | Solo UN rating por sesión. | gotchas.md |
| LIVG11 | El país del visitante anónimo depende de GeoIP. | gotchas.md |
| LIVG12 | Los pasos de bienvenida NO se guardan como `mail.message` hasta la primera interacción. | gotchas.md |
| LIVG13 | `question_selection` sin respuestas rompe el flujo / `ValidationError`. | gotchas.md |
| LIVG14 | La respuesta a email/phone se guarda como `user_raw_answer` (HTML crudo) en `chatbot.message`, no en | gotchas.md |
| LIVG15 | discuss. | integrations.md |
| LIVG17 | website (widget). | integrations.md |
| LIVG18 | crm (`crm_livechat`). | integrations.md |
| LIVG19 | helpdesk. | integrations.md |
| LIVG2 | Reglas con país tienen prioridad ABSOLUTA sobre las sin país. | gotchas.md |
| LIVG3 | Sin operador online no se abre chat (salvo chatbot). | gotchas.md |
| LIVG4 | "Disponible" = `im_status == 'online'`, no es estar dado de alta. | gotchas.md |
| LIVG5 | `chatbot_only_if_no_operator` desactiva el bot si hay CUALQUIER operador disponible. | gotchas.md |
| LIVG6 | Regla con chatbot inactivo o sin pasos se ignora silenciosamente. | gotchas.md |
| LIVG7 | `forward_operator` puede no encontrar a nadie y el script continúa sin avisar. | gotchas.md |
| LIVG8 | GC de sesiones vacías en cada autovacuum, con colchón de 1h. | gotchas.md |
| LIVG9 | GC de sesiones RTC inactivas dentro de `_get_operator`. | gotchas.md |

## functional/lunch

| ID | Título | Fichero |
|----|--------|---------|
| LUG1 | Wallet balance is virtual, not accounting | gotchas.md |
| LUG10 | `topping_quantity='1'` means EXACTLY one, not max one | gotchas.md |
| LUG11 | Each supplier owns its own `ir.cron`, persisted via `ir.model.data` noupdate=True | gotchas.md |
| LUG12 | Cron `nextcall` may skip a day on creation | gotchas.md |
| LUG13 | Removing a weekday cancels EXISTING future orders for that weekday | gotchas.md |
| LUG14 | `automatic_email_time` is bounded 0..12 (am/pm switch) | gotchas.md |
| LUG15 | Supplier `responsible_id` domain in v18 uses `groups_id`, in v19 `all_group_ids` | gotchas.md |
| LUG16 | Alert chat broadcast targets ALL users who ordered in the window, not all employees | gotchas.md |
| LUG17 | `lunch.alert.available_today` operator parsing differs v18 vs v19 | gotchas.md |
| LUG18 | `notified` flag prevents re-sending and is NEVER reset | gotchas.md |
| LUG19 | `lunch_notify_message` is per-company, not per-supplier or per-product | gotchas.md |
| LUG2 | Negative balance allowed up to `lunch_minimum_threshold` | gotchas.md |
| LUG20 | `lunch.product.company_id` follows supplier, not category | gotchas.md |
| LUG21 | `lunch.location.available_location_ids` empty = available everywhere | gotchas.md |
| LUG22 | `automatic_email_time=12.0 + moment='pm'` means END OF DAY, not noon | gotchas.md |
| LUG23 | `_send_auto_email` silently no-ops if supplier not available today or has no orders | gotchas.md |
| LUG24 | Cron only recomputes `nextcall` when a field in `CRON_DEPENDS` changes | gotchas.md |
| LUG25 | Alert `available_today` IGNORES the weekday Booleans whenever `until` is set | gotchas.md |
| LUG26 | Alert `CRON_DEPENDS` is a DIFFERENT set than supplier's | gotchas.md |
| LUG27 | A `chat`-mode alert auto-deletes its own cron once `until` passes | gotchas.md |
| LUG28 | `create()` always forces `state='new'` for matching, so a merge can fold into an existing cart line | gotchas.md |
| LUG29 | `update_quantity` deactivates (does not unlink) a line when quantity drops to ≤0 | gotchas.md |
| LUG3 | `display_add_button` recomputes balance per row | gotchas.md |
| LUG30 | `_compute_display_add_button` does its own per-user `_read_group` over today's `new` orders | gotchas.md |
| LUG31 | Archiving a supplier or category cascades active state to its products via `toggle_active` | gotchas.md |
| LUG32 | `lunch.product.is_available_at` returns the supplier's locations, treating empty as "everywhere" | gotchas.md |
| LUG4 | Orders auto-merge by (user, product, date, note, location, toppings) | gotchas.md |
| LUG5 | v18 only merges into `new`/`ordered` rows; v19 merges any state | gotchas.md |
| LUG6 | Deleting an order is restricted to states `new`/`cancelled` | gotchas.md |
| LUG7 | `action_reorder` always sets `state='ordered'`, ignoring deadline | gotchas.md |
| LUG8 | Cancelled orders stay in DB but disappear from wallet calc | gotchas.md |
| LUG9 | The 3 topping fields share one many2many table | gotchas.md |

## functional/mail

| ID | Título | Fichero |
|----|--------|---------|
| IG9 | (Indaws) — desactivar un cron de notificación corta el aviso, | config_guide.md |
| MCG1 | Fetchmail merged into mail module | gotchas.md |
| MCG10 | Failed message_new uses separate cursor for bounce | gotchas.md |
| MCG100CP | `mail.scheduled.message` (NUEVO model) es DISTINTO de `composer.scheduled_date`. Scheduled.message e | integrations.md |
| MCG101CP | `auto_commit=True` en `action_send_mail` hace `cr.commit()` entre chunks. | integrations.md |
| MCG102DC | `_mail_post_access='read'` permite a CUALQUIER user con read access postear. En `channel_type='chann | integrations.md |
| MCG103DC | `channel_partner_ids` inverse (`_inverse_channel_partner_ids`) crea/borra `discuss.channel.member` d | integrations.md |
| MCG104DC | Mentions (`@user`) en body: front-end parsea body buscando `<a data-oe-model='res.partner' data-oe-i | integrations.md |
| MCG105DC | `channel_type='chat'` máximo 2 members (`_constraint_partners_chat`). Intentar añadir 3º via `channe | integrations.md |
| MCG106DC | `notification_type` en res.users: `'inbox'` (visible en /odoo/discuss, no email) o `'email'`. Inbox  | integrations.md |
| MCG107DC | `_notify_get_recipients` override en discuss.channel hace SQL custom que asume `mail.message.model=' | integrations.md |
| MCG108DC | Sub-channels (`parent_channel_id`) heredan ACL del parent. Members del parent NO son auto-members de | integrations.md |
| MCG109DC | `last_interest_dt` (Datetime indexed) se actualiza en cada message_post. Sirve para sort kanban Disc | integrations.md |
| MCG11 | mail.message message_id contains model+res_id | gotchas.md |
| MCG110DC | el `mute` nativo de Discuss (`mute_until_dt`) | config_guide.md |
| MCG111DC | Bus notifications (`bus.bus._sendone`) NO se persisten | integrations.md |
| MCG112MC | `alias_domain_id` en `mail.alias` es | integrations.md |
| MCG113MC | Migración v16→v17/v18 (`_migrate_icp_to_domain`): mueve `mail.catchall.domain`, `mail.bounce.alias`, | integrations.md |
| MCG114MC | `_check_alias_domain_id_mc` solo se dispara si `alias_domain_id.company_ids` NO vacío. Domains "glob | integrations.md |
| MCG115MC | Crear alias en company A con `alias_parent_model_id+thread_id` apuntando a record de company B → Val | integrations.md |
| MCG116MC | `default_from` en `mail.alias.domain` puede ser left-part (`notifications`) o full email (`notificat | integrations.md |
| MCG117MC | `record_alias_domain_id` en `mail.message` se snapshotea AT POST TIME (`message_post` MCG93MP). Si r | integrations.md |
| MCG118MC | Bounce y catchall son UNIQUE per (alias, name) via SQL constraint. Crear 2 domains con `name='odoo.c | integrations.md |
| MCG119AT | Composer attachments con `res_model='mail.compose.message'` y `res_id=0` quedan | integrations.md |
| MCG12 | References header limited to 3 ancestors | gotchas.md |
| MCG120AT | `_process_attachments_for_post` solo transfiere ownership si `create_uid == self.uid`. Si admin sube | integrations.md |
| MCG121AT | CID inline: body `<img src="cid:image1">` requiere matching `attachment.cid='image1'` (campo `cid` a | integrations.md |
| MCG122AT | `data:image/png;base64,...` inline embebido en body → se EXTRAE a ir.attachment en background con `i | integrations.md |
| MCG123AT | `mail.thread.main.attachment` mixin auto-asigna `message_main_attachment_id` al PRIMER attachment de | integrations.md |
| MCG124AT | `attachment.public=True` + `access_token` set → URL `/web/image/<id>?access_token=X` accesible sin l | integrations.md |
| MCG125AT | Mass mail mode: attachments del template se REUSAN sin copia (M2M directo). Mismo `ir.attachment` li | integrations.md |
| MCG126AT | "Odoo arrastra todos los adjuntos previos del thread en cada respuesta del cliente" — | integrations.md |
| MCG13 | reply_to_force_new breaks return threading | gotchas.md |
| MCG14 | Notification emails auto-delete by default | gotchas.md |
| MCG15 | Pessimistic lock writes exception BEFORE sending | gotchas.md |
| MCG16 | _notify_thread_by_email force_send cap: 100 | gotchas.md |
| MCG17 | Subtype internal=True hides from portal/public | gotchas.md |
| MCG18 | `message_post` auto-suscribe al autor/asignado → te vuelves follower sin querer; revisa suscripcione | config_guide.md |
| MCG19 | mail.alias.domain replaces ICPs | gotchas.md |
| MCG2 | Fetchmail commits per email | gotchas.md |
| MCG20 | body on mail.message is sanitized HTML, body_html on mail.mail is raw | gotchas.md |
| MCG21 | Three separate batch ICPs control throughput | gotchas.md |
| MCG22 | Reply-To formatted to 68 chars max | gotchas.md |
| MCG23 | SO confirmation email requires context send_email=True | gotchas.md |
| MCG24 | CRM lead message_post_after_hook does cross-record partner linking | gotchas.md |
| MCG25 | project.task auto-creates partner from unknown emails; crm.lead does NOT | gotchas.md |
| MCG26 | `stock_move_email_validation` por defecto False y solo salidas; no todos los albaranes emiten. | config_guide.md |
| MCG27 | mail.thread.cc does NOT subscribe CC recipients — only stores them | gotchas.md |
| MCG28 | _track_template sends stage emails as mt_note (internal) | gotchas.md |
| MCG29 | rating_apply auto-sets task state from customer satisfaction | gotchas.md |
| MCG3 | IMAP toggles \Seen around processing, but a CAUGHT failure still marks it read | gotchas.md |
| MCG30 | account.move deletes unrecognized attachments from alias emails | gotchas.md |
| MCG31 | mail.thread variants not interchangeable | gotchas.md |
| MCG32 | m2m/o2m fields ARE tracked in v18 (as char display-name lists) | gotchas.md |
| MCG32T | `_mail_post_access='read'` permite a usuarios portal/share comentar el registro. Bajar de `'write'`  | integrations.md |
| MCG33 | Tracking only fires for fields with `tracking=` AND a real value change | gotchas.md |
| MCG33T | `tracking=N` numérico decide ORDEN de display en chatter, NO prioridad de procesamiento. N=1 sale an | integrations.md |
| MCG34 | Tracking display order is by field `tracking` integer, not write order | gotchas.md |
| MCG34T | `_track_finalize()` corre en | integrations.md |
| MCG35 | mail.activity `state` is computed, never stored; `done` ≠ a state value | gotchas.md |
| MCG35T | `_track_template(changes)` se llama con dict de cambios pero NO está garantizado el orden. Si la res | integrations.md |
| MCG36 | Activity overdue/today depends on the assignee timezone | gotchas.md |
| MCG36T | `_track_filter_for_display(tracking_values)` se llama por record-with-lang en `_notify_get_recipient | integrations.md |
| MCG37 | alias_name is silently sanitized (lowercased, accents stripped, bad chars → '-') | gotchas.md |
| MCG37T | `auto_join=True` en `message_ids` significa que cualquier search sobre el modelo HEREDA un JOIN impl | integrations.md |
| MCG38 | Blacklist "removal" is a soft-archive, not a delete | gotchas.md |
| MCG38T | Campo M2M `message_partner_ids` tiene `inverse=_inverse_message_partner_ids` que LLAMA `message_subs | integrations.md |
| MCG39 | Two different "scheduled message" models with opposite meanings | gotchas.md |
| MCG39T | `_primary_email` debe ser un campo Char/Email real para que loop detection (`_detect_loop_sender_dom | integrations.md |
| MCG4 | POP3 batches of 50 with reconnect | gotchas.md |
| MCG40 | message reactions/translations are deduped by partial SQL unique indexes, not _sql_constraints | gotchas.md |
| MCG40T | `tracking_disable=True` context | integrations.md |
| MCG41 | process_email_queue ignores future-scheduled mails but still respects the batch cap | gotchas.md |
| MCG41T | Tracking de campos `Many2one` guarda el display_name del registro AL MOMENTO del tracking, NO el id. | integrations.md |
| MCG42AM | `mail_activity_automation_skip=True` context | integrations.md |
| MCG43AM | `activity_state` computed usa SQL puro (no @api.depends estándar) con `_search_activity_state` que r | integrations.md |
| MCG44AM | `activity_date_deadline` tiene `store=False` (siempre recompute on-read). Domain `[('activity_date_d | integrations.md |
| MCG45AM | `write({'active': False})` o `unlink()` borra | integrations.md |
| MCG46AM | `activity_type.res_model` filtra qué types son válidos por modelo. Si type tiene `res_model=False` ( | integrations.md |
| MCG47AM | `default_user_id` en `mail.activity.type` se usa SOLO si `activity_schedule` no recibe `user_id` exp | integrations.md |
| MCG48AM | Feedback (`activity_feedback`) llama `action_feedback` que internamente: (a) `message_post` con body | integrations.md |
| MCG49AM | `my_activity_date_deadline` `@depends_context('uid')`. Cambiar de usuario (impersonate) NO invalida  | integrations.md |
| MCG5 | Forward detection can override threading | gotchas.md |
| MCG50TM | `lang` field expression se renderiza con `_render_lang(res_ids)`. Si la expresión es `{{ object.part | integrations.md |
| MCG51TM | `_unrestricted_rendering=True` permite acceso a TODO el env desde la expression QWeb. Templates pued | integrations.md |
| MCG52TM | `model_id` con `ondelete='cascade'`: desinstalar el módulo que provee el modelo → borra templates de | integrations.md |
| MCG53TM | `partner_to` parsing acepta tanto `"1,2,3"` (CSV) como `"[1, 2, 3]"` (Python literal). `_parse_partn | integrations.md |
| MCG54TM | `use_default_to=True` LLAMA `_message_get_default_recipients` del modelo (`mail/models/models.py:204 | integrations.md |
| MCG55TM | `attachment_ids` (M2M) se reasignan a `mail.template.<id>` en `_fix_attachment_ownership()` en cada  | integrations.md |
| MCG56TM | `report_template_ids` se renderiza POR record EN send_mail (no en preview). PDF grandes + batches gr | integrations.md |
| MCG57TM | `scheduled_date` parsea expression DESPUÉS de render. Si expresión devuelve naive datetime sin tz →  | integrations.md |
| MCG58TM | `find_or_create_partners=True` en `_generate_template_recipients` crea | integrations.md |
| MCG59TM | `auto_delete=True` borra `mail.mail` post-send. Pero `mail.message` queda en chatter siempre (except | integrations.md |
| MCG6 | Catchall guard bounces if ALL TOs are catchall | gotchas.md |
| MCG60TM | `email_layout_xmlid` debe ser XMLID válido AL momento de send_mail. Si XMLID no resuelve → fallback  | integrations.md |
| MCG61TM | Template QWeb body inline (`{{ object.X }}`) vs view-based (`email_template_X` view ref): inline es  | integrations.md |
| MCG62AC | `chaining_type='trigger'` y `chaining_type='suggest'` son MUTUAMENTE EXCLUSIVOS via `_compute_sugges | integrations.md |
| MCG63AC | `delay_from='previous_activity'` requiere `activity_previous_deadline` en context. Triggered chains  | integrations.md |
| MCG64AC | `category='upload_file'` cambia UX: el botón "Mark Done" del kanban widget aparece como upload dialo | integrations.md |
| MCG65AC | `default_user_id` archivado → `activity_schedule` cae a `self.env.uid` SIN warning. Para audit puro: | integrations.md |
| MCG66AC | `_unlink_except_todo` protege 5 master types contra delete via `@api.ondelete(at_uninstall=False)`.  | integrations.md |
| MCG67AC | `res_model` change tras crear: si type ya tiene activities asociadas en el modelo viejo, esas activi | integrations.md |
| MCG68AC | Triggered chain falla silenciosamente si `triggered_next_type_id.res_model` != `previous.res_model`  | integrations.md |
| MCG69AL | `alias_force_thread_id=0` (Integer field) y `alias_force_thread_id=False` son AMBIGUOS en Python (0  | integrations.md |
| MCG7 | alias_contact=followers requires existing record | gotchas.md |
| MCG70AL | `alias_defaults` validated con `literal_eval` | integrations.md |
| MCG71AL | `alias_name` sanitization (`_sanitize_alias_name`) hace `remove_accents().lower().split('@')[0]` + r | integrations.md |
| MCG72AL | `alias_contact='followers'` requiere RECORD existente (force_thread o parent_thread). Alias que CREA | integrations.md |
| MCG73AL | SQL UNIQUE INDEX en `(alias_name, COALESCE(alias_domain_id, 0))` | integrations.md |
| MCG74AL | `alias_incoming_local=True` matchea SOLO localpart (`info` en `info@anything.com`). Útil para multi- | integrations.md |
| MCG75AL | `alias_parent_model_id` + `alias_parent_thread_id` no fuerzan que `parent` exista. Si parent record  | integrations.md |
| MCG76AL | `alias_bounced_content` (Html, translate=True) usa `lang_author` (del partner que envió email). Si p | integrations.md |
| MCG77GW | Loop detection (`_detect_loop_sender`) usa `mail.message.message_type='email'` para contar replies e | integrations.md |
| MCG78GW | `mail.gateway.allowed` whitelist bypasea loop detection PERO no bypasea otras checks (alias_contact, | integrations.md |
| MCG79GW | `mail.catchall.domain.allowed` ICP es CSV adicional a `mail.alias.domain.name`. Aliases con domain e | integrations.md |
| MCG8 | Loop detection threshold: 20 records in 2 hours | gotchas.md |
| MCG80GW | IMAP `\Seen` marcado ANTES de processing (`mail_thread.py` flujo): si proceso falla DESPUÉS de marca | integrations.md |
| MCG81GW | Fetchmail commit POR-email evita rollback masivo pero rompe transaccionalidad: parte del batch puede | integrations.md |
| MCG82GW | `bounce_alias@alias_domain.name` se usa para Return-Path del SMTP outgoing Y como detector de bounce | integrations.md |
| MCG83GW | `message_route` busca aliases sorted by domain match BUT cuando dos aliases tienen mismo `alias_name | integrations.md |
| MCG84GW | Bounce handling actualiza `mail.notification.notification_status='bounce'` solo si `bounced_message` | integrations.md |
| MCG85MP | `subtype_xmlid` se resuelve con `_xmlid_to_res_id` (raise si no existe). `subtype_xmlid='mail.mt_ine | integrations.md |
| MCG86MP | `_get_allowed_message_post_params` por defecto retorna SET `{attachment_ids, body, email_add_signatu | integrations.md |
| MCG87MP | `body` puede ser `str` o `Markup`. Strings se ESCAPAN (`escape(body)`) → HTML literal aparece como t | integrations.md |
| MCG88MP | `mail_post_autofollow` context (default True) auto-suscribe `partner_ids`. Útil pero | integrations.md |
| MCG89MP | Author auto-subscribe: si `mail_create_nosubscribe` no está y user.active=True → real_author_id = us | integrations.md |
| MCG9 | mail.gateway.allowed whitelist bypasses loop detection | gotchas.md |
| MCG90MP | `parent_id` con `_mail_flat_thread=True`: si pasas `parent_id` explícito a mensaje en thread que ya  | integrations.md |
| MCG91MP | `partner_ids` validation: SOLO list of int. Pasar `[(6, 0, [ids])]` (command) → `is_list_of(partner_ | integrations.md |
| MCG92MP | kwargs no-fields-de-mail.message van a `notif_kwargs` y se propagan a `_notify_thread`. Override en  | integrations.md |
| MCG93MP | `record_alias_domain_id` y `record_company_id` se RESUELVEN per-message en `message_post` desde el r | integrations.md |
| MCG94CP | `composition_batch` (computed) = `len(res_ids) > 1`. Cambia comportamiento de render: comment mode c | integrations.md |
| MCG95CP | `mass_mail` mode: `subtype_id=False` HARDCODED (`_prepare_mail_values_static`:945). Esto evita notif | integrations.md |
| MCG96CP | `_compute_attachment_ids` en comment mode genera attachments AL VUELO desde template reports. Si use | integrations.md |
| MCG97CP | `reply_to_force_new=True` (boolean) NO es lo mismo que `reply_to_mode='new'`. `reply_to_mode` es UX  | integrations.md |
| MCG98CP | `template_id` set + `subject` previamente lleno: `_compute_subject` SOBRESCRIBE subject con render d | integrations.md |
| MCG99CP | `composition_mode='mass_mail'` + `force_send=True` (default False) hace send inline en chunks. Sin f | integrations.md |

## functional/mailing

| ID | Título | Fichero |
|----|--------|---------|
| MMCAP1 | ¿Editor de email drag-and-drop (snippets)? | capabilities.md |
| MMCAP10 | ¿Throttling / límite diario de envío? | capabilities.md |
| MMCAP2 | ¿Listas de correo y contactos? | capabilities.md |
| MMCAP3 | ¿Segmentar destinatarios por dominio sobre cualquier modelo? | capabilities.md |
| MMCAP4 | ¿A/B testing de campañas? | capabilities.md |
| MMCAP5 | ¿Métricas (sent/delivered/opened/clicked/bounced/replied)? | capabilities.md |
| MMCAP6 | ¿Unsubscribe y lista negra? | capabilities.md |
| MMCAP7 | ¿Programar el envío? | capabilities.md |
| MMCAP8 | ¿Envío por SMS? | capabilities.md |
| MMCAP9 | ¿Tracking de clics con UTM? | capabilities.md |
| MMG1 | mailing.trace survives mail.mail deletion | gotchas.md |
| MMG10 | Unsubscribe behavior differs by model | gotchas.md |
| MMG11 | HMAC tokens in URLs use database.secret | gotchas.md |
| MMG12 | No record rules on mass_mailing | gotchas.md |
| MMG13 | mail_server_id on mailing overrides company default | gotchas.md |
| MMG14 | body_arch vs body_html | gotchas.md |
| MMG15 | medium_id auto-computed to "email" | gotchas.md |
| MMG16 | Campaign required for A/B testing | gotchas.md |
| MMG17 | marketing.trace.process_event() is the orchestration hub | gotchas.md |
| MMG18 | Social posts are NOT mass_mailing | gotchas.md |
| MMG19 | FROM address silently replaced | gotchas.md |
| MMG2 | Opt-in on ANY list overrides opt-out — keyed on email_normalized, not contact | gotchas.md |
| MMG20 | Preemptive exception state on send | gotchas.md |
| MMG21 | No SSL certificate validation in v18 | gotchas.md |
| MMG22 | Empty from_filter = catch-all server | gotchas.md |
| MMG23 | No SMTP reconnection logic | gotchas.md |
| MMG24 | A click implies an open | gotchas.md |
| MMG25 | mailing.trace links via mass_mailing_id, not mailing_id | gotchas.md |
| MMG26 | Bridge KPIs count by source_id, not by mailing | gotchas.md |
| MMG27 | "In queue" does not mean "sending" | gotchas.md |
| MMG28 | A/B testing config is SPLIT between the mailing and utm.campaign | gotchas.md |
| MMG29 | trace_status labels are NOT their technical values | gotchas.md |
| MMG3 | Blacklist is global, opt-out is per-list | gotchas.md |
| MMG30 | set_clicked() does not change trace_status | gotchas.md |
| MMG31 | set_opened() refuses to downgrade open/reply | gotchas.md |
| MMG32 | Send batch is 50, not 1000 | gotchas.md |
| MMG33 | Setting opt_out_reason_id or opt_out_datetime silently forces opt_out=True | gotchas.md |
| MMG34 | Cron resumes 'sending' mailings — duplicate-safe but not crash-clean | gotchas.md |
| MMG35 | _process_mass_mailing_queue runs as user_root but switches to the mailing owner's context | gotchas.md |
| MMG36 | mass_mailing does not add Return-Path | gotchas.md |
| MMG37 | The "Delivered" / `received_ratio` KPI is NOT a real delivery receipt | gotchas.md |
| MMG38 | KPI buckets are mutually exclusive by latest status — lower buckets hide advanced traces | gotchas.md |
| MMG39 | clicks_ratio is a separate raw-SQL compute with its own denominator | gotchas.md |
| MMG4 | Auto-blacklist after 5 bounces — but partner-gated | gotchas.md |
| MMG40 | `total` (expected recipients) is itself scaled down by ab_testing_pc | gotchas.md |
| MMG41 | reply_to_mode is auto-computed from the target model, silently | gotchas.md |
| MMG42 | reply_to_mode='update' keeps the mail.mail log even when keep_archives is off | gotchas.md |
| MMG43 | Statistics read traces with sudo — they ignore all access scoping | gotchas.md |
| MMG44 | mailing.contact email is NOT auto-normalized into `email` | gotchas.md |
| MMG45 | mailing.trace has a hard `res_id != 0` SQL constraint | gotchas.md |
| MMG46 | trace→campaign is a STORED related, trace→medium/source are NOT | gotchas.md |
| MMG47 | link.tracker.click→mailing_trace_id is nulled if the trace was deleted | gotchas.md |
| MMG5 | mailing.list is NOT the recipient model | gotchas.md |
| MMG6 | A/B test recipients are random per send | gotchas.md |
| MMG7 | auto_commit per batch during send | gotchas.md |
| MMG8 | Tracking pixel requires image loading | gotchas.md |
| MMG9 | Link shortening modifies ALL URLs except a blacklist | gotchas.md |

## functional/maintenance

| ID | Título | Fichero |
|----|--------|---------|
| MC1 | `location` (Char) REMOVED from core → `location_id` (M2O) in new `stock_maintenance` | v19_changes.md |
| MC2 | `mrp_maintenance` now depends on `stock_maintenance`, not `maintenance` | v19_changes.md |
| MC3 | Email alias moved from `maintenance.equipment.category` → `maintenance.team` | v19_changes.md |
| MC4 | `_sql_constraints` → `models.Constraint` | v19_changes.md |
| MC5 | `schedule_end` is a NEW field (v19 only) | v19_changes.md |
| MC6 | `duration`: free Float (v18) → computed+stored from `schedule_end` (v19) | v19_changes.md |
| MC7 | Recurrence copy propagates `schedule_end` | v19_changes.md |
| MC8 | New `index='btree_not_null'` on hot M2O fields | v19_changes.md |
| MC9 | `hr_maintenance` restructured into 3 model files | v19_changes.md |
| MTCAP1 | ¿Equipos con categorías? | capabilities.md |
| MTCAP10 | ¿Subequipos / ubicación / nº de serie? | capabilities.md |
| MTCAP11 | ¿Calendario de mantenimientos (vista planning)? | capabilities.md |
| MTCAP12 | ¿Estados/etapas configurables del flujo? | capabilities.md |
| MTCAP13 | ¿Prioridad y duración estimada de la solicitud? | capabilities.md |
| MTCAP14 | ¿Repuestos/piezas del stock consumidas en la reparación? | capabilities.md |
| MTCAP15 | ¿Generación automática de la solicitud preventiva recurrente? | capabilities.md |
| MTCAP2 | ¿Mantenimiento correctivo y preventivo? | capabilities.md |
| MTCAP3 | ¿Preventivo programado/recurrente? | capabilities.md |
| MTCAP4 | ¿Equipos de mantenimiento y kanban? | capabilities.md |
| MTCAP5 | ¿KPIs MTBF/MTTR/próximo fallo? | capabilities.md |
| MTCAP6 | ¿Vincular equipo a centro de trabajo MRP? | capabilities.md |
| MTCAP7 | ¿Crear solicitud desde una orden/centro de fabricación? | capabilities.md |
| MTCAP8 | ¿Asignar equipo a empleado/departamento? | capabilities.md |
| MTCAP9 | ¿Coste del mantenimiento? | capabilities.md |
| MTG1 | `archive` field instead of `active` on maintenance.request | gotchas.md |
| MTG10 | Category deletion blocked even with archived items | gotchas.md |
| MTG11 | Equipment visibility is via followers, not ownership | gotchas.md |
| MTG12 | Activity deadline uses date part of datetime | gotchas.md |
| MTG13 | `schedule_end` recomputed when schedule_date changes | gotchas.md |
| MTG14 | `_get_default_team_id()` falls back to ANY company's team | gotchas.md |
| MTG15 | No cron jobs in core module | gotchas.md |
| MTG16 | Stages are company-agnostic | gotchas.md |
| MTG17 | Recurrence boundary uses `<=` comparison | gotchas.md |
| MTG18 | `hr_maintenance` is auto_install | gotchas.md |
| MTG19 | `hr_maintenance` makes `equipment.owner_user_id` non-writable but `equipment_assign_to='other'` rese | gotchas.md |
| MTG2 | Serial number uniqueness is global, not per-company | gotchas.md |
| MTG20 | `hr_maintenance` overrides `equipment_id` domain on request, hiding equipment | gotchas.md |
| MTG21 | `hr_maintenance.message_new` looks up the WRONG employee (v18) | gotchas.md |
| MTG22 | mrp_maintenance: only `block_workcenter=True` requests actually block the work center | gotchas.md |
| MTG23 | mrp_maintenance reschedules your request silently when the slot is taken | gotchas.md |
| MTG24 | v19 moved location/serial-matching OUT of core into `stock_maintenance` | gotchas.md |
| MTG25 | v19 moved the email alias from equipment category to maintenance team | gotchas.md |
| MTG26 | v19 `duration` became a stored compute derived from `schedule_end` | gotchas.md |
| MTG27 | `_compute_user_id` SILENTLY CLEARS the technician on multi-company company change | gotchas.md |
| MTG28 | MTBF/MTTR use Date arithmetic, not Datetime — sub-day repairs report MTTR=0 | gotchas.md |
| MTG29 | Only `corrective` + `done` requests feed MTBF/MTTR — preventive maintenance is invisible to reliabil | gotchas.md |
| MTG3 | `_default_stage()` picks FIRST stage globally | gotchas.md |
| MTG30 | `maintenance_count` counts ALL requests including archived; `maintenance_open_count` excludes archiv | gotchas.md |
| MTG31 | `_get_activity_note` / activity reassigns to a fallback chain | gotchas.md |
| MTG32 | `_need_new_activity` rebuilds the activity ONLY when `equipment_id` changes | gotchas.md |
| MTG33 | Pre-v17 preventive recurrence was CRON-driven, not write()-driven | gotchas.md |
| MTG34 | `request_date` and `close_date` are Date, `schedule_date` is Datetime — mixing them in domains/compu | gotchas.md |
| MTG35 | `_default_stage()` / `_read_group_stage_ids` bypass record rules with `sudo()` | gotchas.md |
| MTG36 | Equipment kanban shows ALL categories via sudo group-expand, even ones the user can't read | gotchas.md |
| MTG37 | `every base.group_user` (i.e. every internal user) has FULL CRUD on `maintenance.request` | gotchas.md |
| MTG38 | Manager group implies `base.group_user` AND root/admin are hardcoded into it via data | gotchas.md |
| MTG39 | `maintenance.team` is NOT a `mail.thread`/`mail.alias.mixin` in v18 core | gotchas.md |
| MTG4 | Kanban state auto-resets on stage change | gotchas.md |
| MTG40 | `equipment.write()` re-subscribes the owner on every owner change but NEVER unsubscribes the old one | gotchas.md |
| MTG41 | Team dashboard counters silently EXCLUDE done and archived requests, and bucket scheduled by YEAR | gotchas.md |
| MTG42 | `MaintenanceTeam._compute_equipment` writes `equipment_count` but that field is NOT declared on the  | gotchas.md |
| MTG43 | `category_id` on the request is a stored related, read-only — you cannot set a category without equi | gotchas.md |
| MTG44 | `_compute_maintenance_team_id` runs on BOTH mixin and request, with different triggers — equipment t | gotchas.md |
| MTG45 | `mail.activity.type` ACL is granted to managers by THIS module (cross-model grant) | gotchas.md |
| MTG46 | The recurrence copy uses `_default_stage()` (global first stage), NOT "the first stage by sequence" | gotchas.md |
| MTG47 | `repeat_interval` constraint raises a NON-translated, bare string | gotchas.md |
| MTG48 | `done` is a non-stored related to `stage_id.done` — filtering/searching by `done` always hits the st | gotchas.md |
| MTG5 | Recurrence copies happen synchronously in `write()` | gotchas.md |
| MTG6 | `recurring_maintenance` forced False for corrective | gotchas.md |
| MTG7 | `close_date` managed in BOTH create() and write() | gotchas.md |
| MTG8 | Category `fold` has mutual dependency | gotchas.md |
| MTG9 | `maintenance.mixin.maintenance_ids` has no inverse_name | gotchas.md |

## functional/membership

| ID | Título | Fichero |
|----|--------|---------|
| MEMCAP1 | ¿Vender membresías como productos de factura? | capabilities.md |
| MEMCAP10 | ¿Renovación automática de la membresía? | capabilities.md |
| MEMCAP11 | ¿Recordatorio de vencimiento al socio? | capabilities.md |
| MEMCAP12 | ¿Carnet/tarjeta de socio imprimible? | capabilities.md |
| MEMCAP13 | ¿Cobro recurrente automático de la cuota? | capabilities.md |
| MEMCAP2 | ¿Facturar miembros en lote? | capabilities.md |
| MEMCAP3 | ¿Membresías gratuitas? | capabilities.md |
| MEMCAP4 | ¿Membresías asociadas/familiares? | capabilities.md |
| MEMCAP5 | ¿Seguimiento automático del ciclo de vida? | capabilities.md |
| MEMCAP6 | ¿Informe de análisis de membresías? | capabilities.md |
| MEMCAP7 | ¿Precio negociado por socio? | capabilities.md |
| MEMCAP8 | ¿Directorio público de socios? | capabilities.md |
| MEMCAP9 | ¿Estados del socio (en proceso/facturado/pagado/vencido/gratis)? | capabilities.md |
| MEMG1 | La máquina de estados la conduce la FACTURA, no un campo manual. | gotchas.md |
| MEMG10 | Cancelar = revertir/cancelar la factura, no editar la línea. | gotchas.md |
| MEMG11 | Las líneas se auto-crean tanto en `create` como en `write` de la línea de factura, con dedup | gotchas.md |
| MEMG12 | Multi-línea/solapamiento: el socio elige la PRIMERA línea que contiene hoy. | gotchas.md |
| MEMG13 | `date_from` se recorta a la fecha de factura al facturar a mitad de ventana. | gotchas.md |
| MEMG14 | Borrar la línea de factura borra la membresía. | gotchas.md |
| MEMG15 | `account` (única dependencia dura, `depends=['account']`). | integrations.md |
| MEMG16 | `product`. | integrations.md |
| MEMG17 | `partner` (`res.partner`). | integrations.md |
| MEMG18 | `sale` — NO es dependencia. | integrations.md |
| MEMG19 | `website_membership` (CE, módulo separado). | integrations.md |
| MEMG2 | `paid` exige factura PAGADA, no solo posted. | gotchas.md |
| MEMG3 | `old` vs `none` dependen de ventana de fechas Y pago. | gotchas.md |
| MEMG4 | `free_member` cortocircuita antes de mirar líneas, pero pierde contra `paid`. | gotchas.md |
| MEMG5 | `free_member` bloquea la facturación. | gotchas.md |
| MEMG6 | `associate_member` hereda completamente el estado del padre y salta todo lo demás. | gotchas.md |
| MEMG7 | Recursión de associate bloqueada. | gotchas.md |
| MEMG8 | El cron diario solo "empuja" partners `invoiced`/`paid` — la expiración a `old` es perezosa. | gotchas.md |
| MEMG9 | El cron usa `add_to_compute`, no escribe directamente. | gotchas.md |

## functional/partnership

| ID | Título | Fichero |
|----|--------|---------|
| PAG1 | No hard territorial boundaries | gotchas.md |
| PAG10 | Portal write ACL uses `commercial_partner_id` hierarchy | gotchas.md |
| PAG11 | No lead capacity tracking | gotchas.md |
| PAG12 | (v19) Sale-based grade assignment is immediate and hits the COMMERCIAL parent | gotchas.md |
| PAG13 | (v19) One SO cannot mix products of different grades | gotchas.md |
| PAG14 | (v19) `assigned_grade_id` only counts `service_tracking='partnership'` lines | gotchas.md |
| PAG15 | (v19) Grade-pricelist uses `specific_property_product_pricelist`, not the legacy field | gotchas.md |
| PAG16 | (v19) Grade and weight live in two different modules on the same model | gotchas.md |
| PAG17 | `implemented_partner_count` only counts PUBLISHED implementations | gotchas.md |
| PAG18 | `partner_interested` auto-converts the lead to an opportunity (sudo) | gotchas.md |
| PAG19 | Declining a lead does NOT delete the decline once cleared | gotchas.md |
| PAG2 | Single assigned partner per lead | gotchas.md |
| PAG20 | `date_partner_assign` is computed, resets on every reassignment | gotchas.md |
| PAG21 | Grade auto-default only fires with a context flag | gotchas.md |
| PAG22 | `assign_partner()` leaks the first partner across a multi-lead recordset | gotchas.md |
| PAG23 | `assign_partner()` always returns False | gotchas.md |
| PAG24 | Leads that fail to route get tagged, not errored | gotchas.md |
| PAG25 | Assigned partners can post chatter without write access | gotchas.md |
| PAG26 | Portal Many2one writes are silently re-ACL'd, not the whole write | gotchas.md |
| PAG27 | `create_opp_portal` forces the assigned partner to the commercial parent | gotchas.md |
| PAG28 | `_get_access_action` carries an un-migrated portal hack | gotchas.md |
| PAG3 | Requires geocoded partners | gotchas.md |
| PAG4 | Leads without `country_id` are skipped entirely | gotchas.md |
| PAG5 | Probabilistic, not deterministic | gotchas.md |
| PAG6 | Weight 0 = permanently excluded | gotchas.md |
| PAG7 | (v19) Grade pricelist write coupling raises on conflict | gotchas.md |
| PAG8 | Enterprise module is paper-thin | gotchas.md |
| PAG9 | No automatic periodic re-assignment | gotchas.md |

## functional/pos

| ID | Título | Fichero |
|----|--------|---------|
| POG1 | Session close = ONE aggregated journal entry | gotchas.md |
| POG10 | POS picking auto-validates immediately | gotchas.md |
| POG100ES | Multi-company Veri*Factu: `l10n_es_edi_verifactu_required = related('company_id.l10n_es_edi_verifact | integrations.md |
| POG101ES | `simplified_partner_id` se compute via `env.ref("l10n_es.partner_simplified")` (`point_of_sale/model | integrations.md |
| POG102LY | `pos_loyalty` NO compatible con `pos_blackbox_be` (POG53BB). Si admin instala ambos → ValidationErro | integrations.md |
| POG103LY | `loyalty.program` con `pos.load.mixin` carga TODOS los programs activos al abrir POS → pesado en cli | integrations.md |
| POG104LY | `loyalty.card.points` se actualiza en `pos.order.action_pos_order_paid` async; si session cae mid-up | integrations.md |
| POG105EV | `pos_event` permite vender registrations en POS. Pero `event.registration` NO recibe `partner_id` si | integrations.md |
| POG106EV | Capacity check en `event.event.seats_available` NO está atómica con POS sale → 2 POS vendiendo simul | integrations.md |
| POG107EV | Refund de event ticket en POS NO libera el seat automáticamente (`event.registration.state` requiere | integrations.md |
| POG108UP | Delivery orders externos (Swiggy/Zomato/UberEats) llegan via webhook a `pos_urban_piper_request`. Si | integrations.md |
| POG109UP | `product.template.urban_piper_status` per-provider (Swiggy/Zomato/UberEats); update de stock un prov | integrations.md |
| POG11 | Combo products: pricing computed server-side | gotchas.md |
| POG110UP | Cancellation Urban Piper → solo marca order canceled, NO genera refund automatic en POS payment → co | integrations.md |
| POG12 | POS payments: cash bypasses account.payment, bank/terminal create it | gotchas.md |
| POG13 | Negative POS order lines = return | gotchas.md |
| POG14 | pos_settle_due (ENTERPRISE): paying invoices uses pay_later method | gotchas.md |
| POG15 | Multiple open sessions per config not allowed | gotchas.md |
| POG16 | FEFO does NOT apply in POS | gotchas.md |
| POG17 | No scrap button in POS frontend | gotchas.md |
| POG18 | POS Kit explosion: UoM conversion at stock level | gotchas.md |
| POG19S | `_create_picking_from_pos_order_lines` | integrations.md |
| POG2 | update_stock_at_closing defers ALL stock moves | gotchas.md |
| POG20S | Lines con qty negativa SIEMPRE generan picking separado (return). Si `picking_type.return_picking_ty | integrations.md |
| POG21S | `_should_create_picking_real_time` se decide a partir de la SESIÓN, no del config. Si `update_stock_ | integrations.md |
| POG22S | `_link_owner_on_return_picking` infiere `owner_id` del move_line original SOLO si `refunded_order_id | integrations.md |
| POG23S | `_create_production_lots_for_pos_order` busca lots con `company_id IN (False, current)` (`point_of_s | integrations.md |
| POG24S | UoM mismatch en move: `_add_mls_related_to_order` convierte `product_uom_qty` a `uom_id`. Si la conv | integrations.md |
| POG25S | `_send_confirmation_email` excluye | integrations.md |
| POG26S | Anglo-saxon COGS adicional: cuando `shipping_date AND NOT to_invoice` en `_action_done`, se crea un  | integrations.md |
| POG27A | Session close intenta SAVEPOINT para `_create_account_move`. Si lanza `UserError` distinto a unbalan | integrations.md |
| POG28A | `always_tax_exigible=True` se setea en moves con `pos_session_ids` (`point_of_sale/models/account_mo | integrations.md |
| POG29A | `button_draft` sobre un invoice POS NO bloquea pero envía bus_notification "Warning: Invoice Reset R | integrations.md |
| POG3 | Invoiced orders with Anglo-Saxon = real-time picking | gotchas.md |
| POG30A | `_stock_account_get_anglo_saxon_price_unit` usa el coste calculado de la POS order (`_get_pos_anglo_ | integrations.md |
| POG31A | Cash difference se publica como `account.bank.statement.line` con `counterpart_account_id` = `journa | integrations.md |
| POG32A | `_recompute_cash_rounding_lines` se SALTA si el reversed_entry_id (refund) ya tiene rounding line co | integrations.md |
| POG33A | `pos.payment.method` con `type='pay_later'` (journal=False) NO crea movimientos cash en cierre. El r | integrations.md |
| POG34A | `_compute_amount` (account.move) invierte signo si `reversed_pos_order_id AND move_type='entry'` (`p | integrations.md |
| POG35SL | `amount_unpaid = max(amount_total - total_invoice_paid - total_pos_paid - amount_paid, 0.0)` (`pos_s | integrations.md |
| POG36SL | `_compute_qty_delivered` SOLO suma qty si TODOS los pickings de POS están `state='done'` (`pos_sale/ | integrations.md |
| POG37SL | Refunds: `pos_order_line.refund_orderline_ids` también propaga `qty_delivered` (`pos_sale/models/sal | integrations.md |
| POG38SL | Down payment line: SO crea automáticamente `sale.order.line(is_downpayment=True, product_uom_qty=0)` | integrations.md |
| POG39SL | `unlink` sobre `sale.order.line` filtra downpayment lines con `pos_order_line_ids` (`pos_sale/models | integrations.md |
| POG4 | POS doesn't use sale.order | gotchas.md |
| POG40SL | `_force_create_picking_real_time` se evalúa `True` si `any(self.lines.mapped('sale_order_origin_id') | integrations.md |
| POG41SL | `currency_rate = res.currency._get_conversion_rate(company.currency, order.currency, company, date_o | integrations.md |
| POG42SL | `action_pos_order_paid` solo confirma `sale_orders` cuando payment es `_is_online_payment()` (`pos_s | integrations.md |
| POG43R | Borrar `restaurant.floor` o `table` con session OPEN → `UserError` (`pos_restaurant/models/pos_resta | integrations.md |
| POG44R | `iface_splitbill` se fuerza a `True` en create si `module_pos_restaurant=True` AND no se explicita e | integrations.md |
| POG45R | `set_tip_after_payment=False` se fuerza al desactivar `module_pos_restaurant` O al desactivar `iface | integrations.md |
| POG46R | `_update_payment_line_for_tip` NO valida que `payment.payment_method_id.type ∈ {cash, bank}` (`pos_r | integrations.md |
| POG47R | `restaurant.floor.write({'pos_config_ids': ..., 'active': ...})` lanza UserError si **CUALQUIERA** d | integrations.md |
| POG48R | `display_name` de table = `f"{floor.name}, {table_number}"` (`pos_restaurant/models/pos_restaurant.p | integrations.md |
| POG49R | `sync_from_ui` extra-fetcha `table_orders` en draft del mismo `table_ids` context (`pos_restaurant/m | integrations.md |
| POG5 | Cash difference posted as separate statement line | gotchas.md |
| POG50R | `takeaway_fp_id` se aplica vía `account.fiscal.position` _inherit MAP_TAX. Si `pos.order.takeaway=Tr | integrations.md |
| POG51BB | Una vez `certified_blackbox_identifier` se setea (1ª apertura), `iface_fiscal_data_module` NO se pue | integrations.md |
| POG52BB | Si EXISTE algún pos.config con `certified_blackbox_identifier`, NINGÚN otro POS de la BD puede opera | integrations.md |
| POG53BB | Loyalty y self_order (kiosk + mobile w/ payment) se bloquean COMPLETAMENTE para POS con blackbox (`p | integrations.md |
| POG54BB | Modificar pos.order con blackbox cert solo permite 8 fields whitelisted (`point_of_sale/models/pos_o | integrations.md |
| POG55BB | `_check_pos_category` exige que la `pos_category_fdm` esté en `iface_available_categ_ids` si `limit_ | integrations.md |
| POG56BB | Floors NO compartibles entre `pos.config` con blackbox (`point_of_sale/models/pos_config.py:191-196` | integrations.md |
| POG57BB | `cash_rounding` debe ser exactamente `0.05` y `HALF-UP` (`point_of_sale/models/pos_config.py:209-218 | integrations.md |
| POG58BB | `pos_blackbox_be.log` SE CREA en sudo() siempre (`point_of_sale/models/pos_config.py:37-42`, `PosCon | integrations.md |
| POG59BB | ir.module.module._unlink bloqueado para `pos_blackbox_be` (`pos_blackbox_be/models/module.py:10-15`, | integrations.md |
| POG6 | POS tax = always_tax_exigible (no cash basis) | gotchas.md |
| POG60P | `adyen_latest_response` es | integrations.md |
| POG61P | `_is_write_forbidden` excluye `adyen_latest_response` para permitir writes async (`pos_adyen/models/ | integrations.md |
| POG62P | Adyen constraint `_check_adyen_terminal_identifier` valida unicidad GLOBAL (sudo), pero el mensaje d | integrations.md |
| POG63P | Stripe `_stripe_calculate_amount` redondea `amount/currency.rounding` (`pos_stripe/models/pos_paymen | integrations.md |
| POG64P | Stripe capture amount: si pasas `amount=None`, captura el authorized total. Si pasas `amount > autho | integrations.md |
| POG65P | `pos_six` (no IoT) | integrations.md |
| POG66P | `pos_iot.payment_method.iot_device_id` se RESETEA a False cada vez que `_onchange_use_payment_termin | integrations.md |
| POG67P | Mercado Pago, Razorpay, PayTM, Viva Wallet — todos usan | integrations.md |
| POG68P | Refund vía terminal Adyen: requiere `originalReference` del payment original (`point_of_sale/models/ | integrations.md |
| POG69SO | `access_token` regeneración rotal **TODOS** los `restaurant.table.identifier` (`pos_self_order/model | integrations.md |
| POG7 | Anonymous orders use generic POS receivable | gotchas.md |
| POG70SO | `consteq(access_token, ...)` solo valida acceso al MENU. Crear `pos.order` desde self_order usa `sel | integrations.md |
| POG71SO | `_prepare_self_order_splash_screen` lee 3 archivos del filesystem en `pos_self_order/static/img/land | integrations.md |
| POG72SO | Multi-pos: `pos_self_order.custom_link.pos_config_ids` es M2M, pero el botón default "Order Now" se  | integrations.md |
| POG73SO | `self_ordering_mode='kiosk'` impide `is_cash_count` payment methods (`point_of_sale/models/res_confi | integrations.md |
| POG74SO | `pos_online_payment_self_order` exige `payment.provider` configurado a nivel company. Cross-company  | integrations.md |
| POG75SO | `pos_self_order_preparation_display` (auto_install con `pos_self_order + pos_preparation_display`) p | integrations.md |
| POG76SO | Self order en `service_mode='table'` requires `pos_module_pos_restaurant=True` (constraint). Si lo d | integrations.md |
| POG77SO | `self_ordering_default_language_id` se compute con `env.lang` al CREATE. Multi-tenant en distintos t | integrations.md |
| POG78PD | `process_order` solo se llama si `state == 'paid' AND NOT module_pos_restaurant` (`point_of_sale/mod | integrations.md |
| POG79PD | `quantity_data` key incluye `(product_id, note, attrs, uuid)` (`point_of_sale/models/pos_order.py:51 | integrations.md |
| POG8 | Ship Later uses procurement routes, not direct picking | gotchas.md |
| POG80PD | `note_history` consume `used_qty` mutable (`point_of_sale/models/pos_order.py:81-114`, `PosOrder._pr | integrations.md |
| POG81PD | `product_cancelled` se incrementa pero NO se decrementa al re-añadir qty del mismo product+note+uuid | integrations.md |
| POG82PD | `pdis_general_note` se actualiza solo si `pdis_lines` NO está vacío (`point_of_sale/models/pos_order | integrations.md |
| POG83PD | Bus channels (`pos.bus.mixin`) son per-display. Si el cajero crea un display nuevo en runtime (admin | integrations.md |
| POG84PD | `pos_restaurant_preparation_display` agrega "courses" (entrada / plato / postre). Course transitions | integrations.md |
| POG85IO | `iface_printer_id` domain incluye `subtype='receipt_printer'` (`pos_iot/models/pos_config.py:12`, `c | integrations.md |
| POG86IO | Multi-company iot.device: domain `'\|', company_id=False, company_id=current` (`point_of_sale/models/ | integrations.md |
| POG87IO | `iot_device_ids` computed `printer + display + scanner + scale` excluye payment terminals (esos está | integrations.md |
| POG88IO | `pos_iot.payment_method.iot_device_id` reseteado a False en cualquier write a `use_payment_terminal` | integrations.md |
| POG89IO | `_compute_payment_terminal_ids` busca devices por subtype hardcoded: `ingenico_subtype`, `worldline_ | integrations.md |
| POG9 | Loyalty programs shared between POS and eCommerce | gotchas.md |
| POG90IO | IoT box offline: POS sigue funcionando "sin printer" pero | integrations.md |
| POG91IO | `iot.box.identifier` (MAC-based) puede cambiar tras factory-reset → todas las references desde POS a | integrations.md |
| POG92ES | `_process_saved_order` Veri*Factu bloquea con `UserError` si `to_invoice=False AND amount_total > 30 | integrations.md |
| POG93ES | Refunds Veri*Factu: el campo `l10n_es_edi_verifactu_refund_reason` (Selection R1-R5) es required si  | integrations.md |
| POG94ES | `l10n_es_edi_verifactu_button_cancel` está vacío (`l10n_es_edi_verifactu_pos/models/pos_order.py:135 | integrations.md |
| POG95ES | TBAI: el retry-on-chain-head NO vive en `_l10n_es_tbai_post` (`l10n_es_edi_tbai_pos/models/pos_order | integrations.md |
| POG96ES | `is_simplified=True` SIEMPRE en TBAI (`point_of_sale/models/pos_order.py:181`, `PosOrder._process_pa | integrations.md |
| POG97ES | Veri*Factu `_l10n_es_edi_verifactu_get_record_values` asume | integrations.md |
| POG98ES | Refund Veri*Factu: `refunded_document` se busca primero en `refunded_order.account_move.documents`,  | integrations.md |
| POG99ES | `_compute_l10n_es_edi_verifactu_warning` (computa `l10n_es_edi_verifactu_warning_level`) solo lee `l | integrations.md |
| POSCAP1 | ¿Funciona offline (sin conexión)? | capabilities.md |
| POSCAP10 | ¿Display de cliente? | capabilities.md |
| POSCAP11 | ¿Hardware (impresora/balanza/escáner/cajón)? | capabilities.md |
| POSCAP12 | ¿Propinas? | capabilities.md |
| POSCAP13 | ¿Combos / menús? | capabilities.md |
| POSCAP14 | ¿Facturar desde el POS? | capabilities.md |
| POSCAP15 | ¿Pantalla de cocina (preparation)? | capabilities.md |
| POSCAP16 | ¿Cumplimiento fiscal (blackbox/Veri*Factu/TicketBAI)? | capabilities.md |
| POSCAP17 | ¿Cliente + programa de puntos? | capabilities.md |
| POSCAP18 | ¿Stock en tiempo real desde POS? | capabilities.md |
| POSCAP2 | ¿Varios pagos en un ticket (split)? | capabilities.md |
| POSCAP3 | ¿Control de caja (apertura/cierre/arqueo)? | capabilities.md |
| POSCAP4 | ¿Restaurante: mesas/salas/comandas? | capabilities.md |
| POSCAP5 | ¿Autopedido / kiosko? | capabilities.md |
| POSCAP6 | ¿Fidelización / promociones / cupones? | capabilities.md |
| POSCAP7 | ¿Multi-tienda? | capabilities.md |
| POSCAP8 | ¿Personalizar el ticket? | capabilities.md |
| POSCAP9 | ¿Devoluciones / reembolsos? | capabilities.md |
| POSCG1 | Create and Configure POS Terminal | config_guide.md |
| POSCG10 | Configure Self-Order (Kiosk/QR) | config_guide.md |
| POSCG11 | Configure Kitchen Display (Enterprise) | config_guide.md |
| POSCG2 | Configure Payment Methods | config_guide.md |
| POSCG3 | Configure Real-Time vs Deferred Stock | config_guide.md |
| POSCG4 | Configure Ship Later | config_guide.md |
| POSCG5 | Configure POS Invoicing | config_guide.md |
| POSCG6 | Configure Restaurant Mode | config_guide.md |
| POSCG7 | Configure Loyalty Programs in POS | config_guide.md |
| POSCG8 | Configure Cash Control | config_guide.md |
| POSCG9 | Configure Fiscal Positions in POS | config_guide.md |
| PSO1 | URL access_token rotation invalidates ALL printed QRs | extensions_self_order.md |
| PSO10 | Per-session ir.sequence created on session OPEN | extensions_self_order.md |
| PSO11 | Public route uses `auth='public'` | extensions_self_order.md |
| PSO12 | Floor-shared configs break QR uniqueness | extensions_self_order.md |
| PSO13 | Splash image attachments default to demo data on EVERY create | extensions_self_order.md |
| PSO14 | `default_fiscal_position_id` vs `takeaway_fp_id` | extensions_self_order.md |
| PSO15 | Cancelling kiosk session deletes all draft orders | extensions_self_order.md |
| PSO16 | v19 short_url via link.tracker | extensions_self_order.md |
| PSO17 | v19 attachments forced public=True | extensions_self_order.md |
| PSO18 | v19 source field replaces older detection | extensions_self_order.md |
| PSO2 | `module_pos_restaurant=False` forces self_ordering_mode=nothing | extensions_self_order.md |
| PSO3 | Kiosk forbids cash payments | extensions_self_order.md |
| PSO4 | self_ordering_default_user_id MUST be POS user | extensions_self_order.md |
| PSO5 | Pay-after=meal forces table service | extensions_self_order.md |
| PSO6 | Pay-after=each requires Enterprise | extensions_self_order.md |
| PSO7 | Server recomputes ALL prices, ignores frontend | extensions_self_order.md |
| PSO8 | Combo last-child absorbs rounding remainder | extensions_self_order.md |
| PSO9 | Takeaway flag locked once order exists | extensions_self_order.md |
| PSP1 | Empty category_ids = ALL categories | extensions.md |
| PSP10 | v19 SPLIT pos_preparation_display into 3 bridges | extensions.md |
| PSP2 | Empty pos_config_ids = ALL configs | extensions.md |
| PSP3 | Stage edits forbidden during ANY open session | extensions.md |
| PSP4 | Reset uses 'last stage' (sequence-implicit) | extensions.md |
| PSP5 | Auto-cleanup deletes after 1 DAY (not configurable) | extensions.md |
| PSP6 | order.create_uid as "responsible" | extensions.md |
| PSP7 | order_count today-only | extensions.md |
| PSP8 | average_time excludes non-done orders | extensions.md |
| PSP9 | Stages stored as One2many but reset also touches "stageless" | extensions.md |
| PSR1 | Default floor with single 130×130 table created EVERY enable | extensions_restaurant.md |
| PSR10 | Draft order survives floor deactivation only via cancel | extensions_restaurant.md |
| PSR11 | Onboarding session XML conditional on main_company | extensions_restaurant.md |
| PSR12 | iface_splitbill=True default applied only ONCE on create | extensions_restaurant.md |
| PSR13 | v19 takeaway replaced by pos.preset | extensions_restaurant.md |
| PSR14 | v19 master presets undeletable | extensions_restaurant.md |
| PSR15 | v19 courses model adds new lifecycle | extensions_restaurant.md |
| PSR16 | v19 default_screen changes UX | extensions_restaurant.md |
| PSR17 | v19 _get_open_order disambiguates by config_id | extensions_restaurant.md |
| PSR18 | v19 table cycle detection on set_parent_id | extensions_restaurant.md |
| PSR19 | v19 background_color default removed | extensions_restaurant.md |
| PSR2 | floor_ids in forbidden_change_fields | extensions_restaurant.md |
| PSR3 | Disabling restaurant module wipes all floors | extensions_restaurant.md |
| PSR4 | Floor sharing across configs | extensions_restaurant.md |
| PSR5 | Table parent_id grouping is one-way | extensions_restaurant.md |
| PSR6 | takeaway_fp_id required for tax-different takeaway | extensions_restaurant.md |
| PSR7 | send_table_count_notification is a NO-OP | extensions_restaurant.md |
| PSR8 | customer_count is readonly | extensions_restaurant.md |
| PSR9 | table_id readonly after creation | extensions_restaurant.md |

## functional/project

| ID | Título | Fichero |
|----|--------|---------|
| PRJCAP1 | ¿Tareas y subtareas con jerarquía? | capabilities.md |
| PRJCAP10 | ¿Portal de cliente (seguimiento de tareas)? | capabilities.md |
| PRJCAP11 | ¿SLA / plazos? | capabilities.md |
| PRJCAP12 | ¿Sub-proyectos / proyectos jerárquicos? | capabilities.md |
| PRJCAP13 | ¿Field Service (intervenciones en campo)? | capabilities.md |
| PRJCAP14 | ¿Plantillas de proyecto? | capabilities.md |
| PRJCAP15 | ¿Gastos imputados al proyecto? | capabilities.md |
| PRJCAP16 | ¿Compras imputadas (analítica)? | capabilities.md |
| PRJCAP17 | ¿Burndown / informes de avance? | capabilities.md |
| PRJCAP18 | ¿Asignación múltiple (varios asignados por tarea)? | capabilities.md |
| PRJCAP2 | ¿Etapas/Kanban personalizables por proyecto? | capabilities.md |
| PRJCAP3 | ¿Partes de horas (timesheets) en tareas? | capabilities.md |
| PRJCAP4 | ¿Facturar el proyecto (horas/fijo/hitos)? | capabilities.md |
| PRJCAP5 | ¿Hitos (milestones) facturables? | capabilities.md |
| PRJCAP6 | ¿Tareas recurrentes? | capabilities.md |
| PRJCAP7 | ¿Rentabilidad del proyecto (ingresos vs costes)? | capabilities.md |
| PRJCAP8 | ¿Dependencias entre tareas? | capabilities.md |
| PRJCAP9 | ¿Planificación de recursos / forecast? | capabilities.md |
| PRJG1 | project_task_user_rel serves triple duty | gotchas.md |
| PRJG10 | Milestone cascading to subtasks (OPEN subtasks only) | gotchas.md |
| PRJG11 | Analytic account mandatory for timesheets | gotchas.md |
| PRJG12 | SO cancellation doesn't delete project/task | gotchas.md |
| PRJG13 | Cannot make a task private while timesheets are linked to it | gotchas.md |
| PRJG14 | Quick-create parsing in display_name inverse | gotchas.md |
| PRJG15 | Deletion blocked if timesheets exist | gotchas.md |
| PRJG16 | project_timesheet_holidays has TWO similarly-named methods, one with a TYPO | gotchas.md |
| PRJG17 | Global time-off creates TS for ALL matching employees | gotchas.md |
| PRJG18 | Recurrence triggers on LAST task close only | gotchas.md |
| PRJG19 | sale_line_id priority chain on task | gotchas.md |
| PRJG2 | Portal write ACL inactive by default | gotchas.md |
| PRJG20 | Employee rate cost override is per-project | gotchas.md |
| PRJG21 | Disabling allow_billable clears so_line on ALL timesheets | gotchas.md |
| PRJG22 | Profitability double-counting prevention pattern | gotchas.md |
| PRJG23 | Favorite project uses statistics.mode() | gotchas.md |
| PRJG24 | project_enterprise_hr is the scheduling pivot | gotchas.md |
| PRJG25 | Subscription close unlinks task recurrence | gotchas.md |
| PRJG26 | FSM is project.task with is_fsm=True, not a separate model | gotchas.md |
| PRJG27 | industry_fsm_sale allow_material requires allow_billable | gotchas.md |
| PRJG28 | FSM task has no "rejected by technician" state | gotchas.md |
| PRJG29 | FSM view is web responsive, not native mobile | gotchas.md |
| PRJG3 | State forced to 04_waiting_normal on blocked tasks | gotchas.md |
| PRJG30 | action_fsm_validate() is the key hook for post-completion logic | gotchas.md |
| PRJG31 | Profitability panel skips `category=='vendor_bill'` AAL | gotchas.md |
| PRJG32 | Panel only reads the ROOT analytic plan (`account_id`) | gotchas.md |
| PRJG33 | to_bill/to_invoice read DRAFT `account.move.line` directly, not AAL | gotchas.md |
| PRJG34 | project_purchase PO count mixes direct link AND analytic match | gotchas.md |
| PRJG35 | project_purchase rewrites PO line analytic on project_id change | gotchas.md |
| PRJG36 | project_hr_expense only counts posted/done sheets in margin | gotchas.md |
| PRJG37 | project.role assignment is consumed once then wiped | gotchas.md |
| PRJG38 | Converting a task to template clears its roles | gotchas.md |
| PRJG39 | Subtask silently inherits its parent's project unless display_in_project | gotchas.md |
| PRJG4 | allocated_hours is a plain manual Float in v18 (NOT computed in Enterprise) | gotchas.md |
| PRJG40 | Two CHECK constraints on project.task: recurring-no-parent and private-no-parent | gotchas.md |
| PRJG41 | A To-Do IS a project.task with no project — same model, no new model | gotchas.md |
| PRJG42 | To-Do name auto-derived from the first line of the description | gotchas.md |
| PRJG43 | Onboarding To-Do is generated lazily, once per user, gated by a group | gotchas.md |
| PRJG44 | Personal stage default rows are created lazily per user | gotchas.md |
| PRJG5 | progress (aggregator="avg") lives in hr_timesheet, not core project | gotchas.md |
| PRJG6 | Timesheet amount recalculated when unit_amount/employee_id/account_id change | gotchas.md |
| PRJG7 | Invoiced timesheets become read-only and undeletable | gotchas.md |
| PRJG8 | is_so_line_edited flag stops auto-recompute | gotchas.md |
| PRJG9 | date_end auto-set on folded stages | gotchas.md |

## functional/purchase

| ID | Título | Fichero |
|----|--------|---------|
| PCG1 | Configure Vendor Pricelist on Product | config_guide.md |
| PCG10 | Configure Dropship Purchase | config_guide.md |
| PCG2 | Configure Bill Control Policy | config_guide.md |
| PCG3 | Enable 3-Way Matching | config_guide.md |
| PCG4 | Configure Purchase to Different Warehouse | config_guide.md |
| PCG5 | Configure Blanket Order (Framework Agreement) | config_guide.md |
| PCG6 | Configure Double Validation on POs | config_guide.md |
| PCG7 | Configure Automatic Purchase from Reordering Rule | config_guide.md |
| PCG8 | Configure Vendor Bill from PO | config_guide.md |
| PCG9 | Import Vendor Pricelists in Bulk | config_guide.md |
| PG1 | PO currency from vendor, not order date | gotchas.md |
| PG10 | Anglo-Saxon price difference solo cuando qty_invoiced <= qty_received | gotchas.md |
| PG11 | Supplier lead time fija date_planned, no date_order | gotchas.md |
| PG12 | Multiple vendors: el sequence más bajo fija el vendor | gotchas.md |
| PG13 | Vendor bills auto-complete desde líneas de PO; bills manuales no actualizan qty_invoiced | gotchas.md |
| PG14 | Kit purchases: qty_received se computa desde los componentes (bottleneck) | gotchas.md |
| PG15 | Fiscal position auto-set desde vendor al cambiar partner | gotchas.md |
| PG16 | Servicios/consumibles usan `qty_received_method='manual'` permanente | gotchas.md |
| PG17 | price_unit deja de recomputarse en cuanto la línea tiene invoice_lines | gotchas.md |
| PG18 | Sin seller, el precio cae a standard_price con conversión de moneda a fecha de orden | gotchas.md |
| PG19 | Añadir/editar una línea en un PO confirmado crea o ajusta el picking al instante | gotchas.md |
| PG2 | _select_seller: sequence elige el VENDOR, luego price_discounted dentro de ese vendor | gotchas.md |
| PG20 | Bajar product_qty por debajo de lo recibido lanza UserError | gotchas.md |
| PG21 | Multi-step receipt: qty_received solo cuenta el move done al destino final | gotchas.md |
| PG22 | Devoluciones a proveedor restan de qty_received solo bajo condiciones | gotchas.md |
| PG23 | Cancelar un PO con bills posted lanza UserError | gotchas.md |
| PG24 | qty_invoiced ignora bills cancel pero NO las legacy | gotchas.md |
| PG25 | Bill matching: tolerancia ABSOLUTA de 0,02 (no porcentaje) | gotchas.md |
| PG26 | partner_ref de la PO se copia a ref Y payment_reference de la bill | gotchas.md |
| PG27 | Editar date_planned en el header pisa las fechas de TODAS las líneas | gotchas.md |
| PG28 | requisition_type en v18 es {blanket_order, purchase_template} — NO existe "tender" | gotchas.md |
| PG3 | Confirming PO auto-adds vendor to product (límite 10, puede llegar a 11) | gotchas.md |
| PG4 | picking_type_id controla el almacén destino | gotchas.md |
| PG5 | Bill on "Received Quantities" bloquea facturar antes de recibir | gotchas.md |
| PG6 | 3-Way Matching requiere policy "Received Quantities" | gotchas.md |
| PG7 | _run_buy() mergea procurements en POs draft existentes | gotchas.md |
| PG8 | Blanket order confirmation injects a `product.supplierinfo` that competes in seller selection | gotchas.md |
| PG9 | Cerrar/cancelar el blanket order borra las supplierinfo | gotchas.md |
| PU3W1 | Price check STRICT, no tolerancia. POL=100, AML=100.01 → exception automático. No hay config para to | integrations.md |
| PU3W2 | Currency conversion en price check usa rate del día. PO en USD + bill EUR + rate cambiado desde PO → | integrations.md |
| PU3W3 | AML sin `purchase_line_id` → siempre `'exception'`. No hay bypass. Bills manuales (no de PO) quedan  | integrations.md |
| PU3W4 | `qty_received=0` + bill posted (invoice-before-receipt) en method='receive' → exception. Pago bloque | integrations.md |
| PU3W5 | Exception propaga UP. 1 AML de 10 en exception → entire invoice en exception. AND logic strict. `acc | integrations.md |
| PUAC1 | `purchase_method` (NOT `invoice_policy` | integrations.md |
| PUAC10 | Refund parcial: qty_to_invoice NO se restaura. Para segunda refund debes manualmente reajustar POL.  | integrations.md |
| PUAC11 | Cantidad PD capped a `qty_received`. Invoice qty > received qty → resto ignorado (no PD). `purchase_ | integrations.md |
| PUAC12 | History replay determinístico pero complejo. SVL `create_date > move date` puede reordenar history i | integrations.md |
| PUAC13 | Refund reverse PD SVL pero NO crea anti-PD automático. Refund qty > received qty → cantidad incorrec | integrations.md |
| PUAC14 | Returned moves "fakean" price_unit con POL price para evitar double-counting. Confuso en reports. `p | integrations.md |
| PUAC15 | Expense account lookup via `product.get_product_accounts(fiscal_pos=move.fiscal_position_id)`. Cambi | integrations.md |
| PUAC16 | `_replay_history()` con >500 stock.move+invoices puede timeout. Mitigated pero real risk. `purchase_ | integrations.md |
| PUAC17 | Multi-currency triple-rounding: PO currency ≠ layer currency ≠ company currency. Precision loss posi | integrations.md |
| PUAC2 | `qty_to_invoice` computa SOLO si `state in ['purchase', 'done']`. Antes de confirmar PO → siempre 0  | integrations.md |
| PUAC3 | `qty_invoiced` resta refunds (`in_refund`). Moves en `cancel` ignorados SALVO `payment_state='invoic | integrations.md |
| PUAC4 | `TOLERANCE = 0.02` en bill matching es importe ABSOLUTO (2 céntimos), NO 2%. Se aplica como `amount_ | integrations.md |
| PUAC5 | `_prepare_account_move_line()` niega qty AUTOMÁTICAMENTE si move_type='in_refund'. No pases qty nega | integrations.md |
| PUAC6 | Currency conversion a bill currency: usa `fields.Date.today()` (no `po.date_order`). PO histórica +  | integrations.md |
| PUAC7 | Multi-company: bill se fuerza a `with_company(po.company_id)`. Si journal solo existe en child compa | integrations.md |
| PUAC8 | Autocomplete `purchase_id` con `force_write=True` (default) borra ALL invoice_line_ids antes de agre | integrations.md |
| PUAC9 | Grouping bills por `(company_id, partner_id, currency_id)`. Para custom grouping (sequence, salesman | integrations.md |
| PUAN1 | `_compute_analytic_distribution()` silent si no rule match. Distribution vacío sin warning. `purchas | integrations.md |
| PUAN2 | Override manual de analytic en POL se borra en recompute (readonly=False permite override pero compu | integrations.md |
| PUAN3 | AML.analytic_distribution es fallback. Si UI lo populó previamente → NO hereda de POL aunque POL ten | integrations.md |
| PUAN4 | SVL analytic propagation SOLO en `valuation='real_time'`. Periodic (FIFO/LIFO sin auto-post) → SVL s | integrations.md |
| PUAN5 | Parent-child analytic accounts sin auto-split. 100% al parent → no se distribuye children. Manual %  | integrations.md |
| PUAN6 | Distribution dict keys DEBEN ser integer IDs. String IDs o names → ignorados silenciosamente. | integrations.md |
| PUCAP1 | ¿RFQ → pedido de compra? | capabilities.md |
| PUCAP10 | ¿Aprobación de pedidos (doble validación por importe)? | capabilities.md |
| PUCAP11 | ¿Portal de proveedor? | capabilities.md |
| PUCAP12 | ¿Costes en destino (landed costs) desde compra? | capabilities.md |
| PUCAP13 | ¿Recepción parcial / backorders? | capabilities.md |
| PUCAP14 | ¿Fechas de entrega planificadas / lead times? | capabilities.md |
| PUCAP15 | ¿Devoluciones a proveedor? | capabilities.md |
| PUCAP16 | ¿Análisis de compras (informes)? | capabilities.md |
| PUCAP17 | ¿Compra de servicios facturados por entregado? | capabilities.md |
| PUCAP18 | ¿Alertas/bloqueo por proveedor (warnings)? | capabilities.md |
| PUCAP2 | ¿Tarifas de proveedor (precio por proveedor/cantidad)? | capabilities.md |
| PUCAP3 | ¿Reabastecimiento automático genera compras? | capabilities.md |
| PUCAP4 | ¿Control de factura: por pedido vs por recibido (3-way match)? | capabilities.md |
| PUCAP5 | ¿Pedidos abiertos / acuerdos marco (blanket orders)? | capabilities.md |
| PUCAP6 | ¿Comparación de ofertas / licitación (tenders)? | capabilities.md |
| PUCAP7 | ¿Multi-moneda en compras? | capabilities.md |
| PUCAP8 | ¿Compra inter-compañía (PO en Co.A genera SO en Co.B)? | capabilities.md |
| PUCAP9 | ¿Dropshipping (compra directa a cliente)? | capabilities.md |
| PUIC1 | Currency mismatch entre `company_partner.property_product_pricelist.currency_id` y PO currency → Use | integrations.md |
| PUIC2 | Ciclo intercompany (A→B→C→A) prevenido por `auto_generated` flag pero relación cyclical SO/PO desinc | integrations.md |
| PUIC3 | `picking_type_id` NO asignado en `_prepare_purchase_order_data`. Default warehouse. Si destino tiene | integrations.md |
| PUIC4 | Dropship con `dest_address_id`: PO→SO copia a `partner_shipping_id` pero SO permite `partner_shippin | integrations.md |
| PUIC5 | Fiscal position en PO→SO derivada de `partner_id` via `_get_fiscal_position()`. User override en SO  | integrations.md |
| PUIC6 | Taxes NO se transfieren PO→SO. SO taxes default del customer (sale taxes), no del PO supplier taxes. | integrations.md |
| PUM1 | `_get_mrp_productions()` une `move_dest_ids.group_id.mrp_production_ids` (MTO) + `order_line.group_i | integrations.md |
| PUM2 | Kit `qty_received` bottleneck: PO kit=10, BOM=[Comp1×2, Comp2×3]. Recibes 6 Comp1 (=3 kits) + 6 Comp | integrations.md |
| PUM3 | `cost_share` constraint suma 100% | integrations.md |
| PUM4 | Kit price_unit cascade: `kit_pu × cost_share × bom.product_qty / bom_line.product_qty` con UoM conve | integrations.md |
| PUM5 | Bajar qty en PO kit MTO con previous_product_qty contextual NO recalcula `move_dest_ids` para compon | integrations.md |
| PUM6 | Kit dropship + componentes buy: kit move a customer, componentes a warehouse. 2 pickings diferentes, | integrations.md |
| PUM7 | Nested BOM (sub-bom phantom): `_round_in_svl_value()` redistribuye rounding pero multi-nesting NO re | integrations.md |
| PUQ1 | Supplier info cleanup en cancel hace `unlink()` ANTES de validar POs confirmadas. POs en `purchase`  | integrations.md |
| PUQ2 | `purchase.order.group` con ≤1 POs se borra automáticamente. Pero `.unlink()` sin cascada explícita d | integrations.md |
| PUQ3 | Validación `price_unit > 0` SOLO en `blanket_order` al confirm. `purchase_template` permite price=0. | integrations.md |
| PUQ4 | UoM conversion `requisition.line.product_uom_id → product.uom_po_id` con factor fraccionario (1 box= | integrations.md |
| PUQ5 | Constraint `price_unit > 0` solo en create, NO en write. Update posterior a 0 en línea de requisitio | integrations.md |
| PUQ6 | `origin` concatena nombre requisition sin deduplicación. Re-link de misma requisition (context bug)  | integrations.md |
| PUS1 | `qty_received` no incrementa en edge case: move con `origin_returned_move_id` original-dropship Y mo | integrations.md |
| PUS10 | NO existe owner_id/consignment en `purchase_stock` CE/EE v18. Requiere módulo custom o legacy `consi | integrations.md |
| PUS2 | Cambiar `company_id` en PO draft recalcula `picking_type_id` SOLO si actual no es 'incoming' o wareh | integrations.md |
| PUS3 | `reception_steps='three_step'` → `propagate_cancel=True`. Cancel de PO cierra input→control→stock en | integrations.md |
| PUS4 | `procurement.group.run()` con `action='buy'` añade `reception_route_id` auto. Warehouse sin `buy_pul | integrations.md |
| PUS5 | `effective_date` filtra pickings con `location_dest != supplier`. Si una línea va a customer (dropsh | integrations.md |
| PUS6 | `stock.move._get_price_unit()` invoice-before-receipt: si `qty_invoiced > qty_received`, calcula pri | integrations.md |
| PUS7 | PO cancel con move.state='done' → UserError. Con partial picking + `propagate_cancel=True`, backorde | integrations.md |
| PUS8 | Bajar `product_qty` con `qty_received > new qty` → UserError. Si `qty_invoiced > new qty` Y hay invo | integrations.md |
| PUS9 | Return-to-vendor: `_get_purchase_line_and_partner_from_chain()` navega `origin_returned_move_id` rec | integrations.md |

## functional/recruitment

| ID | Título | Fichero |
|----|--------|---------|
| RECCAP1 | ¿Publicar ofertas con nº de vacantes? | capabilities.md |
| RECCAP10 | ¿Enviar oferta/contrato a firmar? | capabilities.md |
| RECCAP11 | ¿Programa de referidos con puntos/recompensas? | capabilities.md |
| RECCAP12 | ¿Banco de talento / reutilizar candidatos? | capabilities.md |
| RECCAP13 | ¿Seguimiento del origen de la candidatura? | capabilities.md |
| RECCAP14 | ¿Competencias/skills del candidato? | capabilities.md |
| RECCAP15 | ¿Candidato reutilizable independiente de la candidatura (arquitectura v18)? | capabilities.md |
| RECCAP16 | ¿Publicar la oferta en portales de empleo externos (Monster, etc.)? | capabilities.md |
| RECCAP17 | ¿Comunicación con candidatos por SMS y cuestionario de evaluación? | capabilities.md |
| RECCAP18 | ¿Planes de actividad de contratación (checklist automatizado)? | capabilities.md |
| RECCAP19 | ¿Informes / analítica de reclutamiento? | capabilities.md |
| RECCAP2 | ¿Portal de empleo web y candidatura online? | capabilities.md |
| RECCAP3 | ¿Pipeline kanban con etapas configurables? | capabilities.md |
| RECCAP4 | ¿Candidatura por email (alias del puesto)? | capabilities.md |
| RECCAP5 | ¿Formularios de entrevista? | capabilities.md |
| RECCAP6 | ¿Agendar entrevista con el candidato? | capabilities.md |
| RECCAP7 | ¿OCR del CV para rellenar datos? | capabilities.md |
| RECCAP8 | ¿Motivos de rechazo con email automático? | capabilities.md |
| RECCAP9 | ¿Crear el empleado al contratar? | capabilities.md |
| RECG1 | La candidatura SIEMPRE cuelga de un candidato (v18). | gotchas.md |
| RECG10 | Quitar `hired_stage` de un stage con candidatos avisa. | gotchas.md |
| RECG11 | En CE core el puesto NO tiene flag de publicación. | gotchas.md |
| RECG12 | La "Evaluación" (priority) del candidato es la MEDIA de sus candidaturas. | gotchas.md |
| RECG13 | Al desarchivar/mover NO se envía el email de stage. | gotchas.md |
| RECG14 | El CV se copia candidato→applicant al crear, validando misma compañía. | gotchas.md |
| RECG15 | El CV y el OCR cuelgan del `hr.candidate`, NO del applicant. | integrations.md |
| RECG16 | El OCR se valida al llegar a un stage `hired_stage`, no antes. | integrations.md |
| RECG17 | `website_published` e `is_published` se desincronizan fuera del onchange. | integrations.md |
| RECG18 | `survey_id` del applicant es related readonly del puesto. | integrations.md |
| RECG19 | Las firmas se enganchan al empleado vía partner, no vía applicant. | integrations.md |
| RECG2 | `kanban_state` se resetea al cambiar de stage. | gotchas.md |
| RECG20 | Documents requiere activar `documents_recruitment_settings` por compañía. | integrations.md |
| RECG21 | Referidos: cambiar `ref_user_id` BORRA y recalcula todas las líneas de puntos. | integrations.md |
| RECG22 | `message_new` desactiva el autor y maneja plataformas. | integrations.md |
| RECG23 | El sub-alias de fuentes exige `alias_domain_id`. | integrations.md |
| RECG24 | `_notify_get_reply_to` enruta los hilos del applicant por el alias del puesto | integrations.md |
| RECG3 | `date_closed` es bidireccional con `hired_stage`. | gotchas.md |
| RECG4 | `application_status` no almacenado, con prioridad fija. | gotchas.md |
| RECG5 | Rechazar archiva pero conserva el dato. | gotchas.md |
| RECG6 | Desarchivar reinyecta al pipeline y "des-rechaza". | gotchas.md |
| RECG7 | Email entrante de-duplica candidato por email. | gotchas.md |
| RECG8 | Rechazar "duplicados" rechaza en cascada candidatos similares. | gotchas.md |
| RECG9 | Reply-to apunta al alias del puesto. | gotchas.md |

## functional/repair

| ID | Título | Fichero |
|----|--------|---------|
| BC1 | `float_compare()` → `uom.compare()` throughout | v19_changes.md |
| BC16 | Integración repair→`qty_delivered` migra de `_compute_qty_delivered` a `_prepare_qty_delivered` | v19_changes.md |
| BC2 | `float_is_zero()` → `uom.is_zero()` throughout | v19_changes.md |
| BC3 | `expression.AND()` → `Domain()` operator | v19_changes.md |
| BC4 | `_search_date_category` operator | v19_changes.md |
| BC5 | `groupby` → `unique` in `_get_picking_type` | v19_changes.md |
| BC6 | Manifest dependency changed | v19_changes.md |
| BC7 | BC15. Additional behavioral changes | v19_changes.md |
| NF1 | `picking_type_visible` computed field | v19_changes.md |
| NF2 | `action_generate_serial` method | v19_changes.md |
| NF3 | `message_post` override | v19_changes.md |
| NF4 | `_is_display_stock_in_catalog` method | v19_changes.md |
| NF5 | `_compute_reference` on stock.move | v19_changes.md |
| NF6 | `stock.reference` integration | v19_changes.md |
| NF7 | `action_create_sale_order` refactorizado en hooks `_create_sale_order()` + `_get_sale_order_values() | v19_changes.md |
| RC1 | Class renamed | v19_changes.md |
| RC10 | New indexes on repair fields | v19_changes.md |
| RC11 | RC16. Additional field removals and renames | v19_changes.md |
| RC17 | `product.template.create_repair` ELIMINADO → `service_tracking == 'repair'` | v19_changes.md |
| RC18 | Feature 'Create Repair from Return' RETIRADA (`is_repairable` / `return_type_of_ids`) | v19_changes.md |
| RC2 | `procurement_group_id` field REMOVED | v19_changes.md |
| RC3 | `product_uom_category_id` field REMOVED | v19_changes.md |
| RC4 | `is_returned` field REMOVED | v19_changes.md |
| RC5 | `picking_id` semantics changed | v19_changes.md |
| RC6 | SOL `product_uom` renamed | v19_changes.md |
| RC7 | Decimal precision renamed | v19_changes.md |
| RC8 | `stock.move.create()` values changed | v19_changes.md |
| RC9 | `_sql_constraints` → `models.Constraint` | v19_changes.md |
| REPCAP1 | ¿Orden de reparación con estados? | capabilities.md |
| REPCAP10 | ¿Vincular a una orden de fabricación? | capabilities.md |
| REPCAP11 | ¿Presupuesto/cotización de la reparación al cliente? | capabilities.md |
| REPCAP12 | ¿Mano de obra (tiempo/tarifa) además de piezas? | capabilities.md |
| REPCAP13 | ¿El cliente sigue el estado de su reparación en el portal? | capabilities.md |
| REPCAP14 | ¿Documento imprimible de la orden de reparación? | capabilities.md |
| REPCAP15 | ¿Prioridad y etiquetas en las órdenes de reparación? | capabilities.md |
| REPCAP2 | ¿Piezas: añadir/quitar/reciclar componentes? | capabilities.md |
| REPCAP3 | ¿Reparación bajo garantía? | capabilities.md |
| REPCAP4 | ¿Facturar la reparación? | capabilities.md |
| REPCAP5 | ¿Reparación desde un ticket de helpdesk? | capabilities.md |
| REPCAP6 | ¿Reparación a partir de una devolución (RMA)? | capabilities.md |
| REPCAP7 | ¿Controles de calidad en la reparación? | capabilities.md |
| REPCAP8 | ¿Consumir/producir contra stock (pickings/ubicaciones)? | capabilities.md |
| REPCAP9 | ¿Reparar producto con nº de serie/lote? | capabilities.md |
| RG1 | `qty_available` includes consignment stock | gotchas.md |
| RG10 | Removing a part doesn't delete its SOL | gotchas.md |
| RG11 | MTO rule exists but rarely triggers | gotchas.md |
| RG12 | No auto-PO on confirm for consignment | gotchas.md |
| RG13 | Phantom BoM explosion runs on EVERY write() | gotchas.md |
| RG14 | BoM explosion only for phantom type | gotchas.md |
| RG15 | Owner detection is for the PRODUCT, not parts | gotchas.md |
| RG16 | Consignment parts consumed without valuation | gotchas.md |
| RG17 | `helpdesk_repair` is NOT auto_install | gotchas.md |
| RG18 | `default_lot_id` context pollution | gotchas.md |
| RG19 | Quality blocks completion, not confirmation | gotchas.md |
| RG2 | Repair moves are NEVER assigned to pickings | gotchas.md |
| RG20 | Quantity-type quality checks forbidden on repairs | gotchas.md |
| RG21 | Zero-quantity moves auto-cancelled | gotchas.md |
| RG22 | Can't complete repair if state != 'under_repair' | gotchas.md |
| RG23 | No partial repair completion | gotchas.md |
| RG24 | `procurement_group_id` removed in v19 | gotchas.md |
| RG25 | Float comparison API changed in v19 | gotchas.md |
| RG26 | `helpdesk_repair` state notification only fires for `use_product_repairs` tickets | gotchas.md |
| RG27 | `quality_repair` rebuilds product checks on `lot_id`/`product_id` write | gotchas.md |
| RG28 | `industry_fsm_repair` exposes the picking, not the repair | gotchas.md |
| RG29 | Serial product silently forces `product_qty=1` on `product_id` change | gotchas.md |
| RG3 | No backorders on repairs | gotchas.md |
| RG30 | `move_ids` domain EXCLUDES the product-to-repair move | gotchas.md |
| RG31 | `picked=True` forced on completion even if technician never confirmed picking | gotchas.md |
| RG32 | `qty_delivered` only syncs when the SOL service is `ordered_prepaid` | gotchas.md |
| RG33 | `copy_data()` strips `sale_line_id` from copied repair moves | gotchas.md |
| RG34 | The repair module was COMPLETELY rewritten in v17 — v14/v15/v16 use an incompatible model | gotchas.md |
| RG35 | v14/v15/v16: invoice state lives on the RO, not the SO | gotchas.md |
| RG36 | v14/v15/v16: parts consumed as `repair.line.state`, not `stock.move` | gotchas.md |
| RG37 | v14/v15/v16: `b4repair` invoice blocks repair start | gotchas.md |
| RG38 | v19: `create_repair` eliminado y su `copy_data` de protección desaparece | gotchas.md |
| RG39 | v19: nuevo `account.move.line._eligible_for_stock_account` excluye moves de reparación 'add' ya cont | gotchas.md |
| RG4 | Product move created at completion, not confirmation | gotchas.md |
| RG40 | v19: cambiar la UoM de un product.product se bloquea si hay repair.order con otra UoM | gotchas.md |
| RG41 | v19: la creación de lotes/series en reparaciones la gobierna `use_create_lots` del picking_type de r | gotchas.md |
| RG42 | v19: `_create_repair_order` (SO→RO) ya no multiplica por unidades serial ni fija producto/qty/uom | gotchas.md |
| RG5 | `_is_consuming()` only for 'add' type | gotchas.md |
| RG6 | `has_valued_move_ids()` returns False for repair moves | gotchas.md |
| RG7 | SOLs created immediately, not on completion | gotchas.md |
| RG8 | `under_warranty` zeroes ALL part prices | gotchas.md |
| RG9 | Repair always installs sales | gotchas.md |

## functional/sale

| ID | Título | Fichero |
|----|--------|---------|
| SAC1 | Commission calculada sobre `SO.amount_total` en momento confirm, NO sobre `qty_to_invoice`. Down pay | integrations.md |
| SAC2 | Cambio de `crm.team.commission_plan_id` NO retroaplica a SOs existentes. Out-of-sync. | integrations.md |
| SAC3 | Salesman en 2 teams con planes distintos → toma uno arbitrario o default. Sin validación. | integrations.md |
| SAC4 | Refund de SO via credit note NO reversa commission. Acreedor cobra commission de venta cancelada. | integrations.md |
| SAC5 | Multi-currency: commission en SO currency pero payout en company currency con FX stale. | integrations.md |
| SAD1 | `recompute_delivery_price=True` se setea con changes pero NO recompute automático. Usuario debe abri | integrations.md |
| SAD2 | `free_over` con `amount_without_delivery` recompute → si agregas items oscila por encima/debajo del  | integrations.md |
| SAD3 | Tax aplicada en `rate_shipment()` mediante fiscal_position, pero `SO.pricelist_id` NO consultado. Pr | integrations.md |
| SAD4 | Margin + fixed_margin con multi-currency → conversion ANTES de margin → margin varía con FX diario.  | integrations.md |
| SAD5 | `zip_prefix_ids` se compila como regex con `\|`, pero NO se escapan metachars. Prefix con `.` literal | integrations.md |
| SAD6 | `partner_shipping_id` NULL → `_match_address()` peta con AttributeError (no chequea nullable). `deli | integrations.md |
| SAD7 | Cambiar `integration_level` de `rate_and_ship` a `rate` deja `return_label_on_delivery=True` sin sen | integrations.md |
| SAE1 | `sale_order_id` domain restringe a SO state='sale'. Si SO regresa a draft (cancel manual), expense.s | integrations.md |
| SAE2 | Propagación de analytic_distribution: si expense NO tiene analytic poblado, SOL queda vacío aunque p | integrations.md |
| SAE3 | `expense_policy='cost'` NO mapea taxes supplier → customer. SOL queda sin tax aunque expense tenía s | integrations.md |
| SAE4 | Multiple expenses al mismo SO → 1 SOL por expense (sin agrupación). 10 expenses = 10 SOLs duplicadas | integrations.md |
| SAE5 | `qty` del SOL = `expense.unit_amount` (importe EUR), no número de items. Confusión UX. | integrations.md |
| SAE6 | SO NO se locked durante expense rebilling. Edits paralelos en SO mientras se procesa expense → datos | integrations.md |
| SAG10 | Cambiar `product.invoice_policy` post-confirm NO redispara `_compute_qty_to_invoice`. Quedaría stale | integrations.md |
| SAG11 | Refund creada fuera del SO context (manualmente en account.move) NO aparece en `invoice_ids` porque  | integrations.md |
| SAG12 | Down payment rounding deltas asumen presence de receivable/product/tax lines. SO sin impuestos → del | integrations.md |
| SAG13 | `grouped=False` (default) → 1 invoice por (company, partner, currency, fiscal_position). `grouped=Tr | integrations.md |
| SAG14 | `order.locked=True` protege fields de SOL pero NO bloquea `_create_invoices()`. SO locked se puede f | integrations.md |
| SAG15 | `invoice_policy='manual'` permitido en código pero NO en UI selection. Hay que setear vía ORM. `qty_ | integrations.md |
| SAG16 | Múltiples SOs agrupados con down payment cada uno → "Down Payment Section" duplicada en invoice. `sa | integrations.md |
| SAG17 | Cambiar `partner_shipping_id` → fiscal_position recompute → `show_update_fpos=True` flag, pero NO re | integrations.md |
| SAG18 | `_get_delivered_quantity_by_analytic()` NO filtra por accrual date. Expense postada retroactivamente | integrations.md |
| SAG19 | `_compute_analytic_distribution()` no filtra `account.analytic.distribution.model` por company_id en | integrations.md |
| SAG20 | `stock.move` NO hereda `analytic_distribution` de SOL. `sale_stock._prepare_procurement_values` no o | integrations.md |
| SAG21 | `analytic_distribution` está en `protected_fields` de SO locked pero NO de invoice locked. Post-invo | integrations.md |
| SALECAP1 | ¿Descuentos por línea y descuento global? | capabilities.md |
| SALECAP10 | ¿Política de facturación: por pedido vs por entregado? | capabilities.md |
| SALECAP11 | ¿Anticipos / down payments? | capabilities.md |
| SALECAP12 | ¿Facturación por hitos (milestones)? | capabilities.md |
| SALECAP13 | ¿Servicios facturados por horas reales (timesheet→factura)? | capabilities.md |
| SALECAP14 | ¿Multi-moneda (presupuesto en divisa del cliente)? | capabilities.md |
| SALECAP15 | ¿Impuestos automáticos por posición fiscal (extranjero/intracom)? | capabilities.md |
| SALECAP16 | ¿Venta inter-compañía (pedido en Co.A genera compra en Co.B)? | capabilities.md |
| SALECAP17 | ¿Entregas parciales y backorders desde la venta? | capabilities.md |
| SALECAP18 | ¿Margen de venta (coste vs precio) en el pedido? | capabilities.md |
| SALECAP19 | ¿Comisiones de venta? | capabilities.md |
| SALECAP2 | ¿Listas de precios por cliente/cantidad/fecha? | capabilities.md |
| SALECAP20 | ¿Alquiler de productos desde el pedido de venta? | capabilities.md |
| SALECAP21 | ¿Vender el mismo producto en varias UoM/packaging en el pedido? | capabilities.md |
| SALECAP22 | ¿Cambiar la UoM de la línea recalcula el precio? | capabilities.md |
| SALECAP23 | ¿En qué UoM se guarda `qty_delivered`? | capabilities.md |
| SALECAP3 | ¿Precios con IVA incluido o sin según cliente? | capabilities.md |
| SALECAP4 | ¿Productos opcionales / cross-sell en el presupuesto? | capabilities.md |
| SALECAP5 | ¿Variantes de producto (talla/color) en la venta? | capabilities.md |
| SALECAP6 | ¿Plantillas de presupuesto reutilizables? | capabilities.md |
| SALECAP7 | ¿PDF de presupuesto personalizado con secciones/imágenes? | capabilities.md |
| SALECAP8 | ¿Firma electrónica y pago online del presupuesto (portal)? | capabilities.md |
| SALECAP9 | ¿Flujo de aprobación antes de confirmar (p.ej. descuento alto)? | capabilities.md |
| SAM1 | Phantom BOM explota a nivel components en picking, NO crea MO. Multi-warehouse rompe coordinación. ` | integrations.md |
| SAM2 | `_compute_kit_quantities()` filtra `incoming_moves`/`outgoing_moves` con lógica INVERTIDA respecto a | integrations.md |
| SAM3 | `bom_line_id` se asigna por `[:1]` | integrations.md |
| SAM4 | Kit con sub-BOM (BOM dentro de BOM) → `explode()` es 1-level only. Subassemblies NO se generan en sa | integrations.md |
| SAM5 | Dropship kit con mix MTS/Dropship → `qty_delivered` es "all-or-nothing". 80% dropship + 20% MTS → qt | integrations.md |
| SAM6 | `_ensure_bom_is_free()` previene cambios de phantom BOM si hay SOs activos, pero desactivar BOM prim | integrations.md |
| SAM7 | `sale_mrp_margin`: standard_price NO refleja cost actual de MO. MO con scrap → margin computado con  | integrations.md |
| SAMA1 | `purchase_price` NO se actualiza si `product.standard_price` cambia post-confirm sin context `sale_w | integrations.md |
| SAMA2 | `margin_percent = price_subtotal and margin/price_subtotal` | integrations.md |
| SAMA3 | Alternative calc (`product_uom_qty=0` pero `qty_delivered>0`) para expense rebilling NO valida que p | integrations.md |
| SAMA4 | `cost_currency_id ≠ company_currency` con conversion `round=False` → demasiados decimales en purchas | integrations.md |
| SAMA5 | `sale_mrp_margin`: MO cost actual (con scrap/desviación) NO propagado al SOL.margin. Standard_price  | integrations.md |
| SAP1 | Cambio de `category_id`/`service_tracking` borra+recrea approvers/milestones de tareas relacionadas  | integrations.md |
| SAP10 | Reinvoiced SOL hereda `product_uom_qty` del move sin conversión UoM. Si `move.product_uom ≠ product. | integrations.md |
| SAP2 | `task_global_project` requiere `product.project_id` pero NO hay constraint que lo valide. Si product | integrations.md |
| SAP3 | `project_id` en SO se asigna SOLO al primer SOL que genera proyecto. SOLs posteriores con tipo disti | integrations.md |
| SAP4 | Cambiar `product_uom_qty` en SOL `sale` auto-actualiza `task.allocated_hours` sin context check. Si  | integrations.md |
| SAP5 | `analytic_distribution` heredado del project NO se propaga a `stock.move` (sale_project_stock no ove | integrations.md |
| SAP6 | `milestone.quantity_percentage` default `=1` (100%). Crear 3 milestones manuales para 1 SOL → `qty_d | integrations.md |
| SAP7 | Sub-tasks de `project_template_id` heredan `sale_line_id` solo si tienen `parent_id != False` (la wr | integrations.md |
| SAP8 | `reinvoiced_sale_order_id ≠ picking.sale_id`. Si project.reinvoiced apunta a otro SO, el SOL de rein | integrations.md |
| SAP9 | `picking_type.analytic_costs=False` por default en cualquier nuevo warehouse → moves NO reinvoiced.  | integrations.md |
| SAPU1 | Constraint `service_to_purchase=True` requiere `type='service'` Y al menos un `seller_id`. Si borras | integrations.md |
| SAPU10 | `analytic_distribution` propagado a PO line. Si luego duplicas PO line, analytic NO se limpia → dobl | integrations.md |
| SAPU2 | Bajar `product_uom_qty` en SO confirmado con PO existente → warning onchange pero NO bloquea ni decr | integrations.md |
| SAPU3 | Subir `product_uom_qty` con PO ya en `done` → crea NUEVA PO para el delta. Inconsistencias si la PO  | integrations.md |
| SAPU4 | UoM conversion `SO.product_uom → product.uom_po_id` puede acumular rounding si precisiones difieren. | integrations.md |
| SAPU5 | Currency conversion en PO creation usa rate del DÍA del crear PO, no del SO. Cambios de divisa entre | integrations.md |
| SAPU6 | Multi-company: `_purchase_service_get_company()` default a `self.company_id` pero `_select_seller()` | integrations.md |
| SAPU7 | Cancel de SO crea activity en PO solo si PO line state ≠ 'cancel' y `service_to_purchase=True`. PO y | integrations.md |
| SAPU8 | `sale_purchase_stock`: en v19 el cómputo se refactorizó a `stock_reference_ids` (M2M), simplificando | integrations.md |
| SAPU9 | Dropshipping con `dest_address_id` pero ruta sin picking_type apropiada → moves fallan al buscar loc | integrations.md |
| SAS1 | `qty_delivered` SOLO suma `stock.move.state='done'`. Picking en `assigned` no cuenta → `invoice_stat | integrations.md |
| SAS2 | Cambiar `product_uom_qty` en SOL `sale` redispara `_action_launch_stock_rule()` SIN cancelar moves p | integrations.md |
| SAS3 | Float rounding en `_get_qty_procurement()`: si `qty ≈ product_uom_qty` por redondeo (UoM conversions | integrations.md |
| SAS4 | `route_id.warehouse_id` puede diferir de `SO.warehouse_id` → la regla usa el warehouse de la ruta, p | integrations.md |
| SAS5 | `picking_policy='one'` espera todos los items pero si una línea queda agotada → genera 2 pickings (u | integrations.md |
| SAS6 | `customer_lead`/`security_lead` mal configurados → `date_planned = date_deadline - security_lead` pu | integrations.md |
| SAS7 | `to_refund` flag en move NO decrementa `qty_delivered` hasta que `state='done'`. RMA parcial pendien | integrations.md |
| SAS8 | Cambiar `customer_lead` en SOL post-`sale` solo propaga a `move.date_deadline` si NO hay `commitment | integrations.md |
| SAX1 | External tax LIMPIA `tax_id=False` en TODAS las líneas eligible ANTES de la API call. Si la API fall | integrations.md |
| SAX2 | `price_tax`/`price_subtotal` manualmente override en `_set_external_taxes()`. User edita `price_unit | integrations.md |
| SAX3 | `order.locked` check para external tax pero NO para invoice_status recalc. Líneas invoiceable status | integrations.md |
| SAX4 | Down payments excluidos de external (Odoo-computed) pero final invoice incluye ambos → mixed tax sou | integrations.md |
| SAX5 | Recálculo no debounced. Editar 100 SOLs → 100 API calls. Rate limiting silent fail. | integrations.md |
| SAX6 | `_compute_amounts()` asume external amounts siempre correctos. Si AvaTax retorna 0 (exemption erróne | integrations.md |
| SCG1 | Configure Pricelist with Volume Discounts | config_guide.md |
| SCG10 | Configure Gift Cards | config_guide.md |
| SCG11 | Configure Fiscal Position for Different Countries | config_guide.md |
| SCG2 | Configure Invoice Policy per Product | config_guide.md |
| SCG3 | Configure Down Payments | config_guide.md |
| SCG4 | Configure Subscriptions | config_guide.md |
| SCG5 | Configure Subscription Renewal | config_guide.md |
| SCG6 | Configure Subscription Upsell | config_guide.md |
| SCG7 | Configure Loyalty Program (Points) | config_guide.md |
| SCG8 | Configure Promotion (Automatic Discount) | config_guide.md |
| SCG9 | Configure Coupon Codes | config_guide.md |
| SG1 | Pricelist only computed in draft state | gotchas.md |
| SG10 | Subscription auto-close after payment failure = 15 days default | gotchas.md |
| SG11 | Subscription upsell prorata discount can confuse customers | gotchas.md |
| SG12 | Cancelled SO with posted invoices cannot be re-cancelled cleanly | gotchas.md |
| SG13 | Payment token archived = all linked subs lose auto-payment | gotchas.md |
| SG14 | Gift card points = money, not reward units | gotchas.md |
| SG15 | Nominative vs non-nominative coupons — different lifecycle | gotchas.md |
| SG16 | Programs restricted by pricelist_ids field | gotchas.md |
| SG17 | Subscription invoice consolidation is optional | gotchas.md |
| SG18 | Deferred revenue requires company configuration | gotchas.md |
| SG19 | Multiple programs can stack — no automatic limit | gotchas.md |
| SG2 | qty_to_invoice ≠ product_uom_qty | gotchas.md |
| SG20 | Subscription stock delivery creates picking per invoice period | gotchas.md |
| SG21 | Kit + invoice_policy: order shows kit, delivery shows components | gotchas.md |
| SG22 | commitment_date vs expected_date — different purposes | gotchas.md |
| SG23 | Subscription pricing overrides pricelist for recurring products | gotchas.md |
| SG24 | Loyalty rewards don't match kit components | gotchas.md |
| SG25 | Two distinct date computations — don't conflate them | gotchas.md |
| SG26 | Pricelist + price_include: tax-inclusive pricing | gotchas.md |
| SG27 | Gift card revenue recognized at sale, not at redemption | gotchas.md |
| SG28 | Payment terms on subscriptions: due date only, not billing trigger | gotchas.md |
| SG29 | Cancel SO with partial deliveries: done pickings survive | gotchas.md |
| SG3 | Pricelist rule priority is SQL-based, not UI order | gotchas.md |
| SG30 | Pricelist per warehouse: not supported natively | gotchas.md |
| SG4 | Chained pricelists have no caching | gotchas.md |
| SG5 | Fiscal position locks at SO confirmation, not at invoice | gotchas.md |
| SG6 | Down payment rounding issues on final invoice | gotchas.md |
| SG7 | Invoice grouping merges multiple SOs silently | gotchas.md |
| SG8 | Loyalty discounts apply AFTER pricelist pricing | gotchas.md |
| SG9 | Subscription plan required if ANY line has recurring_invoice=True | gotchas.md |

## functional/slides

| ID | Título | Fichero |
|----|--------|---------|
| SLDCAP1 | ¿Cursos con contenido variado (vídeo, PDF, artículo, infografía, quiz)? | capabilities.md |
| SLDCAP10 | ¿Foro de preguntas por curso? | capabilities.md |
| SLDCAP11 | ¿Reseñas / valoración de cursos? | capabilities.md |
| SLDCAP12 | ¿Generar competencias del empleado al completar? | capabilities.md |
| SLDCAP13 | ¿Visibilidad (público/solo miembros) y publicación? | capabilities.md |
| SLDCAP14 | ¿Tiempo de finalización / duración por lección? | capabilities.md |
| SLDCAP15 | ¿Recursos descargables adjuntos a la lección? | capabilities.md |
| SLDCAP16 | ¿Email/recordatorio automático a los inscritos? | capabilities.md |
| SLDCAP17 | ¿Acceso al curso vía compra/suscripción (no solo abierto)? | capabilities.md |
| SLDCAP18 | ¿Curso multi-idioma? | capabilities.md |
| SLDCAP19 | ¿Certificación con certificado al aprobar? | capabilities.md |
| SLDCAP2 | ¿Vídeo embebido de YouTube/Google Drive/Vimeo? | capabilities.md |
| SLDCAP20 | ¿Contenido secuencial obligatorio (no saltar lecciones)? | capabilities.md |
| SLDCAP3 | ¿Inscripción pública / por invitación / de pago? | capabilities.md |
| SLDCAP4 | ¿Vender cursos? | capabilities.md |
| SLDCAP5 | ¿Quiz dentro de la lección con puntuación? | capabilities.md |
| SLDCAP6 | ¿Certificación con certificado descargable? | capabilities.md |
| SLDCAP7 | ¿Gamificación: karma, rangos, insignias? | capabilities.md |
| SLDCAP8 | ¿Seguimiento de progreso y % completado por alumno? | capabilities.md |
| SLDCAP9 | ¿Prerrequisitos entre cursos / contenido secuencial? | capabilities.md |
| SLDG1 | Completar un curso es AUTOMÁTICO, no manual. | gotchas.md |
| SLDG10 | Publicar un slide recomputa la completion de TODOS (y avisa a followers si hay plantilla). | gotchas.md |
| SLDG11 | Las categorías (`is_category`) se autopublican/autopreview y se excluyen del recuento. | gotchas.md |
| SLDG12 | Quitar membresía ARCHIVA, no borra, y conserva el karma. | gotchas.md |
| SLDG13 | Solo una review por curso y cuesta karma. | gotchas.md |
| SLDG14 | `enroll='invite'` requiere permiso de escritura para auto-enrolar. | gotchas.md |
| SLDG15 | El badge NO se otorga si el survey no tiene `certification_give_badge=True`. | integrations.md |
| SLDG16 | El badge solo aparece en "cursos" si su challenge tiene `challenge_category='slides'`. | integrations.md |
| SLDG17 | Fallar la ÚLTIMA certificación te EXPULSA del curso (silenciosamente). | integrations.md |
| SLDG18 | `_remove_membership` desliga los `user_input` del `slide_partner_id` pero NO los borra. | integrations.md |
| SLDG19 | La inscripción por compra solo dispara con `enroll='payment'` Y al confirmar el pedido | integrations.md |
| SLDG2 | "Completed" es irreversible por progreso. | gotchas.md |
| SLDG20 | Publicar el curso de pago publica el producto y viceversa. | integrations.md |
| SLDG21 | Un producto-curso publicado se puede añadir al carrito aunque las reglas normales lo | integrations.md |
| SLDG22 | Borrar un survey usado como certificación está bloqueado. | integrations.md |
| SLDG23 | Cambiar `forum_id` reconfigura la privacidad de DOS foros | integrations.md |
| SLDG24 | hr_skills_slides crea la línea de CV con `sudo()` solo si el partner es empleado | integrations.md |
| SLDG3 | El karma de fin de curso se suma — y se RESTA si se descompleta. | gotchas.md |
| SLDG4 | El karma del quiz es DECRECIENTE por intento, y se revierte al descompletar. | gotchas.md |
| SLDG5 | Un slide con preguntas NO se marca completado a mano. | gotchas.md |
| SLDG6 | Marcar visto/completado exige ser member. | gotchas.md |
| SLDG7 | `visibility='members'` fuerza `enroll='invite'` por constraint SQL. | gotchas.md |
| SLDG8 | Ver el curso ≠ ser miembro. | gotchas.md |
| SLDG9 | Las invitaciones caducan a los 3 meses. | gotchas.md |

## functional/spreadsheet

| ID | Título | Fichero |
|----|--------|---------|
| SPRG1 | Large datasets slow the UI | gotchas.md |
| SPRG10 | Inactive records included | gotchas.md |
| SPRG11 | Multi-company accounting cross-talk | gotchas.md |
| SPRG12 | Currency rates not refreshed automatically | gotchas.md |
| SPRG13 | Unposted entries excluded by default | gotchas.md |
| SPRG14 | Field rename breaks formulas silently | gotchas.md |
| SPRG15 | ACL on shared spreadsheet | gotchas.md |
| SPRG16 | Batched RPC dedup pitfall | gotchas.md |
| SPRG17 | Cardinality explosion | gotchas.md |
| SPRG18 | Date timezone confusion | gotchas.md |
| SPRG19 | Measure column re-render lag | gotchas.md |
| SPRG2 | JSON size and DB bloat | gotchas.md |
| SPRG20 | Threshold default 50 | gotchas.md |
| SPRG21 | Sort by computed field | gotchas.md |
| SPRG22 | AND-merge with base domain | gotchas.md |
| SPRG23 | Chart cache not invalidated by filter change | gotchas.md |
| SPRG24 | Label localisation bug | gotchas.md |
| SPRG25 | Relative date filter with server-vs-user TZ | gotchas.md |
| SPRG26 | Built-in dashboards overwritten on `-u` | gotchas.md |
| SPRG27 | Public share data source fallthrough | gotchas.md |
| SPRG28 | Sample dashboard masks real data | gotchas.md |
| SPRG29 | Group access vs source-model access | gotchas.md |
| SPRG3 | Float precision | gotchas.md |
| SPRG30 | `ir.attachment.max_upload_size` silently rejects | gotchas.md |
| SPRG31 | Image token in public URL | gotchas.md |
| SPRG32 | Concurrent edit conflict → client must rebase | gotchas.md |
| SPRG33 | Revision log unbounded growth | gotchas.md |
| SPRG34 | Snapshot can fail mid-write | gotchas.md |
| SPRG35 | Cell threads not synced to source | gotchas.md |
| SPRG36 | Read-only public share allows full data export | gotchas.md |
| SPRG37 | Spreadsheet `copy()` behaviour (EE) | gotchas.md |
| SPRG38 | Public spreadsheet with live data → error page | gotchas.md |
| SPRG39 | Browser tab idle drops Bus connection | gotchas.md |
| SPRG4 | Locale and decimal separator | gotchas.md |
| SPRG40 | Excel export is the published version | gotchas.md |
| SPRG41 | Plugin tier restructuring | gotchas.md |
| SPRG42 | `version` field removed from JSON | gotchas.md |
| SPRG43 | `spreadsheet.dependencies` asset bundle renamed | gotchas.md |
| SPRG44 | `spreadsheet_dashboard_hr_contract` removed in v19 (EE module) | gotchas.md |
| SPRG45 | `=ODOO.LIST` fetches ALL rows up to the highest referenced index, no pagination | gotchas.md |
| SPRG46 | `LOADING_ERROR` is a sentinel, not a real error | gotchas.md |
| SPRG47 | 10 s notify timeout masks a hung RPC as "stale but no error" | gotchas.md |
| SPRG48 | `KeepLast` drops in-flight results on rapid filter changes | gotchas.md |
| SPRG49 | `_lastUpdate` only set on success → "Last refreshed" lies after an error | gotchas.md |
| SPRG5 | Plugin registration timing | gotchas.md |
| SPRG50 | `=ODOO.BALANCE`/`DEBIT`/`CREDIT` aggregate via `_read_group(aggregates=['debit:sum'])` | gotchas.md |
| SPRG51 | Balance-sheet vs P&L date semantics differ inside the SAME formula family | gotchas.md |
| SPRG52 | Account-code `=like` matching catches unintended children | gotchas.md |
| SPRG6 | Volatile functions recompute every cycle | gotchas.md |
| SPRG7 | Circular references not prevented | gotchas.md |
| SPRG8 | Tax / currency / field not found = silent partial result | gotchas.md |
| SPRG9 | Many2one display: stale name | gotchas.md |

## functional/stock

| ID | Título | Fichero |
|----|--------|---------|
| CG1 | Enable Multi-Step Routes | config_guide.md |
| CG10 | Configure Delivery Carrier with Route | config_guide.md |
| CG11 | Configure Product for Stock Tracking | config_guide.md |
| CG12 | Configure Reservation Method on Picking Type | config_guide.md |
| CG13 | Configure Inter-Company Stock Transfer | config_guide.md |
| CG14 | Configure Scrap Location | config_guide.md |
| CG15 | Configure Return Location | config_guide.md |
| CG16 | Configure Landed Costs | config_guide.md |
| CG17 | Configure Inventory Adjustments (Cycle Counts) | config_guide.md |
| CG18 | Configure Dual UoM (Buy in Barrels, Stock in Liters) | config_guide.md |
| CG19 | Configure Shrinkage Tracking (Merma) | config_guide.md |
| CG2 | Configure Warehouse Steps (1/2/3) | config_guide.md |
| CG3 | Configure Inter-Warehouse Resupply | config_guide.md |
| CG4 | Configure MTO (Make to Order) | config_guide.md |
| CG5 | Configure Reordering Rules (Orderpoint) | config_guide.md |
| CG6 | Configure Cross-Dock | config_guide.md |
| CG7 | Configure Dropshipping | config_guide.md |
| CG8 | Configure Putaway Rules | config_guide.md |
| CG9 | Configure FEFO (First Expiry First Out) | config_guide.md |
| STKCAP1 | ¿Multi-almacén y multi-ubicación? | capabilities.md |
| STKCAP10 | ¿Caducidad / fechas de vencimiento? | capabilities.md |
| STKCAP11 | ¿Paquetes y pallets? | capabilities.md |
| STKCAP12 | ¿Picking por lotes / oleadas (batch/wave)? | capabilities.md |
| STKCAP13 | ¿Código de barras (app móvil)? | capabilities.md |
| STKCAP14 | ¿Consigna / stock de terceros? | capabilities.md |
| STKCAP15 | ¿Inventario físico / ajustes con conteo? | capabilities.md |
| STKCAP16 | ¿Trazabilidad upstream/downstream? | capabilities.md |
| STKCAP17 | ¿Reservas (manual/automática) y reglas? | capabilities.md |
| STKCAP18 | ¿Devoluciones (returns) desde entrega? | capabilities.md |
| STKCAP19 | ¿La demanda y lo realmente movido pueden ir en UoM distintas? | capabilities.md |
| STKCAP2 | ¿Rutas configurables (MTO, dropship, push/pull, multi-paso)? | capabilities.md |
| STKCAP20 | ¿La reserva redondea a favor de tener stock suficiente? | capabilities.md |
| STKCAP21 | ¿El albarán muestra el packaging del pedido (caja/palet) en v19? | capabilities.md |
| STKCAP22 | ¿Puedo cambiar la UoM de stock de un producto que ya tiene movimientos? | capabilities.md |
| STKCAP3 | ¿Lotes y números de serie (trazabilidad)? | capabilities.md |
| STKCAP4 | ¿Valoración FIFO / coste medio / estándar? | capabilities.md |
| STKCAP5 | ¿Valoración automática contable (asientos de stock)? | capabilities.md |
| STKCAP6 | ¿Costes en destino (landed costs)? | capabilities.md |
| STKCAP7 | ¿Reglas de reabastecimiento (min/max)? | capabilities.md |
| STKCAP8 | ¿Recepción/entrega multi-paso (calidad, packing)? | capabilities.md |
| STKCAP9 | ¿Putaway y estrategias de removal (FEFO/FIFO/LIFO)? | capabilities.md |
| STKG1 | Product type='consu' is storable in v18 | gotchas.md |
| STKG10 | Return wizard unreserves downstream moves | gotchas.md |
| STKG11 | Multi-step returns are single-step by default | gotchas.md |
| STKG12 | move_type='one' blocks partial delivery | gotchas.md |
| STKG13 | route_ids on SO line overrides EVERYTHING | gotchas.md |
| STKG14 | Dropship pickings have no warehouse | gotchas.md |
| STKG15 | Quant reserved_quantity is readonly but critical | gotchas.md |
| STKG16 | total_route_ids is recursive up category tree | gotchas.md |
| STKG17 | Picking state is computed from moves | gotchas.md |
| STKG18 | Reservation method on picking type controls timing | gotchas.md |
| STKG19 | Orderpoint route_id overrides normal resolution | gotchas.md |
| STKG2 | Move line locations differ from move locations | gotchas.md |
| STKG20 | Inter-company transit uses special location | gotchas.md |
| STKG21 | MTO route is archived by default | gotchas.md |
| STKG22 | MTO always procures even with stock available | gotchas.md |
| STKG23 | Cancelling SO does NOT cancel linked MTO procurement | gotchas.md |
| STKG24 | 0/0/1 reordering rule vs MTO | gotchas.md |
| STKG25 | Cross-dock requires 2+ steps on BOTH incoming and outgoing | gotchas.md |
| STKG26 | Push rules don't fire if pull rules already created the transfer | gotchas.md |
| STKG27 | Visibility days only activates when stock already below min | gotchas.md |
| STKG28 | Multi-Step Routes setting gates ALL warehouse step options | gotchas.md |
| STKG29 | Internal transfers create NO valuation layer | gotchas.md |
| STKG3 | Warehouse creates 8 picking types even for 1-step | gotchas.md |
| STKG30 | AVCO updates standard_price BEFORE creating the SVL | gotchas.md |
| STKG31 | FIFO negative stock uses estimated price corrected later | gotchas.md |
| STKG32 | Landed costs only work with AVCO and FIFO | gotchas.md |
| STKG33 | Changing cost method empties and replenishes ALL stock | gotchas.md |
| STKG34 | Valuation report requires Developer Mode | gotchas.md |
| STKG35 | Anglo-Saxon COGS only recognized on invoice posting, not delivery | gotchas.md |
| STKG36 | Return valuation uses original move's SVL price | gotchas.md |
| STKG37 | stock.quant.value is average, not specific | gotchas.md |
| STKG38 | Location accounts override category accounts | gotchas.md |
| STKG39 | Price difference entry when PO != Invoice price | gotchas.md |
| STKG4 | Procurement destination is Stock, not Customer | gotchas.md |
| STKG40 | group_propagation_option='fixed' kills date_deadline | gotchas.md |
| STKG41 | mts_else_mto is converted to MTS at move creation | gotchas.md |
| STKG42 | Push moves are always procure_method='make_to_order' | gotchas.md |
| STKG43 | Scheduler assignment priority is NOT FIFO by creation | gotchas.md |
| STKG44 | _run_buy merges procurements into existing draft POs | gotchas.md |
| STKG45 | Kit explosion happens BEFORE route resolution | gotchas.md |
| STKG46 | Inter-company != inter-warehouse | gotchas.md |
| STKG47 | FEFO orders by removal_date, NOT expiration_date | gotchas.md |
| STKG48 | Expired lots excluded from reservation but NOT hard-blocked | gotchas.md |
| STKG49 | Carrier route_ids inject into procurement silently | gotchas.md |
| STKG5 | Changing PO picking_type changes destination warehouse | gotchas.md |
| STKG50 | push_domain cannot reference picking_id fields directly | gotchas.md |
| STKG51 | Costing method is per-category, NOT per-warehouse | gotchas.md |
| STKG52 | reservation_method='by_date' needs reservation_days_before | gotchas.md |
| STKG53 | FIFO valuation is GLOBAL per product | gotchas.md |
| STKG54 | Anglo-Saxon partial invoicing: Stock Output stays open | gotchas.md |
| STKG55 | Consignment stock (owner_id) is not valued in YOUR inventory | gotchas.md |
| STKG56 | Serial numbers flow through multi-step automatically | gotchas.md |
| STKG57 | Orderpoint unique constraint: one per (product, location, company) | gotchas.md |
| STKG58 | Putaway rules priority: product > category > package type | gotchas.md |
| STKG59 | Splitting a picking manually creates backorder | gotchas.md |
| STKG6 | Push rules can loop infinitely | gotchas.md |
| STKG60 | Large-scale stock: performance bottlenecks | gotchas.md |
| STKG61 | Dual UoM: purchase UoM != stock UoM != sale UoM | gotchas.md |
| STKG62 | Shrinkage has no native process | gotchas.md |
| STKG63 | Inventory adjustment creates move to Inventory Loss location | gotchas.md |
| STKG64 | lot_properties: dynamic attributes hidden by default | gotchas.md |
| STKG65 | sale_stock qty_at_date only checks ONE warehouse | gotchas.md |
| STKG66 | Stripe capture_manually IS natively supported | gotchas.md |
| STKG67 | MTO-triggered MO does NOT auto-confirm unless orderpoint or MTS move_dest | gotchas.md |
| STKG68 | Carrier lives in stock_delivery, not delivery | gotchas.md |
| STKG69 | POS picking is created at order paid, bypassing procurement rules | gotchas.md |
| STKG7 | Removal strategy is NOT always FIFO | gotchas.md |
| STKG70 | Repair moves are NOT created until action_repair_done | gotchas.md |
| STKG71 | MRP explodes into raw + finished moves, both linked via production_id | gotchas.md |
| STKG72 | Internal moves between valued locations create no SVL even across bridges | gotchas.md |
| STKG73 | Reservation logic moved out of `stock.quant._update_reserved_quantity` | gotchas.md |
| STKG74 | `_do_unreserve` skips moves with `picked=True` | gotchas.md |
| STKG75 | An orphaned internal location (no parent) belongs to no warehouse → MTO/Manufacture procurement find | gotchas.md |
| STKG8 | Negative move quantities trigger location swap | gotchas.md |
| STKG9 | Backorders skip procurement creation | gotchas.md |

## functional/survey

| ID | Título | Fichero |
|----|--------|---------|
| SUCG1 | Choose Question Type | config_guide.md |
| SUCG10 | One Response per User / Anonymous vs Identified | config_guide.md |
| SUCG2 | Configure Scoring | config_guide.md |
| SUCG3 | Configure Certification | config_guide.md |
| SUCG4 | Configure Conditional Questions (Jumps) | config_guide.md |
| SUCG5 | Configure Access Mode | config_guide.md |
| SUCG6 | Configure Attempts Limit | config_guide.md |
| SUCG7 | Configure Time Limit | config_guide.md |
| SUCG8 | Configure Live Sessions / Speed Rating | config_guide.md |
| SUCG9 | Configure Layout & Progression | config_guide.md |
| SVCAP1 | ¿Tipos de pregunta variados? | capabilities.md |
| SVCAP10 | ¿Lógica condicional (mostrar pregunta según respuesta)? | capabilities.md |
| SVCAP11 | ¿Reintentos limitados? | capabilities.md |
| SVCAP12 | ¿Preguntas obligatorias? | capabilities.md |
| SVCAP13 | ¿Respuesta anónima vs identificada? | capabilities.md |
| SVCAP14 | ¿Precargar datos del participante al invitar? | capabilities.md |
| SVCAP15 | ¿Envío masivo de la encuesta por email? | capabilities.md |
| SVCAP16 | ¿NPS / satisfacción tras cerrar un ticket? | capabilities.md |
| SVCAP17 | ¿Usar la encuesta como test de candidato o en eLearning? | capabilities.md |
| SVCAP18 | ¿Live session (preguntas en directo con ranking tipo kahoot)? | capabilities.md |
| SVCAP2 | ¿Modos: encuesta / quiz / evaluación / live? | capabilities.md |
| SVCAP3 | ¿Puntuación y nota de aprobado? | capabilities.md |
| SVCAP4 | ¿Paginación (una página / por sección / por pregunta)? | capabilities.md |
| SVCAP5 | ¿Tiempo límite? | capabilities.md |
| SVCAP6 | ¿Aleatorizar preguntas? | capabilities.md |
| SVCAP7 | ¿Compartir por enlace público y por invitación? | capabilities.md |
| SVCAP8 | ¿Estadísticas / resultados agregados? | capabilities.md |
| SVCAP9 | ¿Certificado descargable al aprobar? | capabilities.md |
| SVG1 | `simple_choice` scoring uses MAX positive answer_score; `multiple_choice` uses SUM | gotchas.md |
| SVG10 | `test_entry=True` user_inputs are EXCLUDED from `attempts_count` and stats | gotchas.md |
| SVG11 | `invite_token` has NO unique constraint | gotchas.md |
| SVG12 | `page_per_question` skips sections that have NO description, but DOES render sections that have one | gotchas.md |
| SVG13 | `users_can_go_back` CANNOT be combined with `scoring_with_answers_after_page` — it raises at save | gotchas.md |
| SVG14 | `last_displayed_page_id` persists between sessions | gotchas.md |
| SVG15 | `session_code` is computed but editable | gotchas.md |
| SVG16 | DELETING questions during an in-progress live session is blocked by an ondelete guard, but ADDING is | gotchas.md |
| SVG17 | Session leaderboard displays the `nickname` field only | gotchas.md |
| SVG18 | `session_speed_rating` requires `session_speed_rating_time_limit > 0` | gotchas.md |
| SVG19 | Certification badge is unique per survey | gotchas.md |
| SVG2 | Negative `answer_score` values are dropped from max calculation but applied to user score | gotchas.md |
| SVG20 | PDF certificate is ONE parametrized QWeb template, not 6 | gotchas.md |
| SVG21 | `certification_mail_template_id` requires `model='survey.user_input'` domain | gotchas.md |
| SVG22 | `survey_time_limit_reached` IS enforced server-side in the controller (correction) | gotchas.md |
| SVG23 | `time_limit` on question (live session) is in SECONDS; on survey (regular) is in MINUTES | gotchas.md |
| SVG24 | The print route takes the answer as a QUERY param, not a path segment | gotchas.md |
| SVG25 | v19 `lang_ids` empty = all installed languages, not just default | gotchas.md |
| SVG26 | v19: deactivating the LAST language of a survey raises | gotchas.md |
| SVG27 | `_compute_attempts_info` uses raw SQL, but flushes 6 fields first | gotchas.md |
| SVG28 | `predefined_question_ids` set at attempt start, not refreshed | gotchas.md |
| SVG29 | `scale` question type is HARD-CAPPED 0..10 | gotchas.md |
| SVG3 | `scoring_percentage` rounds to 0 if score is non-positive | gotchas.md |
| SVG30 | `scoring_success_min` default is 80%, constrained 0..100 | gotchas.md |
| SVG31 | `certification_badge_id` is `copy=False` | gotchas.md |
| SVG32 | `session_code` unique constraint AND `scoring_success_min` are both DB-level | gotchas.md |
| SVG33 | Two different stat paths with DIFFERENT filters — don't conflate them | gotchas.md |
| SVG34 | `hr.applicant.survey_id` es | integrations.md |
| SVG35 | `action_send_survey()` crea el `res.partner` del candidato | integrations.md |
| SVG36 | El `survey.survey` creado por `action_new_survey()` NO nace con `survey_type='recruitment'` (`hr_rec | integrations.md |
| SVG37 | `_compute_allowed_survey_types` añade `'recruitment'` solo si el usuario tiene `hr_recruitment.group | integrations.md |
| SVG38 | La certificación solo llega al resume si el partner del input corresponde a un `hr.employee` con `us | integrations.md |
| SVG39 | La línea de resume es | integrations.md |
| SVG4 | `scoring_success` checks `>= scoring_success_min` AT compute time | gotchas.md |
| SVG40 | `expiration_status` se computa solo desde `date_end` con ventana fija de 3 meses para `'expiring'` ( | integrations.md |
| SVG41 | `certification_validity_months=0` ⇒ `date_end=False` ⇒ `expiration_status` se queda en `'valid'` par | integrations.md |
| SVG42 | Al fallar el ÚLTIMO intento de la certificación, el alumno es | integrations.md |
| SVG43 | `_remove_membership` desvincula los `user_input.slide_partner_id` (`website_slides_survey/models/sli | integrations.md |
| SVG44 | Cada enrolment genera un `invite_token` NUEVO (`website_slides_survey/models/slide_slide.py:156`, `S | integrations.md |
| SVG45 | No miembro abriendo la certificación genera un `user_input` con `test_entry=True` (`website_slides_s | integrations.md |
| SVG46 | Un slide de tipo `certification` no puede ponerse como preview (`check_certification_preview` SQL, ` | integrations.md |
| SVG47 | Borrar un `survey.survey` usado como certificación de curso lanza `ValidationError` listando los cur | integrations.md |
| SVG48 | El badge se otorga ejecutando el | integrations.md |
| SVG49 | `certification_badge_id` es `copy=False` + `badge_uniq` UNIQUE (`survey/models/survey_survey.py:137` | integrations.md |
| SVG5 | Scored question with NO positive-score answer makes the attempt's `total_possible_score=0` → percent | gotchas.md |
| SVG50 | El goal definition usa `batch_user_expression='user.partner_id.id'` y `batch_distinctive_field=partn | integrations.md |
| SVG51 | Los mails de invitación se crean con `model=None, res_id=None` (`survey/wizard/survey_invite.py:245- | integrations.md |
| SVG52 | `_send_mail` usa `email_normalize` para resolver partner por email (`survey/wizard/survey_invite.py: | integrations.md |
| SVG53 | El empleado evaluado tiene PROHIBIDO ver los resultados de su propio 360º (`survey/controllers/main. | integrations.md |
| SVG54 | El template de survey 360º se resuelve en cascada | integrations.md |
| SVG55 | Seleccionar `survey_type='appraisal'` SOBREESCRIBE `access_mode`, `is_attempts_limited` y `users_can | integrations.md |
| SVG56 | `appraisal_manager_user_ids` se computa desde `user_input_ids.appraisal_id.manager_ids` (`hr_apprais | integrations.md |
| SVG57 | `_prepare_survey_anwers` deduplica por (survey, appraisal, partner\|email) (`hr_appraisal_survey/wiza | integrations.md |
| SVG6 | Triggering answers must come from EARLIER questions | gotchas.md |
| SVG7 | The survey-level `scoring_max_obtainable` counts ALL questions, but the per-attempt percentage denom | gotchas.md |
| SVG8 | `triggering_question_ids` is computed as `triggering_answer_ids.question_id` | gotchas.md |
| SVG9 | `attempts_count` SQL groups by `(partner_id OR email)` AND optional `invite_token` | gotchas.md |

## functional/website

| ID | Título | Fichero |
|----|--------|---------|
| ECG1 | Stock check is optimistic | ecommerce_deep.md |
| ECG10 | Kit component stock not warehouse-scoped | ecommerce_deep.md |
| ECG11 | Cart re-rate updates only price_unit on delivery line | ecommerce_deep.md |
| ECG12 | Margins skip fixed carriers entirely | ecommerce_deep.md |
| ECG13 | free_over excludes base_on_rule carriers | ecommerce_deep.md |
| ECG14 | Click & collect skips cart stock verification | ecommerce_deep.md |
| ECG15 | Unpublished products remain purchasable in existing carts | ecommerce_deep.md |
| ECG16 | Cart silently discarded on FP change between sessions | ecommerce_deep.md |
| ECG17 | Pricelist cached in session for 1 hour | ecommerce_deep.md |
| ECG18 | _recompute_prices resets ALL discounts to zero | ecommerce_deep.md |
| ECG19 | Affiliate ID accepted from URL without validation | ecommerce_deep.md |
| ECG2 | free_qty uses website warehouse context | ecommerce_deep.md |
| ECG20 | _cart_update silently casts quantities to int | ecommerce_deep.md |
| ECG21 | Combo products bypass prevent_zero_price_sale | ecommerce_deep.md |
| ECG22 | Wire transfer leaves SO in 'sent' state | ecommerce_deep.md |
| ECG23 | Same SO can have multiple payment transactions | ecommerce_deep.md |
| ECG24 | Ghost carts after cancelled external payment | ecommerce_deep.md |
| ECG25 | Cart recovery merge preserves original prices | ecommerce_deep.md |
| ECG26 | Express checkout pricelist differs from standard | ecommerce_deep.md |
| ECG3 | Delivery rate recomputed on every cart change | ecommerce_deep.md |
| ECG4 | Express checkout freezes pricelist | ecommerce_deep.md |
| ECG5 | Automatic invoice requires ICP | ecommerce_deep.md |
| ECG6 | Refund does NOT create credit notes | ecommerce_deep.md |
| ECG7 | GeoIP fiscal position only for public users | ecommerce_deep.md |
| ECG8 | Cart recovery email only for identified users | ecommerce_deep.md |
| ECG9 | Click & collect changes warehouse AND fiscal position | ecommerce_deep.md |
| WEBCAP1 | ¿Catálogo con variantes en la tienda? | capabilities.md |
| WEBCAP10 | ¿Mostrar stock disponible en la ficha? | capabilities.md |
| WEBCAP11 | ¿Multi-web (varias tiendas en una BD)? | capabilities.md |
| WEBCAP12 | ¿SEO (meta, sitemap, urls)? | capabilities.md |
| WEBCAP13 | ¿Cross-sell / upsell en la tienda? | capabilities.md |
| WEBCAP14 | ¿Cupones / promociones online? | capabilities.md |
| WEBCAP15 | ¿Compra como invitado (sin registro)? | capabilities.md |
| WEBCAP16 | ¿Multi-idioma / multi-moneda en la web? | capabilities.md |
| WEBCAP17 | ¿Blog / foro / contenido? | capabilities.md |
| WEBCAP18 | ¿Impuestos por geolocalización del visitante? | capabilities.md |
| WEBCAP2 | ¿Listas de precios por web/cliente (B2B vs B2C)? | capabilities.md |
| WEBCAP3 | ¿Precios con/sin IVA según visitante? | capabilities.md |
| WEBCAP4 | ¿Pasarelas de pago online (varias)? | capabilities.md |
| WEBCAP5 | ¿Métodos de envío con tarifas? | capabilities.md |
| WEBCAP6 | ¿Cuenta de cliente / portal con pedidos? | capabilities.md |
| WEBCAP7 | ¿Carritos abandonados (recuperación)? | capabilities.md |
| WEBCAP8 | ¿Lista de deseos (wishlist)? | capabilities.md |
| WEBCAP9 | ¿Comparador de productos? | capabilities.md |
| WG1 | COW creates silent copies on edit | gotchas.md |
| WG10 | Snippet assets disabled by weekly cron | gotchas.md |
| WG11 | website.published.mixin checks write access for can_publish | gotchas.md |
| WG12 | specific_user_account scopes login to website | gotchas.md |
| WG13 | _cart_find_product_line never matches combo products | gotchas.md |
| WG14 | _verify_updated_quantity is the stock hook | gotchas.md |
| WG15 | Express checkout matches partners by ADDRESS fields, not email | gotchas.md |
| WG16 | Pricelist resolution uses GeoIP country -> country group | gotchas.md |
| WG17 | Cart survives across sessions for logged users | gotchas.md |
| WG18 | Checkout address writes to res.partner | gotchas.md |
| WG19 | website_crm auto-installs with website+crm | gotchas.md |
| WG2 | COW creates copies for ALL other websites on delete | gotchas.md |
| WG20 | Contact form sets medium_id="Website" by default | gotchas.md |
| WG21 | Delivery carrier visibility is website-specific | gotchas.md |
| WG22 | _auto_apply_rewards runs on every cart update | gotchas.md |
| WG23 | Abandoned cart detection: state=='draft' AND date_order <= (now - cart_abandoned_delay, default 1h)  | gotchas.md |
| WG24 | B2B/B2C tax display is per-website | gotchas.md |
| WG25 | Module count is 90 (58 CE + 32 EE) | gotchas.md |
| WG26 | Web form lead links visitor by email+phone match, can mis-attribute partner | gotchas.md |
| WG27 | crm.lead type (lead vs opportunity) decided by the team's use_leads, not global setting | gotchas.md |
| WG28 | Web form GeoIP auto-fills state_id only for crm.lead and only if blank | gotchas.md |
| WG29 | Helpdesk web form creates a res.partner per unknown email, no dedup beyond exact email | gotchas.md |
| WG3 | filter_duplicate() is the resolution mechanism | gotchas.md |
| WG30 | account.move.website_id is computed from sale_line origin and blanks on multi-website merges | gotchas.md |
| WG31 | ir.rule cache key gains `website_id` — same user, different rows per website | gotchas.md |
| WG32 | QWeb template cache is keyed by `website_id` + `cookies_allowed` | gotchas.md |
| WG33 | Anonymous visitor token is exactly 32 chars; logged-in token IS the partner id | gotchas.md |
| WG34 | Page track dedup is 30 min, NOT per request | gotchas.md |
| WG35 | Only TRACKED pages create/refresh a visitor | gotchas.md |
| WG36 | Visitor lookup returns None (not empty recordset) without a bound request env | gotchas.md |
| WG37 | assets_frontend has minimal + lazy split — order matters | gotchas.md |
| WG38 | Editor edit-mode disables CDN and the public-user render path | gotchas.md |
| WG4 | Menu create without website_id duplicates for ALL websites | gotchas.md |
| WG5 | Visitor UPSERT uses raw SQL | gotchas.md |
| WG6 | Anonymous visitor token is SHA1(IP+UA+session) | gotchas.md |
| WG7 | Visit count increments after 8h gap | gotchas.md |
| WG8 | Page visibility='password' stores hash | gotchas.md |
| WG9 | CDN filters are regex per line | gotchas.md |

## enterprise/account_3way_match

| ID | Título | Fichero |
|----|--------|---------|
| A3M1 | Default value 'exception' on install | gotchas.md |
| A3M10 | on-ordered formula subtracts current line qty | gotchas.md |
| A3M11 | on-received: received=0 path | gotchas.md |
| A3M12 | release_to_pay store=True, recomputes on every payment state change | gotchas.md |
| A3M13 | Dashboard query override is purchase-aware | gotchas.md |
| A3M14 | v19 `line_subsection` skipped in compute | gotchas.md |
| A3M15 | v19 `'Product Unit'` decimal-precision rename | gotchas.md |
| A3M16 | Module NOT auto_install | gotchas.md |
| A3M2 | Lines without purchase_line_id → exception | gotchas.md |
| A3M3 | Currency conversion uses TODAY's rate | gotchas.md |
| A3M4 | force_release_to_pay auto-set on onchange only | gotchas.md |
| A3M5 | `release_to_pay='no'` for paid bills (overloaded value) | gotchas.md |
| A3M6 | Only invoice_line_ids checked, section/note skipped | gotchas.md |
| A3M7 | Non-invoice / out-invoice moves silently 'no' | gotchas.md |
| A3M8 | Mixed line statuses → exception (NOT logical OR) | gotchas.md |
| A3M9 | Decimal precision dependency | gotchas.md |
| A3MCAP1 | ¿Conciliación a 3 bandas (PO↔recepción↔factura)? | capabilities.md |
| A3MCAP10 | ¿Configurable por compañía o por factura? | capabilities.md |
| A3MCAP2 | ¿Estado "Should Be Paid"? | capabilities.md |
| A3MCAP3 | ¿Override manual del estado? | capabilities.md |
| A3MCAP4 | ¿Considera PO y recepción? | capabilities.md |
| A3MCAP5 | ¿Aviso visual en la factura? | capabilities.md |
| A3MCAP6 | ¿Requiere purchase? | capabilities.md |
| A3MCAP7 | ¿Bloquea el pago o solo informa? | capabilities.md |
| A3MCAP8 | ¿Recepción parcial? | capabilities.md |
| A3MCAP9 | ¿Se recalcula al validar recepción/factura? | capabilities.md |

## enterprise/account_accountant

| ID | Título | Fichero |
|----|--------|---------|
| AAG1 | bank.rec.widget and bank.rec.widget.line are NEVER persisted to DB | gotchas.md |
| AAG10 | EPD (early payment discount) in bank reconciliation requires SAME currency AND amounts that apply | gotchas.md |
| AAG11 | Partner detection cascades through 4 strategies before giving up | gotchas.md |
| AAG12 | Partner bank account lookup uses ilike on sanitized number | gotchas.md |
| AAG13 | Deferred entries use 30-day months, not actual calendar days | gotchas.md |
| AAG14 | Single-month deferrals are silently skipped | gotchas.md |
| AAG15 | Resetting deferred invoice to draft reverses (not deletes) deferred entries | gotchas.md |
| AAG16 | Grouped deferral entries block reset to draft | gotchas.md |
| AAG17 | Changing account on a line with existing deferrals is blocked | gotchas.md |
| AAG18 | Deferred end date is INCLUSIVE — off-by-one trap | gotchas.md |
| AAG19 | Lock date exceptions have time-limited duration | gotchas.md |
| AAG2 | Widget state 'invalid' means suspense account is still present | gotchas.md |
| AAG20 | Setting lock dates does NOT auto-post draft entries | gotchas.md |
| AAG21 | Fiscal year records on child companies are forbidden | gotchas.md |
| AAG22 | Invoicing switch threshold uses raw SQL to mass-cancel moves | gotchas.md |
| AAG23 | Manual reconciliation wizard allows max 2 different accounts | gotchas.md |
| AAG24 | Auto-reconcile wizard 'one_to_one' matches by absolute rounded amount | gotchas.md |
| AAG25 | Auto-reconcile 'zero_balance' requires EXACT zero sum per group | gotchas.md |
| AAG26 | Follow-up status search queries ALL partners | gotchas.md |
| AAG27 | Follow-up 'total_due' only considers receivable accounts, not payable | gotchas.md |
| AAG28 | Batch payment state is computed from individual payment states | gotchas.md |
| AAG29 | Batch payment only accepts payments in specific states | gotchas.md |
| AAG3 | Auto-balance line uses partner ranks to choose receivable vs payable | gotchas.md |
| AAG30 | bank.rec.widget creates a new widget instance per statement line | gotchas.md |
| AAG31 | Invoice matching uses REGEXP_SPLIT_TO_ARRAY + UNNEST in SQL | gotchas.md |
| AAG32 | Predictive bill fields rank a full-text search over the last 100 posted entries | gotchas.md |
| AAG33 | Deferred journal and account are per-company, not shared | gotchas.md |
| AAG34 | Reconcile model company filtering uses exact match, not parent_of | gotchas.md |
| AAG35 | You cannot reset a reconciliation on a hashed (secured) bank transaction | gotchas.md |
| AAG36 | Deferred dates survive reversal ONLY under the `move_reverse_cancel` context | gotchas.md |
| AAG37 | bank.rec.widget(.line) are hidden from model selectors | gotchas.md |
| AAG4 | Editing the liquidity line directly modifies the bank statement line | gotchas.md |
| AAG5 | Auto-reconcile CRON only processes lines from the last 3 months | gotchas.md |
| AAG6 | Auto-reconcile CRON silently swallows UserError exceptions | gotchas.md |
| AAG7 | Auto-reconcile CRON has a 180-second hard time limit | gotchas.md |
| AAG8 | Reconciliation model first-match-wins, NOT best-match | gotchas.md |
| AAG9 | Token matching requires minimum 4 characters | gotchas.md |

## enterprise/account_asset

| ID | Título | Fichero |
|----|--------|---------|
| ASCAP1 | ¿Activos fijos con amortización? | capabilities.md |
| ASCAP10 | ¿Pausar y reanudar la amortización? | capabilities.md |
| ASCAP2 | ¿Métodos de amortización (lineal/degresivo/mixto)? | capabilities.md |
| ASCAP3 | ¿Generar el activo automáticamente desde una cuenta/factura? | capabilities.md |
| ASCAP4 | ¿Tabla de asientos de amortización automáticos? | capabilities.md |
| ASCAP5 | ¿Venta/disposición del activo (pérdida/ganancia)? | capabilities.md |
| ASCAP6 | ¿Modificar/revalorizar el activo? | capabilities.md |
| ASCAP7 | ¿Prorrateo (prorata temporis)? | capabilities.md |
| ASCAP8 | ¿Duración en meses/años? | capabilities.md |
| ASCAP9 | ¿Ingresos/gastos diferidos con el mismo motor? | capabilities.md |
| ASG1 | 30-day month assumption breaks real-world expectations | gotchas.md |
| ASG10 | Disposal of asset with children requires disposing children first | gotchas.md |
| ASG11 | gain_account_id / loss_account_id on res.company are easily missed | gotchas.md |
| ASG12 | Disposal move date = disposal_date, not today | gotchas.md |
| ASG13 | net_gain_on_sale only set on parent, not children | gotchas.md |
| ASG14 | Cancelling an asset reverses ALL posted moves, not just future ones | gotchas.md |
| ASG15 | Pausing and resuming does NOT extend the asset lifetime | gotchas.md |
| ASG16 | onchange_consistent_board wipes ALL draft depreciation moves | gotchas.md |
| ASG17 | validate() posts ALL depreciation moves including future ones | gotchas.md |
| ASG18 | Auto-create only fires on positive lines of invoices, models filtered by parent company | gotchas.md |
| ASG19 | Multiple models per account create N assets per line | gotchas.md |
| ASG2 | Prorata date silently resets to fiscal year start when prorata_computation_type='none' | gotchas.md |
| ASG20 | Reversing/cancelling an invoice deactivates assets, doesn't delete them | gotchas.md |
| ASG21 | Asset always uses company currency, not invoice currency | gotchas.md |
| ASG22 | Multi-company rule uses parent_of, not child_of | gotchas.md |
| ASG23 | Depreciation Schedule report uses raw SQL via _cr.execute and caches assets in a dict | gotchas.md |
| ASG24 | Report groups children (gross increases) under parent automatically | gotchas.md |
| ASG25 | Book value shows 0 for fully depreciated closed assets (salvage removed) | gotchas.md |
| ASG26 | Modify wizard date + 1 day for gross increase and board restart | gotchas.md |
| ASG27 | Modify wizard creates and auto-validates gross increase in one step | gotchas.md |
| ASG28 | Resume after pause requires date strictly after pause date | gotchas.md |
| ASG29 | Modify wizard pre-fills params but residual recomputes per date | gotchas.md |
| ASG3 | asset_lifetime_days is 0 when method_number is 0 — division by zero guard | gotchas.md |
| ASG30 | _create_move_before_date can produce zero-amount entries that are silently skipped | gotchas.md |
| ASG31 | Disposal lines with a falsy account are silently dropped from the move | gotchas.md |
| ASG32 | initial_account picks the original invoice account only if it is unique | gotchas.md |
| ASG33 | depreciated_amount in disposal includes already_depreciated_amount_import and uses copysign | gotchas.md |
| ASG34 | _get_residual_value_at_date ignores reversed entries by design | gotchas.md |
| ASG35 | depreciation_value on a closing/disposal move is computed, not the raw line balance | gotchas.md |
| ASG36 | Tax is NOT recomputed on depreciation/disposal entries — _get_computed_taxes returns existing tax_id | gotchas.md |
| ASG37 | Reversing a depreciation move silently rolls its value into the next draft, or appends a new period | gotchas.md |
| ASG38 | copy() of an asset always returns to draft and re-defaults the asset account | gotchas.md |
| ASG39 | Changing depreciation/expense account or journal rewrites only future unlocked draft+posted move lin | gotchas.md |
| ASG4 | Rounding accumulation strategy — cumulative vs per-period | gotchas.md |
| ASG40 | name and original_value compute are store=True readonly=False — manual edits stick but throw warning | gotchas.md |
| ASG41 | gross_increase_value and analytic distribution are computed sudo / weighted by balance | gotchas.md |
| ASG5 | degressive_then_linear child assets use parent's transition point | gotchas.md |
| ASG6 | already_depreciated_amount_import silently skips initial depreciation entries | gotchas.md |
| ASG7 | Fiscal year mismatch silently shifts depreciation dates | gotchas.md |
| ASG8 | Monthly depreciation capped at fiscal year end | gotchas.md |
| ASG9 | Lock date blocks disposal but not depreciation posting | gotchas.md |

## enterprise/account_budget

| ID | Título | Fichero |
|----|--------|---------|
| ABCAP1 | ¿Presupuestos por cuenta analítica? | capabilities.md |
| ABCAP10 | ¿Alerta o bloqueo al superar el presupuesto? | capabilities.md |
| ABCAP2 | ¿Líneas por periodo? | capabilities.md |
| ABCAP3 | ¿Presupuestado vs real conseguido? | capabilities.md |
| ABCAP4 | ¿% de cumplimiento? | capabilities.md |
| ABCAP5 | ¿Presupuestar por varios planes/ejes analíticos? | capabilities.md |
| ABCAP6 | ¿Comprometido (committed) vs real? | capabilities.md |
| ABCAP7 | ¿Presupuesto de compras? | capabilities.md |
| ABCAP8 | ¿Estados del presupuesto? | capabilities.md |
| ABCAP9 | ¿Generar líneas por mes/trimestre/año? | capabilities.md |
| ABG1 | NULL Analytic on Budget Line = Match Everything | gotchas.md |
| ABG10 | Budget Line Deletion Blocked by base.group_user ACL | gotchas.md |
| ABG11 | Company NULL on Budget = Cross-Company Matching in SQL | gotchas.md |
| ABG12 | Revised State Logic Is Asymmetric | gotchas.md |
| ABG13 | _read_group Override for Computed Fields Is Fragile | gotchas.md |
| ABG14 | Date Constraint Only on Budget Lines, Not on Budget Header | gotchas.md |
| ABG15 | PO Budget Matching Uses date_order, Not Planned Receipt | gotchas.md |
| ABG16 | Currency Conversion Not Applied in Budget Report | gotchas.md |
| ABG17 | PO budget matching Uses 'any' Operator on Subquery | gotchas.md |
| ABG18 | Dynamic Analytic Fields Require post_init_hook | gotchas.md |
| ABG19 | The whole model changed between v17 and v18 (no 1:1 migration) | gotchas.md |
| ABG2 | Theoretical Amount Off-by-One at Day Boundaries | gotchas.md |
| ABG20 | v17 practical_amount reads account.move.line with `-balance` and posted-only | gotchas.md |
| ABG21 | budget_type has copy=False | gotchas.md |
| ABG3 | Budget Type "both" Has No Sign Flip for Achieved | gotchas.md |
| ABG4 | Committed Amount Double-Counts During Invoicing Window | gotchas.md |
| ABG5 | budget.report Flushes 5 Models on Every Access | gotchas.md |
| ABG6 | _compute_all Uses Context Key for Performance | gotchas.md |
| ABG7 | analytic_json Stored Field Not Auto-Refreshed | gotchas.md |
| ABG8 | Budget Split Wizard Hardcodes expense Type | gotchas.md |
| ABG9 | is_above_budget on PO Considers Uncommitted Amounts Only Before Confirmation | gotchas.md |

## enterprise/account_consolidation

| ID | Título | Fichero |
|----|--------|---------|
| ACG1 | account_consolidation REMOVED in v18+ | core.md |
| ACG10 | `using_ids`/`used_in_ids` M2M poorly named | core.md |
| ACG11 | _name_search supports code prefix shortcut | core.md |
| ACG12 | Export to FY rotation requires manual period chain | core.md |
| ACG13 | Sign cascades multiplicatively | gotchas.md |
| ACG14 | Sign doesn't change stored balance | gotchas.md |
| ACG15 | composition_id forbids parallel company_period_id | gotchas.md |
| ACG16 | children_ids M2M self-ref via custom rel table | gotchas.md |
| ACG17 | Sub-consolidation triggers RECURSIVE journal regen | gotchas.md |
| ACG18 | dashboard_sections is JSON Text field | gotchas.md |
| ACG19 | company_unmapped_accounts_counts is JSON Text | gotchas.md |
| ACG2 | consolidation.* tables persist after upgrade | core.md |
| ACG20 | generate_guessed_company_periods runs ONCE on onchange | gotchas.md |
| ACG21 | show_on_dashboard at GROUP level only | gotchas.md |
| ACG22 | consolidated_balance_report uses a CUSTOM HANDLER, not the standard engine | gotchas.md |
| ACG23 | balance computed via _read_group of journal lines | gotchas.md |
| ACG24 | Onboarding steps tied to module install | gotchas.md |
| ACG25 | setting_consolidation_action returns "last chart" | gotchas.md |
| ACG26 | Auto-generated journal lines are read-only | gotchas.md |
| ACG27 | _check_conditional_unicity exempts hist mode | gotchas.md |
| ACG28 | hist accounts explode journal line count | gotchas.md |
| ACG29 | No upstream migration script | gotchas.md |
| ACG3 | action_generate_journals deletes auto_generated entries | core.md |
| ACG30 | account.account orphan columns | gotchas.md |
| ACG31 | Onboarding records require manual cleanup | gotchas.md |
| ACG4 | Closed periods FORBID new journals | core.md |
| ACG5 | Group with mixed children type forbidden | core.md |
| ACG6 | period.copy() shifts dates by +1 day, end=None | core.md |
| ACG7 | consolidation.account.account_ids M2M smart-write | core.md |
| ACG8 | Currency modes don't auto-apply | core.md |
| ACG9 | show_on_dashboard at GROUP level only | core.md |

## enterprise/account_disallowed

| ID | Título | Fichero |
|----|--------|---------|
| ADE1 | `unique_code` is GLOBAL, not per-company | gotchas.md |
| ADE10 | `active=False` hides but does NOT unlink | gotchas.md |
| ADE11 | `auto_install=True` — installs with `account_reports`, no easy opt-out | gotchas.md |
| ADE12 | v19 report label is "Fiscal Report" / "Account Fiscal Report" | gotchas.md |
| ADE13 | v19 replaces `_sql_constraints` with `models.Constraint` | gotchas.md |
| ADE14 | `_search_display_name` splits on the first space | gotchas.md |
| ADE15 | v19 category field on `account.account` is indexed (`btree_not_null`); v18 was not | gotchas.md |
| ADE16 | Report rate is per-AML-date, NOT the category's `current_rate` | gotchas.md |
| ADE17 | Cancelled moves excluded, drafts conditionally included | gotchas.md |
| ADE18 | Fleet bridge makes a vehicle mandatory on vendor bills for `car_category` accounts | gotchas.md |
| ADE19 | Fleet applies the MINIMUM of category-rate and vehicle-rate, not the vehicle's | gotchas.md |
| ADE2 | `current_rate` ignores today's-dated rate (strict `<`) | gotchas.md |
| ADE20 | `vehicle_split` defaults to True — the fleet report splits by vehicle out of the box | gotchas.md |
| ADE21 | Fleet `multi_rate_in_period` ORs the vehicle-rate check onto the category check | gotchas.md |
| ADE22 | Fleet warns about expense lines that have a vehicle but whose account has NO disallowed category | gotchas.md |
| ADE3 | `_onchange_internal_group` SILENTLY clears the category | gotchas.md |
| ADE4 | v18 → v19 module/model rename breaks references | gotchas.md |
| ADE5 | v18 → v19 rates move from category to account | gotchas.md |
| ADE6 | v19 adds `'asset'` to the allowed internal groups | gotchas.md |
| ADE7 | Display name renders `(No Rate)`, not blank | gotchas.md |
| ADE8 | `disallowed_expenses_category_id` has `check_company=True` | gotchas.md |
| ADE9 | Rate stored as a whole-number Float (e.g. `25.0`), divided by 100 in the report | gotchas.md |

## enterprise/account_extract

| ID | Título | Fichero |
|----|--------|---------|
| AEXG1 | `account_extract` no contiene código; la lógica está en `account_invoice_extract` | gotchas.md |
| AEXG10 | Detección de partner por "layout" depende de facturas previas del mismo proveedor | gotchas.md |
| AEXG11 | Creación automática de proveedor desde VAT usa Partner Autocomplete (otro servicio IAP) | gotchas.md |
| AEXG12 | `extract_single_line_per_tax` (default True) cambia drásticamente las líneas resultantes | gotchas.md |
| AEXG13 | Polling: webhook + cron diario; el webhook es `auth='public'` | gotchas.md |
| AEXG14 | `error_invalid_account_token`: borra el token IAP para regenerarlo | gotchas.md |
| AEXG15 | `_safe_upload` traga excepciones para no romper el lote (pero deja error_status) | gotchas.md |
| AEXG16 | El OCR de facturas y el de extractos bancarios comparten mixin pero NO endpoint | gotchas.md |
| AEXG17 | Las notificaciones (no-credit, error, success) las provee `iap_mail`, no `iap` | gotchas.md |
| AEXG18 | `is_in_extractable_state` solo True en borrador y siendo factura | gotchas.md |
| AEXG19 | El email de "sin créditos" se manda al usuario id=2, no a los creadores | gotchas.md |
| AEXG2 | El PDF se envía codificado a un servidor externo de Odoo (privacidad) | gotchas.md |
| AEXG20 | Solo se comprueban 20 documentos por click manual (`check_ocr_status`) | gotchas.md |
| AEXG21 | Las trazas del relleno OCR aparecen firmadas por OdooBot, no por ti | gotchas.md |
| AEXG22 | El texto OCR completo se vuelca en `index_content` del adjunto (búsqueda full-text) | gotchas.md |
| AEXG23 | `_validate_ocr` solo actúa sobre `waiting_validation`; postear dos veces no re-envía feedback | gotchas.md |
| AEXG24 | `error_status` por `AccessError` se mapea a "Server not available", enmascarando un problema de cone | gotchas.md |
| AEXG3 | Auto-digitalización por defecto en facturas de proveedor | gotchas.md |
| AEXG4 | El alias de email del diario dispara OCR automáticamente | gotchas.md |
| AEXG5 | Solo se procesa el `message_main_attachment_id`, no todos los adjuntos | gotchas.md |
| AEXG6 | El OCR no sobreescribe campos ya rellenados | gotchas.md |
| AEXG7 | `extract_state` máquina de estados: `done` no significa "validado bien" sino "feedback enviado" | gotchas.md |
| AEXG8 | La validación (feedback al OCR) solo ocurre al POSTEAR la factura | gotchas.md |
| AEXG9 | Al validar se borran las cajas OCR y el prefill data | gotchas.md |

## enterprise/account_followup

| ID | Título | Fichero |
|----|--------|---------|
| AFCAP1 | ¿Niveles de seguimiento configurables? | capabilities.md |
| AFCAP10 | ¿Informe de partidas vencidas por cliente? | capabilities.md |
| AFCAP2 | ¿Recordatorio escalado por nivel (email/SMS/carta)? | capabilities.md |
| AFCAP3 | ¿Cron de envío automático? | capabilities.md |
| AFCAP4 | ¿Estado/nivel de seguimiento por cliente? | capabilities.md |
| AFCAP5 | ¿Plantilla de email/SMS por nivel? | capabilities.md |
| AFCAP6 | ¿Generar carta PDF de recobro? | capabilities.md |
| AFCAP7 | ¿Recargos/intereses por demora? | capabilities.md |
| AFCAP8 | ¿Responsable del recobro? | capabilities.md |
| AFCAP9 | ¿Excluir/posponer facturas? | capabilities.md |
| AFG1 | delay can be NEGATIVE (pre-due reminders) | gotchas.md |
| AFG10 | UserError in cron swallowed, logged as internal note | gotchas.md |
| AFG11 | amount_residual <= 0 → no_action_needed | gotchas.md |
| AFG12 | followup_status excludes liability_payable | gotchas.md |
| AFG13 | with_overdue_invoices DOES NOT trigger reminder | gotchas.md |
| AFG14 | amount_residual aggregate doesn't filter by maturity | gotchas.md |
| AFG15 | send_email defaults True, send_sms False | gotchas.md |
| AFG16 | Manual reminder wizard ignores auto_execute | gotchas.md |
| AFG17 | join_invoices attaches main_attachment only | gotchas.md |
| AFG18 | Manual followup wizard filters attachment_ids | gotchas.md |
| AFG19 | Followup contact = type='followup' children, fallback billing | gotchas.md |
| AFG2 | (company_id, delay) unique constraint | gotchas.md |
| AFG20 | v19 type='followup' REMOVED | gotchas.md |
| AFG21 | aml.followup_line_id assigned via _set_followup_line_on_unreconciled_amls | gotchas.md |
| AFG22 | _update_next_followup_action_date assigns level by date | gotchas.md |
| AFG23 | unreconciled_aml_ids One2many with readonly=False | gotchas.md |
| AFG24 | _included_unreconciled_aml_max_followup uses amount_residual for "most_delayed" | gotchas.md |
| AFG25 | report falls back through 3 candidates | gotchas.md |
| AFG26 | Followup PDF rendered with partner.lang context | gotchas.md |
| AFG27 | Manual reminder default attachment_ids = ALL unreconciled main attachments | gotchas.md |
| AFG28 | Activity scheduled on partner, not invoice | gotchas.md |
| AFG29 | activity_type_id default to "_default_activity_type" | gotchas.md |
| AFG3 | (company_id, name) unique constraint | gotchas.md |
| AFG30 | additional_follower_ids subscribes USERS to PARTNER | gotchas.md |
| AFG31 | v19 _sql_constraints replaced by models.Constraint | gotchas.md |
| AFG32 | v19 send_email label changed to "Email" | gotchas.md |
| AFG33 | v19 type='followup' removed entirely | gotchas.md |
| AFG34 | v19 _get_followup_responsible signature breaks | gotchas.md |
| AFG35 | v18 has NO per-line block flag | gotchas.md |
| AFG35A | `_get_unreconciled_aml_domain` incluye `liability_payable` (vendor bills). Si tu localización mete p | integrations.md |
| AFG36 | v19 ADDS account.move.line.no_followup block flag | gotchas.md |
| AFG36A | El SQL filtra `aml.company_id = ANY(%(company_ids)s)` con `company_ids = child_of env.company`, no ` | integrations.md |
| AFG37 | amount_residual SUM in status query is pre-blocking in v18 | gotchas.md |
| AFG37A | `_set_followup_line_on_unreconciled_amls` (inverse del campo computed): si el usuario edita `followu | integrations.md |
| AFG38 | _included_unreconciled_aml_max_followup skips AMLs without date_maturity | gotchas.md |
| AFG38A | El grupo `followup_overdue` en `_read_group` SOLO funciona si `date_maturity IS NOT NULL`. AML sin f | integrations.md |
| AFG39 | only AMLs in env.company.parent_ids tree drive the level | gotchas.md |
| AFG39A | `_compute_unpaid_invoices` usa `commercial_partner_id`, mientras `_compute_total_due` usa `partner_i | integrations.md |
| AFG4 | auto_execute and create_activity mutually exclusive | gotchas.md |
| AFG40A | `invoice_origin` related en AML añade JOIN extra en cualquier read masivo de `account.move.line` aun | integrations.md |
| AFG41L | `copy_data` añade | integrations.md |
| AFG42L | Onchange `_onchange_auto_execute` solo se dispara en UI form. Crear vía ORM con `auto_execute=True`  | integrations.md |
| AFG43L | `_get_next_date()` con un solo nivel devuelve `today + line.delay` (siempre el mismo intervalo). Si  | integrations.md |
| AFG44L | `_get_next_followup` y `_get_previous_followup` filtran por `env.company.id` (no `parent_of`). En en | integrations.md |
| AFG45L | `additional_follower_ids` añade usuarios como followers del partner | integrations.md |
| AFG46L | `_check_company_auto=True` en la línea valida company contra `mail_template_id`/`sms_template_id`. P | integrations.md |
| AFG47L | `name` field es `translate=True` pero `(company_id, name)` UNIQUE constraint es | integrations.md |
| AFG48P | `_get_followup_responsible` filtra por `u.active`. Si todos los candidatos están archivados → cae a  | integrations.md |
| AFG49P | `followup_responsible_id` con `company_dependent=True` Y `tracking=True`: tracking dispara mensajes  | integrations.md |
| AFG5 | delay=0 is valid (= reminder ON due date) | gotchas.md |
| AFG50P | `_compute_followup_status` depende SOLO de `unreconciled_aml_ids` y `followup_next_action_date`. Si  | integrations.md |
| AFG51P | `type='followup'` en child contacts: ese tipo NO se valida contra ningún esquema. Es un valor "marca | integrations.md |
| AFG52P | `_get_all_followup_contacts` fallback: si NO hay child con `type='followup'` y `address_get(['invoic | integrations.md |
| AFG53P | Inverse `_set_followup_line_on_unreconciled_amls` aplica el nivel | integrations.md |
| AFG54P | `_query_followup_data(all_partners=True)` cachea en `env.cr.cache['res_partner_all_followup']`. La c | integrations.md |
| AFG55M | `subtype_id=mail.mt_note` (note interna) en lugar de `mt_comment`: los recordatorios NO disparan aut | integrations.md |
| AFG56M | `_get_email_from` resuelve `{{ object._get_followup_responsible().email_formatted }}` en runtime. Si | integrations.md |
| AFG57M | `lang` context se setea ANTES de leer template.body_html: `template.with_context(lang=partner.lang o | integrations.md |
| AFG58M | `mail_notify_author=True` (v18 vía context) hace que el responsible reciba copia del propio email en | integrations.md |
| AFG59M | `_get_email_reply_to`: si `mail_template.reply_to=False` → cae a `_notify_get_reply_to()[False]` = c | integrations.md |
| AFG6 | cron processes max 1000 partners per company per run | gotchas.md |
| AFG60M | `_get_main_body` fallback a `default_body` (plaintext con `\n`) si no hay `mail_template`. Concatena | integrations.md |
| AFG61M | Adjuntos: `attachment_ids` mezcla report PDF (siempre) + facturas (si `join_invoices=True`) + templa | integrations.md |
| AFG62PT | Bug latente en `_show_pay_now_button`: `payment.method.search_count([('active','=','True')])` compar | integrations.md |
| AFG63PT | Botón "Pay now" se renderiza en TODOS los emails (todos los niveles) si `_show_pay_now_button()=True | integrations.md |
| AFG64PT | El link `/my/invoices/overdue` NO incluye partner-specific access token. Si el partner NO está logue | integrations.md |
| AFG65PT | `account_payment` es CE, no Enterprise. Si la instalación NO tiene `account_payment` instalado (raro | integrations.md |
| AFG66PT | `enable_portal_payment` se lee con `sudo()`. Si un usuario portal ha perdido permisos al system para | integrations.md |
| AFG67CRM | `activity_default_responsible_type='salesperson'` resuelve a `most_delayed_aml.move_id.invoice_user_ | integrations.md |
| AFG68CRM | `auto_execute=True` + `create_activity=True` están excluidos por `onchange`, pero solo en UI. Crear  | integrations.md |
| AFG69CRM | Activity creada NO se cierra automáticamente al pagar la factura. Si followup pasa de `in_need_of_ac | integrations.md |
| AFG7 | cron name xml_id is misleading | gotchas.md |
| AFG70CRM | No existe `mail.activity_type` predefinido para followup. `activity_type_id=False` cae a `_default_a | integrations.md |
| AFG71CRM | `partner.user_id` se usa como "salesperson general". Es el campo `user_id` estándar de `res.partner` | integrations.md |
| AFG72CRM | `_get_followup_responsible` no devuelve `False`/empty: cae a `super()` (que en partner base NO exist | integrations.md |
| AFG73MC | Asimetría direccional | integrations.md |
| AFG74MC | Cache `res_partner_all_followup` NO se invalida automáticamente entre companies. `_cron_execute_foll | integrations.md |
| AFG75MC | `followup_next_action_date` company_dependent: pausar followup en company A NO pausa en B. Si el par | integrations.md |
| AFG76MC | `followup_responsible_id` company_dependent: cambio en company A se logea en chatter del partner per | integrations.md |
| AFG77MC | El cron itera `res.company.search([])` SIN filtro. Companies archivadas (`active=False`) NO entran ( | integrations.md |
| AFG78MC | `_get_followup_report` usa `forced_companies = child_of allowed_company_ids`. Si no hay `allowed_com | integrations.md |
| AFG79MC | `(company_id, delay)` UNIQUE constraint es por company directa, no `parent_of`. Padre y hijo pueden  | integrations.md |
| AFG8 | Cron iterates ALL companies regardless of allowed_company_ids | gotchas.md |
| AFG80C | `xml_id="ir_cron_auto_post_draft_entry"` es copy-paste leftover, NO tiene relación con post de asien | integrations.md |
| AFG81C | Hard limit `[:1000]` por company por run. Empresa con 5000 partners overdue → solo se procesan 1000  | integrations.md |
| AFG82C | `UserError` capturada Y logged como warning (NO error). Errores recurrentes (partner sin email persi | integrations.md |
| AFG83C | Otras excepciones (no UserError) NO capturadas: `AccessError`, `ValidationError`, `KeyError` en temp | integrations.md |
| AFG84C | `_cron_execute_followup_company` NO commitea per-partner. Toda la batch va en una sola transacción.  | integrations.md |
| AFG85C | `_query_followup_data` se ejecuta dentro del cron CON el cache helper, pero `_execute_followup_partn | integrations.md |
| AFG86C | `nextcall` con hora hardcoded 02:00 server TZ. Empresa con servidor en UTC pero usuarios en America/ | integrations.md |
| AFG87S | Iteración `for to_send_partner in followup_contacts`: SMS se envía a TODOS los followup contacts con | integrations.md |
| AFG88S | `_message_sms(partner_ids=partner.ids, sms_pid_to_number={partner.id: sms_number})`: el `partner_ids | integrations.md |
| AFG89S | `UserError` si ningún contact tiene número Y line.send_sms=True. En cron, esto se captura y logea wa | integrations.md |
| AFG9 | Followup query cache shared per cursor, NOT per company | gotchas.md |
| AFG90S | `_get_sms_body` fallback default body: `"Dear client, we kindly remind you that you still have unpai | integrations.md |
| AFG91S | SMS template lang resolution: `template.with_context(lang=partner.lang or env.user.lang).body`. Si l | integrations.md |
| AFG92S | `sms_pid_to_number` mapea SOLO `partner.id` → number. Si quieres enviar a un child contact distinto, | integrations.md |
| AFG93S | En v19: `sms_number = to_send_partner.phone` (mobile field removido del modelo `res.partner` unifica | integrations.md |

## enterprise/account_iso20022

| ID | Título | Fichero |
|----|--------|---------|
| ISOCAP1 | ¿Generar fichero ISO20022/SEPA (pain.001)? | capabilities.md |
| ISOCAP10 | ¿Propósito / referencia estructurada del pago? | capabilities.md |
| ISOCAP2 | ¿Lote de pagos? | capabilities.md |
| ISOCAP3 | ¿SEPA Credit Transfer como método? | capabilities.md |
| ISOCAP4 | ¿Varias versiones de pain.001? | capabilities.md |
| ISOCAP5 | ¿Validación IBAN/BIC? | capabilities.md |
| ISOCAP6 | ¿Descargar el XML? | capabilities.md |
| ISOCAP7 | ¿Por diario bancario de la compañía? | capabilities.md |
| ISOCAP8 | ¿Multi-divisa en el fichero? | capabilities.md |
| ISOCAP9 | ¿Domiciliación / adeudo SEPA (SDD)? | capabilities.md |
| ISOG1 | `sepa_ct` desaparece del wizard de pago si la moneda no es SEPA-elegible | gotchas.md |
| ISOG10 | Cron cierra mandatos a los 36 meses de la última colección | gotchas.md |
| ISOG11 | `pain.008.001.08` exige dirección estructurada y ciudad/país | gotchas.md |
| ISOG12 | `one_off` mandato se cierra automáticamente tras el primer pago | gotchas.md |
| ISOG13 | `EndToEndId` truncado a 30 caracteres (no 35) | gotchas.md |
| ISOG14 | `iso20022_uetr` solo se genera para el code `iso20022`, no para `sepa_ct` | gotchas.md |
| ISOG15 | `iso20022_ch` con `iso20022_ch_force_sepa` reescribe el code por divisa | gotchas.md |
| ISOG16 | Charge bearer: `sepa_ct` ignora el valor del batch (siempre SLEV) | gotchas.md |
| ISOG17 | `sdd_creditor_identifier` obligatorio y validado con Mod-97; sin él no hay SDD | gotchas.md |
| ISOG18 | No se puede borrar una `res.partner.bank` ligada a mandato activo | gotchas.md |
| ISOG19 | `NbOfTxs` limitado a 15 dígitos (no al nº real de transacciones) | gotchas.md |
| ISOG2 | `iso20022` genérico se auto-excluye de companies en EUR al instalar | gotchas.md |
| ISOG20 | Initiating party name se fija una vez en `create` y no se recomputa | gotchas.md |
| ISOG21 | `EndToEndId` en SCT nunca usa `end_to_end_id`: siempre `PmtInfId+id` | gotchas.md |
| ISOG22 | `MsgId` = `str(time.time())`: dos ficheros en el mismo segundo pueden colisionar | gotchas.md |
| ISOG23 | El nombre del acreedor (`Cdtr/Nm`) prioriza `acc_holder_name` sobre el partner | gotchas.md |
| ISOG24 | Pagos de nómina (`sepa_payroll_sala`) cambian el XML: prioridad HIGH + CtgyPurp SALA | gotchas.md |
| ISOG25 | `LclInstrm` en ISO20022 genérico solo aparece con el config param `local_instrument_code` | gotchas.md |
| ISOG26 | Empleados sin país: ValidationError en `get_postal_address`, no UserError silencioso | gotchas.md |
| ISOG3 | No hay validación XSD del fichero generado | gotchas.md |
| ISOG4 | Sin BIC se emite `NOTPROVIDED` en lugar de fallar | gotchas.md |
| ISOG5 | La versión pain por defecto cambió a 09 y puede romper bancos legacy | gotchas.md |
| ISOG6 | Una `PmtInf` por (fecha, divisa): un batch mixto explota en bloques | gotchas.md |
| ISOG7 | `required_payment_date = max(payment_date, today)`: fechas pasadas se silencian | gotchas.md |
| ISOG8 | SDD `SeqTp` siempre `RCUR`, nunca FRST/OOFF/FNAL | gotchas.md |
| ISOG9 | Mandato `revoked` bloquea el XML; `closed` no | gotchas.md |

## enterprise/account_loans

| ID | Título | Fichero |
|----|--------|---------|
| LOANG1 | el préstamo NO tiene cron propio — depende del cron del core `account` | gotchas.md |
| LOANG10 | `is_wrong_date` exige que `loan.date` sea <= primera línea | gotchas.md |
| LOANG11 | corto/largo plazo se calcula por posición (12 líneas), NO por fecha real | gotchas.md |
| LOANG12 | `outstanding_balance` de la línea es TEÓRICO, el del loan es REAL | gotchas.md |
| LOANG13 | `payment`, `long_term_/short_term_theoretical_balance` están stored "para pivot" | gotchas.md |
| LOANG14 | cancelar o borrar usa `_unlink_or_reverse()` — moses posteados se REVIERTEN, no se borran | gotchas.md |
| LOANG15 | tras reconfirmar con audit trail, hay varios payment moves por línea | gotchas.md |
| LOANG16 | borrar el loan hace `ondelete='cascade'` desde el move generado | gotchas.md |
| LOANG17 | `expense_account_id` no admite cuentas de tipo `income` ni `asset` por domain | gotchas.md |
| LOANG18 | el vínculo préstamo↔activos es indirecto vía `asset_group_id` | gotchas.md |
| LOANG19 | `journal_id` restringido a `type='general'` | gotchas.md |
| LOANG2 | el asiento de pago se fecha siempre a FIN DE MES, no en `line.date` | gotchas.md |
| LOANG20 | `loan_properties` depende de la definición en el diario | gotchas.md |
| LOANG21 | el wizard de cierre sólo borra moves DRAFT posteriores a la fecha — los posteados quedan | gotchas.md |
| LOANG22 | `loan_type` lo decide la presencia de interés, no el usuario | gotchas.md |
| LOANG23 | `first_payment_date` por defecto es el día 1 del mes siguiente, y sólo se usa con `at_anniversary` | gotchas.md |
| LOANG24 | la reclasificación LT→ST y su reversión NO heredan analítica ni tax_tags | gotchas.md |
| LOANG25 | `_compute_is_wrong_date` tiene la dependencia INCOMPLETA — no recomputa al editar la fecha de una lí | gotchas.md |
| LOANG26 | re-ejecutar el wizard de cálculo APILA líneas, no las reemplaza | gotchas.md |
| LOANG27 | `loan_term` del wizard está en AÑOS, no en meses ni en nº de cuotas | gotchas.md |
| LOANG28 | `interest_difference` se guarda CON signo (sin `abs`), a diferencia de `amount_borrowed_difference` | gotchas.md |
| LOANG3 | el signo del asiento de pago es contraintuitivo (debe en deuda LP) | gotchas.md |
| LOANG4 | reclasificación LT→ST sólo cubre 12 meses fijos y vive 1 solo día | gotchas.md |
| LOANG5 | la reversión NO usa `_reverse_moves()` — se crea a mano | gotchas.md |
| LOANG6 | la última línea NO genera reclasificación | gotchas.md |
| LOANG7 | `skip_until_date` salta líneas SILENCIOSAMENTE y afecta al saldo pendiente | gotchas.md |
| LOANG8 | el cierre automático cuenta las líneas skipped como pagadas | gotchas.md |
| LOANG9 | confirmación bloqueada si el cuadro no cuadra al céntimo con las cabeceras | gotchas.md |

## enterprise/account_reports

| ID | Título | Fichero |
|----|--------|---------|
| ARG1 | `sum_if_pos` / `sum_if_neg` evaluates the sign on the | gotchas.md |
| ARG10 | `_compute_expression_totals_for_each_column_group` iterates over ALL column groups independently. Wi | gotchas.md |
| ARG100EV | Engine `external` NO soporta groupby, limit, ni offset (`if current_groupby or next_groupby or offse | integrations.md |
| ARG101EV | `most_recent` con figure_type='string' devuelve `text_value`; con figure_type numérico devuelve `val | integrations.md |
| ARG102EV | `foreign_vat_fiscal_position_id` en external.value: debe coincidir con `target_report_expression_id. | integrations.md |
| ARG103EV | Carryover external_values se ondelete=cascade vía expression. Si borras la expression (config update | integrations.md |
| ARG104EV | Multi-company carryover: `_generate_carryover_external_values` split por company. Si tax_unit cambia | integrations.md |
| ARG105EV | Editable external values (`subformula='editable'`): el UPDATE/CREATE usa `date_to` del column_group_ | integrations.md |
| ARG106EV | `name` field es required pero el engine NO lo usa (solo para list view display). Auto-generación al  | integrations.md |
| ARG11 | The `NO_NEXT_GROUPBY_ENGINES` set (`tax_tags`, `account_codes`) forces `next_groupby=None`. This ena | gotchas.md |
| ARG12 | `unfold_all` with many lines triggers `_fully_unfold_lines_if_needed`, which iterates and recursivel | gotchas.md |
| ARG13 | The currency table is created as a temporary table (`account_currency_table`) for multi-currency rep | gotchas.md |
| ARG14 | `prefix_groups_threshold` (default: 4000) triggers automatic line grouping by name prefix when a rep | gotchas.md |
| ARG15 | Tax tag matching uses `SUBSTRING(tag_name, 2)` to strip the `+`/`-` prefix, then groups by the remai | gotchas.md |
| ARG16 | `tax_tag_invert` on AML is an often-overlooked field. It flips the sign of the tag contribution. Thi | gotchas.md |
| ARG17 | Tax closing (`_generate_carryover_external_values`) handles multi-company by: | gotchas.md |
| ARG18 | Fiscal position filtering on tax reports uses an OR between the fiscal position domain AND a tax-tag | gotchas.md |
| ARG19 | `filter_multi_company='tax_units'` does NOT simply aggregate all companies. It uses `account.tax.uni | gotchas.md |
| ARG2 | `account_codes` D/C suffix checks the | gotchas.md |
| ARG20 | The currency table supports three modes: `monocurrency` (no conversion), `current` (latest rate at r | gotchas.md |
| ARG21 | `_get_sender_company_for_export()` determines the "main" company for exports. In a tax unit, it's th | gotchas.md |
| ARG22 | The `is_opening_report` key in previous_options resets the journal filter to the first journal group | gotchas.md |
| ARG23 | `_get_options_journals_domain()` returns an empty list (no filter) when NO journals are specifically | gotchas.md |
| ARG24 | `forced_domain` in options is additive | gotchas.md |
| ARG25 | Options comparison's `period_order` can be `'ascending'` or `'descending'`. When ascending, column h | gotchas.md |
| ARG26 | Budget columns use a | gotchas.md |
| ARG27 | Line IDs are strings with `\|` as hierarchy delimiter and `~` as field delimiter within each segment. | gotchas.md |
| ARG28 | `_create_hierarchy()` only processes lines whose last segment has `model='account.account'`. Lines f | gotchas.md |
| ARG29 | Totals below sections (`_add_totals_below_sections`) are first gated by `self.env.company.totals_bel | gotchas.md |
| ARG3 | Aggregation engine evaluates **last | gotchas.md |
| ARG30 | PDF export uses `export_mode='print'` which changes behavior: unfold_all becomes True (unless specif | gotchas.md |
| ARG31 | XLSX export calls `_filter_out_folded_children()` which removes children of folded lines. If you wan | gotchas.md |
| ARG32 | `_format_column_values()` skips columns that already have a `name` key. Custom handlers can pre-form | gotchas.md |
| ARG33 | The `show_debug_column` option requires FOUR conditions together (`account_report.py → _init_options | gotchas.md |
| ARG34 | `integer_rounding_enabled` defaults to True in options but is only active when `self.integer_roundin | gotchas.md |
| ARG35 | Report annotations are stored with the | gotchas.md |
| ARG36 | `get_options()` performs a reroute check after computing `report_id`. If `report_id != self.id`, it  | gotchas.md |
| ARG37 | The `no_report_reroute` key in options prevents the reroute and forces options to be computed for th | gotchas.md |
| ARG38 | In the `account_codes` engine, `D`/`C` are reserved as debit/credit total selectors placed right aft | gotchas.md |
| ARG38A | `sum_if_pos`/`sum_if_neg` evalúan el signo del | integrations.md |
| ARG39 | `currency_translation` (the report field, CE `account/models/account_report.py`) only offers two val | gotchas.md |
| ARG39A | Batching de domains: si todos los términos del domain filtran por UN solo M2O field de AML, varias e | integrations.md |
| ARG4 | `cross_report` subformula forces the aggregation to evaluate expressions from OTHER reports using th | gotchas.md |
| ARG40 | The `external` engine's `most_recent` formula returns the value of the | gotchas.md |
| ARG40A | `count_rows` cuenta `COUNT(DISTINCT next_groupby_first_field)`. Si la línea NO tiene groupby → cuent | integrations.md |
| ARG41 | `sum_children` (aggregation engine) is resolved by expanding it into `_expression:<child_expr_id>` t | gotchas.md |
| ARG41A | `display_type IN ('line_section','line_note')` EXCLUIDAS en todo query (`_get_options_domain`:2090). | integrations.md |
| ARG42A | Engine `domain` con term `('account_id.account_type', '=', 'asset_receivable')`: el batching detecta | integrations.md |
| ARG43A | `parent_state='posted'` por defecto: borradores excluidos siempre salvo `all_entries=True` (toggle " | integrations.md |
| ARG44A | Budget shadow table (`account_report_budget_temp_aml`): hereda DDL de `account_move_line` pero | integrations.md |
| ARG45A | `exclude_bank_lines` (campo computed stored) se calcula vía `account_id != journal_id.default_accoun | integrations.md |
| ARG46A | Constraint `_check_taxes_on_closing_entries`: bloquea añadir `tax_ids`/`tax_tag_ids` a AMLs de moves | integrations.md |
| ARG47AN | Filtro `analytic_accounts` NO afecta el cómputo del Balance Sheet ni P&L estándar | integrations.md |
| ARG48AN | El operator `'in'` sobre `analytic_distribution` (JSON) en domain | integrations.md |
| ARG49AN | Si activas `filter_analytic=True` pero el usuario NO tiene `analytic.group_analytic_accounting` → op | integrations.md |
| ARG5 | The `_default` prefix on expression labels (e.g., `_default_balance`) is special: these expressions  | gotchas.md |
| ARG50AN | Analytic distribution con porcentajes (`{plan_acc_id: 60.0, plan_acc_id_2: 40.0}`) NO se aplica como | integrations.md |
| ARG51AN | Multi-plan analytic (v17+): un AML puede tener múltiples planes simultáneos. El filtro `analytic_acc | integrations.md |
| ARG52AN | `account_analytic_report` (handler) opera sobre `account.analytic.line` (no AML). Esto significa que | integrations.md |
| ARG53T | Engine `tax_tags` NO soporta subformula. Si pones `subformula='sum_if_pos'` → se ignora silenciosame | integrations.md |
| ARG54T | Cambiar el `country_id` de un report con expresiones tax_tags existentes → tags antiguas (con countr | integrations.md |
| ARG55T | Una tag puede vivir en `account.account` (applicability='accounts') Y en taxes; pero el engine `tax_ | integrations.md |
| ARG56T | `tax_tag_invert` se computa pero NO siempre se recalcula al cambiar tax_repartition. Reformar un mov | integrations.md |
| ARG57T | SQL del engine usa `SUBSTRING(name, 2)` para quitar el ±. Si alguien crea tag con name `++casilla_01 | integrations.md |
| ARG58T | Tax tags NO respetan `parent_state='posted'` por sí solas | integrations.md |
| ARG59T | Multi-VAT (fiscal_position): el filtro tax_tag se aplica con `('tax_tag_ids.country_id', '=', report | integrations.md |
| ARG6 | `_carryover_` and `_applied_carryover_` prefixed expression labels are special. They trigger carryov | gotchas.md |
| ARG60MC | `filter_multi_company='disabled'` no significa "ignora multi-company" | integrations.md |
| ARG61MC | `tax_units` filtra companies "filtradas". Si user no tiene acceso a TODAS las companies de la unit ( | integrations.md |
| ARG62MC | Currency conversion en multi-company opera vía `currency_table` (ver bridge 8). Si companies tienen  | integrations.md |
| ARG63MC | Branches: `_accessible_branches()` retorna company + descendientes activos. Si tienes branch archive | integrations.md |
| ARG64MC | Cambiar `tax_unit` en UI dispara hack JS que recarga companies (`A js hack ensures the page is reloa | integrations.md |
| ARG65MC | Export de tax report en multi-company con varias VAT (branches con `vat` distinto): export buttons d | integrations.md |
| ARG66MC | Multi-company + journals filter: opciones `available_journals` se calculan POR company. Si un journa | integrations.md |
| ARG67AU | Engine `account_codes` con sufijo `D`/`C` NO puede traducirse a domain AML (depende del signo del TO | integrations.md |
| ARG68AU | `aggregation` con `cross_report` usa `expression.date_scope` (la principal) y NO el de las subexpres | integrations.md |
| ARG69AU | `_get_audit_line_groupby_domain` parsea el `line_dict_id` para extraer groupby keys (`partner_id`, ` | integrations.md |
| ARG7 | `_currency_` prefixed expression labels (e.g., `_currency_balance`) are NOT a forced-currency mechan | gotchas.md |
| ARG70AU | Custom audit action (`column.custom_audit_action_id`) reemplaza el action default pero NO elimina el | integrations.md |
| ARG71AU | Audit de columna con `figure_type='string'` (text external value) abre list de `account.report.exter | integrations.md |
| ARG72AU | El domain del audit incluye `_get_options_domain(column_group_options, date_scope)` que aplica `pare | integrations.md |
| ARG73V | El reroute via `_init_options_report_id` (sequence=17) ocurre TARDE en la secuencia (después de comp | integrations.md |
| ARG74V | `_is_available_for(options)` filtra variants por country (default) | integrations.md |
| ARG75V | Sections con sub-reports de countries distintos: el PDF/XLSX export consolida pero el filtro `fiscal | integrations.md |
| ARG76V | `with_context(active_test=False)` SE USA al cargar variants para mostrar incluso archived. Pero `_is | integrations.md |
| ARG77V | Sections heredan `selected_section_id` SOLO si está en `available_sections.ids`. Si cambias de varia | integrations.md |
| ARG78V | `section_report_ids` es M2M pero el orden depende del `sequence` de cada sub-report (no de la M2M or | integrations.md |
| ARG79RT | Cierre VAT requiere `tax_group.tax_receivable_account_id` Y `tax_group.tax_payable_account_id` confi | integrations.md |
| ARG8 | The `domain` engine batches queries when all domain terms reference a single M2O field on AML. If yo | gotchas.md |
| ARG80RT | `_get_vat_closing_entry_additional_domain()` es hook para override por l10n_*. Sin override devuelve | integrations.md |
| ARG81RT | Reabrir un closing move ya posted (cancel + edit + repost): SQL `_compute_vat_closing_entry` se re-e | integrations.md |
| ARG82RT | Multi-company tax_closing: si `len(env.companies) > 1` y no se pasa `override_tax_closing_warning=Tr | integrations.md |
| ARG83RT | Periodo del cierre = `company._get_tax_closing_period_boundaries(date_to, report)`. Si el user filtr | integrations.md |
| ARG84RT | `repartition_line.use_in_tax_closing=False` por defecto en algunas taxes (e.g. reverse charge en lín | integrations.md |
| ARG85RT | v18 → v19 migration: `account.return` nuevo en v19 reemplaza la lógica embebida de v18. Modules cust | integrations.md |
| ARG86RT | Carryover en tax reports (Modelo 303 casilla 110 → 87 del siguiente periodo): se gestiona via `accou | integrations.md |
| ARG87FX | `currency_translation='cta'` usa LEFT JOIN | integrations.md |
| ARG88FX | Rate type por account_type: la rule `account_type LIKE 'equity%'` matchea TODOS los equity (equity,  | integrations.md |
| ARG89FX | Multicurrency Revaluation Report requiere `len(active_currencies) >= 2` (incluyendo la de la company | integrations.md |
| ARG9 | `count_rows` subformula always forces non-batched execution. If one expression on a line uses `count | gotchas.md |
| ARG90FX | El user puede override rates en options (`options['currency_rates']` mutable). Si modifica rate en U | integrations.md |
| ARG91FX | Comparison period (e.g. Year-over-Year): cada col_group tiene su propio period en `currency_table`.  | integrations.md |
| ARG92FX | `no_impact_on_currency_table=True` en col_group (típico para Initial Balance del Trial Balance): exc | integrations.md |
| ARG93EX | `export_mode='print'` fuerza `unfold_all=True` para PDF/XLSX. Reports con miles de líneas (e.g. Part | integrations.md |
| ARG94EX | PDF rendering usa wkhtmltopdf con `landscape=True` si `len(columns)>5 or horizontal_split`. Pero det | integrations.md |
| ARG95EX | `_get_pdf_export_html` rinde QWeb template `account_reports.pdf_export_main`. Custom handlers que ov | integrations.md |
| ARG96EX | XLSX export inyecta una "Options" sheet con filtros aplicados (`_add_options_xlsx_sheet:6426`). Si t | integrations.md |
| ARG97EX | Multi-section export combina PDFs vía `_merge_pdfs` (PyPDF). Si una sub-section da error en wkhtmlto | integrations.md |
| ARG98EX | Branches+VAT: export buttons mostradas según `branch_allowed`. PDF/XLSX siempre `branch_allowed=True | integrations.md |
| ARG99EX | `account.report.send` wizard (email + print) usa `_get_report_send_recipients` del custom handler. S | integrations.md |

## enterprise/account_saft

| ID | Título | Fichero |
|----|--------|---------|
| SAFTG1 | España NO usa SAF-T (no existe `l10n_es_saft`) | gotchas.md |
| SAFTG10 | Partners sin Zip/City bloquean o se omiten (salvo `saft_allow_empty_address`) | gotchas.md |
| SAFTG11 | La company DEBE tener al menos un contacto hijo | gotchas.md |
| SAFTG12 | `CustomerID`/`SupplierID` usan el ID interno de Odoo, no un código fiscal | gotchas.md |
| SAFTG13 | `partner_detail_map['type']` (singular) es legacy; usar `types` (lista) | gotchas.md |
| SAFTG14 | Líneas con debe=0 Y haber=0 igualmente emiten un importe según move_type | gotchas.md |
| SAFTG15 | `rate` (tipo de cambio) se calcula como abs(amount_currency)/abs(balance), 1.0 si balance=0 | gotchas.md |
| SAFTG16 | RO/AT/PL añaden campos y modelos propios | gotchas.md |
| SAFTG17 | El detalle de impuestos descarta líneas donde el signo de cuota ≠ signo de base | gotchas.md |
| SAFTG18 | `tax_base_amount` y `amount` salen de la query de tax details, NO del campo `tax_base_amount` de la  | gotchas.md |
| SAFTG19 | El nombre de la línea se sintetiza desde el producto si falta, y el `sign` por move puede sorprender | gotchas.md |
| SAFTG2 | El módulo base no genera nada por sí solo | gotchas.md |
| SAFTG20 | El `sign` del bloque `TaxInformation` se decide por línea (debe/haber), no por move | gotchas.md |
| SAFTG21 | `FirstName` siempre se emite como literal `"NotUsed"` | gotchas.md |
| SAFTG22 | `CustomerID`/`SupplierID` a nivel línea se decide con `type` (singular legacy), a nivel transacción  | gotchas.md |
| SAFTG23 | RO: NO es "un botón SAF-T", son DOS exports con secciones distintas (D.406 mensual vs activos) | gotchas.md |
| SAFTG24 | RO: el export valida MUCHO más que NO/LT (taxes, products, partners, company) y bloquea | gotchas.md |
| SAFTG25 | RO: las líneas con `balance == 0` se ELIMINAN del XML (al contrario que el base) | gotchas.md |
| SAFTG26 | AT: cada cuenta debe mapear a EXACTAMENTE un `l10n_at_saft.account` vía un tag numérico | gotchas.md |
| SAFTG27 | LT: `_saft_get_account_type` revienta con KeyError ante un `account_type` no mapeado | gotchas.md |
| SAFTG28 | Import: el wizard solo entiende el namespace genérico `urn:StandardAuditFile-Taxation-Financial` | gotchas.md |
| SAFTG29 | Import: dedup por claves "naturales" → reimportar NO duplica, pero cambios de nombre SÍ | gotchas.md |
| SAFTG3 | SAF-T reutiliza el report del Libro Mayor, no tiene report propio | gotchas.md |
| SAFTG30 | Import: fija `sequence.mixin.constraint_start_date` global para tragar fechas históricas | gotchas.md |
| SAFTG31 | Import opcional de saldos de apertura crea un asiento de AJUSTE, no replica los OpeningBalance | gotchas.md |
| SAFTG4 | Solo un column group permitido | gotchas.md |
| SAFTG5 | Las queries son SQL crudo, no ORM (record rules NO se aplican igual) | gotchas.md |
| SAFTG6 | `xmlns`/`file_version`/`accounting_basis` los pone la localización, no el base | gotchas.md |
| SAFTG7 | `accounting_basis` (`TaxAccountingBasis`) NO significa lo mismo en cada país | gotchas.md |
| SAFTG8 | Norway exige el código de impuesto estándar dentro del NOMBRE del impuesto | gotchas.md |
| SAFTG9 | Norway: warning de upgrade si falta `l10n_no_saft.balance_account` | gotchas.md |

## enterprise/appointment

| ID | Título | Fichero |
|----|--------|---------|
| APG1 | Slot start times use `appointment_tz`, not `tz_offset` of the visitor | gotchas.md |
| APG10 | `linked_resource_ids` is symmetric across two M2M tables | gotchas.md |
| APG100 | `capacity` SQL constraint requires `>= 1`, but `capacity_reserved` only `>= 0` | gotchas.md |
| APG11 | Resource `shareable=True` means same slot can be reserved by multiple attendees with separate bookin | gotchas.md |
| APG12 | `staff_user_ids` domain `[('share','=',False)]` excludes portal users | gotchas.md |
| APG13 | v18 has 3 assign methods (`resource_time`/`time_resource`/`time_auto_assign`); v19 collapses to 2 (` | gotchas.md |
| APG14 | v18 `appointment_manual_confirmation=True` AND v19 `auto_confirm=False` are inverse defaults | gotchas.md |
| APG15 | `appointment.invite.short_code` is validated against `^[\w-]+$` | gotchas.md |
| APG16 | Filter `staff_user_ids`/`resource_ids` on the invite must be SUBSETS of the type's pool | gotchas.md |
| APG17 | `appointment.invite` has many2many to multiple types but no priority order | gotchas.md |
| APG18 | `calendar.event.access_token` is generated lazily for OLD events | gotchas.md |
| APG19 | Two separate computes: `_compute_videocall_redirection` builds the URL, `_compute_videocall_source`  | gotchas.md |
| APG2 | `min_schedule_hours` is added to the FIRST slot only when reference_date is "near now" | gotchas.md |
| APG20 | Reminders only inherit from type at event CREATION | gotchas.md |
| APG21 | `appointment.type` has no `company_id` | gotchas.md |
| APG22 | `_slots_fill_users_availability` reads staff calendar events via `_slot_availability_prepare_users_v | gotchas.md |
| APG23 | `_get_appointment_slots` builds the entire month grid in Python (no SQL aggregation) | gotchas.md |
| APG24 | v19 introduces `appointment.question.is_default` + `is_reusable` + `appointment_type_ids` (M2M) | gotchas.md |
| APG25 | v19 splits frontend assets from `js/` into `interactions/` | gotchas.md |
| APG26P | Booking collision: capacidad puede cambiar entre form submit y pago confirmado. `_filter_unavailable | integrations.md |
| APG27P | SQL constraint: answer input DEBE estar linked a `calendar_booking_id` O `calendar_event_id` (no amb | integrations.md |
| APG28P | `calendar.booking` NO reserva capacidad real. La reserva ocurre solo al crear `calendar.event` post- | integrations.md |
| APG29P | Conversión booking→event requiere invoice en estado `payment_state='paid'`. Pago pending/timeout → b | integrations.md |
| APG3 | Custom appointments rely on `slot.end_datetime > reference_date`, not `start_datetime` | gotchas.md |
| APG30P | `partner_id` en booking sin `required=True` pero usado en `_make_invoice_from_booking()`. Si null →  | integrations.md |
| APG31P | `has_payment_step=True` sin `product_id` → constraint error en save. `_compute_product_id` busca def | integrations.md |
| APG32P | Refund de booking pagado NO está implementado. Cancel invoice (`in_refund`) NO elimina event, NO lib | integrations.md |
| APG33P | Guest IDs en booking se PIERDEN al convertir a event. `_make_event_from_paid_booking` usa solo `part | integrations.md |
| APG34C | Lead se crea SOLO si `partner_ids - user_id.partner_id` NO vacío. Internal user + otro staff user =  | integrations.md |
| APG35C | Lead se crea en company del | integrations.md |
| APG36C | Lead.name = `event.name` directo, | integrations.md |
| APG37C | GDPR risk: `_get_attendee_description()` mete emails + names de attendees en `lead.description` (ful | integrations.md |
| APG38C | Multi-partner event → solo | integrations.md |
| APG39C | `appointment.invite.opportunity_id` pre-linked propaga a `event.opportunity_id`. Si AMBOS activos (p | integrations.md |
| APG4 | `recurring` slots are filtered by `byweekday=int(weekday)-1` | gotchas.md |
| APG40C | `_compute_lead_ids` costoso: `_read_group` en calendar.event + joins crm.lead. 1000+ events → comput | integrations.md |
| APG41C | Booking pagado en pending state NO crea lead (event no existe aún). Si pago timeout → booking huérfa | integrations.md |
| APG42S | Visitor sin `phone` field → SMS falla silenciosamente. Form muestra hint "We will use it to remind y | integrations.md |
| APG43S | SMS via cron (delay 30min típico) vs email reminders inmediato. SMS llega tarde si cron interval gra | integrations.md |
| APG44S | SMS gateway restricciones por país (provider-dependent). Visitor de país no soportado → SMS falla si | integrations.md |
| APG45S | SMS template language hardcoded o por `company.locale`, NO por `visitor.lang`. Visitor ES con compan | integrations.md |
| APG46S | Anonymous partner (guest booking sin login) → `event.partner_id.mobile=NULL` → SMS no se puede envia | integrations.md |
| APG47S | Multiple SMS alarms por event (1h+1day before). Cron procesa cada uno independiente. Cron crash mid- | integrations.md |
| APG48S | GDPR: NO hay `sms_opt_out` ni consentimiento explícito en appointment_sms. SMS enviado sin opt-in. M | integrations.md |
| APG49W | Domain `'\|', (website_id, '=', current_website), (website_id, '=', False)`. Appointment con `website | integrations.md |
| APG5 | Setting `active=False` auto-cancels the appointment, and vice-versa | gotchas.md |
| APG50W | Operator selection view skip si CUALQUIERA: `not active OR schedule_based_on != 'users' OR avatars_d | integrations.md |
| APG51W | Country detection fallback chain: `geoip > user.partner.country > visitor.country`. VPN → geoip stal | integrations.md |
| APG52W | `appointment_tz` es `required=True` con `default=lambda self: self.env.user.tz` | integrations.md |
| APG53W | `appointment.invite` multi-website warning: si `invite.website_id != type.website_id` → `appointment | integrations.md |
| APG54W | `copy_data()` override fuerza `is_published=False` en duplicado. Admin duplica published → copia unp | integrations.md |
| APG55W | SEO search indexing: appointments private (con invite_token) se INDEXAN igual via `_search_get_detai | integrations.md |
| APG56W | Videocall location depende de `request.website.domain` al construir URL. Si admin cambia website dom | integrations.md |
| APG57H | `resource_calendar_id.flexible_hours=True` | integrations.md |
| APG58H | `on_leave_partner_ids` requiere `invalidate_recordset()` para actualizar. Cambios a `resource.calend | integrations.md |
| APG59H | `_work_intervals_batch(resources=[])` con resources vacío causa `KeyError`. Si `employee.resource_id | integrations.md |
| APG6 | `appointment_status` cascades only when the type is set/changed | gotchas.md |
| APG60H | Multi-company employees: busca employee en | integrations.md |
| APG61H | `timezone_datetime()` convierte sin validar tz válida. `start/stop` en UTC pero calendar en tz difer | integrations.md |
| APG62H | `work_hours_activated` field INVISIBLE si `schedule_based_on='resources'` (no users). Admin no puede | integrations.md |
| APG63HR | `applicant_id` propaga SOLO si viene en | integrations.md |
| APG64HR | `appointment.invite.applicant_id` NO crea `hr.applicant` automáticamente. Solo linkea si ya existe.  | integrations.md |
| APG65HR | Answers de appointment NO se copian a `applicant.description` automáticamente. Interview question "s | integrations.md |
| APG66HR | Borrar `appointment.invite` → `calendar.event.applicant_id` se vuelve null (related field). Cascade  | integrations.md |
| APG67GC | `is_google_calendar_synced()` chequea EXISTENCIA del token, NO refresh status. Token expirado pero p | integrations.md |
| APG68GC | `videocall_redirection` (Discuss fallback) se genera PRE-sync con `access_token`. `videocall_locatio | integrations.md |
| APG69GC | Google rechaza `conferenceData.createRequest` si user sin permiso → `hangoutLink=None`. Evento cread | integrations.md |
| APG7 | `_send_invitation_emails()` only fires when status TRANSITIONS to `booked` | gotchas.md |
| APG70GC | Cambiar `event_videocall_source` de `google_meet` a otro (o vacío) NO limpia Google Meet links de ev | integrations.md |
| APG71GC | `users_wo_google_calendar_msg` es INFO solo, NO bloquea booking. Staff users sin Google sync → appoi | integrations.md |
| APG72GR | `idempotency_token` = `access_token`. Mismo token → Odoo retorna event existente (sin duplicate). Si | integrations.md |
| APG73GR | `assign_method='time_auto_assign'` FORZADO al activar google_reserve. Si cambias a resource-based de | integrations.md |
| APG74GR | CRON `ir_cron_iap_google_reserve_availabilities` con `interval_number=9999 months` — | integrations.md |
| APG75GR | CORREGIDO — el sumatorio NO está donde decía. | integrations.md |
| APG76GR | Party size logic en `_google_reserve_format_slot_availabilities_resources()` IGNORA `linked_resource | integrations.md |
| APG77GR | `google_reserve_pending_sync=True` se marca pero CRON puede fallar silenciosamente (sin retry automá | integrations.md |
| APG78GR | Borrar `calendar.event` Odoo creado vía Google Reserve NO notifica Google. Solo actualiza al recibir | integrations.md |
| APG79MC | `check_synchronization_status()['microsoft_calendar']` puede retornar `'missing_credentials'` aunque | integrations.md |
| APG8 | Cancellation via portal token does NOT delete; it archives via `action_cancel_meeting`→`action_archi | gotchas.md |
| APG80MC | Sync NO es bidireccional para appointment-specific fields. Modificar `appointment.answer.input` en O | integrations.md |
| APG81MC | `microsoft_calendar_sync_paused` global pausa | integrations.md |
| APG82MC | Multi-tenant Microsoft: staff users de tenants diferentes en mismo appointment type → solo tenant do | integrations.md |
| APG83 | `end_hour=0` means "next day midnight" (24:00), not "00:00 same day" | gotchas.md |
| APG84 | `_compute_end_hour` auto-shifts `end_hour` when it's below `start_hour` | gotchas.md |
| APG85 | `_check_delta_hours` requires the slot window to fit at least ONE full `appointment_duration` | gotchas.md |
| APG86 | `_compute_restrict_to_user_ids` silently drops users no longer in `staff_user_ids` | gotchas.md |
| APG87 | `slot_type` is computed from category, not directly settable | gotchas.md |
| APG88 | `capacity_used >= capacity_reserved` is a hard SQL constraint, not just a compute | gotchas.md |
| APG89 | Resource `sequence` decides combination priority, lowest first | gotchas.md |
| APG9 | `capacity_used` ≠ `capacity_reserved` when resource is not shareable | gotchas.md |
| APG90 | `appointment.resource.company_id` defaults to `False` (no company) | gotchas.md |
| APG91 | `capacity_reserved=0` zeroes `capacity_used` — a "ghost" booking line | gotchas.md |
| APG92 | Portal cancel is blocked by `min_cancellation_hours`, measured from NOW against event start | gotchas.md |
| APG93 | `_check_appointment_is_valid_slot` re-runs full slot generation on every booking submit (sudo) | gotchas.md |
| APG94 | `allow_guests` lets attendees add arbitrary guest partners — but guests are dropped on paid-booking  | gotchas.md |
| APG95 | `_compute_category` only decides `recurring` vs `punctual`; `custom`/`anytime` come from the inverse | gotchas.md |
| APG96 | `appointment.invite` autovacuums itself after 6 months | gotchas.md |
| APG97 | `short_code` auto-dedupes on default, but a manual duplicate only WARNS until save | gotchas.md |
| APG98 | `appointment.booking.line._check_resources_match_appointment_type` blocks resources not in the type' | gotchas.md |
| APG99 | `appointment.resource` forces `resource_type='material'` on the underlying `resource.resource` | gotchas.md |
| APPTCAP1 | ¿Tipos de cita configurables? | capabilities.md |
| APPTCAP10 | ¿Recordatorios y confirmación automática? | capabilities.md |
| APPTCAP11 | ¿Pago al reservar la cita? | capabilities.md |
| APPTCAP12 | ¿Sincronización con calendario externo (Google/Microsoft)? | capabilities.md |
| APPTCAP13 | ¿Enlaces de invitación compartibles (varios tipos / operador)? | capabilities.md |
| APPTCAP14 | ¿Generar lead/oportunidad CRM desde la cita? | capabilities.md |
| APPTCAP15 | ¿Recordatorios / avisos por SMS? | capabilities.md |
| APPTCAP16 | ¿Reserva desde Google (Reserve with Google)? | capabilities.md |
| APPTCAP2 | ¿Disponibilidad por usuario según calendario y eventos? | capabilities.md |
| APPTCAP3 | ¿Citas con recursos (salas/mesas), no solo usuarios? | capabilities.md |
| APPTCAP4 | ¿Reserva pública desde web sin login? | capabilities.md |
| APPTCAP5 | ¿Capacidad por slot / reservas múltiples? | capabilities.md |
| APPTCAP6 | ¿Antelación mínima y ventana máxima de reserva? | capabilities.md |
| APPTCAP7 | ¿Preguntas personalizadas al reservar? | capabilities.md |
| APPTCAP8 | ¿Asignación del staff: elegido vs automático? | capabilities.md |
| APPTCAP9 | ¿Enlace de videollamada al confirmar? | capabilities.md |

## enterprise/approvals

| ID | Título | Fichero |
|----|--------|---------|
| APPG1 | `_compute_approver_ids` borra y recrea TODO | gotchas.md |
| APPG10 | `action_confirm` con `approver_sequence` y `approval_minimum=0` | gotchas.md |
| APPG11 | `manager_approval='required'` sin parent_id bloquea | gotchas.md |
| APPG12 | Búsqueda de employee toma primero por company | gotchas.md |
| APPG13 | `precompute=True` en product_uom_id | gotchas.md |
| APPG14 | Activities: cancel vs approve/refuse comportamiento diferente | gotchas.md |
| APPG15 | `has_access_to_request` súper restrictivo | gotchas.md |
| APPG16 | `copy()` lanza UserError en `approval.approver` | gotchas.md |
| APPG17 | Resubmit tras withdraw activa "reanudación" silenciosa | gotchas.md |
| APPG18 | `change_request_owner` cambia subscripción | gotchas.md |
| APPG19 | `kanban_view` con `group_expand=True` | gotchas.md |
| APPG2 | Manager siempre `sequence=9` | gotchas.md |
| APPG20 | `automated_sequence` cambia ir.sequence al editar | gotchas.md |
| APPG21 | `attachments_required` no es campo, es `requirer_document` | gotchas.md |
| APPG22 | Domain de approver `user_id` excluye existing pero por context | gotchas.md |
| APPG23 | Doble botón "Submit" gated por `attachment_number` | gotchas.md |
| APPG24 | Approve invisible si eres owner sin adjunto | gotchas.md |
| APPG25 | `action_cancel` botón exige ser owner | gotchas.md |
| APPG26 | `action_withdraw` botón requiere `user_status` no vacío | gotchas.md |
| APPG27 | `_mail_post_access = 'read'` | gotchas.md |
| APPG28 | Record rule "request approver rule" incluye al jefe del owner | gotchas.md |
| APPG29 | Unlink del owner solo en `new`/`cancel`, y vía `function` se restringe más | gotchas.md |
| APPG3 | Manager duplicado se pop silenciosamente | gotchas.md |
| APPG30 | Approver unlink permitido en `new/pending/cancel` (no en terminal) | gotchas.md |
| APPG31 | `group_approval_user` NO es admin: config solo `group_approval_manager` | gotchas.md |
| APPG32 | `request_owner_id` domain excluye portal pero por `env.companies` | gotchas.md |
| APPG33 | `attachment_ids` filtra por `res_field IS NULL` implícito vía `attachment_number` | gotchas.md |
| APPG34 | `ir.attachment` no se puede borrar en estado terminal | gotchas.md |
| APPG4 | `required=True` puede bloquear request "para siempre" | gotchas.md |
| APPG5 | `cancel` tiene prioridad sobre `refused` | gotchas.md |
| APPG6 | `minimal_approver` tolerancia silenciosa | gotchas.md |
| APPG7 | `status='waiting'` solo existe con `approver_sequence=True` | gotchas.md |
| APPG8 | `user_status` devuelve `''` (string vacío) si no eres approver | gotchas.md |
| APPG9 | `_check_approver_ids` también valida duplicados | gotchas.md |
| APPP1 | `action_create_purchase_orders` es manual, no automático | gotchas.md |
| APPP10 | `action_cancel` NO cancela POs ya creadas | gotchas.md |
| APPP11 | `_domain_product_id` requiere context | gotchas.md |
| APPP12 | `action_approve` valida product_id pero no seller | gotchas.md |
| APPP13 | UoM conversion `po_uom_qty` | gotchas.md |
| APPP14 | Re-cancel: PO en draft NO se borra | gotchas.md |
| APPP15 | `action_confirm` (purchase) bloquea request vacío | gotchas.md |
| APPP16 | `currency_id` de la PO sale de `seller_id.currency_id`, no del producto | gotchas.md |
| APPP17 | `_select_seller` se llama con `uom_id=product.uom_po_id` | gotchas.md |
| APPP18 | `_compute_seller_id` tiene fallback a "cualquier vendor" | gotchas.md |
| APPP2 | Doble-click en "Create RFQ's" no duplica | gotchas.md |
| APPP3 | Edit de línea TRAS approve sigue siendo posible | gotchas.md |
| APPP4 | Agrupación PO por (company, vendor, currency, state=draft) | gotchas.md |
| APPP5 | Si encuentra PO existente, mergea o crea línea según UoM | gotchas.md |
| APPP6 | `origin` se appendea con `", "` y dedupe | gotchas.md |
| APPP7 | `search(po_domain)` sin limit; usa `purchase_orders[0]` | gotchas.md |
| APPP8 | `_select_seller` puede cambiar el vendor según qty | gotchas.md |
| APPP9 | `has_no_seller=True` y decoración roja en list | gotchas.md |
| APPRCAP1 | ¿Tipos de aprobación configurables? | capabilities.md |
| APPRCAP10 | ¿Estados del flujo y quién puede qué? | capabilities.md |
| APPRCAP11 | ¿Líneas de producto en la solicitud (artículos + cantidades)? | capabilities.md |
| APPRCAP12 | ¿Aprobador automático = responsable/manager del empleado? | capabilities.md |
| APPRCAP13 | ¿Una solicitud aprobada genera pedido de compra? | capabilities.md |
| APPRCAP14 | ¿Recepción de stock tras aprobar la compra? | capabilities.md |
| APPRCAP15 | ¿Aprobar documentos estándar (PO/gasto/factura) o solo su propio modelo? | capabilities.md |
| APPRCAP2 | ¿Mínimo de aprobaciones + múltiples aprobadores? | capabilities.md |
| APPRCAP3 | ¿Aprobadores por usuario o por grupo/rol? | capabilities.md |
| APPRCAP4 | ¿Flujo secuencial vs paralelo? | capabilities.md |
| APPRCAP5 | ¿Campos activables por tipo (importe, fecha, partner, producto, ubicación…)? | capabilities.md |
| APPRCAP6 | ¿Adjuntar documentos/justificantes (obligatorios)? | capabilities.md |
| APPRCAP7 | ¿Una aprobación genera un pedido de compra? | capabilities.md |
| APPRCAP8 | ¿Notificaciones/actividades automáticas a los aprobadores? | capabilities.md |
| APPRCAP9 | ¿Routing condicional por umbral de importe? | capabilities.md |
| APPS1 | `warehouse_id` solo visible con multi-warehouse | gotchas.md |
| APPS2 | `hide_location` oculta el campo en form | gotchas.md |
| APPS3 | Mismo producto, distintos warehouses → 2 POs | gotchas.md |
| APPS4 | Stock module NO genera pickings al aprobar | gotchas.md |
| APPS5 | `warehouse_id` sin `in_type_id` → PO sin picking_type | gotchas.md |

## enterprise/bank_statement_import

| ID | Título | Fichero |
|----|--------|---------|
| BSICAP1 | ¿Importar extractos en varios formatos? | capabilities.md |
| BSICAP10 | ¿Conciliación tras importar? | capabilities.md |
| BSICAP2 | ¿Mapeo de columnas para CSV? | capabilities.md |
| BSICAP3 | ¿Crear statement + líneas automáticamente? | capabilities.md |
| BSICAP4 | ¿Detección de duplicados? | capabilities.md |
| BSICAP5 | ¿Detectar el diario por la cuenta del fichero? | capabilities.md |
| BSICAP6 | ¿Multi-divisa en las líneas? | capabilities.md |
| BSICAP7 | ¿Importar saldo inicial/final? | capabilities.md |
| BSICAP8 | ¿OCR de extracto en PDF/imagen? | capabilities.md |
| BSICAP9 | ¿Sincronización bancaria automática? | capabilities.md |
| BSIG1 | `unique_import_id` is silently namespaced; re-importing into a different journal duplicates lines | gotchas.md |
| BSIG10 | OFX ledger balance ≠ ending balance | gotchas.md |
| BSIG11 | OFX/QIF partner matching is by payee NAME string, exact match | gotchas.md |
| BSIG12 | QIF date format defaults to day-first (dd/mm/yy), not the QIF spec month-first | gotchas.md |
| BSIG13 | OFX needs the `ofxparse` python lib or import dies at runtime | gotchas.md |
| BSIG14 | There is no import wizard menu; entry is the journal dashboard uploader | gotchas.md |
| BSIG15 | Importing requires `default_account_id` on the journal, else UserError mid-pipeline | gotchas.md |
| BSIG16 | Partial import failures commit the good statements then raise RedirectWarning | gotchas.md |
| BSIG17 | OCR only triggers for pure PDF/JPEG/PNG batches; mixing types is "rejected" (but via a bugged `retur | gotchas.md |
| BSIG18 | OCR auto-send is ON by default and sends scans to Odoo's IAP cloud | gotchas.md |
| BSIG19 | Imported statement gets an auto-generated PDF attachment (v18 only) | gotchas.md |
| BSIG2 | "You already have imported that file" can fire even on a partly-new file | gotchas.md |
| BSIG20 | `set_bank_account` auto-creates a res.partner.bank from the statement's account number | gotchas.md |
| BSIG21 | Norma 43 is not importable on stock Enterprise | gotchas.md |
| BSIG22 | CAMT credit/debit sign is "CRDT or it's negative" | gotchas.md |
| BSIG23 | CAMT raises a hard ValidationError if it can't derive an FX rate for a foreign-currency entry | gotchas.md |
| BSIG24 | CAMT folds bank charges into the line amount (no separate charge line) | gotchas.md |
| BSIG25 | CAMT `unique_import_id` from `AcctSvcrRef` is rejected if all-zeros or `NOTPROVIDED`, falling back t | gotchas.md |
| BSIG26 | `_check_camt` only matches the literal substring `'camt.053'`; camt.052/054 fall through to "install | gotchas.md |
| BSIG27 | CAMT statement currency is sniffed from the FIRST of `Acct/Ccy` OR `Bal/Amt/@Ccy`; a missing account | gotchas.md |
| BSIG28 | CAMT `payment_ref` defaults to the literal `'/'` when no remittance/info is found | gotchas.md |
| BSIG29 | CAMT counter-party party is taken from the OPPOSITE side of the credit/debit indicator | gotchas.md |
| BSIG3 | Lines with `amount == 0` are dropped, not imported | gotchas.md |
| BSIG30 | CAMT re-reads the entry gross sign independently for the rounding fix-up | gotchas.md |
| BSIG31 | OFX v2 sniff is `b"<ofx>" in raw.lower()` ANYWHERE in the file (very loose) | gotchas.md |
| BSIG32 | QIF resets all accumulated transactions on a bare newline marker | gotchas.md |
| BSIG33 | QIF balance_end_real is the SUM of amounts, and there is no balance_start | gotchas.md |
| BSIG34 | `_complete_bank_statement_vals` sets statement `reference` to the attachment filename, and that beco | gotchas.md |
| BSIG35 | CSV/XLS/XLSX share the bank-import path; "CSV" import actually accepts Excel too | gotchas.md |
| BSIG4 | Skipped (already-imported) lines mutate `balance_start` | gotchas.md |
| BSIG5 | CAMT multi-IBAN file silently drops the other accounts | gotchas.md |
| BSIG6 | CAMT requires journal currency to match the statement currency or the whole Stmt is skipped | gotchas.md |
| BSIG7 | CSV rows MUST be date-sorted or the import hard-fails | gotchas.md |
| BSIG8 | CSV needs `amount` XOR (`debit` AND `credit`); wrong combo = ValidationError | gotchas.md |
| BSIG9 | CSV import has NO dedup and NO multicurrency (by design) | gotchas.md |

## enterprise/data_tools

| ID | Título | Fichero |
|----|--------|---------|
| DTG1 | [EE-Merge] No existe módulo `data_merge` independiente | gotchas.md |
| DTG10 | [EE-Clean] `trim_all` quita TODOS los espacios, también los internos | gotchas.md |
| DTG11 | [EE-Clean] la acción phone se procesa en Python (lento), no en SQL | gotchas.md |
| DTG12 | [EE-Clean] el país del teléfono se infiere del registro, no del número | gotchas.md |
| DTG13 | [EE-Clean] solo campos char/text/html; m2o/selection no se limpian | gotchas.md |
| DTG14 | [EE-Merge] la similitud NO usa las reglas de match | gotchas.md |
| DTG15 | [EE-Merge] similitud = exact-match de campos char, no fuzzy | gotchas.md |
| DTG16 | [EE-Merge] match_mode `accent` requiere extensión PostgreSQL `unaccent` | gotchas.md |
| DTG17 | [EE-Merge] merge automático irreversible y silencioso | gotchas.md |
| DTG18 | [EE-Merge] el master genérico es el registro MÁS ANTIGUO | gotchas.md |
| DTG19 | [EE-Merge] res.partner y crm.lead NO usan el merge genérico | gotchas.md |
| DTG2 | [EE-Clean][EE-Merge] data_cleaning auto-instala data_recycle (CE) | gotchas.md |
| DTG20 | [EE-Merge] crm.lead: no se pueden fusionar oportunidades ganadas | gotchas.md |
| DTG21 | [EE-Merge] el atributo de clase "_prevent_merge" bloquea la dedup en un modelo | gotchas.md |
| DTG22 | [EE-Merge] cross-company off por defecto: no agrupa entre companies | gotchas.md |
| DTG23 | [EE-Merge] `create_threshold` borra grupos por debajo del umbral | gotchas.md |
| DTG24 | [EE-Merge] reapuntado de FKs tolera fallos silenciosamente | gotchas.md |
| DTG25 | [EE-Merge] `used_in` (References) puede ser caro de computar | gotchas.md |
| DTG26 | [EE-Merge] `merge_common_lists` agrupa transitivamente (puede sobre-agrupar) | gotchas.md |
| DTG27 | [EE-Merge] subconjuntos no crean grupo nuevo | gotchas.md |
| DTG28 | [CE-Recycle][EE-Clean][EE-Merge] todo es admin-only (group_system) | gotchas.md |
| DTG29 | [EE-Clean][EE-Merge] candidatos "discard" se reactivan en el siguiente barrido | gotchas.md |
| DTG3 | [CE-Recycle] recycle_action default es `unlink` (BORRA, no archiva) | gotchas.md |
| DTG30 | [EE-Clean] el motor de cleaning solo toca registros activos | gotchas.md |
| DTG31 | [EE-Merge] _log_merge solo si el modelo es mail.thread | gotchas.md |
| DTG32 | [EE-Merge] contextual merge crea data_merge.model inactivas ocultas | gotchas.md |
| DTG33 | [CE-Recycle] el filtro temporal usa `now` del servidor según ttype | gotchas.md |
| DTG34 | [EE-Merge] el reapuntado de FKs M2M NO está protegido por savepoint (a diferencia de las FKs normale | gotchas.md |
| DTG35 | [EE-Merge] con `removal_mode='archive'` pero modelo SIN campo active, `_post_merge` BORRA igualmente | gotchas.md |
| DTG36 | [EE-Merge] el master debe EXISTIR o el merge se aborta en silencio (return sin error) | gotchas.md |
| DTG37 | [EE-Merge] `_log_merge` solo escribe la nota de archivado en el source si `removal_mode == 'archive' | gotchas.md |
| DTG38 | [EE-Merge] crm.lead define `_elect_master` (typo): NUNCA se llama, gana el genérico | gotchas.md |
| DTG39 | [EE-Merge] `_disable_data_merge` también en mailing.list y account.analytic.line (no solo partner/le | gotchas.md |
| DTG4 | [CE-Recycle] recycle_mode `automatic` ejecuta sin revisión humana | gotchas.md |
| DTG40 | [EE-Merge] res.partner `_elect_method` prioriza ARCHIVADOS al final, no por "más completo" | gotchas.md |
| DTG41 | [EE-Merge] res.partner merge devuelve `post_merge=False`: el archivado/borrado lo hace el wizard, no | gotchas.md |
| DTG42 | [EE-Merge] crm.lead merge devuelve `log_chatter=False`: sin snapshot data_merge en el chatter | gotchas.md |
| DTG43 | [EE-Merge] `merge_records` SIEMPRE borra los data_merge.record al final, aunque el merge real falle | gotchas.md |
| DTG44 | [EE-Merge] `data_merge.record.create` valida que el registro original EXISTA (con active_test implíc | gotchas.md |
| DTG45 | [CE-Recycle][EE-Clean] `_send_notification` notifica al AUTOR (`notify_author=True`) y por defecto s | gotchas.md |
| DTG5 | [CE-Recycle] `archive` falla si el modelo no tiene `active` | gotchas.md |
| DTG6 | [CE-Recycle] desactivar la regla BORRA todos sus candidatos pendientes | gotchas.md |
| DTG7 | [CE-Recycle] el filtro temporal exige campo date/datetime store | gotchas.md |
| DTG8 | [CE-Recycle] include_archived no es por defecto | gotchas.md |
| DTG9 | [EE-Clean] Format Phone requiere el paquete Python `phonenumbers` | gotchas.md |

## enterprise/documents

| ID | Título | Fichero |
|----|--------|---------|
| DOCAP1 | ¿Capturar email entrante como documento (alias → workspace)? | capabilities.md |
| DOCAP10 | ¿Generar un documento desde un registro de Odoo (adjuntar informe a un workspace)? | capabilities.md |
| DOCAP11 | ¿Convertir un PDF en factura de proveedor directamente? | capabilities.md |
| DOCAP12 | ¿OCR/digitalización automática del PDF para rellenar la factura? | capabilities.md |
| DOCAP13 | ¿Pedir firma electrónica desde un documento del workspace? | capabilities.md |
| DOCAP14 | ¿Crear una hoja de cálculo nativa dentro de Documents? | capabilities.md |
| DOCAP15 | ¿Vincular documentos a proyecto/tareas, con workspace automático? | capabilities.md |
| DOCAP16 | ¿Documents integrado con Field Service (FSM)? | capabilities.md |
| DOCAP2 | ¿Acciones/flujos automáticos sobre documentos (workspace actions)? | capabilities.md |
| DOCAP3 | ¿Versionado de ficheros (subir nueva versión con histórico)? | capabilities.md |
| DOCAP4 | ¿Solicitar un documento a alguien (request/placeholder a subir)? | capabilities.md |
| DOCAP5 | ¿Carpetas/workspaces anidados? | capabilities.md |
| DOCAP6 | ¿Tags/facetas y reglas automáticas de etiquetado? | capabilities.md |
| DOCAP7 | ¿Buscar dentro del contenido del documento (full-text)? | capabilities.md |
| DOCAP8 | ¿Split / merge de PDF desde la interfaz? | capabilities.md |
| DOCAP9 | ¿Dónde se almacenan los ficheros y hay limpieza automática? | capabilities.md |
| DOG1 | `owner_id` ALWAYS gets `edit`, even if `access_via_link='none'` | gotchas.md |
| DOG10 | `active=False` is the trash; physical delete after `documents.deletion_delay` days (default 30) | gotchas.md |
| DOG11 | Deleting a record using `documents.unlink.mixin` keeps the document, just retargets to itself | gotchas.md |
| DOG12 | Folder deletion cascades to child documents only via `_parent_store` | gotchas.md |
| DOG13 | `shortcut_document_id` ondelete='cascade' deletes the shortcut when the source is unlinked | gotchas.md |
| DOG14 | Shortcut name falls back to source name if not set | gotchas.md |
| DOG15 | Only NON-shortcut folders can have `alias_name` | gotchas.md |
| DOG16 | `message_new()` creates ONE document per attachment | gotchas.md |
| DOG17 | `document_message_new=True` context bypasses `no_document=True` guard | gotchas.md |
| DOG18 | `_compute_name_and_preview` fetches OG metadata synchronously over HTTP | gotchas.md |
| DOG19 | URL preview cached as raw image URL, not stored | gotchas.md |
| DOG2 | `access_internal='view'` is COMPANY-WIDE | gotchas.md |
| DOG20 | `previous_attachment_ids` grows unbounded | gotchas.md |
| DOG21 | Replacing the attachment OVERWRITES `name` with the new file's name | gotchas.md |
| DOG22 | `documents.tag.name` UNIQUE across ALL companies | gotchas.md |
| DOG23 | `_unlink_except_used_in_server_action` only checks tags by external_id | gotchas.md |
| DOG24 | `request_activity_id` is M2O on the document, not the activity | gotchas.md |
| DOG25 | Requestee can be a non-internal partner | gotchas.md |
| DOG26 | Changing a folder's `company_id` CASCADES via raw SQL to every descendant document and its shortcuts | gotchas.md |
| DOG27 | Admin (group_documents_system) sees documents from companies in `user.company_ids` even if archived | gotchas.md |
| DOG28 | `_compute_user_permission` is `compute_sudo=True` BUT `depends_context=('uid', 'allowed_company_ids' | gotchas.md |
| DOG29 | Search panel iterates folder tree to build counts | gotchas.md |
| DOG3 | `access_via_link` propagates from parent folder unless `is_access_via_link_hidden=True` | gotchas.md |
| DOG30 | v19 introduces `documents.access.tracking` cron-batched audit | gotchas.md |
| DOG31 | v19 wizards `documents.operation` and `documents.sharing` replace v18 inline actions | gotchas.md |
| DOG32 | v19 `ir.embedded.actions` pinned to a folder require write access on that folder | gotchas.md |
| DOG33 | OCR no vive en Documents — sin créditos IAP la factura se crea VACÍA y sin error | gotchas.md |
| DOG34 | Carpeta→diario es UNIQUE por diario | gotchas.md |
| DOG35 | Documentos firmados se libran del autovacuum de papelera | gotchas.md |
| DOG36 | Firma directa exige UN solo documento | gotchas.md |
| DOG37 | Borrar todos los proyectos de una carpeta la ARCHIVA, no la borra | gotchas.md |
| DOG38 | Carpeta-destino de un bridge debe ser de la misma compañía (o ninguna) | gotchas.md |
| DOG39 | Si archivas/borras la carpeta-destino de un bridge, los adjuntos dejan de convertirse en documentos  | gotchas.md |
| DOG4 | `documents.access` rows allow `role=NULL` if `last_access_date IS NOT NULL` | gotchas.md |
| DOG40 | El ACL de una hoja de cálculo es el de Documents, no el del editor | gotchas.md |
| DOG41 | `documents_fsm` NO integra Documents en FSM — lo DESACTIVA | gotchas.md |
| DOG42 | `owner_id` por defecto es el USUARIO ACTUAL, no OdooBot | gotchas.md |
| DOG43 | Cambiar el `owner_id` MANTIENE al dueño anterior como editor explícito | gotchas.md |
| DOG44 | Solo el dueño (o `manager`) puede cambiar el owner; no-managers no pueden re-asignar docs ajenos | gotchas.md |
| DOG45 | Un usuario portal NO puede ser dueño de un documento raíz | gotchas.md |
| DOG46 | Borrar un documento de tipo carpeta usado por una app falla con ValidationError | gotchas.md |
| DOG47 | Archivar/borrar un doc que NO posees exige `edit` en la CARPETA, no en el doc | gotchas.md |
| DOG48 | `documents.access` es INMUTABLE en `partner_id`/`document_id` — y se auto-purga al expirar | gotchas.md |
| DOG49 | La invitación con alta de usuario solo funciona con política global "Free sign up" (b2c) | gotchas.md |
| DOG5 | Cross-company documents hide silently when company is not in `env.companies` | gotchas.md |
| DOG50 | Los shortcuts a un destino inaccesible se AUTO-BORRAN (no solo se marcan) | gotchas.md |
| DOG51 | La papelera (kanban) usa `is_pinned_folder`, un campo TODO-remove, en una regla de seguridad | gotchas.md |
| DOG6 | `documents.group_documents_system` (admin) can edit documents from companies they DON'T belong to, I | gotchas.md |
| DOG7 | `access_token` is `document_token + 'o' + hex(id)` | gotchas.md |
| DOG8 | Member sign-up token is HMAC over `(access.id, partner.id)` | gotchas.md |
| DOG9 | `_is_signup_available()` checks `_get_signup_invitation_scope() == 'b2c'` | gotchas.md |

## enterprise/enterprise_tools

| ID | Título | Fichero |
|----|--------|---------|
| ETG1 | Access tokens are security-critical — regenerated on cancel/reassign | gotchas.md |
| ETG10 | Signer reassignment has strict conditions | gotchas.md |
| ETG11 | Phone matching is context-sensitive | gotchas.md |
| ETG12 | User settings auto-created on VoIP field write | gotchas.md |
| ETG13 | SQL injection risk in cleaning actions | gotchas.md |
| ETG14 | Batch commits manual and mode-dependent | gotchas.md |
| ETG15 | Phone formatting requires country context | gotchas.md |
| ETG16 | Similarity only compares char/text fields | gotchas.md |
| ETG17 | FK redirect uses raw SQL — no constraint handling | gotchas.md |
| ETG18 | Requires PostgreSQL unaccent extension | gotchas.md |
| ETG19 | Master election extensible per model | gotchas.md |
| ETG2 | Multi-signer requires explicit ordering | gotchas.md |
| ETG20 | Cross-company dedup controlled by mix_by_company | gotchas.md |
| ETG21 | Sign depends on 'certificate' module in v19 | gotchas.md |
| ETG22 | Sign editor components lazy-loaded in v19 | gotchas.md |
| ETG23 | Document integrity is a hash CHAIN, not a single hash | gotchas.md |
| ETG24 | Reminder cron uses raw SQL interval + `last_reminder` defaulting to today | gotchas.md |
| ETG25 | Frame hash binds signer email to DB uuid | gotchas.md |
| ETG26 | Template attachment `copy_data` does not re-stamp `res_model` | gotchas.md |
| ETG27 | Call activity refresh is bus-driven and category-gated | gotchas.md |
| ETG28 | Settings auto-create ignores explicit company context | gotchas.md |
| ETG29 | Automatic mode applies suggestions to PRODUCTION with no review | gotchas.md |
| ETG3 | PDF attachment auto-copied if shared | gotchas.md |
| ETG30 | FK redirect swallows IntegrityError/UniqueViolation as a WARNING | gotchas.md |
| ETG31 | Merge reassigns attachments/activities/messages with no ownership check | gotchas.md |
| ETG32 | Similarity baseline is the FIRST record, order-dependent | gotchas.md |
| ETG33 | `_notify_no_more_credit` emails creators ONCE, then goes silent | gotchas.md |
| ETG34 | Lead enrichment is auto-triggered on EVERY `crm.lead` create (default ON) | gotchas.md |
| ETG35 | Two DIFFERENT switches govern enrichment auto-mode (config param vs cron.active) | gotchas.md |
| ETG36 | Enrich only fills EMPTY lead fields — never overwrites | gotchas.md |
| ETG37 | Generic email domains are skipped and marked done (credit-saving, but no enrichment) | gotchas.md |
| ETG38 | Enrich credit-exhaustion ABORTS the whole run, not just the failed batch | gotchas.md |
| ETG39 | Enrich commits per batch — partial results survive a later crash | gotchas.md |
| ETG4 | Sign item positions normalized 0-1 | gotchas.md |
| ETG40 | Reveal runs ONLY for public (not-logged-in) visitors and ONLY on 200 page responses | gotchas.md |
| ETG41 | Reveal dedups per-IP for 6 months and per-clearbit_id forever | gotchas.md |
| ETG5 | Signature images are Base64 PNG with transparency fix | gotchas.md |
| ETG6 | SMS authentication is IAP credit-based | gotchas.md |
| ETG7 | Requests auto-expire after 6 months | gotchas.md |
| ETG8 | Shared documents bypass role restrictions | gotchas.md |
| ETG9 | Template tag uniqueness is case-sensitive | gotchas.md |

## enterprise/frontdesk

| ID | Título | Fichero |
|----|--------|---------|
| FDCAP1 | ¿Estaciones de recepción configurables? | capabilities.md |
| FDCAP10 | ¿Planificar visitas con antelación? | capabilities.md |
| FDCAP2 | ¿Registro de visitantes con estados? | capabilities.md |
| FDCAP3 | ¿Quiosco web público para auto check-in? | capabilities.md |
| FDCAP4 | ¿Notificar al anfitrión al llegar? | capabilities.md |
| FDCAP5 | ¿Campos del visitante configurables? | capabilities.md |
| FDCAP6 | ¿Anfitrión (empleados a visitar)? | capabilities.md |
| FDCAP7 | ¿Ofrecer bebida? | capabilities.md |
| FDCAP8 | ¿Personalización visual del quiosco? | capabilities.md |
| FDCAP9 | ¿Check-out / registro de salida? | capabilities.md |
| FDG1 | Las notificaciones se disparan en `write()`, no en los `action_*` | gotchas.md |
| FDG10 | Usuario `frontdesk_group_user` solo ve estaciones donde es `responsible_id` | gotchas.md |
| FDG11 | `company` del visitor es texto libre, no un `res.partner` | gotchas.md |
| FDG12 | No existe impresión de etiqueta (badge) en estándar | gotchas.md |
| FDG13 | El `public_user` se añade a la compañía de la estación en un `onchange` (no en write) | gotchas.md |
| FDG14 | Quick check-in solo lista visitas en ventana de ±45 minutos | gotchas.md |
| FDG15 | Quick check-in filtra por el campo `check_in`, no por una fecha de cita planificada | gotchas.md |
| FDG16 | En `write` masivo, `self.drink_ids` se evalúa sobre TODO el recordset, no por registro | gotchas.md |
| FDG17 | `action_served` solo pone `served=True`; NO dispara el flujo de `write(state=...)` | gotchas.md |
| FDG18 | `duration` es `store=True` con `default=1.0` y solo se recalcula con AMBAS fechas | gotchas.md |
| FDG19 | `host_ids` en el visitor NO está filtrado por la estación; el filtro company solo está en el kiosk | gotchas.md |
| FDG2 | Los recepcionistas (`responsible_ids`) siempre reciben Discuss, ignorando los flags `notify_*` | gotchas.md |
| FDG20 | El `create` en v18 NO notifica; solo el `write` lo hace (asimetría create vs write) | gotchas.md |
| FDG21 | `visitor_properties` se hereda de la estación: cambiar la definición rompe propiedades existentes | gotchas.md |
| FDG22 | `frontdesk.drink.notify_user_ids` es required: no puedes ofrecer una bebida sin "camarero" | gotchas.md |
| FDG23 | `prepare_visitor_data` (walk-in) NO tolera kwargs ausentes: `host_ids` y `drink_ids` deben venir sie | gotchas.md |
| FDG24 | `get_hosts` exige el kwarg `name` (posicional en la firma del endpoint) | gotchas.md |
| FDG25 | `frontdesk.visitor` no tiene `_check_company`/`check_company`: `company_id` y `station_id.company_id | gotchas.md |
| FDG26 | `active=True` por defecto pero no hay archivado en la UI estándar del visitor | gotchas.md |
| FDG3 | Sin `host_selection`, los anfitriones NUNCA se notifican | gotchas.md |
| FDG4 | `notify_email` requiere `work_email`; `notify_sms` requiere `work_phone` | gotchas.md |
| FDG5 | El domain de `mail_template_id`/`sms_template_id` apunta a `frontdesk.frontdesk`, pero las plantilla | gotchas.md |
| FDG6 | El QR móvil (self check-in) caduca a la hora exacta | gotchas.md |
| FDG7 | El kiosk cierra la sesión del usuario (logout) al abrirlo | gotchas.md |
| FDG8 | `prepare_visitor_data` con `drink_ids` NO crea ni hace check-in: solo añade bebidas | gotchas.md |
| FDG9 | `served` se fuerza a True en cada check-out aunque la bebida no se sirviera | gotchas.md |

## enterprise/fsm

| ID | Título | Fichero |
|----|--------|---------|
| FSMCAP1 | ¿Tareas de servicio en campo (intervenciones)? | capabilities.md |
| FSMCAP10 | ¿Cronómetro / registro de tiempo in situ? | capabilities.md |
| FSMCAP11 | ¿Intervenciones recurrentes? | capabilities.md |
| FSMCAP12 | ¿Intervenciones multi-día? | capabilities.md |
| FSMCAP13 | ¿Generar tarea FSM desde venta? | capabilities.md |
| FSMCAP14 | ¿Productos bajo garantía (precio 0)? | capabilities.md |
| FSMCAP15 | ¿Informe/parte PDF de la intervención? | capabilities.md |
| FSMCAP16 | ¿Validar y cerrar desde el móvil? | capabilities.md |
| FSMCAP17 | ¿Stock/furgoneta del técnico? | capabilities.md |
| FSMCAP18 | ¿SLA o prioridades en intervenciones? | capabilities.md |
| FSMCAP2 | ¿App móvil para el técnico? | capabilities.md |
| FSMCAP3 | ¿Partes de trabajo / worksheets configurables? | capabilities.md |
| FSMCAP4 | ¿Añadir materiales/productos a la intervención? | capabilities.md |
| FSMCAP5 | ¿Facturar la intervención (tiempo + materiales)? | capabilities.md |
| FSMCAP6 | ¿Generar pedido de venta desde la tarea? | capabilities.md |
| FSMCAP7 | ¿Planificación / asignación de técnicos? | capabilities.md |
| FSMCAP8 | ¿Geolocalización / mapa? | capabilities.md |
| FSMCAP9 | ¿Firma del cliente en sitio? | capabilities.md |
| FSMG1 | Partner phone is bidirectional and writes to the partner record | gotchas.md |
| FSMG10 | Timer rounding silently inflates timesheets | gotchas.md |
| FSMG11 | Timer collisions across devices fail silently | gotchas.md |
| FSMG12 | Resource overlap is warning-only | gotchas.md |
| FSMG13 | Project archive does not cascade to FSM tasks | gotchas.md |
| FSMG14 | Portal access token regenerates on `copy()` | gotchas.md |
| FSMG15 | `action_fsm_validate` does NOT enforce worksheet/signature in core | gotchas.md |
| FSMG16 | Form view is the report | gotchas.md |
| FSMG17 | Studio field rename breaks PDF until re-save | gotchas.md |
| FSMG18 | Worksheet manual delete leaves PostgreSQL table orphaned | gotchas.md |
| FSMG19 | One worksheet record per host record per template | gotchas.md |
| FSMG2 | Date validation silently clears both `planned_date_begin` and `date_deadline` | gotchas.md |
| FSMG20 | Studio Many2one chains across worksheet models can crash reports | gotchas.md |
| FSMG21 | `x_name` orphaning in v18 | gotchas.md |
| FSMG22 | Task created twice if SO confirms before product worksheet template is set | gotchas.md |
| FSMG23 | Task partner inherits SO | gotchas.md |
| FSMG24 | `under_warranty` only visible when material is enabled | gotchas.md |
| FSMG25 | Locked SO silently rejects material additions | gotchas.md |
| FSMG26 | Multi-company SO confirmed in foreign company | gotchas.md |
| FSMG27 | Pricelist missing on SO falls back to product list price | gotchas.md |
| FSMG28 | Materials auto-marked delivered without `industry_fsm_stock` | gotchas.md |
| FSMG29 | Recurring vs subscription tasks: two mechanisms | gotchas.md |
| FSMG3 | `is_fsm` toggle "hides" tasks rather than archives them | gotchas.md |
| FSMG30 | Employee-rate timesheets unlinked from SOL until fsm_done | gotchas.md |
| FSMG31 | `cancel_backorder=True` silently truncates partial picks | gotchas.md |
| FSMG32 | Cannot decrease quantity below the delivered amount | gotchas.md |
| FSMG33 | Warehouse is the user's default, no per-task override | gotchas.md |
| FSMG34 | 2-step / 3-step delivery + warehouse mismatch cancels moves | gotchas.md |
| FSMG35 | Serial tracking = one SOL per serial | gotchas.md |
| FSMG36 | Material price update after invoice | gotchas.md |
| FSMG37 | `industry_fsm_repair` has no FSM→repair workflow | gotchas.md |
| FSMG38 | PDF empty if worksheet template has no `report_view_id` | gotchas.md |
| FSMG39 | Signature widget broken on iOS Safari < 15 | gotchas.md |
| FSMG4 | `allow_material` cascade auto-enables `allow_billable` | gotchas.md |
| FSMG40 | Map view requires Google Maps API key, navigation button does not | gotchas.md |
| FSMG41 | Map lazy-loaded → first navigation has 1-2 s lag | gotchas.md |
| FSMG42 | SMS uses `partner_id.mobile`, display uses `mobile or phone` | gotchas.md |
| FSMG43 | `_is_fsm_report_available()` requires at least one timesheet | gotchas.md |
| FSMG44 | Portal `worksheet_map` not iterated in portal template | gotchas.md |
| FSMG45 | Browser crash mid-signature loses the canvas | gotchas.md |
| FSMG46 | Worksheet template delete cascades to all linked dynamic models | gotchas.md |
| FSMG47 | Gantt removed in v19 | gotchas.md |
| FSMG48 | Timer running with browser tab closed | gotchas.md |
| FSMG49 | Worksheet signature field excluded from CSV export | gotchas.md |
| FSMG5 | `timesheet_product_id` is required, but the error is SQL-only | gotchas.md |
| FSMG50 | Planning slot deletion orphans FSM tasks | gotchas.md |
| FSMG51 | Helpdesk team `fsm_project_id` silent fallback | gotchas.md |
| FSMG52 | Subscription cron timing creates off-cycle tasks | gotchas.md |
| FSMG53 | `documents_fsm` forces `use_documents=False` on FSM projects | gotchas.md |
| FSMG54 | Sign module separate from `worksheet_signature` | gotchas.md |
| FSMG55 | No SLA propagation from helpdesk to FSM task | gotchas.md |
| FSMG56 | FSM date truncation can wipe gantt schedule on bulk edits | gotchas.md |
| FSMG57 | `from odoo.osv import expression` dead in v19 | gotchas.md |
| FSMG58 | `partner_phone` inverse removed in v19 (field persists) | gotchas.md |
| FSMG59 | Security groups: `users` → `user_ids`, `category_id` → `privilege_id` | gotchas.md |
| FSMG6 | Portal user `partner_id` auto-assignment on create (v18 only) | gotchas.md |
| FSMG60 | `data` files renamed in `industry_fsm_report` | gotchas.md |
| FSMG61 | Close method is public `action_fsm_validate` (no leading underscore) | gotchas.md |
| FSMG62 | Warranty task with no material and no prior SO closes with NO sale order | gotchas.md |
| FSMG63 | No offline mode: FSM is PWA-only, native mobile apps discontinued | gotchas.md |
| FSMG64 | v19: timers FSM sobre `account.analytic.line`, no `project.task` | gotchas.md |
| FSMG65 | v19: `project.task.create.timesheet` ELIMINADO (overrides mueren en silencio) | gotchas.md |
| FSMG66 | v19: ajuste `group_industry_fsm_quotations` eliminado (el grupo NO) [PARTIAL] | gotchas.md |
| FSMG67 | v19: `task_custom_report` (Field Service Report) restringido a `group_fsm_user` | gotchas.md |
| FSMG68 | v19: el redondeo de timers FSM migra a `round_time_spent` (amplía FSMG10) | gotchas.md |
| FSMG69 | v19: `partner_phone` se mueve a `project`, NO desaparece (corrige FSMG58) | gotchas.md |
| FSMG7 | Mark-as-Done buttons filtered while timer runs | gotchas.md |
| FSMG8 | Recurrence copies `partner_phone` as a frozen Char | gotchas.md |
| FSMG9 | `SELF_READABLE_FIELDS` rename | gotchas.md |

## enterprise/helpdesk

| ID | Título | Fichero |
|----|--------|---------|
| HDCAP1 | ¿Políticas de SLA? | capabilities.md |
| HDCAP10 | ¿Valoración del cliente (rating/CSAT)? | capabilities.md |
| HDCAP11 | ¿Fusionar tickets? | capabilities.md |
| HDCAP12 | ¿Devoluciones/reembolsos/reparaciones desde ticket? | capabilities.md |
| HDCAP13 | ¿Live chat → ticket? | capabilities.md |
| HDCAP14 | ¿Respuestas predefinidas (canned)? | capabilities.md |
| HDCAP15 | ¿Generar intervención FSM desde ticket? | capabilities.md |
| HDCAP16 | ¿Informes de rendimiento del equipo? | capabilities.md |
| HDCAP17 | ¿Tickets recurrentes? | capabilities.md |
| HDCAP18 | ¿Portal: el cliente ve/cierra sus tickets? | capabilities.md |
| HDCAP2 | ¿Equipos y etapas configurables? | capabilities.md |
| HDCAP3 | ¿Crear tickets por email? | capabilities.md |
| HDCAP4 | ¿Crear tickets desde portal/web? | capabilities.md |
| HDCAP5 | ¿Asignación automática? | capabilities.md |
| HDCAP6 | ¿Escalado automático? | capabilities.md |
| HDCAP7 | ¿Partes de horas en tickets? | capabilities.md |
| HDCAP8 | ¿Facturar el tiempo del ticket? | capabilities.md |
| HDCAP9 | ¿Base de conocimiento / artículos? | capabilities.md |
| HDG1 | SLA `time` is WORKING hours, not calendar hours | gotchas.md |
| HDG10 | `oldest_unanswered_customer_message_date` is cleared on close | gotchas.md |
| HDG11 | Portal close is metadata, not a security gate | gotchas.md |
| HDG12 | "Balanced" counts open tickets ONLY on the current team | gotchas.md |
| HDG13 | Auto-close measures `write_date`, not last customer message | gotchas.md |
| HDG14 | Auto-close silently no-ops without `to_stage_id` | gotchas.md |
| HDG15 | Stage email template skipped when actor IS the customer (or no `partner_email`) | gotchas.md |
| HDG16 | Response metrics are driven by COMMENTS on OPEN tickets, classified by author share-status | gotchas.md |
| HDG17 | Stage email templates are auto-deleted after send | gotchas.md |
| HDG18 | `resource_calendar_id` is MANDATORY for any meaningful SLA | gotchas.md |
| HDG19 | SLA reach CASCADES to lower-sequence stages | gotchas.md |
| HDG2 | `exclude_stage_ids` freezes SLA time | gotchas.md |
| HDG20 | `success_rate` only counts last 7 days and can read negatively | gotchas.md |
| HDG21 | `helpdesk_stock` / `helpdesk_account` / `helpdesk_repair` are NOT auto_install | gotchas.md |
| HDG22 | "Return" button needs an already-delivered (`done`) partner picking + `product_id` | gotchas.md |
| HDG23 | `helpdesk_sale` does NOT create a sale order | gotchas.md |
| HDG24 | `helpdesk_stock_account` assumes ONE sale line per ticket product | gotchas.md |
| HDG25 | Ticket `lot_id` would contaminate ALL repair component moves | gotchas.md |
| HDG26 | FSM task generation needs `team.fsm_project_id` | gotchas.md |
| HDG27 | Incoming alias drops `partner_id` across companies | gotchas.md |
| HDG28 | A new SLA status on a ticket already in the target stage is born REACHED | gotchas.md |
| HDG29 | `keep_reached` preserves only statuses that have a `reached_datetime` | gotchas.md |
| HDG3 | `sla_reached`, `sla_reached_late`, `sla_fail`, `sla_success` are FOUR different things | gotchas.md |
| HDG30 | SLA deadline depends on the ORIGINAL creation hour, not midnight | gotchas.md |
| HDG31 | Folding the FIRST stage closes tickets at creation | gotchas.md |
| HDG32 | `copy()` keeps the same assignee but inactive users vary by version | gotchas.md |
| HDG33 | `ticket_ref` is per-COMPANY sequence, not global | gotchas.md |
| HDG34 | Auto-close cron is enabled/disabled by team writes, not always running | gotchas.md |
| HDG35 | Working-day filter needs employee calendars, else falls back to COMPANY calendar | gotchas.md |
| HDG36 | Inbound email auto-fills `description` from the email BODY, once | gotchas.md |
| HDG37 | `message_new` can leave `partner_id` empty even with a known sender (multi-company) | gotchas.md |
| HDG38 | Stages are SHARED across teams (M2M), not owned | gotchas.md |
| HDG39 | Archiving a stage archives ALL its tickets | gotchas.md |
| HDG4 | Auto-assignment runs ONLY at create, not on team change | gotchas.md |
| HDG40 | `_mail_get_message_subtypes` hides refund/return/repair subtypes when the flag is off | gotchas.md |
| HDG41 | `sla_success` (the COMPUTE) ignores reached/late — it is just "open with a future deadline" | gotchas.md |
| HDG42 | A brand-new SLA defaults its Target Stage to the first FOLDED (closed) stage | gotchas.md |
| HDG43 | `exclude_stage_ids` (freeze) cannot include the Target Stage | gotchas.md |
| HDG44 | `open_hours` is computed-on-read, never stored, and uses `.seconds` | gotchas.md |
| HDG45 | `partner_id.ticket_count` rolls up child contacts AND archived tickets | gotchas.md |
| HDG46 | `to_stage_id` is a STORED computed field that resets to the first folded stage | gotchas.md |
| HDG47 | The "closed N hours before/after SLA" chatter note prefers the LATEST-breaching SLA | gotchas.md |
| HDG5 | SLA matching requires EXACT priority match | gotchas.md |
| HDG6 | SLA tag matching is OR logic, not AND | gotchas.md |
| HDG7 | `kanban_state` auto-resets to 'normal' on EVERY stage change | gotchas.md |
| HDG8 | `assign_date` is set ONLY on first-ever assignment | gotchas.md |
| HDG9 | `close_date` is CLEARED when reopening | gotchas.md |

## enterprise/hr_appraisal

| ID | Título | Fichero |
|----|--------|---------|
| APRCAP1 | ¿Crea appraisals automáticamente por antigüedad? | capabilities.md |
| APRCAP10 | ¿Notificar a varios managers a la vez? | capabilities.md |
| APRCAP11 | ¿Plantillas de cuestionario de evaluación (empleado / manager)? | capabilities.md |
| APRCAP12 | ¿Evaluar y sincronizar competencias en el appraisal (skills)? | capabilities.md |
| APRCAP13 | ¿Feedback 360 vía encuesta a compañeros? | capabilities.md |
| APRCAP14 | ¿Programar las evaluaciones según fechas de contrato? | capabilities.md |
| APRCAP15 | ¿Notas privadas del manager sobre el empleado? | capabilities.md |
| APRCAP2 | ¿Ocultar el feedback del manager hasta publicarlo? | capabilities.md |
| APRCAP3 | ¿Un empleado puede pedir su propia evaluación? | capabilities.md |
| APRCAP4 | ¿Vincular reuniones de calendario? | capabilities.md |
| APRCAP5 | ¿Calcula la fecha de entrevista final desde las reuniones? | capabilities.md |
| APRCAP6 | ¿Rating final no visible al empleado? | capabilities.md |
| APRCAP7 | ¿Propiedades dinámicas por departamento? | capabilities.md |
| APRCAP8 | ¿Objetivos con progreso, deadline y tags? | capabilities.md |
| APRCAP9 | ¿Avisa de evaluaciones duplicadas? | capabilities.md |
| APRG1 | El cron crea appraisals directamente en `pending`, no en `new`. | gotchas.md |
| APRG10 | Cancelar borra reuniones y actividades y rebobina `last_appraisal_id`. | gotchas.md |
| APRG11 | Archivar al empleado borra su `next_appraisal_date` pero NO cancela appraisals abiertos. | gotchas.md |
| APRG12 | La baja (departure wizard) cancela appraisals futuros solo si marcas el check. | gotchas.md |
| APRG13 | No puedes borrar un empleado que es manager de algún goal. | gotchas.md |
| APRG14 | Las plantillas de mail de request son indestructibles. | gotchas.md |
| APRG15 | hr (núcleo). | integrations.md |
| APRG16 | survey (360, `hr_appraisal_survey`). | integrations.md |
| APRG17 | `hr_appraisal_skills`. | integrations.md |
| APRG18 | calendar. | integrations.md |
| APRG2 | `next_appraisal_date` solo se calcula si el empleado NO tiene appraisals en curso. | gotchas.md |
| APRG20 | `hr_appraisal_contract`. | integrations.md |
| APRG3 | No puedes poner `next_appraisal_date` en el pasado. | gotchas.md |
| APRG4 | El mensaje del plan se postea una sola vez y bajo condiciones. | gotchas.md |
| APRG5 | El "Unpublished" del feedback NO es un permiso ORM, es un compute que oculta el valor. | gotchas.md |
| APRG6 | `read()` sobreescrito oculta la nota privada y el rating al propio empleado. | gotchas.md |
| APRG7 | Un empleado NO puede publicar el feedback del manager. | gotchas.md |
| APRG8 | Pasar a `pending` SIEMPRE re-oculta ambos feedbacks. | gotchas.md |
| APRG9 | No se puede borrar un appraisal que no esté en `new`/`cancel`. | gotchas.md |

## enterprise/hr_payroll

| ID | Título | Fichero |
|----|--------|---------|
| PYCAP1 | ¿Reglas salariales personalizadas en Python? | capabilities.md |
| PYCAP10 | ¿Trae reglas españolas l10n_es listas? | capabilities.md |
| PYCAP11 | ¿Genera la nómina a partir de work entries (días trabajados)? | capabilities.md |
| PYCAP12 | ¿Estructuras salariales por tipo/país? | capabilities.md |
| PYCAP13 | ¿Entradas variables por nómina (bonus, comisiones, dietas)? | capabilities.md |
| PYCAP14 | ¿Ausencias reflejadas automáticamente en la nómina? | capabilities.md |
| PYCAP15 | ¿Declaraciones sociales periódicas (tipo DSN/Seg. Social)? | capabilities.md |
| PYCAP16 | ¿Simulación de oferta / paquete salarial (net configurator)? | capabilities.md |
| PYCAP17 | ¿Recibo de nómina en PDF + generación programada? | capabilities.md |
| PYCAP18 | ¿Cuadro de mando con avisos + informe de plantilla (headcount)? | capabilities.md |
| PYCAP19 | ¿Cumplimiento de nómina española (IRPF por tramos, Seg. Social, finiquito, certificado de empresa)? | capabilities.md |
| PYCAP2 | ¿Genera asiento contable de nómina? | capabilities.md |
| PYCAP3 | ¿Registra el pago al banco del empleado? | capabilities.md |
| PYCAP4 | ¿Distintas frecuencias de pago? | capabilities.md |
| PYCAP5 | ¿Nóminas en lote? | capabilities.md |
| PYCAP6 | ¿Nómina rectificativa / refund? | capabilities.md |
| PYCAP7 | ¿Embargos / deducciones recurrentes? | capabilities.md |
| PYCAP8 | ¿Acumulado anual (YTD)? | capabilities.md |
| PYCAP9 | ¿Tablas paramétricas versionadas por fecha (IRPF)? | capabilities.md |
| PYG1 | `compute_sheet` borra TODAS las líneas antes de recomputar. | gotchas.md |
| PYG10 | Inputs del mismo código se agregan vía `ProxyHrPayslipInput` | gotchas.md |
| PYG11 | `date_from`/`date_to` se autocomputan por `schedule_pay` y se recortan a fin de contrato. | gotchas.md |
| PYG12 | `appears_on_payroll_report=True` crea/borra `ir.model.fields` dinámicos en `hr.payroll.report` | gotchas.md |
| PYG13 | Las sumas históricas (`_sum`, `_sum_category`, `_sum_worked_days`, `_get_line_values`) usan SQL | gotchas.md |
| PYG14 | `action_payslip_done` valida work entries del periodo solo para payslips regulares | gotchas.md |
| PYG15 | `hr` (contratos/empleados). | integrations.md |
| PYG16 | `account` (vía `hr_payroll_account`). | integrations.md |
| PYG17 | `hr_work_entry`. | integrations.md |
| PYG18 | `hr_holidays` (vía `hr_payroll_holidays`). | integrations.md |
| PYG19 | `hr_attendance` / `planning`. | integrations.md |
| PYG2 | El flag `edited` protege contra `action_refresh_from_work_entries`, NO contra `compute_sheet`. | gotchas.md |
| PYG20 | Salary attachments ↔ inputs. | integrations.md |
| PYG3 | `_compute_line_ids` NO recalcula salvo contexto explícito. | gotchas.md |
| PYG4 | El `code` de regla es case-sensitive y se usa como variable. | gotchas.md |
| PYG5 | Los errores de `safe_eval` se reempaquetan en `UserError`. | gotchas.md |
| PYG6 | `_get_parameter_from_code` está cacheado con `ormcache` por `(code, date, allowed_company_ids)` | gotchas.md |
| PYG7 | El parámetro vigente = `date_from` más reciente ≤ fecha | gotchas.md |
| PYG8 | Cancelar una payslip `done` requiere Payroll Manager. | gotchas.md |
| PYG9 | Marcar `paid` dispara `record_payment` en salary attachments agrupados por código | gotchas.md |

## enterprise/intercompany

| ID | Título | Fichero |
|----|--------|---------|
| ICCAP1 | ¿Auto-crear PO en la compañía B desde una SO en A? | capabilities.md |
| ICCAP10 | ¿Detección de la compañía por el partner? | capabilities.md |
| ICCAP2 | ¿Factura/bill espejo entre compañías? | capabilities.md |
| ICCAP3 | ¿Movimiento de stock entre compañías? | capabilities.md |
| ICCAP4 | ¿Reglas por compañía? | capabilities.md |
| ICCAP5 | ¿Validación automática vs dejar en borrador? | capabilities.md |
| ICCAP6 | ¿Qué documentos se sincronizan? | capabilities.md |
| ICCAP7 | ¿Almacén destino por compañía? | capabilities.md |
| ICCAP8 | ¿Multi-moneda entre compañías (PO→SO)? | capabilities.md |
| ICCAP9 | ¿Trazabilidad/log del documento intercompañía? | capabilities.md |
| ICG1 | Currency mismatch on PO→SO is a hard error | gotchas.md |
| ICG10 | Inter-company ≠ inter-warehouse transit locations | gotchas.md |
| ICG11 | Branch/sub-company support via `parent_of` | gotchas.md |
| ICG12 | Child contacts trigger intercompany | gotchas.md |
| ICG13 | Config page only visible in developer mode | gotchas.md |
| ICG14 | Analytic distribution merged, not copied | gotchas.md |
| ICG15 | PDF attachment copied to mirrored bill | gotchas.md |
| ICG16 | Auto-posted bills bypass approval workflow | gotchas.md |
| ICG17 | No salesperson on auto-generated SO | gotchas.md |
| ICG18 | Kit products with different BOMs per company | gotchas.md |
| ICG19 | Custom attribute values forwarded via stock move rollup | gotchas.md |
| ICG2 | Discount field is explicitly preserved | gotchas.md |
| ICG20 | Dropship in intercompany redirects to transit | gotchas.md |
| ICG21 | Mirror creation is atomic with the source document | gotchas.md |
| ICG22 | No auto-reconciliation between companies | gotchas.md |
| ICG23 | v19 unified UoM breaks line-prep overrides | gotchas.md |
| ICG24 | Tax-recompute line filter differs v18 vs v19 | gotchas.md |
| ICG25 | PO→SO forces `in_rental_app=False` + `allowed_company_ids=company.ids` in context | gotchas.md |
| ICG26 | Warehouse ownership is a STRICT equality, no fallback | gotchas.md |
| ICG27 | `_post` pops `default_invoice_payment_term_id` from context before mirroring | gotchas.md |
| ICG28 | Tax recompute happens AFTER price is pinned | gotchas.md |
| ICG29 | Delivery→receipt sync skips company-restricted products entirely | gotchas.md |
| ICG3 | UoM normalized to a product UoM, and the two directions use DIFFERENT targets | gotchas.md |
| ICG30 | PO→SO access check runs as the CURRENT user, not the intercompany user | gotchas.md |
| ICG31 | Cross-reference write-back is conditional — pre-set refs are never overwritten | gotchas.md |
| ICG32 | SO→PO is skipped when the source SO has no `company_id` | gotchas.md |
| ICG33 | Invoice mirroring batches per target company and posts in Co.B's company context | gotchas.md |
| ICG34 | Fiscal position for the mirror bill is resolved from Co.A's partner + Co.A's delivery address, evalu | gotchas.md |
| ICG4 | `auto_generated` is the ONLY circuit breaker | gotchas.md |
| ICG5 | Company-specific products stripped from invoice lines | gotchas.md |
| ICG6 | Fiscal position recomputed, not copied | gotchas.md |
| ICG7 | Payment terms only cross-company if shared | gotchas.md |
| ICG8 | Delivery sync depends on PO name = SO client_order_ref | gotchas.md |
| ICG9 | Warehouse must match company | gotchas.md |

## enterprise/intrastat

| ID | Título | Fichero |
|----|--------|---------|
| INTR1 | Default single-digit transaction codes (1–9) are all expired; the active set is 11–99. | gotchas.md |
| INTR10 | Supplementary-unit amount is force-zeroed whenever the commodity code has no supplementary unit. | gotchas.md |
| INTR11 | (= EDIG34). Triangular trade & B2C are NOT excluded — they are reported with substitute VAT placehol | gotchas.md |
| INTR12 | `stock_intrastat`'s `_fill_missing_values` override is DEAD CODE in v18 — the handler it lives on is | gotchas.md |
| INTR13 | Even *if* the stock handler were wired in, stock moves are consulted ONLY for the region code — quan | gotchas.md |
| INTR14 | Report period must be a full month or full quarter, enforced by `UserError` — partial ranges blow up | gotchas.md |
| INTR15 | Commodity code is blanked in the output when the code belongs to a *different* country than the comp | gotchas.md |
| INTR16 | "No journals selected" means ALL sale/purchase journals — empty selection ≠ no data. | gotchas.md |
| INTR17 | The `intrastat_with_vat` filter ADDS `AND partner.vat IS NOT NULL` — it silently drops every B2C/no- | gotchas.md |
| INTR18 | Lines with no product (`prod.id IS NULL`) are KEPT in the query but their value is EXCLUDED from tot | gotchas.md |
| INTR19 | The invoice-type filter is a `forced_domain`, not a SQL clause — `intrastat_type` selection of NEITH | gotchas.md |
| INTR2 | (= EDIG31). Brexit cutoff is hard-coded `2021-01-01`; GB excluded after it, origin GB rewritten to ` | gotchas.md |
| INTR20 | Supplementary-units total is taken from the FIRST subline's flag but SUMMED across all sublines — an | gotchas.md |
| INTR21 | Region code in the declaration comes ONLY from `company.intrastat_region_id` in v18 — the per-wareho | gotchas.md |
| INTR22 | Transaction-code expiry/premature warnings compare against `account_move.invoice_date`, but `_comput | gotchas.md |
| INTR23 | Country/origin-country/incoterm column NAMES vs CODES swap depends on `country_format`/`commodity_fl | gotchas.md |
| INTR3 | The base goods report hard-excludes services (`prodt.type != 'service'`); services need a separate h | gotchas.md |
| INTR4 | (= EDIG33). Transaction code is NULL when the move has no `intrastat_country_id`; warning only fires | gotchas.md |
| INTR5 | Refund sign handling: value = `SIGN(quantity)*SIGN(price_unit)*ABS(balance)`, and refunds flip the a | gotchas.md |
| INTR6 | Cross-border filter excludes domestic lines via `company_country.id != country.id`. | gotchas.md |
| INTR7 | (= SAI1). On sales, `intrastat_country_id` = the SHIPPING partner's country, not the invoice/bill-to | gotchas.md |
| INTR8 | (= PUI1). Bridge-set `intrastat_country_id` is only written when the partner country is flagged `int | gotchas.md |
| INTR9 | Zero weight → `warning_missing_weight`, but ONLY for `consu` products with no supplementary unit. | gotchas.md |
| INTRCAP1 | ¿Declaración Intrastat? | capabilities.md |
| INTRCAP10 | ¿Incoterm en la declaración? | capabilities.md |
| INTRCAP2 | ¿Códigos de mercancía? | capabilities.md |
| INTRCAP3 | ¿Tipo de transacción / naturaleza? | capabilities.md |
| INTRCAP4 | ¿País de origen, peso, valor? | capabilities.md |
| INTRCAP5 | ¿Aplica a ventas y compras? | capabilities.md |
| INTRCAP6 | ¿Datos desde movimientos de stock? | capabilities.md |
| INTRCAP7 | ¿Región/provincia? | capabilities.md |
| INTRCAP8 | ¿Modo de transporte? | capabilities.md |
| INTRCAP9 | ¿Exportar el fichero para la autoridad? | capabilities.md |

## enterprise/iot

| ID | Título | Fichero |
|----|--------|---------|
| IOTG1 | IoT box behind NAT / reverse proxy | gotchas.md |
| IOTG10 | Belgian blackbox end-of-day report | gotchas.md |
| IOTG11 | Blackbox IP lock | gotchas.md |
| IOTG12 | Same blackbox across multiple POS | gotchas.md |
| IOTG13 | Box HTTPS cert expiry → silent disconnect | gotchas.md |
| IOTG14 | Bluetooth devices unreliable under load | gotchas.md |
| IOTG15 | Customer display URL unreachable → blank screen | gotchas.md |
| IOTG16 | Worldline paper status codes | gotchas.md |
| IOTG17 | Token expires after 60 minutes (v18; v19 = 15 min) | gotchas.md |
| IOTG18 | New box format (`YYYY.MM.DD`) skips installed-module drivers | gotchas.md |
| IOTG19 | `pos_iot_*` v19 splits | gotchas.md |
| IOTG2 | Reserva de DHCP o IP fija para la caja: evita que la IP baile. | config_guide.md |
| IOTG20 | Print confirmation 10-second timeout | gotchas.md |
| IOTG21 | Single-tenant device assumption | gotchas.md |
| IOTG22 | Image versioning git-link broken | gotchas.md |
| IOTG23 | `manual_measurement` only for Adam | gotchas.md |
| IOTG24 | Keyboard layout pushed on first setup only | gotchas.md |
| IOTG25 | Box admin can't "create" a box from the UI | gotchas.md |
| IOTG26 | `/iot/setup` token only checked on first registration | gotchas.md |
| IOTG27 | Token mismatch fails silently | gotchas.md |
| IOTG28 | `is_websocket_active` never goes back to False (v18-only) | gotchas.md |
| IOTG29 | Empty report template → cryptic ParserError, now wrapped | gotchas.md |
| IOTG3 | `drivers_auto_update=True` overwrites custom drivers | gotchas.md |
| IOTG30 | `iot.channel` returns empty string for non-internal users | gotchas.md |
| IOTG31 | `take_control` is not in the Python module *(box) | gotchas.md |
| IOTG32 | V17 db ↔ V18.4+ box print callbacks dropped on purpose | gotchas.md |
| IOTG33 | The real print-routing decision is in the JS handler, not Python | gotchas.md |
| IOTG34 | Two devices on a report → prints on BOTH (fan-out, not a choice) | gotchas.md |
| IOTG4 | POS offline mode still needs the IoT box | gotchas.md |
| IOTG5 | Receipt printer paper status only on poll | gotchas.md |
| IOTG6 | Cash drawer ESC/POS not universal | gotchas.md |
| IOTG7 | Scale calibration drift | gotchas.md |
| IOTG8 | Payment terminal certification per country + acquirer | gotchas.md |
| IOTG9 | Multiple POS sessions on one box → race conditions | gotchas.md |
| WOG67B | en tablet MRP, lot/SN creado pero la etiqueta | integrations.md |

## enterprise/knowledge

| ID | Título | Fichero |
|----|--------|---------|
| KNCAP1 | ¿Artículos jerárquicos tipo wiki (parent/child)? | capabilities.md |
| KNCAP10 | ¿Propiedades personalizadas en artículos? | capabilities.md |
| KNCAP11 | ¿Comentarios / hilos en línea sobre partes del artículo? | capabilities.md |
| KNCAP12 | ¿Gestionar artículos como items con etapas (kanban)? | capabilities.md |
| KNCAP13 | ¿Imagen de portada del artículo? | capabilities.md |
| KNCAP14 | ¿Compartir con externos por invitación (miembro por email)? | capabilities.md |
| KNCAP2 | ¿Permisos por artículo y miembros? | capabilities.md |
| KNCAP3 | ¿Workspace / privado / compartido con herencia a hijos? | capabilities.md |
| KNCAP4 | ¿Plantillas de artículo? | capabilities.md |
| KNCAP5 | ¿Items embebidos (vistas Odoo, kanban/lista dinámicas)? | capabilities.md |
| KNCAP6 | ¿Versionado/historial del artículo? | capabilities.md |
| KNCAP7 | ¿Favoritos con orden personal? | capabilities.md |
| KNCAP8 | ¿Papelera con borrado diferido? | capabilities.md |
| KNCAP9 | ¿Publicar un artículo en la web / acceso externo? | capabilities.md |
| KNG1 | `category` (Private/Shared/Workspace) is decided by the ROOT article, not the article itself | gotchas.md |
| KNG10 | Article items appear in TWO places: as `child_ids` and as item rows | gotchas.md |
| KNG11 | `stage_id` is scoped per parent (`domain="[('parent_id','=',parent_id)]"`) and is COMPUTED+stored | gotchas.md |
| KNG12 | `article_properties` definition lives on the PARENT | gotchas.md |
| KNG13 | v18 templates use `template_body` (raw text); rendering happens via `_render_template()` | gotchas.md |
| KNG14 | v19 `origin_template_id` tracks ancestry but doesn't auto-update | gotchas.md |
| KNG15 | Root templates require `template_category_id` | gotchas.md |
| KNG16 | `template_name` is required for templates | gotchas.md |
| KNG17 | `to_delete=True` requires `active=False` | gotchas.md |
| KNG18 | Trash limit = config param `knowledge.knowledge_article_trash_limit_days`, fallback 30 | gotchas.md |
| KNG19 | `deletion_date` is NOT stored — it is computed every read | gotchas.md |
| KNG2 | For internal users a member row OVERRIDES internal permission — it is NOT "max wins" | gotchas.md |
| KNG20 | The GC cron deletes only 100 trashed articles per run | gotchas.md |
| KNG21 | Only `body` is versioned via `html.field.history.mixin` | gotchas.md |
| KNG22 | `body` is `prefetch=False` | gotchas.md |
| KNG23 | `handle_history_divergence` merges concurrent edits silently | gotchas.md |
| KNG24 | Anchor text capped at 1200 chars + ellipsis | gotchas.md |
| KNG25 | Thread `is_resolved=True` HIDES it in the editor; never unlinks | gotchas.md |
| KNG26 | Threads inherit `_mail_post_access='read'` — read access lets you comment | gotchas.md |
| KNG27 | `unique(article_id, user_id)` | gotchas.md |
| KNG28 | Favorite `sequence` is auto-assigned to max+1 on create | gotchas.md |
| KNG29 | `is_article_active` is a stored related — favorites survive archive/trash | gotchas.md |
| KNG3 | Desynchronized article MUST have its own internal_permission | gotchas.md |
| KNG30 | `favorite_count` is stored and recomputed on every favorite write | gotchas.md |
| KNG31 | Permission resolution runs recursive SQL CTEs per user/partner | gotchas.md |
| KNG32 | Article `body` is NOT translatable | gotchas.md |
| KNG33 | Portal invite link is an HMAC over `member.id-create_date-partner_id-article_id` | gotchas.md |
| KNG34 | `website_knowledge` cascades the published state to descendants | gotchas.md |
| KNG35 | v19 drops the `web_editor` dependency | gotchas.md |
| KNG36 | Desync is a SIDE EFFECT of downgrading, not a user-set flag | gotchas.md |
| KNG37 | v19 adds `is_listed_in_templates_gallery`, `template_child_default_create`, `origin_template_id` | gotchas.md |
| KNG38 | `create()` auto-fills `body` with an `<h1>` from the title when `body` is omitted | gotchas.md |
| KNG39 | Only `base.group_system` can create/update/delete/retype templates — checked in Python, not ir.model | gotchas.md |
| KNG4 | Root article cannot be desynchronized | gotchas.md |
| KNG40 | `last_edition_uid`/`last_edition_date` only update when `body` is in the write vals | gotchas.md |
| KNG41 | `handle_history_divergence` (OT merge) runs ONLY for single-record writes | gotchas.md |
| KNG42 | Portal/non-internal users may only write an allowlist of fields; private articles get a slightly wid | gotchas.md |
| KNG43 | `copy()` suffixes "(copy)" ONLY on root articles — child copies keep the original name | gotchas.md |
| KNG44 | Archiving/privatizing does NOT cascade to children you can't write — it DETACHES them as new ROOT ar | gotchas.md |
| KNG45 | `search_fetch` is overridden to support ordering by `is_user_favorite` via a TWO-pass search | gotchas.md |
| KNG46 | `_compute_user_can_write` short-circuits to True for the system user — it is ACL-like, not permissio | gotchas.md |
| KNG47 | Every new non-share user silently gets a private "Welcome <user>" article — and it can fail silently | gotchas.md |
| KNG5 | Two-layer "must have a writer" check; the member-level one fires on unlink | gotchas.md |
| KNG6 | `inherited_permission` is stored+recursive — touching an ancestor cascades a recompute down to the n | gotchas.md |
| KNG7 | `parent_id` is `ondelete='cascade'` — deleting an article hard-deletes the whole subtree | gotchas.md |
| KNG8 | CORRECTION — `move_to` DOES adjust permissions in v18 | gotchas.md |
| KNG9 | `move_to` under an Article Item parent is rejected | gotchas.md |

## enterprise/marketing_automation

| ID | Título | Fichero |
|----|--------|---------|
| MACAP1 | ¿Campañas multi-paso con flujo? | capabilities.md |
| MACAP10 | ¿Cron que ejecuta las actividades programadas? | capabilities.md |
| MACAP2 | ¿Disparadores de actividad (al entrar / tras tiempo / tras acción)? | capabilities.md |
| MACAP3 | ¿Target = cualquier modelo con dominio? | capabilities.md |
| MACAP4 | ¿Acciones: email, SMS, WhatsApp, server action? | capabilities.md |
| MACAP5 | ¿Ramas condicionales según interacción? | capabilities.md |
| MACAP6 | ¿A/B testing de la campaña/mensajes? | capabilities.md |
| MACAP7 | ¿Métricas por actividad? | capabilities.md |
| MACAP8 | ¿Participantes únicos / evitar reprocesar? | capabilities.md |
| MACAP9 | ¿Probar la campaña antes de lanzar? | capabilities.md |
| MAG1 | Domain filters evaluated at EXECUTION time, not scheduling | gotchas.md |
| MAG10 | Deleted records marked 'unlinked', not removed | gotchas.md |
| MAG11 | Mailings with use_in_marketing_automation=True are hidden from normal views | gotchas.md |
| MAG12 | Blacklist/opt-out NOT enforced automatically | gotchas.md |
| MAG13 | Mailing deduplication before sending | gotchas.md |
| MAG14 | Sync cron runs every 12 hours, not real-time | gotchas.md |
| MAG15 | Execute cron runs every 1 hour BUT _trigger fires at exact dates | gotchas.md |
| MAG16 | Batch processing commits every 100/500 records | gotchas.md |
| MAG17 | CRON trigger deduplication | gotchas.md |
| MAG18 | Campaign copy rebuilds tree but resets sync state | gotchas.md |
| MAG19 | Changing target model on a running campaign is only warned, NOT blocked | gotchas.md |
| MAG2 | Opposite triggers cancel each other automatically | gotchas.md |
| MAG20 | last_sync_date NOT copied | gotchas.md |
| MAG21 | Domain class replaces osv.expression | gotchas.md |
| MAG22 | Testing detection changed | gotchas.md |
| MAG23 | Server action template gains `state` | gotchas.md |
| MAG24 | `mass_mailing_id` is computed, reset on type mismatch | gotchas.md |
| MAG25 | UTM tagging only if `default_marketing_activity_id` in context | gotchas.md |
| MAG26 | Linked mailing cannot be deleted, but the activity can | gotchas.md |
| MAG27 | WhatsApp button URL type controls click tracking | gotchas.md |
| MAG28 | Children traces generated BEFORE the activity method runs | gotchas.md |
| MAG29 | process_event drops events on stopped campaigns silently | gotchas.md |
| MAG3 | 'begin' traces created immediately on participant creation | gotchas.md |
| MAG30 | process_event is single-record (N+1 risk) | gotchas.md |
| MAG31 | unique_field_id reset to False when model_id changes | gotchas.md |
| MAG32 | is_test participants are synced but excluded from stats | gotchas.md |
| MAG33 | _execute_email / _execute_sms raise AccessError if the running user lacks the marketing group | gotchas.md |
| MAG34 | Launch-time bounce is recorded as `error`, not `bounce` | gotchas.md |
| MAG35 | action_start_campaign requires at least one activity | gotchas.md |
| MAG36 | Recursive activity hierarchy blocked; parent must match category | gotchas.md |
| MAG37 | UTM campaign/source records are delete-protected | gotchas.md |
| MAG38 | Setting a Favorite Filter OVERWRITES the manual domain | gotchas.md |
| MAG39 | _execute_action runs server actions ONE-BY-ONE, isolating failures | gotchas.md |
| MAG4 | Event-based triggers (mail_open/click) are created UNSCHEDULED | gotchas.md |
| MAG40 | Manual participant creation defaults res_id to the FIRST record of the model | gotchas.md |
| MAG41 | action_update_participants skips a 'not' branch if the opposite was already processed | gotchas.md |
| MAG42 | _execute_whatsapp iterates ALL of the activity's traces, not just the batch | gotchas.md |
| MAG5 | Modifying running campaigns requires manual sync | gotchas.md |
| MAG6 | New activities on running campaigns only affect existing participants forward | gotchas.md |
| MAG7 | Duplicate participants allowed — stats count ALL | gotchas.md |
| MAG8 | unique_field_id only prevents NEW duplicates during sync | gotchas.md |
| MAG9 | Empty relation on unique field = filtered out | gotchas.md |

## enterprise/mrp_plm

| ID | Título | Fichero |
|----|--------|---------|
| PLMCAP1 | ¿Crear una ECO desde una MO? | capabilities.md |
| PLMCAP10 | ¿Ver desde un producto cuántas ECOs tiene? | capabilities.md |
| PLMCAP2 | ¿Generar una BoM nueva desde los cambios reales de una MO? | capabilities.md |
| PLMCAP3 | ¿Crear ECOs por email? | capabilities.md |
| PLMCAP4 | ¿Aprobar/rechazar una ECO? | capabilities.md |
| PLMCAP5 | ¿Comparar dos revisiones de BoM (diff de líneas y byproducts)? | capabilities.md |
| PLMCAP6 | ¿Ver el diff de routing (operaciones/tiempos)? | capabilities.md |
| PLMCAP7 | ¿Aplicar cambios sin incrementar versión? | capabilities.md |
| PLMCAP8 | ¿Adjuntar documentos a la ECO que se propaguen al producto? | capabilities.md |
| PLMCAP9 | ¿Mover la ECO a una etapa final tras aplicar? | capabilities.md |
| PLMG1 | Aplicar una ECO desactiva la BoM anterior automáticamente. | gotchas.md |
| PLMG10 | `will_update_version` se puede desmarcar y resta versión en caliente. | gotchas.md |
| PLMG11 | ECO `type='product'` ignora todo lo de BoM. | gotchas.md |
| PLMG12 | Producciones en curso: solo se actualizan MOs en `draft`/`confirmed`. | gotchas.md |
| PLMG13 | `latest_bom_id` solo se calcula si la BoM de la MO está inactiva. | gotchas.md |
| PLMG14 | Comentar en el chatter aprueba automáticamente los approvals tipo comment. | gotchas.md |
| PLMG15 | `mrp` (BoM/producción). | integrations.md |
| PLMG16 | `mrp_workorder`/routing. | integrations.md |
| PLMG17 | `quality` (hook). | integrations.md |
| PLMG18 | `documents`/`product.document`. | integrations.md |
| PLMG19 | `product`. | integrations.md |
| PLMG2 | Aplicar una ECO mete en rebase TODAS las demás ECOs abiertas de la misma BoM. | gotchas.md |
| PLMG20 | `mail`/actividades. | integrations.md |
| PLMG3 | `is_blocking` del stage solo lo activan approvals mandatory. | gotchas.md |
| PLMG4 | Aprobaciones no-mandatory cuentan como aprobadas siempre. | gotchas.md |
| PLMG5 | Cambiar de etapa cierra o borra los approvals previos. | gotchas.md |
| PLMG6 | No puedes saltar una etapa bloqueante intermedia. | gotchas.md |
| PLMG7 | La restricción de etapa solo aplica al AVANZAR, no al retroceder. | gotchas.md |
| PLMG8 | Editar la BoM activa con una ECO abierta genera líneas de rebase. | gotchas.md |
| PLMG9 | Conflicto de revisión. | gotchas.md |

## enterprise/mrp_workorder

| ID | Título | Fichero |
|----|--------|---------|
| WOG1 | No backward scheduling in `_plan_workorder` — always forward from date_start | gotchas.md |
| WOG10 | Backorder WOs regenerate quality checks from scratch | gotchas.md |
| WOG100T | `_get_fields_for_tablet` hardcoded en código, NO usa `_register_hook` ni introspection. Custom modul | integrations.md |
| WOG101T | `get_summary_data.show_rainbow` requiere `last30op` ranking. Primer MO del operation type → `last30o | integrations.md |
| WOG102T | `record_production` con WO sin `operation_id` (additional WO) cae a backorder lookup por `index` de  | integrations.md |
| WOG103T | `_change_quality_check(position='next')` con `check.quality_state != 'none'` (ya completado) hace | integrations.md |
| WOG104T | `barcodes.barcode_events_mixin` en `_inherit` provoca que cualquier scan global mientras tablet visi | integrations.md |
| WOG105T | `MrpQualityCheckConfirmationDialog` recibe boxes JSON pero los `DeviceController` se construyen en ` | integrations.md |
| WOG11 | BOM changes after confirmation NOT reflected in WOs | gotchas.md |
| WOG12 | Adding components mid-production doesn't adjust consumed quantities | gotchas.md |
| WOG13 | qty_reported_from_previous_wo is a float, not int | gotchas.md |
| WOG14 | Overlapping times WITHIN a loss_type are merged (union), but SUMMED across loss types | gotchas.md |
| WOG15 | Pause without start creates NULL date_start productivity record | gotchas.md |
| WOG16 | Missing 'productive' loss reason raises UserError on completion | gotchas.md |
| WOG17 | Cancelled WOs keep their time_ids | gotchas.md |
| WOG18 | duration_expected=0 → duration_percent/progress safely fall back to 0 (no Infinity) | gotchas.md |
| WOG19 | Quality checks auto-deleted on WO cancel | gotchas.md |
| WOG2 | ~700-day search limit is CONFIGURABLE, returns tuple not UserError | gotchas.md |
| WOG20 | Check chain navigation not concurrent-safe | gotchas.md |
| WOG21 | User can force pass even when measure fails | gotchas.md |
| WOG22 | Failed quality checks do NOT block stock moves | gotchas.md |
| WOG23 | No lot-level quality tracking | gotchas.md |
| WOG24 | Periodical frequency uses create_date, not control_date | gotchas.md |
| WOG25 | Random frequency is truly random, not seeded | gotchas.md |
| WOG26 | on_demand frequency points are simply NEVER auto-created as WO steps (no constraint blocks them) | gotchas.md |
| WOG27 | finished_product_sequence is a float | gotchas.md |
| WOG28 | Worksheet success_conditions parsed with ast.literal_eval() | gotchas.md |
| WOG29 | Component qty changes don't clean up old checks | gotchas.md |
| WOG3 | Alternative workcenter selection is greedy, not optimal | gotchas.md |
| WOG30 | Only SESSION_OWNER can start/stop WOs from tablet | gotchas.md |
| WOG31 | No skill matching for employee assignment | gotchas.md |
| WOG32 | PIN validation reads pin via sudo() (not the barcode lookup) | gotchas.md |
| WOG33 | No auto-replan on predecessor completion | gotchas.md |
| WOG34 | MPS assumes infinite capacity | gotchas.md |
| WOG35 | OEE window is last 1 MONTH (relativedelta months=1), not "30 days", and not configurable | gotchas.md |
| WOG36 | Drag-drop in Gantt doesn't propagate to dependents | gotchas.md |
| WOG37 | No SQL locks on WO operations | gotchas.md |
| WOG38 | quality_point_ids computed with N+1 queries | gotchas.md |
| WOG39 | Gantt unavailability queries per workcenter per WO | gotchas.md |
| WOG4 | No resource contention — multiple WOs run simultaneously on same workcenter | gotchas.md |
| WOG40 | No crons for WO lifecycle | gotchas.md |
| WOG41 | Scrap during WO doesn't adjust qty_producing | gotchas.md |
| WOG42 | Subcontracting and WOs not mutually exclusive | gotchas.md |
| WOG43 | Empty BOM operations still create WOs | gotchas.md |
| WOG44 | Cancel MO with partial production keeps done moves | gotchas.md |
| WOG45 | Backorder on under-production only triggers from the FIRST unfinished WO | gotchas.md |
| WOG45M | `_create_checks` se ejecuta en `_action_confirm` SOLO si `wo.check_ids` vacío. Si se confirma una WO | integrations.md |
| WOG46 | `action_mark_as_done` allowed-employee check compares employee id against a list of WORKORDER ids (l | gotchas.md |
| WOG46M | State machine NO transitiona directamente `pending→done`. Si una WO está `pending` (sin `_should_sta | integrations.md |
| WOG47 | EE `button_pending` stops ONLY the current employee's timer; CE stops only current user's | gotchas.md |
| WOG47M | `record_production` con `qty_producing < qty_remaining` ejecuta `_split_productions` y crea backorde | integrations.md |
| WOG48 | `_check_open_time_ids` constraint forbids the SAME user opening a WO timer twice | gotchas.md |
| WOG48M | `_compute_duration` override calcula `duration_unit = duration / max(qty_produced, 1)`. Si `qty_prod | integrations.md |
| WOG49 | `end_all` on EE clears `employee_ids` entirely | gotchas.md |
| WOG49M | `mrp.workcenter.productivity.employee_cost` snapshot | integrations.md |
| WOG5 | Gantt conflicts are visual warnings only | gotchas.md |
| WOG50 | `_compute_duration_expected` recomputes (overwrites manual edits) when qty_producing changes | gotchas.md |
| WOG50M | `time_ids` con `date_end=False` (cronómetro activo) impide cancelar la WO sin error explícito si pas | integrations.md |
| WOG51 | Changing a WO's workcenter is blocked once in progress/done; otherwise recomputes duration_expected | gotchas.md |
| WOG51M | `verify_quality_checks` auto-pasa checks `register_consumed_materials`/`register_byproducts`/`instru | integrations.md |
| WOG52 | Alternative-workcenter duration uses the ORIGINAL workcenter's settings as a baseline | gotchas.md |
| WOG52H | `EMPLOYEES_CONNECTED` y `SESSION_OWNER` viven en `request.session` (cookie-backed) NO en DB. Reinici | integrations.md |
| WOG53 | `_plan_workorder` requires a resource_calendar on EVERY candidate workcenter | gotchas.md |
| WOG53H | Logout del `SESSION_OWNER` con otros empleados conectados → `owner=False`, NINGUNO toma el rol. Próx | integrations.md |
| WOG54 | WO `state` is a STORED recursive compute — cannot be set freely; buttons write it directly bypassing | gotchas.md |
| WOG54H | `pin_validation(pin=False)` con `employee.pin=False` (sin PIN configurado) → retorna `True`. Cualqui | integrations.md |
| WOG55 | `_set_duration` (manual duration edit) auto-splits into productive + performance loss lines | gotchas.md |
| WOG55H | `start_employee` añade al `employee_ids` pero el `time_id` se crea con `employee_id` específico. Si  | integrations.md |
| WOG56 | v19: `button_start` asigna `employee_assigned_ids`; `button_finish` congela `employee_costs_hour` | gotchas.md |
| WOG56H | `allowed_employees = workcenter.employee_ids`. Si admin cambia `workcenter.employee_ids` con WOs `pe | integrations.md |
| WOG57 | v19: `record_production` ya NO auto-abre la tablet del siguiente WO (WOG9 obsoleto) | gotchas.md |
| WOG57H | `get_employees_connected()` en test cases (sin `request`) cae a `[env.user.employee_id.id]`. Tests a | integrations.md |
| WOG58 | v19: `hr.mixin` también en `mrp.workcenter`; `get_employee_barcode` usa `.sudo().search` (amplía §7  | gotchas.md |
| WOG58Q | `_default_product_ids` lee `default_operation_id` del context y busca BOM products. Si abres `qualit | integrations.md |
| WOG59Q | `_compute_is_workorder_step` requiere `picking_type_ids` TODOS con `code='mrp_operation'`. Si admin  | integrations.md |
| WOG6 | Cannot unplan a single WO — must unplan the whole MO | gotchas.md |
| WOG60Q | `next_check_id`/`previous_check_id` doubly-linked chain MANUAL. Si `quality.check.unlink()` sin re-l | integrations.md |
| WOG61Q | `finished_product_sequence` = `wo.qty_produced` al momento de crear el check. Si MO produce con back | integrations.md |
| WOG62Q | `do_measure` override en `quality_mrp_workorder` llama `_next()` SOLO si hay `workorder_id`. Quality | integrations.md |
| WOG63Q | `quality.point._change_product_ids_for_bom` en `mrp.routing.workcenter.write({'bom_id':...})` → camb | integrations.md |
| WOG64Q | `_check_to_unlink` en `quality_mrp_workorder` añade `not self.workorder_id` → checks linked a WO NO  | integrations.md |
| WOG65B | `mrp.workorder._inherit` añade `barcodes.barcode_events_mixin` directamente. La WO en form view escu | integrations.md |
| WOG66B | `filter_on_barcode` con GS1 nomenclature parsea solo la ÚLTIMA regla feasible (`parsed_results[::-1] | integrations.md |
| WOG67B | `set_lot_producing` ejecuta `_set_lot_producing` (genera SN/lot) y luego retorna `lot.read(_get_fiel | integrations.md |
| WOG68B | `move_raw_line_ids` computed (no stored) | integrations.md |
| WOG69B | `_get_stock_barcode_data` invoca múltiples `search`+`read` por modelo. Si MO tiene componentes en mu | integrations.md |
| WOG7 | Calendar leaves NOT auto-updated on calendar change | gotchas.md |
| WOG70IO | `iot.trigger.device_id` domain `[('type','=','keyboard')]` | integrations.md |
| WOG71IO | `quality.check.boxes` (Char JSON) recomputa por `workcenter_id.trigger_ids` change pero NO por `trig | integrations.md |
| WOG72IO | `DeviceController` JS usa `iot_longpolling` service. Si IoT box pierde conexión mid-WO, pedales NO f | integrations.md |
| WOG73IO | `action_print` con `device_id` (IoT printer) NO valida que la impresora esté en línea pre-dispatch.  | integrations.md |
| WOG74IO | Múltiples triggers con MISMA `key` en mismo workcenter → solo el primero ordenado por `sequence` se  | integrations.md |
| WOG75IO | Test type `measure` con `device_id` (caliper push) escribe `check.measure` automático. Si device env | integrations.md |
| WOG76MN | `_recreate_leaves` lanza `UserError` si MO ya planificada en el slot Y es create-time. Si admin prog | integrations.md |
| WOG77MN | `maintenance.stage.create_leaves=False` desactiva leaves para TODAS las requests en ese stage. Si ad | integrations.md |
| WOG78MN | `_get_unavailability_intervals` JOIN SQL hardcoded a `maintenance_for='equipment'` con overlap. Main | integrations.md |
| WOG79MN | `_compute_workcenter_id` recompute solo on `maintenance_for` or `equipment_id` change. Si admin asig | integrations.md |
| WOG8 | Serial products force qty_producing=1 per WO | gotchas.md |
| WOG80MN | `recurring_maintenance` con `recurring_leaves_count` >0 genera N leaves futuros via loop, cada uno c | integrations.md |
| WOG81MN | MTBF/MTTR calc en `maintenance.mixin` ignora maintenance_type. Una request preventiva con `duration` | integrations.md |
| WOG82PL | `add_check_in_chain` con `notify_bom=False` override default `True` (en core mrp_workorder es True). | integrations.md |
| WOG83PL | Búsqueda ECO `state in ('confirmed','progress')` | integrations.md |
| WOG84PL | `_get_sync_values` matching key = `(test_type_id, title, component_id)`. Si admin renombra `title` e | integrations.md |
| WOG85PL | `eco.action_new_revision` ejecuta sudo: usuario sin permiso PLM puede crear revisiones implícitament | integrations.md |
| WOG86PL | Resequence loop `current_sequence += 10` empieza en 10 SIEMPRE. Si admin tenía steps con sequence 5, | integrations.md |
| WOG87PL | `_create_revision_bom` copia `quality_point_ids` de operation usando tuple `(company_id, name, workc | integrations.md |
| WOG88A | `_create_or_update_analytic_entry` se ejecuta en cada `_compute_duration` recompute, no solo al fini | integrations.md |
| WOG89A | `employee_analytic_account_line_ids` es M2M (no O2M). Si admin manual-borra un `account.analytic.lin | integrations.md |
| WOG9 | record_production() opens next WO's tablet — no dedicated "start next" method exists | gotchas.md |
| WOG90A | `action_cancel` unlinkea las analytic lines del bridge, pero si la WO se canceló y luego se recrea p | integrations.md |
| WOG91A | `_total_cost_per_hour` añade `employee_costs_hour * employee_ratio` SIEMPRE, incluso si los empleado | integrations.md |
| WOG92A | Multiple empleados en misma WO crean MÚLTIPLES analytic lines (una por productivity time_id). Si rep | integrations.md |
| WOG93A | `account.analytic.line.employee_id` añadido pero ningún reporte stock/mrp lo usa por defecto. Custom | integrations.md |
| WOG94EX | `is_expired` related a `lot.product_expiry_alert` computa SOLO en lot fetch | integrations.md |
| WOG95EX | Wizard `confirm_expiry` solo aparece para `register_consumed_materials`. Si `register_byproducts` pr | integrations.md |
| WOG96EX | `product_expiry_alert` (boolean en stock.lot) considera `alert_time` configurado en producto. Si adm | integrations.md |
| WOG97EX | Si MO cancelada después de consumir lot expirado, analytic + stock NO retroactivos (lot ya usado). E | integrations.md |
| WOG98EX | Multi-company: lots tienen `company_id` y la verificación expira | integrations.md |
| WOG99T | `open_tablet_view` auto-login del `env.user.employee_id` sin PIN si no hay session_owner. Si user co | integrations.md |

## enterprise/planning

| ID | Título | Fichero |
|----|--------|---------|
| PLANCAP1 | ¿Turnos por empleado y por rol/puesto? | capabilities.md |
| PLANCAP10 | ¿El empleado ve su planning (portal/self, sin login)? | capabilities.md |
| PLANCAP11 | ¿% de asignación / horas asignadas vs disponibles? | capabilities.md |
| PLANCAP12 | ¿Respetar fechas de contrato del empleado (planning_contract)? | capabilities.md |
| PLANCAP13 | ¿Auto-planificación (auto-asignar turnos abiertos)? | capabilities.md |
| PLANCAP14 | ¿Detección de conflictos / solapes de turnos? | capabilities.md |
| PLANCAP15 | ¿Planificar recursos materiales/equipos (no solo personas)? | capabilities.md |
| PLANCAP16 | ¿Vista Gantt con arrastrar y soltar? | capabilities.md |
| PLANCAP17 | ¿Petición de cambio/intercambio de turno por el empleado? | capabilities.md |
| PLANCAP18 | ¿Los turnos generan work entries para nómina / alquiler? | capabilities.md |
| PLANCAP2 | ¿Turnos recurrentes / plantillas / copiar semana anterior? | capabilities.md |
| PLANCAP3 | ¿Publicar el planning y notificar a los empleados? | capabilities.md |
| PLANCAP4 | ¿Open shifts que el empleado coge (self-assign)? | capabilities.md |
| PLANCAP5 | ¿Respetar horario laboral y vacaciones/ausencias? | capabilities.md |
| PLANCAP6 | ¿Vender horas y planificarlas (sale_planning)? | capabilities.md |
| PLANCAP7 | ¿Forecast / planificar contra proyecto (project_forecast)? | capabilities.md |
| PLANCAP8 | ¿Asignar según skills/competencias (planning_hr_skills)? | capabilities.md |
| PLANCAP9 | ¿Planning integrado con Field Service (industry_fsm)? | capabilities.md |
| PLG1 | Auto-plan only works with non-flexible calendars | gotchas.md |
| PLG10 | Open shift emails go to role-matched employees only | gotchas.md |
| PLG100 | Self-assign and self-unassign run as `sudo()` and bypass the overlap/conflict check. `action_self_as | gotchas.md |
| PLG101 | `allow_self_unassign` is driven by `company.planning_employee_unavailabilities == 'unassign'` (defau | gotchas.md |
| PLG102 | Switching a shift (`request_to_switch=True`) makes the slot visible to OTHER users via the ir.rule ( | gotchas.md |
| PLG103 | Re-assigning a PUBLISHED human shift to a different human resource in `write()` auto-sends the "shif | gotchas.md |
| PLG104 | Copy Previous Week only copies slots with `recurrency_id=False` AND `was_copied=False`, and marks so | gotchas.md |
| PLG105 | `planning.slot.template` does NOT carry a company | gotchas.md |
| PLG106 | `default_planning_role_id` / `planning_role_ids` on hr.employee are RELATED (readonly except for hr. | gotchas.md |
| PLG107 | `planning_contract` overrides `_get_misc_recurrence_stop` so recurring shifts for an employee with a | gotchas.md |
| PLG108 | sale_planning makes `start_datetime`/`end_datetime` OPTIONAL on planning.slot and adds a SQL constra | gotchas.md |
| PLG109 | Dragging the sale_planning unscheduled slot onto the gantt (write with a start_datetime) triggers `a | gotchas.md |
| PLG11 | Employee archiving splits in-progress shifts | gotchas.md |
| PLG110 | project_forecast adds `project_id`/`task_id` to slots and a grid view but is a SEPARATE optional mod | gotchas.md |
| PLG111 | Archiving a MATERIAL resource (resource.resource.action_archive) also splits/unassigns its in-progre | gotchas.md |
| PLG112 | `_compute_overlap_slot_count` for non-managers only counts overlaps against PUBLISHED slots (`S2.sta | gotchas.md |
| PLG12 | Slot properties are per-role | gotchas.md |
| PLG13 | Overlap detection is SQL-based and only counts published for non-managers | gotchas.md |
| PLG14 | Recurrence scope (this/subsequent/all) recreates slots | gotchas.md |
| PLG15 | No stored planning vs actual comparison | gotchas.md |
| PLG16 | Appointment module does NOT check Planning slots | gotchas.md |
| PLG17 | Auto-plan has no zone/geography concept | gotchas.md |
| PLG18 | Unassigned slots after auto-plan generate no notification | gotchas.md |
| PLG19HR | `employee_id` es | integrations.md |
| PLG2 | Auto-plan is greedy first-fit, not optimal | gotchas.md |
| PLG20HR | Roles viven en `resource.resource.role_ids`, | integrations.md |
| PLG21HR | `_default_employee_token` genera UUID | integrations.md |
| PLG22HR | `action_archive()` solo divide slots con `end_datetime >= departure_date+1day`. Si un employee es ar | integrations.md |
| PLG23HR | `has_slots` SQL computed con `EXISTS (... LIMIT 1)`: no devuelve count, solo bool. Filtros tipo "emp | integrations.md |
| PLG24HR | `department_id` related store. Cambiar `employee.department_id` | integrations.md |
| PLG25HR | `resource.resource.create(from_planning=True)` con `resource_type='user'` auto-crea `hr.employee` | integrations.md |
| PLG26PR | `_compute_project_id` se ejecuta cuando `template_id` cambia, | integrations.md |
| PLG27PR | `_check_company_id` en project.project | integrations.md |
| PLG28PR | `_read_group_project_id` group_expand exige contexto `planning_expand_project=True` Y filtros dates  | integrations.md |
| PLG29PR | `total_forecast_time` NO incluye slots con `start_datetime=False` (unscheduled de sale_planning) | integrations.md |
| PLG3 | Recurrence creates unassigned shifts when resource busy | gotchas.md |
| PLG30PR | `project_id` está en `_get_fields_breaking_publication` → cambiar project en slot publicado dispara  | integrations.md |
| PLG31PR | `action_project_forecast_from_project` setea `initialDate` al | integrations.md |
| PLG32PR | `_prepare_template_values` propaga `project_id` cuando se crea template desde slot, pero | integrations.md |
| PLG33FSM | NO hay bridge planning↔FSM | integrations.md |
| PLG34FSM | `_compute_planning_overlap` en FSM | integrations.md |
| PLG35FSM | Auto-plan (`planning.slot.auto_plan_ids`) en FSM project asigna por | integrations.md |
| PLG36FSM | Material resources (`resource_type='material'`) en planning útiles para FSM (vehículos, herramientas | integrations.md |
| PLG37FSM | `industry_fsm_sale` (timesheet to invoice) | integrations.md |
| PLG38SL | SQL constraint permite slots `start=NULL AND end=NULL` SOLO si `sale_line_id IS NOT NULL`. Borrar `s | integrations.md |
| PLG39SL | `ondelete='cascade'` en `sale_line_id` → cancelar SOL borra | integrations.md |
| PLG4 | Draft shifts invisible to non-managers | gotchas.md |
| PLG40SL | Auto-plan excluye `hr.employee.is_flexible=True` (`employee_ids_to_exclude += search([is_flexible=Tr | integrations.md |
| PLG41SL | `_compute_planning_hours_planned` filtra slots con `resource_id=False OR resource_type IN ('user','m | integrations.md |
| PLG42SL | `_planning_slot_generation` solo se ejecuta en | integrations.md |
| PLG43SL | Auto-plan en SO con `sale_order.state='cancel'` → retorna notification danger, NO bloquea otros slot | integrations.md |
| PLG44SL | Constraint `_check_planning_product_uom_is_time` valida en write/create de product pero NO se re-val | integrations.md |
| PLG45SL | `sale_line_plannable` related: si product.planning_enabled cambia a False post-confirm SOL, slots ex | integrations.md |
| PLG46TS | `_get_timesheet_domain` matcha por | integrations.md |
| PLG47TS | `slot_id` en `account.analytic.line` es | integrations.md |
| PLG48TS | `effective_hours` es | integrations.md |
| PLG49TS | `_gantt_progress_bar_project_id` usa `project.allocated_hours` como max_value, | integrations.md |
| PLG5 | Material resources auto-published | gotchas.md |
| PLG50TS | `can_open_timesheets` requiere `group_hr_timesheet_user` Y `env.user == slot.user_id` (o approver).  | integrations.md |
| PLG51TS | `action_open_timesheets` setea `default_unit_amount=allocated_hours` SOLO si `duration < 24h` (plann | integrations.md |
| PLG52TS | `percentage_hours` puede superar 100% (no cap). Employee timesheets >planning → ratio 150% → barra d | integrations.md |
| PLG53CT | `gantt_resource_employees_working_periods` | integrations.md |
| PLG54CT | Empleados | integrations.md |
| PLG55CT | Solo contratos con `state='draft' AND kanban_state='done'` se consideran "futuros válidos". Draft si | integrations.md |
| PLG56CT | `_get_working_hours_over_period` override | integrations.md |
| PLG57CT | `_get_misc_recurrence_stop`: si `initial_slot.start_datetime > end_contract` → NO limita (retorna re | integrations.md |
| PLG58CT | v19 sustituye `hr.contract` por `hr.version` (refactor HR enterprise). Bridge `planning_contract` v1 | integrations.md |
| PLG59HOL | `_compute_leave_warning` usa `min_date = min(start_datetime)` clamped a `today` → leaves | integrations.md |
| PLG6 | publication_warning is one-way | gotchas.md |
| PLG60HOL | `_search_is_absent` reconstruye TODOS los slots con `end_datetime > today` y employee → en BD con >5 | integrations.md |
| PLG61HOL | `resource.calendar.leaves.create` dispara recompute solo si el leave tiene `resource_id` o es global | integrations.md |
| PLG62HOL | `allocated_hours` recompute funciona, pero | integrations.md |
| PLG63HOL | `_handle_flexible_leave_interval`: solo procesa leaves con `request_unit_half=True` Y un único leave | integrations.md |
| PLG64HOL | NO cancela slots automáticamente en leave | integrations.md |
| PLG65HOL | Material resources excluidas del recompute (`resource_type != 'material'` en `_process_shifts_domain | integrations.md |
| PLG67SK | `planning.role` | integrations.md |
| PLG68SK | `_group_expand_resource_id` con domain de skill | integrations.md |
| PLG69SK | `employee_skill_ids` es | integrations.md |
| PLG7 | allocated_hours vs allocated_percentage circular dependency | gotchas.md |
| PLG70SK | Skills levels (`hr.employee.skill.level_id`) | integrations.md |
| PLG71T | `planning.slot.template.end_time < 24` (SQL constraint) — | integrations.md |
| PLG72T | `template_autocomplete_ids` se computa por slot individualmente con search limitada (limit=10). En B | integrations.md |
| PLG73T | `_prepare_template_values` | integrations.md |
| PLG74T | `recurrence_update='subsequent'` o `'all'` con cambios de `start_datetime/end_datetime/role/repeat_* | integrations.md |
| PLG75T | Material resources se publican automáticamente (PLG5 core) pero | integrations.md |
| PLG76T | `action_copy_previous_week` no copia slots con `was_copied=True` para evitar duplicados — pero | integrations.md |
| PLG77T | Cron `_cron_schedule_next` weekly: genera hasta `planning_generation_interval` (default 6 meses) | integrations.md |
| PLG78M | `_send_planning` usa `hr.employee.public` browse (sudo) para render template — solo expone campos no | integrations.md |
| PLG79M | Empleados con `work_email=False` | integrations.md |
| PLG8 | Copy Previous Week splits forecast slots | gotchas.md |
| PLG80M | `_send_shift_assigned` solo se envía si | integrations.md |
| PLG81M | `publication_warning=True` se setea en muchos field changes (`_get_fields_breaking_publication`) per | integrations.md |
| PLG82M | `action_self_unassign` requiere `company.planning_employee_unavailabilities='unassign'` AND `is_unas | integrations.md |
| PLG83M | `action_switch_shift` setea `request_to_switch=True` pero | integrations.md |
| PLG84M | `request_to_switch` se cancela automáticamente en write si cambia `resource_id`, `start_datetime`, o | integrations.md |
| PLG85R | Material resources con `calendar_id=False` → `_work_intervals_batch` retorna interval vacío o crash  | integrations.md |
| PLG86R | `from_planning` context flag determina behavior crítico (auto-create employee, calendar_id nullable) | integrations.md |
| PLG87R | `display_name` de material resource incluye `default_role_id.name` entre paréntesis (linea 74). Camb | integrations.md |
| PLG88R | Material resource sin calendar **siempre disponible | integrations.md |
| PLG89R | Auto-publish de material (PLG5 core) | integrations.md |
| PLG9 | Self-unassign has deadline | gotchas.md |
| PLG90R | `_send_planning` filter excluye material → si planning incluye humans + material en mismo publish, e | integrations.md |
| PLG91 | Recurrence is generated lazily, only 6 months ahead by default | gotchas.md |
| PLG92 | The recurrency cron is shipped with NO `interval_number` and NO `nextcall` (`planning_cron.xml` only | gotchas.md |
| PLG93 | `_repeat_slot` re-reads the LATEST slot of the recurrence as its template and copies it forward | gotchas.md |
| PLG94 | Recurring shift on a non-working day / public holiday is skipped silently. `_repeat_slot.can_slot_be | gotchas.md |
| PLG95 | `repeat_type='until'` boundary uses company calendar tz at midnight-max | gotchas.md |
| PLG96 | Editing recurrence parameters (repeat_interval/unit/type/until/number) in `write()` DELETES draft fu | gotchas.md |
| PLG97 | `allocated_hours` for an open (unassigned) shift uses the COMPANY calendar's `hours_per_day`, not an | gotchas.md |
| PLG98 | Flexible vs fully-flexible resources compute hours completely differently. Fully-flexible (`_is_full | gotchas.md |
| PLG99 | `_get_slot_duration` (the `duration` field) and `allocated_hours` are NOT the same number. `duration | gotchas.md |

## enterprise/pos_blackbox

| ID | Título | Fichero |
|----|--------|---------|
| PBB1 | Self-compiled pos_blackbox_be is NOT legally certified | gotchas.md |
| PBB10 | INSZ required for current user AND every potential cashier | gotchas.md |
| PBB11 | Registered orders write-locked except whitelist | gotchas.md |
| PBB12 | vat_letter cannot be SET/changed, but the guard is truthy-only | gotchas.md |
| PBB13 | Refund of WORK IN/OUT forbidden | gotchas.md |
| PBB14 | Cancel of registered order zeros tax_categories | gotchas.md |
| PBB15 | Order delete blocked once registered | gotchas.md |
| PBB16 | Letter assigned by amount only — group taxes confused | gotchas.md |
| PBB17 | Purchase taxes have no letter | gotchas.md |
| PBB18 | pos_blackbox_be.log immutable but no `_inherit = mail.thread` | gotchas.md |
| PBB19 | _create_log_description duplicates per pro-forma | gotchas.md |
| PBB2 | Cannot uninstall while ANY certified config exists | gotchas.md |
| PBB20 | INSZ validation has 2-pass for post-2000 births | gotchas.md |
| PBB21 | Clocked-in employees block session close | gotchas.md |
| PBB22 | cash_box_opening_number incremented manually | gotchas.md |
| PBB23 | Sequences auto-created on FDM link | gotchas.md |
| PBB24 | v19 self_order compatibility added | gotchas.md |
| PBB25 | v19 pos_blackbox_log_ip is a real model | gotchas.md |
| PBB26 | WORK IN/OUT orders are EXEMPT from the write-lock | gotchas.md |
| PBB27 | `backend_recomputation` context is the ONLY sanctioned write bypass | gotchas.md |
| PBB28 | Fiscal-position tax mapping must keep a letter on BOTH sides | gotchas.md |
| PBB29 | Correction counter is a separate manual increment | gotchas.md |
| PBB3 | IP cross-contamination locks workstations | gotchas.md |
| PBB30 | IP lock is asymmetric by certification state | gotchas.md |
| PBB31 | v19 sequences are company-agnostic (`company_id=False`) | gotchas.md |
| PBB32 | plu_hash is NOT from the FDM — it is computed client-side in v18 | gotchas.md |
| PBB33 | The Z/X report letters come from `tax_group_id.pos_receipt_label`, NOT `account.tax.identification_l | gotchas.md |
| PBB34 | Z report injects empty A/B/C/D rows and re-totals categories | gotchas.md |
| PBB35 | EVERY product.template create/delete is logged globally once the module is installed | gotchas.md |
| PBB36 | INSZ field is hidden behind hr.group_hr_manager and validated on BOTH models | gotchas.md |
| PBB37 | `_log_ip` reads geoip and is a no-op without a bound request | gotchas.md |
| PBB38 | certified_blackbox_identifier is a STORED compute that fires a log + sequence side-effects on link | gotchas.md |
| PBB4 | Loyalty programs FULLY blocked | gotchas.md |
| PBB5 | WORK IN/OUT products require single, SHARED pos_categ_ids | gotchas.md |
| PBB6 | Cash rounding 0.05 HALF-UP HARDCODED | gotchas.md |
| PBB7 | Auto-print + skip-preview MANDATORY | gotchas.md |
| PBB8 | Self-order with payment INCOMPATIBLE | gotchas.md |
| PBB9 | Floors cannot be shared across configs | gotchas.md |

## enterprise/quality

| ID | Título | Fichero |
|----|--------|---------|
| QCAP1 | ¿Puntos de control por producto/categoría/operación? | capabilities.md |
| QCAP10 | ¿Equipos de calidad y asignación? | capabilities.md |
| QCAP11 | ¿Control de calidad mediante hoja de cálculo (spreadsheet)? | capabilities.md |
| QCAP12 | ¿Control en picking por lotes / olas (batch)? | capabilities.md |
| QCAP13 | ¿Control de calidad en reparaciones? | capabilities.md |
| QCAP14 | ¿Registrar la medida/control por lote o número de serie? | capabilities.md |
| QCAP15 | ¿Al fallar, mover a ubicación de rechazo / gestionar el fallo? | capabilities.md |
| QCAP2 | ¿Tipos de control (pass/fail, medida con tolerancia, instrucción, foto, worksheet)? | capabilities.md |
| QCAP3 | ¿Controles disparados en recepción/entrega/fabricación? | capabilities.md |
| QCAP4 | ¿Alertas de calidad con etapas y equipo? | capabilities.md |
| QCAP5 | ¿Controles en órdenes de fabricación y por orden de trabajo? | capabilities.md |
| QCAP6 | ¿Hojas de trabajo configurables como control? | capabilities.md |
| QCAP7 | ¿Medición/captura automática vía IoT? | capabilities.md |
| QCAP8 | ¿Frecuencia/periodicidad del control? | capabilities.md |
| QCAP9 | ¿Un control pendiente bloquea el flujo? | capabilities.md |
| QG1 | quality vs quality_control confusion | gotchas.md |
| QG10 | on_demand frequency excluded from auto-creation | gotchas.md |
| QG11 | check deduplication key is (point_id, team_id, product_id) | gotchas.md |
| QG12 | move_line checks created on line.create() with quantity != 0 | gotchas.md |
| QG13 | context flag 'no_checks' suppresses check creation | gotchas.md |
| QG14 | MRP move_line checks bypass normal SML path | gotchas.md |
| QG15 | wildcard points excluded for raw materials | gotchas.md |
| QG16 | quality checks BLOCK picking validation (two-stage) | gotchas.md |
| QG17 | show_validate hidden when checks pending | gotchas.md |
| QG18 | MRP validation blocks differently from wizard vs direct | gotchas.md |
| QG19 | repair orders also block on quality | gotchas.md |
| QG2 | stock.group_stock_user implies quality user | gotchas.md |
| QG20 | failure location rerouting is complex and fragile | gotchas.md |
| QG21 | partial failure creates invisible new auto-PASSED check | gotchas.md |
| QG22 | failure location only works for measure_on='move_line' | gotchas.md |
| QG23 | MTO links broken on failure rerouting | gotchas.md |
| QG24 | worksheet success conditions use ast.literal_eval | gotchas.md |
| QG25 | worksheet opens INSTEAD of wizard, not alongside | gotchas.md |
| QG26 | spreadsheet check_cell defaults to A1 | gotchas.md |
| QG27 | spreadsheet revisions auto-cleaned after 1 day of inactivity | gotchas.md |
| QG28 | SPC is monitoring only — no alerts or actions | gotchas.md |
| QG29 | Welford algorithm includes ALL non-none checks (pass AND fail) | gotchas.md |
| QG3 | quality.check users cannot delete (but sudo bypasses it) | gotchas.md |
| QG30 | tolerance_max auto-set from norm on first change only | gotchas.md |
| QG31 | measure precision is 'Quality Tests' (2 decimals) | gotchas.md |
| QG32 | cancel deletes only pending checks | gotchas.md |
| QG33 | move cancellation requires ALL product moves cancelled | gotchas.md |
| QG34 | backorder deletes original picking's pending checks | gotchas.md |
| QG35 | quality.alert.team company_id can be False (shared) | gotchas.md |
| QG36 | quality.check record rule has OR condition | gotchas.md |
| QG37 | sequences are cross-company | gotchas.md |
| QG38 | alert stages shared unless team-filtered | gotchas.md |
| QG39 | title and note are snapshotted at creation, not live-related | gotchas.md |
| QG39M | `mrp.production._compute_check` NO filtra por `_is_checkable()` (a diferencia de stock.picking). Cue | integrations.md |
| QG4 | test types are records, not a Selection field | gotchas.md |
| QG40M | `_create_quality_checks_for_mo()` crea checks `measure_on='move_line'` usando `production.product_id | integrations.md |
| QG41M | `quality.point` con `operation_id` (WO-step) Y `measure_frequency_type='on_demand'` → `UserError` en | integrations.md |
| QG42M | `qty_producing=0` antes de `check_quality()` → `UserError "You cannot perform a quality check if the | integrations.md |
| QG43 | a point with no product AND no category matches EVERY product | gotchas.md |
| QG43M | `pre_button_mark_done()` con `from_wizard=True` retorna | integrations.md |
| QG44 | quality.alert.team email alias defaults company_id onto created alerts | gotchas.md |
| QG44M | `quality.point.product_ids` con `domain="[('type', '=', 'consu')]"` excluye servicios pero NO valida | integrations.md |
| QG45 | alert date_close set on ANY write that lands in a done stage | gotchas.md |
| QG45M | Cambio de `lot_id` en `stock.move.line` propaga a `register_consumed_materials`/`register_byproducts | integrations.md |
| QG46 | lot auto-fills from move line and can override a manually set lot | gotchas.md |
| QG46M | `_action_confirm_mo_backorders()` re-crea checks para `move_raw_ids \| move_finished_ids` (`quality_m | integrations.md |
| QG47 | qty_passed/qty_failed are stored and recompute on every state change | gotchas.md |
| QG47P | `_compute_show_validate` (v18) oculta el botón Validate cuando hay checks pendientes (`quality_contr | integrations.md |
| QG48 | SML write creates checks only when the line has NO checks yet | gotchas.md |
| QG48P | `res.partner` (vendor) NO tiene `quality_alert_count` ni `quality_alert_ids` nativo. La trazabilidad | integrations.md |
| QG49P | `quality.check.do_alert()` NO propaga `partner_id` al `quality.alert` creado (`quality_control/model | integrations.md |
| QG5 | picking_type_ids is required on quality.point | gotchas.md |
| QG50P | `purchase.order.button_cancel()` → `picking.action_cancel()` → unlink checks `quality_state='none'`, | integrations.md |
| QG51P | Vendor reabre PO modificando líneas → re-creación de moves pero `_create_quality_checks` deduplica p | integrations.md |
| QG52P | Quality point con `measure_frequency_type='random'` usa `random.random()` | integrations.md |
| QG53P | `quality.point.product_category_ids` con `parent_of` matching → punto en categoría "All" aplica a TO | integrations.md |
| QG54S | `_compute_check_state` en SML NO contempla checks `pass` mixto con `fail`: si una línea tiene 2 chec | integrations.md |
| QG55S | `_create_quality_check_at_write({'quantity': N})` crea check si la línea no tiene ya. Pero | integrations.md |
| QG56S | SML con `quantity=0` NO crea checks (`_filter_move_lines_applicable_for_quality_check` filtra). Si l | integrations.md |
| QG57S | `_unlink_quality_check` en SML.unlink() solo borra checks que `_check_to_unlink()` retorne True. El  | integrations.md |
| QG58S | Wizard `quality.check.on.demand` se cierra si `picking.state in ('draft', 'done', 'cancel')` con Use | integrations.md |
| QG59S | `failure_location_id` rerouting (`_move_line_to_failure_location`) NO valida que la location esté en | integrations.md |
| QG6 | product domain is type='consu' only | gotchas.md |
| QG60S | `stock.lot.quality_check_qty` y `quality_alert_qty` están | integrations.md |
| QG61FSM | Sin bridge: FSM task `project.task` NO tiene `quality_check_ids`. Para QC en intervenciones, el técn | integrations.md |
| QG62FSM | Worksheets FSM (`project.task.worksheet_template_id`) y worksheets Quality (`quality.check.worksheet | integrations.md |
| QG63FSM | Custom dev típico: añadir `quality_check_ids` a `project.task` con domain `[('product_id', '=', task | integrations.md |
| QG64FSM | Si el cliente requiere QC en FSM, el bridge actual recomendado es | integrations.md |
| QG65MN | `quality.alert` NO tiene `maintenance_request_id` field. Para vincular alert → maintenance request h | integrations.md |
| QG66MN | `maintenance.request.equipment_id` (M2o `maintenance.equipment`) NO tiene `quality_alert_ids` relate | integrations.md |
| QG67MN | Workcenter de MRP (`mrp.workcenter`) tiene `maintenance_request_count` (vía `mrp_maintenance`), pero | integrations.md |
| QG68MN | `quality.alert.reason_id` (M2o `quality.reason`) tiene records noupdate predefinidos: "Workcenter Fa | integrations.md |
| QG69MN | Si el cliente necesita escalation Quality→Maintenance: server.action en `quality.alert.write` que de | integrations.md |
| QG7 | team is required everywhere, and the default is company-scoped | gotchas.md |
| QG70IO | `qcp_test_type` mapping NO incluye `barcode_scanner`, `keyboard`, `display`. Si añades device de tip | integrations.md |
| QG71IO | `ondelete='restrict'` en `point.device_id` impide borrar `iot.device` si está usado en algún quality | integrations.md |
| QG72IO | `quality.check.ip` y `identifier` son | integrations.md |
| QG73IO | `iot.trigger` con `action='measure'` (quality_mrp_workorder_iot) requiere `iot.device.qcp_test_type= | integrations.md |
| QG74IO | `device_name` related con `size=30` trunca silenciosamente nombres de IoT devices largos en views. C | integrations.md |
| QG75IO | Domain de `device_id` en quality.point: `['\|', ('company_id', '=', False), ('company_id', '=', compa | integrations.md |
| QG76IO | `quality_control_iot` bridge tiene **0 archivos en `models/` | integrations.md |
| QG77BT | `_add_to_wave_post_picking_split_hook` solo deshace checks `measure_on in ('operation', 'product')`  | integrations.md |
| QG78BT | `_create_quality_checks` tras split NO chequea check_execute_now de nuevo. Si tienes `measure_freque | integrations.md |
| QG79BT | `quality.check.batch_id` es | integrations.md |
| QG8 | random frequency is truly random — not reproducible | gotchas.md |
| QG80BT | `batch.quality_check_todo` no es | integrations.md |
| QG81BT | `batch.action_open_quality_check_wizard()` agrega checks de TODOS los pickings | integrations.md |
| QG82BT | Cancelación de batch NO cancela los pickings individuales (es un acción de batch.unlink). Si unlinke | integrations.md |
| QG83AL | `date_close` se setea cuando `stage.done=True` pero NO se limpia si vuelves a un stage non-done. Rea | integrations.md |
| QG84AL | `date_assign` NO se setea automáticamente al crear el alert. Solo es Datetime field editable manualm | integrations.md |
| QG85AL | Email alias: `message_new` override (`quality_control/models/quality.py:551-562`, `QualityAlert.name | integrations.md |
| QG86AL | Email subject con HTML/Markup en `description = msg_dict['body']` (HTML del email) → renderizado cor | integrations.md |
| QG87AL | `_get_default_stage_id` busca stage con `team_ids=False` primero, luego con team match (`OR`). Si ti | integrations.md |
| QG88AL | `do_alert()` propaga `team_id` desde el check, pero `check.team_id` puede ser distinto a `picking.te | integrations.md |
| QG89AL | `partner_id` (vendor) en alert NO tiene domain restriction (`check_company=False` para partner). Mul | integrations.md |
| QG9 | periodical frequency checks create_date, not control_date | gotchas.md |
| QG90WS | `ast.literal_eval(worksheet_success_conditions)` con string mal formado → `ValueError` no capturado, | integrations.md |
| QG91WS | `worksheet.template` con `_get_quality_check_access_all_groups` = `quality.group_quality_manager` (n | integrations.md |
| QG92WS | `worksheet_template_id` en check viene del | integrations.md |
| QG93WS | `worksheet_count` recomputed solo cuando `worksheet_template_id` cambia. Si el usuario crea registro | integrations.md |
| QG94WS | `quality_mrp_workorder_worksheet` override (`action_worksheet_check`): si `workorder_id` y `not from | integrations.md |
| QG95WS | Modelo dinámico `x_quality_check_<N>` se crea por `worksheet.template`. Si templates dupes (mismo no | integrations.md |
| QG96WS | `worksheet_success_conditions` por defecto vacía → `ast.literal_eval('') → SyntaxError`. Mitigation: | integrations.md |

## enterprise/rental

| ID | Título | Fichero |
|----|--------|---------|
| RENTCAP1 | ¿Alquilar productos con periodo (pickup/return)? | capabilities.md |
| RENTCAP10 | ¿Estado del flujo y cantidades recogida/devuelta? | capabilities.md |
| RENTCAP11 | ¿Informe/análisis de alquiler? | capabilities.md |
| RENTCAP12 | ¿Alquiler con tracking por número de serie? | capabilities.md |
| RENTCAP13 | ¿Tiempo de preparación/margen entre alquileres (padding)? | capabilities.md |
| RENTCAP14 | ¿Firmar el contrato de alquiler? | capabilities.md |
| RENTCAP15 | ¿Integrar alquiler con planning / proyecto / CRM? | capabilities.md |
| RENTCAP2 | ¿Tarifas escalonadas por duración (hora/día/semana/mes)? | capabilities.md |
| RENTCAP3 | ¿Fianza / depósito de seguridad? | capabilities.md |
| RENTCAP4 | ¿Penalización por devolución tardía (late fees)? | capabilities.md |
| RENTCAP5 | ¿Stock real: pickup/return generan movimientos (sale_stock_renting)? | capabilities.md |
| RENTCAP6 | ¿Disponibilidad / evitar overbooking en el periodo? | capabilities.md |
| RENTCAP7 | ¿Vender y alquilar en el mismo pedido? | capabilities.md |
| RENTCAP8 | ¿Alquiler desde eCommerce/web? | capabilities.md |
| RENTCAP9 | ¿Facturación del alquiler? | capabilities.md |
| RNTG1 | `is_rental_order` is sticky once set | gotchas.md |
| RNTG10 | `rent_ok` incompatible with subscription products | gotchas.md |
| RNTG11 | Combo products excluded from pickup/return | gotchas.md |
| RNTG12 | Rental returns are an internal move, never a refund | gotchas.md |
| RNTG13 | Pickup more than reserved auto-expands qty | gotchas.md |
| RNTG14 | Orderpoint ignores rental returns + custom in-progress qty | gotchas.md |
| RNTG15 | `group_rental_stock_picking` is a user-group, not a company setting | gotchas.md |
| RNTG16 | Switching picking mode mid-operation breaks existing orders | gotchas.md |
| RNTG17 | `reservation_begin` ignores preparation_time without sale_stock_renting | gotchas.md |
| RNTG18 | Delay product auto-creates itself and silently stops if deactivated | gotchas.md |
| RNTG19 | `min_extra_hour` grace period checked in two places with different math | gotchas.md |
| RNTG2 | `is_rental` stored to survive product edits | gotchas.md |
| RNTG20 | Account eligibility returns False for rental lines (revenue never hits COGS/valuation) | gotchas.md |
| RNTG21 | Daily price in report ignores time components | gotchas.md |
| RNTG22 | Overnight recurrence is v19-only and stores as 24h internally | gotchas.md |
| RNTG23 | `account.move.line` eligibility hook renamed v18→v19 | gotchas.md |
| RNTG24 | Return picking is a real make_to_order chain, not a copy | gotchas.md |
| RNTG25 | Two pricing modes share NO code path — wizard vs picking diverge in `write()` | gotchas.md |
| RNTG26 | Rental moves are force-merged onto the sale picking in hybrid orders | gotchas.md |
| RNTG27 | _read_group_product_ids caps rows at 80; SQL _color() builds colors. | core.md |
| RNTG28 | `available_reserved_lots` is a WARNING-only flag, never blocks | gotchas.md |
| RNTG29 | Setting all `pickedup_lot_ids` silently unreserves the non-picked lots | gotchas.md |
| RNTG3 | `is_rental` only True in `in_rental_app` context | gotchas.md |
| RNTG30 | Manually editing pickedup/returned M2M creates phantom stock moves & double-counts on invoice | gotchas.md |
| RNTG31 | `_action_assign` reassigns reserved serials only when the quant is free | gotchas.md |
| RNTG32 | Rental `qty_delivered` is computed from done OUTGOING moves in picking mode, manual in wizard mode | gotchas.md |
| RNTG33 | `_action_done` on a return move auto-generates the delay line and bumps `qty_returned` | gotchas.md |
| RNTG34 | Rental pickings always create backorders | gotchas.md |
| RNTG35 | A non-rental move on a rental order gets its `location_dest_id` rewritten to `location_final_id` | gotchas.md |
| RNTG36 | `_is_incoming` treats rental_loc→internal as incoming (valuation/forecast) | gotchas.md |
| RNTG37 | Availability falls back to first-day forecast for FUTURE rentals (not the min over the window) | gotchas.md |
| RNTG38 | Availability window bound depends on `rental_pivot_date` (preparation_time only counted then) | gotchas.md |
| RNTG39 | Early pickups/returns shift quantities to `now`, not to the actual event time | gotchas.md |
| RNTG4 | `rental_status` strings are inverted | gotchas.md |
| RNTG40 | `product.pricing._compute_price` returns the flat price for duration≤0 or fixed-price (duration=0) r | gotchas.md |
| RNTG41 | `qty_in_rent` = delivered − returned over CONFIRMED rental lines, no stock moves involved | gotchas.md |
| RNTG42 | With pickings OFF, `_action_launch_stock_rule` makes rental lines create NO procurements at all | gotchas.md |
| RNTG43 | Cannot change product on a rental line once it has non-cancelled moves | gotchas.md |
| RNTG44 | Rental price recompute is filtered by `is_rental` on BOTH ends | gotchas.md |
| RNTG45 | `_compute_product_updatable` forces rental lines to stay editable | gotchas.md |
| RNTG46 | The auto-created delay product is created ACTIVE and shared, never auto-deactivated | gotchas.md |
| RNTG5 | Pricelist rules completely bypassed for rentals | gotchas.md |
| RNTG6 | Best pricing picks cheapest absolute, not most appropriate | gotchas.md |
| RNTG7 | Month = 31-day approximation in unit conversion, but `relativedelta` for month count | gotchas.md |
| RNTG8 | `remaining_hours` uses ceil producing unintuitive results | gotchas.md |
| RNTG9 | Lot tracking NOT supported — serial only | gotchas.md |

## enterprise/sale_amazon

| ID | Título | Fichero |
|----|--------|---------|
| AMZCAP1 | ¿Conector con Amazon (cuenta, SP-API)? | capabilities.md |
| AMZCAP10 | ¿Publicar stock disponible hacia Amazon? | capabilities.md |
| AMZCAP2 | ¿Importar pedidos de Amazon como `sale.order`? | capabilities.md |
| AMZCAP3 | ¿FBA vs FBM con flujos distintos? | capabilities.md |
| AMZCAP4 | ¿Crear contacto desde el pedido Amazon? | capabilities.md |
| AMZCAP5 | ¿Importar comisiones/fees de Amazon como gasto? | capabilities.md |
| AMZCAP6 | ¿Multi-marketplace / varios países? | capabilities.md |
| AMZCAP7 | ¿Confirmar el envío hacia Amazon? | capabilities.md |
| AMZCAP8 | ¿Mapeo de productos por SKU? | capabilities.md |
| AMZCAP9 | ¿Cron de sincronización de pedidos y cancelaciones? | capabilities.md |
| AMZG1 | refresh_token stored plaintext | gotchas.md |
| AMZG10 | Anonymous partner pool grows unbounded | gotchas.md |
| AMZG11 | SKU mismatch → silent fallback | gotchas.md |
| AMZG12 | Tax recomputation diverges from Amazon | gotchas.md |
| AMZG13 | Gift wrap creates separate lines per item | gotchas.md |
| AMZG14 | Multi-currency: FX captured at sync time | gotchas.md |
| AMZG15 | Refund / item cancellation not re-synced | gotchas.md |
| AMZG16 | Buyer cancellation after FBM picking done | gotchas.md |
| AMZG17 | Address change at Amazon post-sync | gotchas.md |
| AMZG18 | DE address swap is hard-coded | gotchas.md |
| AMZG19 | Status filter is strict | gotchas.md |
| AMZG2 | Refresh token revocation = failure surfaced only by email | gotchas.md |
| AMZG20 | Forced picking sync on "Shipped" update | gotchas.md |
| AMZG21 | Multi-line one-to-one mapping | gotchas.md |
| AMZG22 | FBA inventory NOT actively synced (v18) | gotchas.md |
| AMZG23 | FBA stock move auto-done at order time | gotchas.md |
| AMZG24 | One FBA location per account (Pan-EU FBA loss) | gotchas.md |
| AMZG25 | FBM late ship confirmation hurts scorecard | gotchas.md |
| AMZG26 | Wrong tracking number cannot be corrected via API | gotchas.md |
| AMZG27 | Partial-line shipment blocked | gotchas.md |
| AMZG28 | No automatic return workflow | gotchas.md |
| AMZG29 | Carrier code mapping fallback | gotchas.md |
| AMZG3 | Wrong `base_marketplace_id` → wrong region endpoint | gotchas.md |
| AMZG30 | Carrier + tracking ref mandatory for FBM | gotchas.md |
| AMZG31 | Ship-from address validation strict | gotchas.md |
| AMZG32 | `UNIQUE(account, sku)` excludes marketplace | gotchas.md |
| AMZG33 | No ASIN field on products | gotchas.md |
| AMZG34 | `amazon_feed_ref` JSON cache staleness (v18) | gotchas.md |
| AMZG35 | Default products archived but auto-restored | gotchas.md |
| AMZG36 | New marketplaces not auto-discovered | gotchas.md |
| AMZG37 | Currency mismatch creates fresh pricelist | gotchas.md |
| AMZG38 | Offer search in-memory | gotchas.md |
| AMZG39 | AvaTax bridge only clears default FP | gotchas.md |
| AMZG4 | Hardcoded app ID | gotchas.md |
| AMZG40 | Spanish simplified invoice forced for all amounts | gotchas.md |
| AMZG41 | No bridge for Modelo 349 / OSS | gotchas.md |
| AMZG42 | Custom marketplace warehouse routing requires override | gotchas.md |
| AMZG43 | `amazon_email` → `email` partner field rename | gotchas.md |
| AMZG44 | `amazon.offer.amazon_channel` is new in v19 | gotchas.md |
| AMZG45 | `is_follow_up_displayed` removed in v19 | gotchas.md |
| AMZG46 | `tax_id` → `tax_ids` field rename (Odoo-wide) | gotchas.md |
| AMZG47 | `auto_join` → `bypass_search_access` (Odoo-wide) | gotchas.md |
| AMZG48 | `stock.move` extended in v19 | gotchas.md |
| AMZG49 | v19 adds ghost-listing prevention | gotchas.md |
| AMZG5 | No test/sandbox mode | gotchas.md |
| AMZG6 | 429 rate limit defers remaining batch | gotchas.md |
| AMZG7 | RDT failure invalidates whole sync | gotchas.md |
| AMZG8 | `last_orders_sync` is user-editable | gotchas.md |
| AMZG9 | Token expiry mid-run | gotchas.md |

## enterprise/sign

| ID | Título | Fichero |
|----|--------|---------|
| SIGNCAP1 | ¿Firma de PDF con campos arrastrables (firma, texto, checkbox…)? | capabilities.md |
| SIGNCAP10 | ¿Campos autocompletados desde datos del partner/usuario? | capabilities.md |
| SIGNCAP11 | ¿Pago + firma combinados en el mismo flujo? | capabilities.md |
| SIGNCAP12 | ¿Pedir firma desde un documento de Documents? | capabilities.md |
| SIGNCAP13 | ¿Rechazar / declinar la firma con motivo? | capabilities.md |
| SIGNCAP14 | ¿Enlace público compartible para firmar (share link)? | capabilities.md |
| SIGNCAP15 | ¿Tipos de campo avanzados (selección, radio, opciones)? | capabilities.md |
| SIGNCAP16 | ¿Autenticación reforzada tipo eID (validez legal alta)? | capabilities.md |
| SIGNCAP2 | ¿Múltiples firmantes con orden y roles? | capabilities.md |
| SIGNCAP3 | ¿Plantillas reutilizables de documento a firmar? | capabilities.md |
| SIGNCAP4 | ¿Enviar a firmar a externos sin cuenta (acceso por token)? | capabilities.md |
| SIGNCAP5 | ¿Autenticación del firmante (SMS / identidad)? | capabilities.md |
| SIGNCAP6 | ¿Audit trail / certificado / log inalterable? | capabilities.md |
| SIGNCAP7 | ¿Recordatorios automáticos y caducidad? | capabilities.md |
| SIGNCAP8 | ¿Firma en persona / in situ? | capabilities.md |
| SIGNCAP9 | ¿Firmar contratos de RRHH en el onboarding? | capabilities.md |
| SIGNCG1 | Crear un template de firma (subir PDF + colocar campos) | config_guide.md |
| SIGNCG10 | Plantilla con campos selection / radio | config_guide.md |
| SIGNCG11 | Branding y T&C de los emails (multi-company) | config_guide.md |
| SIGNCG12 | Enlazar una firma a un registro (reference_doc) | config_guide.md |
| SIGNCG13 | Control de acceso a plantillas (authorized users/groups) | config_guide.md |
| SIGNCG14 | PDF de origen protegido con contraseña | config_guide.md |
| SIGNCG15 | Verificar integridad / auditoría legal | config_guide.md |
| SIGNCG16 | Parámetros del sistema relevantes | config_guide.md |
| SIGNCG2 | Definir roles de firmante (responsables) | config_guide.md |
| SIGNCG3 | Configurar campos de auto-relleno (auto_field) | config_guide.md |
| SIGNCG4 | Enviar una petición de firma (wizard) | config_guide.md |
| SIGNCG5 | Firma secuencial (orden de firmantes) | config_guide.md |
| SIGNCG6 | Autenticación extra por SMS (OTP) | config_guide.md |
| SIGNCG7 | Autenticación por itsme® (BE/NL) | config_guide.md |
| SIGNCG8 | Recordatorios automáticos | config_guide.md |
| SIGNCG9 | Firma por link público (Share / shared request) | config_guide.md |
| SNG1 | El template se congela para siempre tras la primera request | gotchas.md |
| SNG10 | `auto_field` es SOLO de res.partner en v18 | gotchas.md |
| SNG11 | `auth_method='itsme'` solo existe con `sign_itsme` instalado | gotchas.md |
| SNG12 | `sign.request.item.value` sin acceso de lectura para usuarios | gotchas.md |
| SNG13 | `reference_doc` solo postea/adjunta si el modelo es `mail.thread` | gotchas.md |
| SNG14 | El rechazo NO es un estado en v18 (a diferencia de v17) | gotchas.md |
| SNG15 | Requests `shared` solo se borran manualmente, NO por autovacuum | gotchas.md |
| SNG16 | Atributo de geoloc/IP del log oculto para usuarios normales | gotchas.md |
| SNG17 | El primer hash de la cadena es el del PDF del template, NO del documento enviado | gotchas.md |
| SNG18 | El OTP por SMS es predecible en rango y se resetea en cada reenvío | gotchas.md |
| SNG19 | La firma comparte siempre se DUPLICA al firmar; la original nunca se cierra | gotchas.md |
| SNG2 | Reasignar firmante regenera tokens y reglas estrictas | gotchas.md |
| SNG20 | Constraint: un template solo se comparte por link si tiene UN solo rol | gotchas.md |
| SNG21 | Selection items sin opciones bloquean el envío | gotchas.md |
| SNG22 | Radio buttons: agrupación automática en un radio.set, y la copia crea sets nuevos | gotchas.md |
| SNG23 | PDF de origen encriptado → el firmante debe meter la contraseña para descargar el | gotchas.md |
| SNG24 | `validity` por defecto del wizard = hoy + 6 meses, NO infinito | gotchas.md |
| SNG25 | El orden secuencial de firma se activa con `set_sign_order` en el wizard, no en el | gotchas.md |
| SNG26 | El email de finalización adjunta el PDF al chatter de la request Y al reference_doc, | gotchas.md |
| SNG27 | `sms_number` es escribible y se puede cambiar al disparar el OTP | gotchas.md |
| SNG28 | `attachment_id.copy()` al crear template si el PDF ya está en otro record | gotchas.md |
| SNG3 | Logs y requests firmadas son inmutables / no borrables | gotchas.md |
| SNG4 | Cancelar regenera el access_token de la request | gotchas.md |
| SNG5 | `toggle_active` (archivar) cancela las requests pendientes | gotchas.md |
| SNG6 | SMS sin créditos → firma sin autenticación (bypass silencioso) | gotchas.md |
| SNG7 | `communication_company_id` NO fija el dominio del link | gotchas.md |
| SNG8 | El link de email caduca independiente de `validity` | gotchas.md |
| SNG9 | Escribir `validity` en pasado expira la request al instante | gotchas.md |

## enterprise/social

| ID | Título | Fichero |
|----|--------|---------|
| SOCAP1 | ¿Cuentas de redes y media? | capabilities.md |
| SOCAP10 | ¿Tracking UTM de los enlaces? | capabilities.md |
| SOCAP2 | ¿Publicar en varias redes a la vez? | capabilities.md |
| SOCAP3 | ¿Programar publicaciones? | capabilities.md |
| SOCAP4 | ¿Bandeja de interacciones/comentarios (streams)? | capabilities.md |
| SOCAP5 | ¿Responder comentarios desde Odoo? | capabilities.md |
| SOCAP6 | ¿Métricas/insights por cuenta y post? | capabilities.md |
| SOCAP7 | ¿Crear lead/oportunidad desde una interacción social? | capabilities.md |
| SOCAP8 | ¿Push notifications web a visitantes? | capabilities.md |
| SOCAP9 | ¿Qué redes soporta? | capabilities.md |
| SOG1 | UTM source auto-generated per post — enables post-level sales tracking | gotchas.md |
| SOG10 | Push needs Firebase + visitor opt-in | gotchas.md |
| SOG11 | Push timezone processing is incremental | gotchas.md |
| SOG12 | Individual live_post failures don't fail the parent | gotchas.md |
| SOG13 | Message edits update the utm.source NAME (not ID) | gotchas.md |
| SOG14 | Multi-company: accounts with `company_id` empty are shared across ALL companies | gotchas.md |
| SOG15 | social_crm partner matching requires >3 chars AND exactly one hit | gotchas.md |
| SOG16 | Facebook 90-day insights lookback, looped 4× for 360 days | gotchas.md |
| SOG18 | YouTube refreshes the token before EVERY operation | gotchas.md |
| SOG19 | Token-refresh failure disconnects the account; success silently reconnects it | gotchas.md |
| SOG2 | Stats are NOT auto-computed — manual refresh or no cron at all | gotchas.md |
| SOG20 | Facebook / LinkedIn / Twitter tokens have NO refresh logic | gotchas.md |
| SOG21 | `is_media_disconnected` is set by `_action_disconnect_accounts` and rarely auto-cleared | gotchas.md |
| SOG22 | There is NO cron polling streams — refresh is manual/UI-only | gotchas.md |
| SOG23 | `refresh_all` swallows only ConnectionError/Timeout per stream | gotchas.md |
| SOG24 | Stream-post dedup is provider-specific by external post-id | gotchas.md |
| SOG25 | Feed "time ago" only for posts under 12h | gotchas.md |
| SOG26 | `_action_post` does an explicit `cr.commit()` mid-flow | gotchas.md |
| SOG27 | Scheduled posts fire near their datetime, not just on the hourly cron | gotchas.md |
| SOG28 | Cannot schedule in the past; cannot reschedule a posted post | gotchas.md |
| SOG29 | Every account auto-creates a required utm.medium (restrict ondelete) | gotchas.md |
| SOG3 | Instagram posting is ASYNC with retry via cron | gotchas.md |
| SOG30 | account_ids / message constraints run under sudo and per-media length | gotchas.md |
| SOG31 | `account_ids` auto-fills only when ≤3 accounts exist | gotchas.md |
| SOG32 | Twitter @mentions are KEPT by default | gotchas.md |
| SOG33 | Creating a live_post immediately re-evaluates parent completion | gotchas.md |
| SOG4 | Instagram requires JPEG only with strict aspect ratio | gotchas.md |
| SOG5 | Twitter uses OAuth 1.0a (HMAC-SHA1) | gotchas.md |
| SOG6 | YouTube limited to ONE account per post | gotchas.md |
| SOG7 | Facebook GIFs use a DIFFERENT host + the video endpoint | gotchas.md |
| SOG8 | LinkedIn URN format and 4KB API limit | gotchas.md |
| SOG9 | LinkedIn link preview fetches og:image separately | gotchas.md |

## enterprise/spreadsheet

| ID | Título | Fichero |
|----|--------|---------|
| SPRGE1 | Concurrent edit conflict has NO server-side merge | gotchas.md |
| SPRGE10 | Dashboard `group_ids` controls visibility but NOT source-model access | gotchas.md |
| SPRGE11 | Editing a built-in dashboard is lost on `-u`; all managers edit all dashboards | gotchas.md |
| SPRGE12 | XLSX import is capped (1000 files / 50 MB unzipped) and silently truncated/blocked | gotchas.md |
| SPRGE13 | Image upload has no client-side size guard → 413 silently fails | gotchas.md |
| SPRGE14 | `write(spreadsheet_binary_data=...)` silently nukes all collab data unless `preserve_spreadsheet_rev | gotchas.md |
| SPRGE15 | `restore_spreadsheet_version` unlinks by revision | gotchas.md |
| SPRGE16 | `current_revision_uuid` is per-transaction and MUST NOT be reused to dispatch | gotchas.md |
| SPRGE17 | `_copy_revisions_to` re-checks write access on the TARGET and strips comment commands | gotchas.md |
| SPRGE18 | `fork_history` return value differs v18 ↔ v19 (breaks code that reads `params.next`) | gotchas.md |
| SPRGE19 | Copying a spreadsheet (or instantiating a template) silently strips ALL comments | gotchas.md |
| SPRGE2 | One child per parent → branching is structurally impossible (intentional) | gotchas.md |
| SPRGE20 | Cell-thread access is gated by the HOST record's read check, not by the thread's own ACL | gotchas.md |
| SPRGE21 | Cell position lives in the JSON, not on `spreadsheet.cell.thread` → orphaned threads on out-of-band  | gotchas.md |
| SPRGE22 | Comment-mention emails route through a fixed mail layout and an `act_url`, not a client action | gotchas.md |
| SPRGE23 | XLSX import silently drops everything outside `SUPPORTED_PATHS` | gotchas.md |
| SPRGE24 | Image attachments are deep-copied AND re-pathed on every copy/fork; missing `_post_add_create` break | gotchas.md |
| SPRGE25 | `_contains_live_data` scans snapshot AND replayed revision commands | gotchas.md |
| SPRGE26 | Thumbnails are per-user `ir.attachment`s, never derived from the file | gotchas.md |
| SPRGE27 | `spreadsheet.contributor` "recently opened" is `handler=='spreadsheet'` only and needs write+create  | gotchas.md |
| SPRGE3 | Snapshot only fires after 2 h of inactivity AND only if the joiner can write | gotchas.md |
| SPRGE4 | Snapshot revision-id mismatch raises (not silently ignored) | gotchas.md |
| SPRGE5 | Revision GC is permanent and silent: 365-day inactive purge (keys on `write_date`, not `create_date` | gotchas.md |
| SPRGE6 | Frozen spreadsheet rejects writes at THREE layers, all raising AccessError/return False | gotchas.md |
| SPRGE7 | Public share with live Odoo data → error page, not empty cells | gotchas.md |
| SPRGE8 | Read-only token still lets the viewer download the full JSON + XLSX | gotchas.md |
| SPRGE9 | Portal/share users can NEVER hold edit role on a spreadsheet | gotchas.md |

## enterprise/stock_barcode

| ID | Título | Fichero |
|----|--------|---------|
| BG1 | `_get_stock_barcode_data()` fetches ALL UoMs | gotchas.md |
| BG10 | Default data modifies ALL outgoing types on every upgrade | gotchas.md |
| BG11 | `_get_allowed_company_ids()` reads cookies directly | gotchas.md |
| BG12 | Selected line gets priority in matching tiebreaker | gotchas.md |
| BG13 | ExtraProductDialog blocks on unreserved products | gotchas.md |
| BG14 | Package scan creates lines from quants | gotchas.md |
| BG15 | RFID batch timer buffers scans | gotchas.md |
| BG16 | Camera scanner has `delay_between_scan` | gotchas.md |
| BG17 | GS1 barcode padding in cache | gotchas.md |
| BG18 | Split on location change | gotchas.md |
| BG19 | Missing separator regex defaults to never-match | gotchas.md |
| BG2 | `qty_done` is a virtual ("Dummy") field | gotchas.md |
| BG20 | `barcode_write` creates lots without company_id | gotchas.md |
| BG21 | Internal picking from location scan also sets user_id=False | gotchas.md |
| BG22 | `_get_stock_barcode_data` returns `nomenclature_id` as the COMPANY's, but the client overlays the co | gotchas.md |
| BG22S | `qty_done` es un | integrations.md |
| BG23 | `restrict_scan_dest_location` from config depends on multi-locations group | gotchas.md |
| BG23S | `_get_stock_barcode_data()` hace 12 reads en bloque. Si el picking tiene 1000+ move_lines, devuelve  | integrations.md |
| BG24 | `_compute_show_barcode_validation` duplicates view invisible logic | gotchas.md |
| BG24S | `restrict_scan_source_location='mandatory'` + `restrict_scan_dest_location='optional'` en pick.type  | integrations.md |
| BG25 | `split_uncompleted_moves` UNLINKS picked-but-zero move lines | gotchas.md |
| BG25S | `barcode_validation_full=True` se | integrations.md |
| BG26 | Constraint forbids internal type with src=mandatory + dest=optional | gotchas.md |
| BG26S | `_search` de product/lot/location/package está parchado con `nomenclature._preprocess_gs1_search_arg | integrations.md |
| BG27 | `_try_new_internal_picking` swallows `dest_loc` walk to top internal parent | gotchas.md |
| BG27S | `split_uncompleted_moves()` crea | integrations.md |
| BG28 | `_get_picking_type_update_values` rewires source-scan across pick/delivery on multi-step changes | gotchas.md |
| BG28S | `filter_on_barcode` (kanban list) parsea GS1, pero | integrations.md |
| BG29S | `nomenclature_id` se lee de `request.env.company`, NO de `picking.company_id`. En multi-company con  | integrations.md |
| BG3 | `create()` on move line manipulates context | gotchas.md |
| BG30S | `barcode_separator_regex` (default `.^` = no-match) permite escanear varios códigos a la vez en un i | integrations.md |
| BG31M | `move_raw_line_ids` y `move_byproduct_line_ids` son | integrations.md |
| BG32M | `split_uncompleted_moves` en MRP | integrations.md |
| BG33M | Bridge stock.picking añade `kit_variants` (variantes de productos cuyo BOM es `phantom`) al cache ba | integrations.md |
| BG34M | `manual_consumption` es related-inverse: setear `move_line.manual_consumption=True` actualiza `move_ | integrations.md |
| BG35M | `_compute_parent_location_id` override | integrations.md |
| BG36M | `set_lot_producing` devuelve `(lot_read_dict, action_print)` | integrations.md |
| BG37M | `filter_on_barcode` busca MO con `state in ['confirmed','progress']` + `reservation_state='assigned' | integrations.md |
| BG38M | `is_subcontract_stock_barcode` se desactiva si line.state es `done`. Tras validar parcialmente (line | integrations.md |
| BG39M | El override de `create` reasigna `location_id` solo si line.location_id != move.location_id. Si crea | integrations.md |
| BG4 | `barcode_validation_full` silently forced to False | gotchas.md |
| BG40Q | `_is_checkable_from_context` con `barcode_trigger=True` requiere `line.picked=True`. | integrations.md |
| BG41Q | `_action_done` de quality_control bloquea con UserError si `_check_for_quality_checks` devuelve pick | integrations.md |
| BG42Q | `check_state` es un computed por línea, pero el QC en muchos casos está atado al MOVE, no a la line  | integrations.md |
| BG43Q | El JS patch hace `for line of pageLines: if check_state in ['pass','fail']: line.reserved_uom_qty := | integrations.md |
| BG44Q | `action_open_on_demand_quality_check` permite crear check on-demand desde barcode, pero solo si `pic | integrations.md |
| BG45Q | Sin este módulo (depende de ambos auto-install), un batch picking con sub-pickings que tengan QC NO  | integrations.md |
| BG46BUS | NO hay live sync entre dos operarios sobre el mismo picking. | integrations.md |
| BG47BUS | El `EventBus` interno `new EventBus()` en `stock_barcode/static/src/components/main.js:30`, `bus` es | integrations.md |
| BG48BUS | Tras validar parcial → backorder creado → cliente refresca con `/get_barcode_data` con res_id ORIGIN | integrations.md |
| BG49IO | stock_barcode NO usa el `iot.device` model de iot/pos_iot. | integrations.md |
| BG5 | Typo in constraint method | gotchas.md |
| BG50IO | `barcode_max_time_between_keys_in_ms` (default 100ms) es CRÍTICO. Si el scanner manda más lento que  | integrations.md |
| BG51IO | `mobile_reader_scanned` solo se dispara en Odoo Mobile app (web_mobile service). En navegador deskto | integrations.md |
| BG52IO | RFID batch buffers se LIMPIAN tras `RFIDCountTimeout` (5s auto-close). Si el operario tarda >5s entr | integrations.md |
| BG53IO | `vibrate(100)` se dispara en TODO scan (line 360 main.js). En tablets sin motor de vibración → silen | integrations.md |
| BG54IA | `barcode_write` usa formato `[[1, id, vals], [0, 0, vals], ...]` (tupla tipo O2M command), | integrations.md |
| BG55IA | Al crear un quant nuevo, | integrations.md |
| BG56IA | `barcode_write` crea `stock.lot` automáticamente si `lot_name` viene sin `lot_id`. Usa `product_id`  | integrations.md |
| BG57IA | `action_validate` solo aplica quants con `inventory_quantity_set=True`. Si el operario escanea solo  | integrations.md |
| BG58IA | Filter inicial `inventory_date <= today`: quants planificados para mañana NO aparecen. Si configuras | integrations.md |
| BG59IA | `dummy_id` se usa para enlazar create del cliente con el server: cliente genera UUID local, lo pasa  | integrations.md |
| BG6 | Typo in method name | gotchas.md |
| BG60BP | `group_lines_by_product=True` en `picking.type` agrupa móvil-friendly, pero el "help" dice | integrations.md |
| BG61BP | `move_line._domain_picking_id` añade filtro `batch_id=default_batch_id` SOLO si el context lo trae.  | integrations.md |
| BG62BP | `action_get_new_batch_status` filtra `allowed_picking_ids.state=='assigned'` y picking_states convie | integrations.md |
| BG63BP | `filter_on_barcode` en batch delega a `stock.picking.filter_on_barcode`, pero la action devuelta es  | integrations.md |
| BG64BP | Cuando batch no tiene picking_ids aún, `_get_stock_barcode_data` retorna estructura DIFERENTE (recor | integrations.md |
| BG65DEL | En GS1, AI 31xx (weight) se decodifica como | integrations.md |
| BG66DEL | AI 17 (expiration_date) requiere `stock_barcode_product_expiry` instalado. Sin él, el campo `expirat | integrations.md |
| BG67DEL | `getGs1Filters` solo añade filter `{product_id}` para `stock.lot`. | integrations.md |
| BG68DEL | `product.packaging._barcode_field='barcode'`: escanear barcode de packaging (caja de 12) → cliente f | integrations.md |
| BG69DEL | `stock.scrap` herencia de `barcodes.barcode_events_mixin` permite scan EN VIVO en form (NO en barcod | integrations.md |
| BG7 | `_create_new_picking()` sets `user_id=False` | gotchas.md |
| BG70DEL | `_create_new_picking` desde escaneo de location (`_try_new_internal_picking`) busca pick.type `inter | integrations.md |
| BG71DEL | `stock_barcode.usable_packages_limit` (ir.config_parameter) limita cuántos `stock.quant.package` reu | integrations.md |
| BG8 | `barcode_write()` is NOT ORM write | gotchas.md |
| BG9 | Missing `usable_packages_limit` loads ALL packages | gotchas.md |

## enterprise/studio

| ID | Título | Fichero |
|----|--------|---------|
| STUCAP1 | ¿Crear modelos y campos sin código? | capabilities.md |
| STUCAP10 | ¿Qué NO puede hacer Studio (límite clave)? | capabilities.md |
| STUCAP2 | ¿Editar vistas (form/list/kanban) arrastrando? | capabilities.md |
| STUCAP3 | ¿Crear menús y acciones de ventana? | capabilities.md |
| STUCAP4 | ¿Crear automatizaciones (base.automation) sin código? | capabilities.md |
| STUCAP5 | ¿Editor de informes QWeb? | capabilities.md |
| STUCAP6 | ¿Reglas de aprobación en botones? | capabilities.md |
| STUCAP7 | ¿Exportar la personalización como módulo instalable? | capabilities.md |
| STUCAP8 | ¿Campos computados con Python desde Studio? | capabilities.md |
| STUCAP9 | ¿Editar la pantalla de inicio / apps? | capabilities.md |
| STUG1 | SQL-view models cannot get Studio fields | gotchas.md |
| STUG10 | `studio-view-group-ids` is UI-only, NOT security | gotchas.md |
| STUG11 | Editing a default view materializes (freezes) it | gotchas.md |
| STUG12 | Only groups WITH an external ID can be set in view attributes | gotchas.md |
| STUG13 | Approval rules require mail capability for notifications | gotchas.md |
| STUG14 | Webhook automations need `_model` + `_id` in the POST payload | gotchas.md |
| STUG15 | `on_change` automations are synchronous and block the UI | gotchas.md |
| STUG16 | Server-action code is sandboxed — no `import` | gotchas.md |
| STUG17 | Studio data survives module uninstall | gotchas.md |
| STUG18 | `noupdate=1` shields Studio records from base data fixes | gotchas.md |
| STUG19 | Export does NOT include record data by default | gotchas.md |
| STUG2 | Archive field is `x_active`, NOT `x_studio_active` | gotchas.md |
| STUG20 | Many fields are silently excluded from export | gotchas.md |
| STUG21 | Exported module ALWAYS depends on `web_studio` → no Community install | gotchas.md |
| STUG22 | No auto-indexing on Studio fields | gotchas.md |
| STUG23 | Stage/tag secondary models can't be deleted independently | gotchas.md |
| STUG24 | `__` (double underscore) is forbidden in custom field names | gotchas.md |
| STUG25 | Custom kanban `t-name="kanban-box"` BREAKS in v19 | gotchas.md |
| STUG26 | Chatter arch shape changed v17→v18 | gotchas.md |
| STUG27 | `name_create` in Studio context builds a whole MODEL, not a record | gotchas.md |
| STUG28 | `use_stages` kanban-state values are fixed `normal/done/blocked` | gotchas.md |
| STUG29 | Tag M2M relation columns are hardcoded | gotchas.md |
| STUG3 | M2M relation-table name collisions auto-increment | gotchas.md |
| STUG30 | Custom-model records are undeletable by regular users by default | gotchas.md |
| STUG31 | Stat-button count field name is truncated to 63 chars | gotchas.md |
| STUG32 | `studio.export.model` is one-row-per-model (unique constraint) | gotchas.md |
| STUG4 | Binary field silently creates a `_filename` companion | gotchas.md |
| STUG5 | Selection edit deletes data; confirmation only without `force_edit` | gotchas.md |
| STUG6 | `related` is immutable after creation | gotchas.md |
| STUG7 | Computed fields exist but not in the visual editor | gotchas.md |
| STUG8 | Broken XPaths silently skip in production | gotchas.md |
| STUG9 | Studio views ALWAYS apply last | gotchas.md |

## enterprise/subscription

| ID | Título | Fichero |
|----|--------|---------|
| SUBCAP1 | ¿Facturación por uso / metered / consumo variable? | capabilities.md |
| SUBCAP10 | ¿Upsell (añadir productos a una sub viva) con prorrateo? | capabilities.md |
| SUBCAP11 | ¿Contratos con fecha de fin y auto-cierre vs renovación? | capabilities.md |
| SUBCAP12 | ¿Reactivación automática de una sub cancelada al pagar una factura pendiente? | capabilities.md |
| SUBCAP13 | ¿El cliente puede pausar/reanudar él mismo? | capabilities.md |
| SUBCAP14 | ¿El cliente puede cancelar él mismo, con opciones? | capabilities.md |
| SUBCAP15 | ¿El cliente puede cambiar de plan desde el portal? | capabilities.md |
| SUBCAP16 | ¿Analítica de MRR/ARR, churn y expansion/contraction? | capabilities.md |
| SUBCAP17 | ¿Alertas automáticas por salud/MRR/estado (riesgo de churn)? | capabilities.md |
| SUBCAP18 | ¿Plantillas de cotización de suscripción reutilizables? | capabilities.md |
| SUBCAP19 | ¿Suscripciones con productos físicos que se envían cada periodo? | capabilities.md |
| SUBCAP2 | ¿Periodo de prueba gratuito (free trial)? | capabilities.md |
| SUBCAP20 | ¿Vender suscripciones en eCommerce (web) y en TPV (POS)? | capabilities.md |
| SUBCAP3 | ¿Precios distintos por periodicidad (mensual vs anual más barato)? | capabilities.md |
| SUBCAP4 | ¿Descuentos / cupones en una suscripción? | capabilities.md |
| SUBCAP5 | ¿Suscripciones multi-moneda (y al renovar en otra moneda)? | capabilities.md |
| SUBCAP6 | ¿Facturas consolidadas (varias subs de un cliente en una factura)? | capabilities.md |
| SUBCAP7 | ¿Cobro automático recurrente con tarjeta/token? | capabilities.md |
| SUBCAP8 | ¿Reintentos de pago fallido y avisos (dunning)? | capabilities.md |
| SUBCAP9 | ¿Prorrateo automático en upgrade/downgrade a mitad de periodo? | capabilities.md |
| SUBG1 | is_subscription is False for upsells | gotchas.md |
| SUBG10 | Two SQL constraints enforce state↔subscription_state coherence | gotchas.md |
| SUBG100PR | Pricing por variant: `product_variant_ids` M2M. Si la variante se borra (combination change en attri | integrations.md |
| SUBG101PR | `display_subscription_pricing` muestra solo el PRIMER pricing (`sale_subscription/models/product.py: | integrations.md |
| SUBG102AA | `sale.order.alert` usa `_inherits` (delegation) con `base.automation`. Borrar la `base.automation` ( | integrations.md |
| SUBG103AA | Alert con `subscription_state_from` requiere transición exacta. Si SO pasa `3_progress → 4_paused →  | integrations.md |
| SUBG104AA | `mrr_change_period` ('1month'\|'3months') usa `kpi_1month_mrr_delta` / `kpi_3months_mrr_delta` que so | integrations.md |
| SUBG105AA | `activity_user='contract'` usa `sub.user_id` que puede ser False (subscriptions sin salesperson). Ac | integrations.md |
| SUBG106AA | `_get_selection_mrr_change_unit` y companion fields (mrr_min/max) usan `currency_field='currency_id' | integrations.md |
| SUBG107AA | Studio server actions sobre `sale.order` con código Python que escriba a `subscription_state` direct | integrations.md |
| SUBG108AA | `action='set_health_value'` cambia `order.health` sin tracking adicional. `health` está stored con d | integrations.md |
| SUBG11 | origin_order_id IS stored and set at creation | gotchas.md |
| SUBG12 | Invoice consolidation groups by payment_token + partner_invoice | gotchas.md |
| SUBG13 | "Free" subscriptions (amount/MRR ≤ 0) still advance next_invoice_date | gotchas.md |
| SUBG14 | KPIs are daily snapshots, not live | gotchas.md |
| SUBG15 | Upsell start_date = today; next_invoice_date stays parent's | gotchas.md |
| SUBG16 | Renewal quote dates auto-overwritten on parent invoice post | gotchas.md |
| SUBG17 | Upsell parent-line link is a snapshot via parent_line_id | gotchas.md |
| SUBG18 | Renewed subscription is hard-locked | gotchas.md |
| SUBG19 | Churned subscription auto-reopens when an invoice is paid | gotchas.md |
| SUBG2 | next_invoice_date incremented on invoice POST, in account.move | gotchas.md |
| SUBG20 | _get_invoiceable_lines routes recurring lines through the subscription path | gotchas.md |
| SUBG21 | Invoice email template comes from the plan, overriding the default | gotchas.md |
| SUBG22 | Pause/resume protects the discount field from recompute | gotchas.md |
| SUBG23 | Reminder cron catches up missed days but sends ≤1 reminder per sub per day | gotchas.md |
| SUBG24 | last_invoice_date is plan-derived, not from real invoices | gotchas.md |
| SUBG25 | v19 adds user_pause_start (pause analytics) | gotchas.md |
| SUBG26 | plan.billing_period is a Python @property, not a stored field | gotchas.md |
| SUBG26SL | `action_confirm` envuelve TODO con `env.protecting([order_line._fields['discount']], order_line)` pa | integrations.md |
| SUBG27 | INTERVAL_FACTOR uses 30.437 days/month and is on the line module | gotchas.md |
| SUBG27SL | Renewal con `start_date < parent.next_invoice_date` → `ValidationError` ("starting before the next i | integrations.md |
| SUBG28 | account.move write reopen hook fires on ANY matched_payment_ids change | gotchas.md |
| SUBG28SL | `_confirm_renewal` AUTO-CANCELA cualquier otro renewal SO de `2_renewal` state que esté abierto sobr | integrations.md |
| SUBG29SL | `subscription_state='7_upsell'` hace que `is_subscription` se compute como False (`sale_subscription | integrations.md |
| SUBG3 | Postpaid (all-delivered) lines add an extra billing period | gotchas.md |
| SUBG30SL | `update_existing_subscriptions` (upsell confirm) propaga `_upsell_context={'skip_next_invoice_update | integrations.md |
| SUBG31SL | `_should_be_locked()` override hace que las subscriptions confirmadas | integrations.md |
| SUBG32SL | `create_alternative()` (`sale_subscription/models/sale_order.py:1030-1041`, `SaleOrder.create_altern | integrations.md |
| SUBG33SL | `_subscribe_followers_to_new_order` (`sale_subscription/models/sale_order.py:2189-2199`, `SaleOrder. | integrations.md |
| SUBG34SL | `_unlink_except_draft_or_cancel` (`sale_subscription/models/sale_order.py:600-604`, `SaleOrder`) blo | integrations.md |
| SUBG35A | `next_invoice_date` se actualiza SOLO en `account_move._post()` (`sale_subscription/models/account_m | integrations.md |
| SUBG36A | Postpaid lines (`invoice_policy='delivery'`, `_is_all_postpaid()=True`): `_post()` añade `+billing_p | integrations.md |
| SUBG37A | `_get_max_invoiced_date` (`sale_subscription/models/account_move_line.py:110-136`, `AccountMoveLine. | integrations.md |
| SUBG38A | `_compute_mrr` (`sale_subscription/models/account_move_line.py:35-57`, `AccountMoveLine._compute_mrr | integrations.md |
| SUBG39A | `_compute_name` (`sale_subscription/models/account_move_line.py:67-79`, `class AccountMoveLine`) NO  | integrations.md |
| SUBG4 | Renewal 30-day gap → churn-log instead of clean transfer (MRR only) | gotchas.md |
| SUBG40A | `_sale_determine_order` (`sale_subscription/models/account_move_line.py:81-108`, `AccountMoveLine._s | integrations.md |
| SUBG41A | `_message_auto_subscribe_followers` override (`sale_subscription/models/account_move.py:75-90`, `Acc | integrations.md |
| SUBG42A | Renewal SOs en estado `2_renewal\|sent` se ACTUALIZAN automáticamente cuando el parent posts una invo | integrations.md |
| SUBG43P | `payment_exception` queda True tras fallo y SOLO se borra explícitamente en: | integrations.md |
| SUBG44P | `payment_token` heredado de commercial_partner.child_ids (`sale_subscription/models/payment_token.py | integrations.md |
| SUBG45P | El token se guarda SOLO si pasa `_check_token_saving_conditions` (`sale_subscription/models/sale_ord | integrations.md |
| SUBG46P | `_do_payment` ejecuta `SELECT 1 FROM payment_transaction WHERE id=X FOR NO KEY UPDATE` para lockear  | integrations.md |
| SUBG47P | `_subscription_reminder_parameters` (`sale_subscription/models/sale_order.py:2101-2136`, `SaleOrder. | integrations.md |
| SUBG48P | `_handle_automatic_invoices` con `invoice.amount_total_signed < 0` (factura negativa por edición man | integrations.md |
| SUBG49P | Token con `provider.maximum_amount > 0` y `sale_order.amount_total > maximum_amount` → provider excl | integrations.md |
| SUBG5 | Currency change in renewal creates TWO transfer logs | gotchas.md |
| SUBG50P | `_get_partial_payment_subscription_transaction` (`sale_subscription/models/payment_transaction.py:10 | integrations.md |
| SUBG51P | `_post_subscription_action` (`sale_subscription/models/payment_transaction.py:168-191`, `class Payme | integrations.md |
| SUBG52CRM | `_update_revenues_from_so` actualiza valor SOLO SI nuevo > anterior (`crm_sale_subscription/models/c | integrations.md |
| SUBG53CRM | Currency check estricto: `order.currency_id == opportunity.company_id.currency_id`. Si lead está en  | integrations.md |
| SUBG54CRM | `recurring_plan` se setea a monthly por default SOLO la primera vez (`crm_sale_subscription/models/c | integrations.md |
| SUBG55CRM | No hay auto-creación de lead desde sale.order para subscriptions (a diferencia de `appointment_crm`, | integrations.md |
| SUBG56CRM | `_track_filter_for_display` oculta tracking values de `recurring_revenue` SIN borrarlos. Si user sin | integrations.md |
| SUBG57CRM | `team_id` en sale.order PROPAGA a `sale.order.log.team_id` como related-stored (`sale_subscription/m | integrations.md |
| SUBG58PT | `_can_be_edited_on_portal` retorna False si `next_invoice_date > start_date` (ya hubo al menos un bi | integrations.md |
| SUBG59PT | `change_plan` (`sale_subscription/controllers/portal.py:251-259`, `class CustomerPortal`) cambia `pl | integrations.md |
| SUBG6 | Deferred period dates live on the line, not the SO | gotchas.md |
| SUBG60PT | `subscription_portal_upsell` (`sale_subscription/controllers/portal.py:261-269`, `class CustomerPort | integrations.md |
| SUBG61PT | `close_account` (`sale_subscription/controllers/portal.py:238-249`, `class CustomerPortal`) requiere | integrations.md |
| SUBG62PT | `assign_token` ejecuta validation transaction (operation='validation'). Si provider no soporta valid | integrations.md |
| SUBG63PT | `my_subscription` lista incluye `subscription_state in (3_progress, 4_paused, 6_churn)` por default  | integrations.md |
| SUBG64PT | Cliente con `partner_shipping_id` ≠ `partner_invoice_id` puede acceder al portal del subscription vi | integrations.md |
| SUBG65M | Cron de invoicing usa `with_context(mail_notrack=True)` (`sale_subscription/models/sale_order.py:150 | integrations.md |
| SUBG66M | `_create_renewal_transfer_log` el cutoff de 30 días es HARDCODED (`sale_subscription/models/sale_ord | integrations.md |
| SUBG67M | `_create_currency_transfer_log` crea AMBOS logs en el MISMO `order_id` (no en parent vs renewal). Re | integrations.md |
| SUBG68M | `_compute_origin_order_id` en log usa `log.order_id.origin_order_id or log.order_id` (`sale_subscrip | integrations.md |
| SUBG69M | `_get_subscription_delta` (`sale_subscription/models/sale_order.py:1929-1941`, `SaleOrder._get_subsc | integrations.md |
| SUBG7 | Upsell prorata is a discount computed over remaining period | gotchas.md |
| SUBG70M | `_cancel_renewal_logs` (`sale_subscription/models/sale_order_log.py:101-109`, `SaleOrderLog._cancel_ | integrations.md |
| SUBG71M | `sale.subscription.report` SQL view (`report/sale_subscription_report.py`) hace LEFT JOIN con `sale_ | integrations.md |
| SUBG72M | `_cron_update_kpi` (`sale_subscription/models/sale_order.py:1140-1142`, `SaleOrder._cron_update_kpi` | integrations.md |
| SUBG73M | `recurring_monthly` en `sale.order` es COMPUTED-STORED con tracking. Cualquier cambio en `order_line | integrations.md |
| SUBG74S | `_get_outgoing_incoming_moves` filter por `date_deadline.date() in [period_start, period_end]` (`sal | integrations.md |
| SUBG75S | `_reset_subscription_quantity_post_invoice` se llama desde `_post_invoice_hook` que está envuelto en | integrations.md |
| SUBG76S | Upsell con `skip_procurement=True` (`sale_subscription_stock/models/sale_order.py:12-15`, `SaleOrder | integrations.md |
| SUBG77S | `_action_done` override solo suma `qty_delivered` al parent_line si `subscription_state='7_upsell'`  | integrations.md |
| SUBG78S | Postpaid renewal: el override de `_get_invoice_line_parameters` (`sale_subscription_stock/models/sal | integrations.md |
| SUBG79S | `customer_lead` propaga a `date_deadline = period_start + customer_lead`. Si `customer_lead > billin | integrations.md |
| SUBG8 | payment_exception is a one-day lock, auto-cleared by the cron | gotchas.md |
| SUBG80S | `_get_incoming_outgoing_moves_filter` (`sale_subscription/models/sale_order_line.py:111-126`, `SaleO | integrations.md |
| SUBG81S | Cancel de picking de período pasado → `move.state='cancel'` → `qty_delivered` recomputa restando. Pe | integrations.md |
| SUBG82POS | `action_pos_order_paid` llama `_update_next_invoice_date` DIRECTAMENTE (`pos_sale_subscription/model | integrations.md |
| SUBG83POS | POS pago parcial de subscription (split payment, gift card) NO está soportado. Si POS captura solo 5 | integrations.md |
| SUBG84POS | `_get_subscription_qty_invoiced` suma `pos_order_line.qty` convertido via `_convert_qty(line, qty, ' | integrations.md |
| SUBG85POS | No hay sync de `payment_token`: POS captures payment one-shot, NO guarda token para próximos billing | integrations.md |
| SUBG86POS | POS solo procesa subscriptions ya CONFIRMED con `is_subscription=True`. POS no puede confirmar una ` | integrations.md |
| SUBG87POS | Refund POS sobre pos.order paid → NO desencadena rollback de `next_invoice_date`. Subscription queda | integrations.md |
| SUBG88W | `_cart_update_order_line` BLOQUEA mezclar planes (`website_sale_subscription/models/sale_order.py:37 | integrations.md |
| SUBG89W | `_get_pricelist_pricings` (`website_sale_subscription/models/product_template.py:26-39`, `ProductTem | integrations.md |
| SUBG9 | Cron re-triggers itself sequentially in batches of 30 | gotchas.md |
| SUBG90W | `_website_can_be_added` (`website_sale_subscription/models/product_template.py:41-54`, `ProductTempl | integrations.md |
| SUBG91W | `is_plan_possible` (combination_info) propaga a `is_combination_possible` via controller (`website_s | integrations.md |
| SUBG92W | `list_price = price` override (`website_sale_subscription/models/product_template.py:62`, `ProductTe | integrations.md |
| SUBG93W | `prevent_zero_price_sale` (website setting) bloquea producto si `currency.is_zero(unit_price)`. Pero | integrations.md |
| SUBG94W | `optional_product_ids` constraint (`website_sale_subscription/models/product_template.py:13-24`, `Pr | integrations.md |
| SUBG95PR | `_get_first_suitable_recurring_pricing` retorna PRIMER match en orden (`pricelist_id` first, luego s | integrations.md |
| SUBG96PR | `recurring_invoice` está en `_get_incompatible_types` (`sale_subscription/models/product.py:21-23`,  | integrations.md |
| SUBG97PR | `_onchange_recurring_invoice` (`sale_subscription/models/product.py:25-42`, `class product_template` | integrations.md |
| SUBG98PR | `copy()` de product con pricings requiere `group_sale_salesman` (`sale_subscription/models/product.p | integrations.md |
| SUBG99PR | Pricelist archive (`sale_subscription/models/product_pricelist.py:19-30`, `Pricelist.toggle_active`) | integrations.md |

## enterprise/timesheet

| ID | Título | Fichero |
|----|--------|---------|
| TCAP1 | ¿Registrar horas por tarea/proyecto? | capabilities.md |
| TCAP10 | ¿Control de horas vs lo planificado/vendido? | capabilities.md |
| TCAP11 | ¿Coste del empleado en el parte (para rentabilidad/analítica)? | capabilities.md |
| TCAP12 | ¿Facturar a tarifa distinta por empleado/rol (no un único producto)? | capabilities.md |
| TCAP13 | ¿Margen (coste vs venta) en timesheets facturables? | capabilities.md |
| TCAP14 | ¿Ausencias generan parte automáticamente (time off → timesheet)? | capabilities.md |
| TCAP15 | ¿Imputar horas desde tickets de asistencia (helpdesk)? | capabilities.md |
| TCAP16 | ¿Recordatorios automáticos para rellenar partes? | capabilities.md |
| TCAP17 | ¿Integración con planificación / forecast (horas previstas)? | capabilities.md |
| TCAP18 | ¿Comparar asistencia vs horas imputadas? | capabilities.md |
| TCAP2 | ¿Vista grid editable por celda? | capabilities.md |
| TCAP3 | ¿Temporizador start/stop? | capabilities.md |
| TCAP4 | ¿Facturar timesheets en un pedido de venta? | capabilities.md |
| TCAP5 | ¿Aprobación/validación de partes? | capabilities.md |
| TCAP6 | ¿Redondeo y duración mínima del timer? | capabilities.md |
| TCAP7 | ¿Comparar asistencia vs horas imputadas? | capabilities.md |
| TCAP8 | ¿Registro desde portal? | capabilities.md |
| TCAP9 | ¿Rentabilidad por proyecto? | capabilities.md |
| TG1 | Forced cascade uninstall | gotchas.md |
| TG10 | Timer on task vs timer on timesheet are different | gotchas.md |
| TG11 | Minimum duration and rounding only apply to timer | gotchas.md |
| TG12 | try_to_match merging is silent | gotchas.md |
| TG13 | _compute_project_id skipped for validated TS | gotchas.md |
| TG14 | No overtime storage | gotchas.md |
| TG15 | check_if_allowed same-day exception | gotchas.md |
| TG16 | Config params are noupdate=1 | gotchas.md |
| TG17 | Orphan employees validatable by ANY approver | gotchas.md |
| TG18 | Reset to draft can roll back lock date significantly | gotchas.md |
| TG19C | A timesheet is just an `account.analytic.line` with a `project_id`. | gotchas.md |
| TG19PR | `allow_timesheets` y `account_id` son interdependientes con dependencia circular en `_compute_allow_ | integrations.md |
| TG2 | Validated TS create rolling lock date | gotchas.md |
| TG20C | `unit_amount` is the hours field — but it's stored in the company time UoM, not always hours. | gotchas.md |
| TG20PR | `total_hours_spent = effective_hours + subtask_effective_hours` IGNORA el árbol completo del task: s | integrations.md |
| TG21C | `amount` (cost) is recomputed on every write touching unit_amount/employee_id/account_id and is NEGA | gotchas.md |
| TG21PR | `progress` se almacena (`store=True`) y se calcula como `round(total_hours_spent / allocated_hours,  | integrations.md |
| TG22C | `name` is force-defaulted to `'/'` on create AND write. | gotchas.md |
| TG22PR | `_compute_remaining_hours` resta `effective + subtask_effective`. Si una subtask tiene `allocated_ho | integrations.md |
| TG23C | Creating a TS for an archived employee RAISES; for an inactive analytic account RAISES. | gotchas.md |
| TG23PR | `project._unlink_except_contains_entries` levanta `RedirectWarning` (no `UserError`): el usuario VE  | integrations.md |
| TG24C | The analytic account comes from the PROJECT's mandatory plans, validated at create/write time. | gotchas.md |
| TG24PR | `internal_project_id` se crea automáticamente al crear company vía `create()` con `sudo()` (`hr_time | integrations.md |
| TG25C | Timesheets cannot be created on a private task. | gotchas.md |
| TG25PR | Task con `project_id=False` (private task) NO puede tener timesheets: `_check_project_root` constrai | integrations.md |
| TG26C | `project_id` auto-fills from `task_id`, and `task_id` auto-clears when project changes. | gotchas.md |
| TG26PR | `_compute_partner_id` en analytic.line va `task.partner_id OR project.partner_id` (línea 134-136), N | integrations.md |
| TG27C | Multi-company guard: project, task and analytic accounts must all be the same company. | gotchas.md |
| TG27HR | Crear analytic.line para empleado de OTRA company en multi-company da `ValidationError` (línea 277): | integrations.md |
| TG28C | Basic users can only touch their OWN timesheets — enforced in 3 layers. | gotchas.md |
| TG28HR | Resolución employee_id↔user_id es | integrations.md |
| TG29C | (sale). The SO line (`so_line`) is what makes a timesheet billable — auto-deduced unless manually ed | gotchas.md |
| TG29HR | `_hourly_cost` por defecto = `employee.hourly_cost` (línea 423-425). Para multi-currency: `_timeshee | integrations.md |
| TG3 | Timer start on non-today dates creates NEW timesheet | gotchas.md |
| TG30C | (sale). `timesheet_invoice_type` is computed, stored, and drives portal/invoicing filters. | gotchas.md |
| TG30HR | `action_unlink_wizard` en `hr.employee`: si el employee tiene timesheets Y el current user NO es app | integrations.md |
| TG31C | (sale). An INVOICED timesheet is locked harder than a validated one. | gotchas.md |
| TG31HR | `has_timesheet` en `hr.employee` se calcula via SQL directo `EXISTS(...)` con `tuple(self.ids)` (lín | integrations.md |
| TG32C | (sale). `qty_delivered` on the SOL is the SUM of timesheet `unit_amount`, only for service products  | gotchas.md |
| TG32HR | `hr.timesheet.attendance.report` es view SQL inmutable: el `init()` DROPea y crea la view. Custom mo | integrations.md |
| TG33C | (sale). Employee-rate cost overrides the employee's hourly_cost via the map. | gotchas.md |
| TG33HR | `attendance_cost - timesheets_cost` (campo `cost_difference`) usa `employee.hourly_cost` actual (sna | integrations.md |
| TG34AA | Cost histórico no se snapshot | integrations.md |
| TG34C | (grid). Grid `grid_update_cell` NEVER edits a validated or multi-match cell — it COPIES. | gotchas.md |
| TG35AA | `_check_allow_timesheet` constraint exige `account_id` cuando `allow_timesheets=True`. Auto-create e | integrations.md |
| TG35C | (grid). `value=0` in the grid is a silent no-op. | gotchas.md |
| TG36AA | `_timesheet_postprocess` chequea `len(companies) > 1` mezclando `timesheet.company_id \| accounts.com | integrations.md |
| TG36C | (grid). Timer rounding & minimum apply ONLY to timer stop, via `ir.config_parameter`, NOT to manual  | gotchas.md |
| TG37AA | Conversión de currency en `amount`: `employee.currency_id._convert(amount, account.currency_id or ti | integrations.md |
| TG37C | (grid). `action_timer_increase`/`decrease` default step is 15min and decrease floors at 0. | gotchas.md |
| TG38AA | Mandatory plans cross-project: si añades un plan secundario mandatory (`x_studio_costcenter_plan`) A | integrations.md |
| TG38C | (holidays). Validating a time-off auto-generates one timesheet PER WORKDAY in the internal/vacation  | gotchas.md |
| TG39AA | AA inactiva (`account.active=False`) bloquea timesheet write (línea 382): `"Timesheets must be creat | integrations.md |
| TG39C | (holidays). Refusing/cancelling/zeroing a leave UNLINKS its timesheets — even validated ones, via su | gotchas.md |
| TG4 | Timer stop with 0 hours DELETES the timesheet | gotchas.md |
| TG40AA | `_timesheet_preprocess_get_accounts` lee los planes del project y los inyecta en vals. Funciona para | integrations.md |
| TG40C | (holidays). Flexible-calendar leaves compute hours differently. | gotchas.md |
| TG41C | `_disable_data_merge = True` on account.analytic.line (grid). | gotchas.md |
| TG41SL | Auto-binding so_line agresivo | integrations.md |
| TG42C | Approver can validate own line per `user_can_validate` but the validation ACTION silently skips it. | gotchas.md |
| TG42SL | `_is_not_billed` retorna True si `timesheet_invoice_id.state == 'cancel' AND payment_state != 'invoi | integrations.md |
| TG43SL | `_check_can_write` impide modificar `unit_amount/employee_id/project_id/task_id/so_line/date` si `so | integrations.md |
| TG44SL | timesheet_invoice_type recalcula sobre amount | integrations.md |
| TG45SL | upsell activity con side-effect | integrations.md |
| TG46SL | Credit note (`out_refund` con `reversed_entry_id`): `action_post` libera timesheets del invoice reve | integrations.md |
| TG47SL | `_check_timesheet_can_be_billed` (línea 99-100) valida que `so_line ∈ project.sale_line_employee_ids | integrations.md |
| TG48SL | `_get_employee_mapping_entry` multi-company: si `len(env.companies) == 1` → search simple. Si multi  | integrations.md |
| TG49EX | `expense.sale_order_id` se cambia → `_onchange_sale_order_id` invalida `analytic_distribution`. Si e | integrations.md |
| TG5 | Validation stops OTHER people's timers | gotchas.md |
| TG50EX | Expense con `expense_policy='cost'` re-factura al | integrations.md |
| TG51EX | `_reverse_moves` resetea qty del SOL via `expense_sheet._sale_expense_reset_sol_quantities()`. Si el | integrations.md |
| TG52EX | Domain de `_search_display_name` con context `sale_expense_all_order` (línea 14-30): permite al user | integrations.md |
| TG53EX | Bridge module `project_sale_expense` añade campos al `project` para track expenses-as-cost, pero NO  | integrations.md |
| TG54EX | Expense flow para | integrations.md |
| TG55GR | Validation NOT atomic across employees | integrations.md |
| TG56GR | `_disable_data_merge = True` (línea 20) impide al módulo `data_merge` proponer merges en account.ana | integrations.md |
| TG57GR | `grid_update_cell` semantics (línea 346-382): | integrations.md |
| TG58GR | `last_validated_timesheet_date` lock per employee es global: TODOS los timesheets del empleado en/an | integrations.md |
| TG59GR | Exception "today's date always editable" (`check_if_allowed`, línea 287-288): `date != fields.Date.t | integrations.md |
| TG6 | Validation cascade-stops timers on other timesheets | gotchas.md |
| TG60GR | Cascade uninstall | integrations.md |
| TG61GR | Reminder crons (daily but only fire when `nextdate <= today`): si nextdate se desfasa (e.g. server d | integrations.md |
| TG62GR | Timer auto-stop on validate: validar timesheets con timer RUNNING → `_stop_all_users_timer()` los pa | integrations.md |
| TG63HD | Mutua exclusión task vs ticket | integrations.md |
| TG64HD | Tickets sin team change warning | integrations.md |
| TG65HD | `display_timesheet_timer` requires `encode_uom_in_days=False`. Encoding UoM en días → ticket no mues | integrations.md |
| TG66HD | timesheet_invoice_type sobre ticket | integrations.md |
| TG67HD | `_inverse_project_id` (línea 42-52): si user cambia `project_id` del timesheet a uno DIFERENTE del t | integrations.md |
| TG68HD | Portal access | integrations.md |
| TG69FSM | Timesheet validation locks task editing | integrations.md |
| TG7 | Cannot validate own timesheets as Approver | gotchas.md |
| TG70FSM | `under_warranty` toggle re-prices ALL SOLs | integrations.md |
| TG71FSM | FSM task copy resets sale_order_id | integrations.md |
| TG72FSM | Materials NO en analytic timesheet flow | integrations.md |
| TG73FSM | Cross-bridge timer rounding | integrations.md |
| TG74FSM | FSM con encoding_uom=days | integrations.md |
| TG75HOL | Refuse leave borra timesheets validados sin warning | integrations.md |
| TG76HOL | Flexible hours bypass | integrations.md |
| TG77HOL | Global leaves DOBLES | integrations.md |
| TG78HOL | timesheet_grid_holidays + timer | integrations.md |
| TG79HOL | Unlink/refuse de leave PIERDE timesheets manuales sobre task de leave | integrations.md |
| TG8 | Future timesheets cannot be validated | gotchas.md |
| TG80HOL | Multi-day leave SPAN cambio de company | integrations.md |
| TG81AD | Distribution format quirk | integrations.md |
| TG82AD | Plan column_name dinámico | integrations.md |
| TG83AD | Fallback silent a project.account_id | integrations.md |
| TG84AD | Mandatory plans solo valida `timesheet` business_domain | integrations.md |
| TG85AD | Multi-plan SOL distribution sin root plan | integrations.md |
| TG86AD | Distribución partial percentage NOT split | integrations.md |
| TG9 | Grid cell update creates copies for validated rows | gotchas.md |

## enterprise/voip

| ID | Título | Fichero |
|----|--------|---------|
| VOIPG1 | `voip_secret` stored plaintext | gotchas.md |
| VOIPG10 | DTMF not received by IVR | gotchas.md |
| VOIPG11 | DTMF pre-call ignored | gotchas.md |
| VOIPG12 | Phone number format mismatch | gotchas.md |
| VOIPG13 | USA area-code heuristic | gotchas.md |
| VOIPG14 | Future phonecalls invisible in queue | gotchas.md |
| VOIPG15 | Activity deleted after marked done → name preserved on call only | gotchas.md |
| VOIPG16 | Registration expiry & dead-peer detection | gotchas.md |
| VOIPG17 | WSS self-signed certificate | gotchas.md |
| VOIPG18 | External device number with blind transfer | gotchas.md |
| VOIPG19 | `how_to_call_on_mobile='ask'` interrupts every call | gotchas.md |
| VOIPG2 | Browser autoplay blocks first ringtone | gotchas.md |
| VOIPG20 | Demo mode misleading | gotchas.md |
| VOIPG21 | No voicemail integration | gotchas.md |
| VOIPG22 | Recording responsibility on provider, not Odoo (v18); v19 DOES store audio | gotchas.md |
| VOIPG23 | Anonymous caller ID | gotchas.md |
| VOIPG24 | Auto-reject vs user-reject use DIFFERENT SIP codes | gotchas.md |
| VOIPG25 | Multi-monitor: ringtone audio output | gotchas.md |
| VOIPG26 | v18 lacks call transcription | gotchas.md |
| VOIPG27 | Users can write their OWN voip_secret and provider (self-service) | gotchas.md |
| VOIPG28 | `voip.provider` with `company_id=False` is visible to every user | gotchas.md |
| VOIPG29 | Users cannot delete their own call history | gotchas.md |
| VOIPG3 | Microphone permission must be granted | gotchas.md |
| VOIPG30 | `neutralize.sql` forces demo mode on staging | gotchas.md |
| VOIPG31 | ICE gathering capped at 1 s | gotchas.md |
| VOIPG32 | Settings record created lazily on first edit | gotchas.md |
| VOIPG33 | Lazily-created "Call" activity type is global and sticks | gotchas.md |
| VOIPG34 | `create_call_activity` raises `UserError` and rolls back the WHOLE batch | gotchas.md |
| VOIPG35 | Recent-calls search is `ilike` but contact search is `like` (case-sensitive) | gotchas.md |
| VOIPG36 | Click-to-call resolves the partner via `_mail_get_partners`, not the phone field | gotchas.md |
| VOIPG4 | WSS blocked by corporate firewalls | gotchas.md |
| VOIPG5 | iOS Safari WebRTC limitations | gotchas.md |
| VOIPG6 | Android Chrome muting non-focused tabs | gotchas.md |
| VOIPG7 | Headset hot-swap mid-call → audio glitch | gotchas.md |
| VOIPG8 | Multi-tab → wrong tab rings | gotchas.md |
| VOIPG9 | Background tab → softphone frozen | gotchas.md |

## enterprise/website_sale_subscription

| ID | Título | Fichero |
|----|--------|---------|
| WSSCAP1 | ¿Vender suscripciones desde la tienda web? | capabilities.md |
| WSSCAP10 | ¿Validar 1 plan por carrito? | capabilities.md |
| WSSCAP2 | ¿Mostrar plan y precio recurrente (/mes, /año)? | capabilities.md |
| WSSCAP3 | ¿El checkout crea la suscripción? | capabilities.md |
| WSSCAP4 | ¿Pago recurrente con token desde la web? | capabilities.md |
| WSSCAP5 | ¿Portal: ver/gestionar/cancelar la suscripción? | capabilities.md |
| WSSCAP6 | ¿Combinar recurrente y normal en el mismo pedido? | capabilities.md |
| WSSCAP7 | ¿Periodo de prueba (free trial) desde la web? | capabilities.md |
| WSSCAP8 | ¿Renovación / upsell desde el portal? | capabilities.md |
| WSSCAP9 | ¿Elegir variante de plan (mensual vs anual) en la web? | capabilities.md |
| WSSG1 | Auto-install wires in silently | gotchas.md |
| WSSG10 | Variant vs template pricing dispatch is via `_get_pricelist_pricings(pricelist, product=...)` | gotchas.md |
| WSSG11 | Optional subscription products must share a plan with the parent | gotchas.md |
| WSSG12 | Three independent add-to-cart gates all funnel through `_website_can_be_added` | gotchas.md |
| WSSG13 | `is_plan_possible` folds into `is_combination_possible` at the controller | gotchas.md |
| WSSG14 | Empty `pricings` returns a stub combination_info, not an error | gotchas.md |
| WSSG15 | Search results show ONLY the default pricing string | gotchas.md |
| WSSG16 | NO checkout/address override in v18 | gotchas.md |
| WSSG17 | Token capture lives in `sale_subscription`; this module only flags the form | gotchas.md |
| WSSG18 | Partial / unauthorized payment doesn't save the token → next renewal fails | gotchas.md |
| WSSG19 | "Reorder / Buy again" is silently disabled for subscription lines | gotchas.md |
| WSSG2 | Cart plan lock — but the guard only fires when `plan_id` is explicitly passed | gotchas.md |
| WSSG20 | v18 portal has NO pause/resume/change_address ROUTES | gotchas.md |
| WSSG21 | v18 re-renders a `<select>.plan_select`, not radio buttons | gotchas.md |
| WSSG22 | Optional-product combination info carries the plan from the `.plan_select` value | gotchas.md |
| WSSG3 | Plan is cleared only when the LAST recurring line is removed | gotchas.md |
| WSSG4 | Plan assignment happens on add only when the cart has NO plan yet | gotchas.md |
| WSSG5 | `list_price` is overwritten with the computed price → pricelist strike-through dies | gotchas.md |
| WSSG6 | No discount % at all in v18 — there is a `to_minimum_billing_period` label instead | gotchas.md |
| WSSG7 | `subscription_pricing_select` (radios/select shown?) = `possible_pricing_count > 1` | gotchas.md |
| WSSG8 | `can_be_added` per pricing is gated by the cart's existing plan | gotchas.md |
| WSSG9 | `_get_pricelist_pricings` returns at most ONE pricing per plan, and only for ACTIVE plans | gotchas.md |

## enterprise/whatsapp

| ID | Título | Fichero |
|----|--------|---------|
| WACAP1 | ¿Envío vía WhatsApp Business API oficial (Meta)? | capabilities.md |
| WACAP10 | ¿Envío masivo de marketing por WhatsApp? | capabilities.md |
| WACAP2 | ¿Plantillas aprobadas con variables? | capabilities.md |
| WACAP3 | ¿Conversación bidireccional (recepción)? | capabilities.md |
| WACAP4 | ¿Disparar una plantilla desde un registro? | capabilities.md |
| WACAP5 | ¿Integración con Discuss? | capabilities.md |
| WACAP6 | ¿Ventana de servicio de 24h? | capabilities.md |
| WACAP7 | ¿Botones/respuestas interactivas? | capabilities.md |
| WACAP8 | ¿Adjuntos/multimedia? | capabilities.md |
| WACAP9 | ¿Multi-cuenta / varios números? | capabilities.md |
| WAG1 | Template model locks if linked to a server action | gotchas.md |
| WAG10 | RED quality OR non-approved status = automatic send failure | gotchas.md |
| WAG11 | notify_user_ids required — inactive users = no notifications | gotchas.md |
| WAG12 | Outdated-channel resend marks cancel, not error | gotchas.md |
| WAG13 | Location header strict 4-var + regex validation | gotchas.md |
| WAG14 | Blacklist shared with SMS; any inbound text auto-opts-IN | gotchas.md |
| WAG16 | Safe phone-field whitelist blocks non-admin users | gotchas.md |
| WAG18 | Dynamic button variable name must equal button name | gotchas.md |
| WAG19 | Batch sending ordered by template (non-template first) | gotchas.md |
| WAG2 | Phone formatting = country code + number, NO `+` | gotchas.md |
| WAG20 | v19 class rename WhatsAppMessage → WhatsappMessage + Constraint API | gotchas.md |
| WAG21 | 10 template status values, not 3 | gotchas.md |
| WAG22 | Re-pointing a template at another account silently discards approval | gotchas.md |
| WAG23 | Composer & server actions only see APPROVED templates | gotchas.md |
| WAG24 | Only the CUSTOMER's reply extends the window | gotchas.md |
| WAG25 | Missing app_secret → webhook Forbidden, NOT open | gotchas.md |
| WAG26 | webhook_verify_token generated once, never regenerated | gotchas.md |
| WAG27 | Unsupported inbound types are warned and dropped | gotchas.md |
| WAG28 | Outbound WA only triggers for `whatsapp_message` type on a whatsapp channel | gotchas.md |
| WAG29 | Reply parent only honored with a unique matching outbound message | gotchas.md |
| WAG3 | Duplicate messages silently cancelled | gotchas.md |
| WAG30 | You cannot LEAVE a whatsapp channel — only delete it | gotchas.md |
| WAG31 | Resend skips whatsapp_unrecoverable | gotchas.md |
| WAG32 | Attachments cloned before send; report headers regenerate per retry | gotchas.md |
| WAG33 | app_secret & token are admin-only fields | gotchas.md |
| WAG34 | All WhatsApp API calls are disabled in test threads | gotchas.md |
| WAG35 | `_send` sends inline only for a SINGLE message; >1 (or forced) queues to cron | gotchas.md |
| WAG36 | Batch composer SWALLOWS phone errors; single send RAISES | gotchas.md |
| WAG37 | A `user_mobile` template variable blocks the WHOLE send if the sender has no mobile | gotchas.md |
| WAG38 | Inbound only re-uses a channel if the number got a TEMPLATE before | gotchas.md |
| WAG39 | Template send posts a visible "moved to new chat" notification when the agent isn't a channel member | gotchas.md |
| WAG4 | Variable index gaps cause validation error | gotchas.md |
| WAG5 | 24h window based on message create_date, not current time | gotchas.md |
| WAG6 | Audio messages split into a SEPARATE mail.message | gotchas.md |
| WAG7 | Template sync OVERWRITES local edits | gotchas.md |
| WAG8 | Webhook signature must be EXACTLY 71 chars | gotchas.md |
| WAG9 | Messages garbage collected after 15 days | gotchas.md |

## cross-module/analytic

| ID | Título | Fichero |
|----|--------|---------|
| ANG1 | analytic_distribution is a JSON field, not a relation | gotchas.md |
| ANG10 | Stock move _get_analytic_distribution() returns {} by default | gotchas.md |
| ANG11 | Analytic lines created on POST, not on invoice creation — and unlinked on un-post/draft | gotchas.md |
| ANG12 | Back-sync recomputes distribution as `-aal.amount / line.balance * 100` when analytic lines are modi | gotchas.md |
| ANG13 | Reward/discount lines don't inherit analytic | gotchas.md |
| ANG14 | Expense split copies analytic to all fragments | gotchas.md |
| ANG15 | MO has NO analytic_distribution field | gotchas.md |
| ANG16 | CE: analytic distribution uniform on all MO moves | gotchas.md |
| ANG17 | Work center costs are negative | gotchas.md |
| ANG18 | extra_cost doesn't generate separate journal entry | gotchas.md |
| ANG19 | Analytic account mandatory for timesheets | gotchas.md |
| ANG2 | Each root plan creates real database columns via ALTER TABLE | gotchas.md |
| ANG20 | Timesheet amount only recalculates on specific field changes | gotchas.md |
| ANG21 | Profitability can double-count if not careful | gotchas.md |
| ANG22 | POS closing entry has NO analytic_distribution | gotchas.md |
| ANG23 | POS order lines don't inherit analytic.mixin | gotchas.md |
| ANG24 | Budget matching is exact plan match, not parent/child | gotchas.md |
| ANG25 | Committed amount only from confirmed POs | gotchas.md |
| ANG26 | Achieved amount comes from analytic lines, not invoices directly | gotchas.md |
| ANG27 | GIN index only works on stored analytic_distribution | gotchas.md |
| ANG28 | Analytic shadow table negates amounts | gotchas.md |
| ANG29 | Multi-plan groupby creates column explosion | gotchas.md |
| ANG3 | Plan applicability competes by score, not sequence — but distribution MODELS use sequence + dedup | gotchas.md |
| ANG30 | Distribution model company check uses SQL array intersection | gotchas.md |
| ANG31 | Plan applicability is company_dependent | gotchas.md |
| ANG32 | analytic_distribution search converts display_name to IDs | gotchas.md |
| ANG33 | AAL exist only for POSTED moves, but the project margin reads DRAFT AML | gotchas.md |
| ANG34 | Stock analytic lines are unlinked, not zeroed, when the move nets to zero | gotchas.md |
| ANG35 | Non-real-time (continental/manual) moves estimate analytic at `standard_price`, not at actual cost | gotchas.md |
| ANG36 | Stock analytic distribution comes ONLY from the move, not from the MO/picking | gotchas.md |
| ANG37 | Landed costs have no dedicated analytic hook — they ride the SVL | gotchas.md |
| ANG38 | v19 replaces `_condition_to_sql` analytic override with a real `search=` method | gotchas.md |
| ANG39 | `_get_all_plans` is ormcached and HARD-FAILS if no Project plan is configured | gotchas.md |
| ANG4 | Mandatory plans block invoice posting if not 100% — but ONLY on `display_type='product'` lines | gotchas.md |
| ANG40 | Reparenting/replanning an account REFUSES to silently move history — it raises RedirectWarning | gotchas.md |
| ANG41 | account.analytic.line has NO `move_id`, NO `auto_account_id` column, and `code` is capped at 8 chars | gotchas.md |
| ANG42 | AAL general_account_id is constrained to equal the journal item's account | gotchas.md |
| ANG43 | Account balance converts every line at TODAY's rate, not the line's date | gotchas.md |
| ANG44 | Invoice/bill counts on an analytic account only see POSTED moves of the matching type | gotchas.md |
| ANG5 | Project plan is special — uses account_id, not x_plan{N}_id | gotchas.md |
| ANG6 | _merge_distribution uses cartesian product | gotchas.md |
| ANG7 | Sanitization rounds percentages on every create/write — and the `__update__` sentinel is exempt | gotchas.md |
| ANG8 | _read_group on analytic_distribution only supports __count | gotchas.md |
| ANG9 | SOL → AML propagation is direct copy, not recompute | gotchas.md |

## cross-module/cron

| ID | Título | Fichero |
|----|--------|---------|
| CRONG1 | `--max-cron-threads=0` → NO cron runner at all | gotchas.md |
| CRONG10 | `_trigger` on an inactive cron drops immediate triggers | gotchas.md |
| CRONG11 | Triggers don't wake workers when PG is in recovery (replica) | gotchas.md |
| CRONG12 | SKIP LOCKED — another worker already took it (normal) | gotchas.md |
| CRONG13 | Row left LOCKED after a worker crash blocks edits/reruns | gotchas.md |
| CRONG14 | Neutralized DB cannot re-enable crons via side effects | gotchas.md |
| CRONG15 | Long job killed by `--limit-time-real-cron` | gotchas.md |
| CRONG16 | Batch cap of 10 per pass — big backlogs need multiple passes | gotchas.md |
| CRONG17 | Crons NEVER fire inside tests; `_trigger()` is a no-op in tests | gotchas.md |
| CRONG18 | `numbercall`/`doall` in data XML errors on 18+ | gotchas.md |
| CRONG19 | Time automations share ONE cron — worst-case latency = shared interval | gotchas.md |
| CRONG2 | DB not in the cron worker's database list | gotchas.md |
| CRONG20 | Deleting/disabling the base_automation cron silently kills ALL time automations | gotchas.md |
| CRONG21 | Outgoing email is queued, not sent — needs the mail scheduler cron | gotchas.md |
| CRONG22 | `with_delay()` jobs stuck in PENDING forever — the jobrunner is not the cron runner | gotchas.md |
| CRONG23 | queue_job channel capacity 0 / unreachable base URL stalls jobs | gotchas.md |
| CRONG24 | Cron fails with `AccessError` when the scheduler user lacks rights on the action's model | gotchas.md |
| CRONG25 | Model-driven (de)activation via `toggle()` is silently SKIPPED if the cron is mid-run | gotchas.md |
| CRONG26 | FUTURE-dated triggers SURVIVE a regular run and fire again | gotchas.md |
| CRONG27 | Failing crons sink to the BOTTOM of the ready queue (ordering is `failure_count` FIRST) | gotchas.md |
| CRONG28 | Worst-case ~60 s latency to pick up a due `nextcall` (no trigger) | gotchas.md |
| CRONG3 | Code/DB version mismatch silently skips the whole DB | gotchas.md |
| CRONG4 | Pending module install/upgrade blocks crons | gotchas.md |
| CRONG5 | `active=False` overrides everything | gotchas.md |
| CRONG6 | Auto-deactivated after repeated failures (NOT after N calls) | gotchas.md |
| CRONG7 | Exception → rollback → reschedule, work lost silently | gotchas.md |
| CRONG8 | No catch-up: missed slots collapse to one run (old `doall` gone) | gotchas.md |
| CRONG9 | On an INACTIVE cron, immediate triggers are dropped | core.md |

## cross-module/discuss_bus

| ID | Título | Fichero |
|----|--------|---------|
| BUSG1 | BUSG40. Citas `path:line` verificadas contra `odoo18/addons/bus/` y `mail/models/discuss/`. Notas v1 | gotchas.md |
| BUSG10 | Guest presence es 100% capa mail, no bus | gotchas.md |
| BUSG11 | Multi-tab concurrent presence updates race | gotchas.md |
| BUSG12 | Longpolling no escribe el `last_poll` propio | gotchas.md |
| BUSG13 | im_status NO es N+1: una sola query por recordset, pero NO almacenado | gotchas.md |
| BUSG14 | Member-specific broadcasts don't reach new members immediately | gotchas.md |
| BUSG15 | Typing status has no server-side rate limit | gotchas.md |
| BUSG16 | Reaction spam via bus — no rate limit | gotchas.md |
| BUSG17 | RTC state broadcasts spike bus load | gotchas.md |
| BUSG18 | Sub-channel GC UNPINEA (no borra) miembros idle 2 días | gotchas.md |
| BUSG19 | Guest token constant-time compare but ID lookup is not | gotchas.md |
| BUSG2 | PostgreSQL NOTIFY payload limit 8000 bytes | gotchas.md |
| BUSG20 | Channel UUID = 10 chars de alfabeto 58, con `secrets.choice` (SÍ cripto-seguro) | gotchas.md |
| BUSG21 | No per-notification authorization in bus.bus | gotchas.md |
| BUSG22 | Guest has no logout mechanism | gotchas.md |
| BUSG23 | is_member NO es N+1: una sola query batched | gotchas.md |
| BUSG24 | Group auto-subscription SÍ es idempotente | gotchas.md |
| BUSG25 | First longpoll (last=0) returns 50s buffer only | gotchas.md |
| BUSG26 | `_bus_send` exige Model y ensure_one por record | gotchas.md |
| BUSG27 | `_sendone` persiste un message con shape fijo `{type, payload}` | gotchas.md |
| BUSG28 | NOTIFY se emite contra la BD "postgres", no la del tenant | gotchas.md |
| BUSG29 | `bus_bus` crece sin parar entre GCs; retención configurable | gotchas.md |
| BUSG3 | Deduplication window only 10 seconds | gotchas.md |
| BUSG30 | Worker reconnect: backoff exponencial ×1.5 hasta 60s | gotchas.md |
| BUSG31 | SharedWorker comparte UNA conexión entre todas las tabs | gotchas.md |
| BUSG32 | `update_presence()` hace `cr.commit()` por cada poll | gotchas.md |
| BUSG33 | `_on_websocket_closed` sólo marca offline a usuarios autenticados (guest aparte) | gotchas.md |
| BUSG34 | `_authenticate` del websocket degrada a public_user, no rechaza | gotchas.md |
| BUSG35 | El cliente sólo puede suscribir canales STRING; los de modelo los pone el server | gotchas.md |
| BUSG36 | `last` se resetea a 0 si supera el último id del bus | gotchas.md |
| BUSG37 | Canales de presencia perdidos: ventana de 2s al suscribir | gotchas.md |
| BUSG38 | El subchannel "members" sólo lo añaden canales DONDE eres miembro | gotchas.md |
| BUSG39 | `_compute_im_status` de partner reduce con preferencia a "online" | gotchas.md |
| BUSG4 | Channel format mixing causes missed notifications | gotchas.md |
| BUSG40 | `bus.gc_retention_seconds` es un `ir.config_parameter`, leído en cada autovacuum | gotchas.md |
| BUSG5 | LISTEN lost on DB reconnect | gotchas.md |
| BUSG6 | Subscribe REPLACES channels, doesn't add | gotchas.md |
| BUSG7 | Expired session kills connection with code 4001 | gotchas.md |
| BUSG8 | Outdated worker version → clean close "OUTDATED_VERSION" (no clear error) | gotchas.md |
| BUSG9 | Closing handshake can hang up to 15s per connection | gotchas.md |

## cross-module/landed_costs

| ID | Título | Fichero |
|----|--------|---------|
| LCCAP1 | ¿Repartir costes adicionales (flete/aduana/seguro) sobre el coste de productos recibidos? | capabilities.md |
| LCCAP10 | ¿Usar peso/volumen del producto para el reparto? | capabilities.md |
| LCCAP2 | ¿Métodos de reparto? | capabilities.md |
| LCCAP3 | ¿Crear el coste en destino desde la factura del transportista? | capabilities.md |
| LCCAP4 | ¿Aplicar a recepciones de compra? | capabilities.md |
| LCCAP5 | ¿Aplicar a órdenes de fabricación? | capabilities.md |
| LCCAP6 | ¿Funciona con valoración manual o requiere automática + FIFO/AVCO? | capabilities.md |
| LCCAP7 | ¿Ajusta la valoración de stock (SVL) y el asiento contable? | capabilities.md |
| LCCAP8 | ¿Revertir/cancelar un landed cost ya validado? | capabilities.md |
| LCCAP9 | ¿Coste en destino en subcontratación? | capabilities.md |
| LCG1 | No se puede cancelar tras validar. | gotchas.md |
| LCG10 | El SVL del LC NO impacta el price_unit del receipt si viene de factura. | gotchas.md |
| LCG11 | La reconciliación solo en anglosajón con factura posteada. | gotchas.md |
| LCG12 | El producto LC debe ser tipo servicio. | gotchas.md |
| LCG13 | `split_method='equal'` reparte por nº de líneas, NO por importe. | gotchas.md |
| LCG14 | Dropshipping desvía el débito a la cuenta de GASTO. | gotchas.md |
| LCG15 | El botón "Create Landed Costs" solo aparece si la línea de factura es LC. | integrations.md |
| LCG16 | En fabricación el reparto va a los productos ACABADOS, no a los componentes. | integrations.md |
| LCG17 | La reconciliación factura↔LC solo en anglosajón + factura posteada. | integrations.md |
| LCG18 | Un LC creado desde factura NO impacta dos veces. | integrations.md |
| LCG19 | La subcontratación NO añade un modelo/objetivo propio. | integrations.md |
| LCG2 | Solo `draft` se puede validar. | gotchas.md |
| LCG20 | Cambiar/desactivar un producto LC ya usado está bloqueado. | integrations.md |
| LCG3 | Solo productos FIFO/AVCO entran en el reparto. | gotchas.md |
| LCG4 | El asiento contable solo se crea si la valoración es `real_time`. | gotchas.md |
| LCG5 | El `standard_price` sube prorrateando solo el stock REMANENTE. | gotchas.md |
| LCG6 | Si toda la mercancía ya salió, no se ajusta el coste. | gotchas.md |
| LCG7 | `_check_sum` bloquea la validación si las líneas no cuadran. | gotchas.md |
| LCG8 | `compute_landed_cost` BORRA y regenera los adjustment lines | gotchas.md |
| LCG9 | Valoración por lote: el reparto se subdivide por lote. | gotchas.md |

## cross-module/loyalty

| ID | Título | Fichero |
|----|--------|---------|
| LOYCAP1 | ¿Qué tipos de programa hay? | capabilities.md |
| LOYCAP10 | ¿Solo con código vs automática? | capabilities.md |
| LOYCAP11 | ¿Recompensa de envío gratis? | capabilities.md |
| LOYCAP12 | ¿Aplicar en eCommerce y POS? | capabilities.md |
| LOYCAP13 | ¿Cupones de siguiente pedido? | capabilities.md |
| LOYCAP14 | ¿Combinar varios programas en un mismo pedido? | capabilities.md |
| LOYCAP15 | ¿Programa de referidos ("trae un amigo")? | capabilities.md |
| LOYCAP16 | ¿Niveles/tiers de fidelidad (Silver/Gold VIP)? | capabilities.md |
| LOYCAP17 | ¿El cliente ve su saldo de puntos (portal)? | capabilities.md |
| LOYCAP18 | ¿Código de barras/QR en cupones y tarjetas (escaneo en POS)? | capabilities.md |
| LOYCAP19 | ¿Cupón/tarjeta nominativo (ligado a un cliente concreto)? | capabilities.md |
| LOYCAP2 | ¿Descuentos % o fijo, sobre pedido / productos / el más barato? | capabilities.md |
| LOYCAP20 | ¿Reglas por importe mínimo o cantidad mínima de producto? | capabilities.md |
| LOYCAP3 | ¿Compra X llévate Y? | capabilities.md |
| LOYCAP4 | ¿Cupones con código (uso único/multiuso) y generación masiva? | capabilities.md |
| LOYCAP5 | ¿Tarjetas regalo (compra y canje)? | capabilities.md |
| LOYCAP6 | ¿Monedero / ewallet (saldo del cliente)? | capabilities.md |
| LOYCAP7 | ¿Puntos por dinero/unidad/pedido y canje por recompensas? | capabilities.md |
| LOYCAP8 | ¿Caducidad de puntos/cupones? | capabilities.md |
| LOYCAP9 | ¿Restricción por producto, importe mínimo, fechas, pricelist, web? | capabilities.md |
| LOYG1 | Cambiar `program_type` BORRA y recrea reglas/recompensas/comunicaciones. | gotchas.md |
| LOYG10 | (Des)archivar el programa propaga a hijos. | gotchas.md |
| LOYG11 | No se puede borrar un programa activo. | gotchas.md |
| LOYG12 | `trigger_product_ids` se descarta salvo en gift_card/ewallet. | gotchas.md |
| LOYG13 | El cálculo de productos descontables se puede desactivar por parámetro. | gotchas.md |
| LOYG14 | Los datos demo de gift_card/ewallet pueden no existir. | gotchas.md |
| LOYG15 | Las líneas de recompensa NO se copian. | integrations.md |
| LOYG16 | `ondelete='restrict'` en `reward_id`/`coupon_id`. | integrations.md |
| LOYG17 | Los puntos solo se mueven en estado `sale`. | integrations.md |
| LOYG18 | Cancelar revierte puntos e historial pero NO regenera cupones. | integrations.md |
| LOYG19 | Un cupón "future" no se canjea en el pedido que lo genera. | integrations.md |
| LOYG2 | `is_nominative` decide gift_card/ewallet/loyalty vs promoción. | gotchas.md |
| LOYG20 | `_get_no_effect_on_threshold_lines` es el punto de extensión de líneas neutras. | integrations.md |
| LOYG21 | gift_card/ewallet se aplican sobre el total CON impuestos y envío. | integrations.md |
| LOYG22 | Programas nominativos solo para usuarios logueados en web. | integrations.md |
| LOYG23 | Des-reclamar una recompensa web la mete en `disabled_auto_rewards`. | integrations.md |
| LOYG24 | Solo una recompensa de envío gratis simultánea. | integrations.md |
| LOYG25 | Programa y pricelist DEBEN compartir moneda. | gotchas.md |
| LOYG26 | La divisa del programa la fija la COMPAÑÍA, no la eliges. | gotchas.md |
| LOYG27 | El descuento nunca supera el total actual del pedido. | gotchas.md |
| LOYG28 | `per_point` recalcula el coste real de puntos por divisa y redondeo. | gotchas.md |
| LOYG29 | Los puntos `per_point` se truncan a múltiplos enteros de la recompensa. | gotchas.md |
| LOYG3 | `is_payment_program` distingue "pagar" de "descontar". | gotchas.md |
| LOYG30 | "Cheapest" mezcla base con-IVA y sin-IVA en dos acumuladores. | gotchas.md |
| LOYG31 | El producto de recompensa NO puede ser `combo`. | gotchas.md |
| LOYG32 | Gift card recomputa el price_unit metiendo los impuestos price_include del producto. | gotchas.md |
| LOYG33 | La caducidad del cupón se mide contra la fecha del PAGO, no "hoy". | gotchas.md |
| LOYG34 | El orden de aplicación de programas es por `sequence`, NO por mejor descuento. | gotchas.md |
| LOYG35 | Cambiar `program_type` en write inyecta el tipo en el contexto para el `default_get` del | gotchas.md |
| LOYG36 | gift/ewallet descuentan sobre el total CON impuestos fijos y envío. | gotchas.md |
| LOYG37 | Un cupón `future` nominativo cambia quién puede canjear dónde. | gotchas.md |
| LOYG4 | `reward_point_split` prohibido en loyalty/ewallet. | gotchas.md |
| LOYG5 | El código de una regla no puede coincidir con el de ninguna tarjeta (y viceversa). | gotchas.md |
| LOYG6 | Las tarjetas de fidelización NO pueden caducar. | gotchas.md |
| LOYG7 | Email automático al crear la tarjeta (si tiene partner). | gotchas.md |
| LOYG8 | Email "al alcanzar X puntos": solo el hito MÁS ALTO cruzado. | gotchas.md |
| LOYG9 | Cada recompensa crea un producto de descuento OCULTO. | gotchas.md |

## cross-module/mrp

| ID | Título | Fichero |
|----|--------|---------|
| KITCAP1 | ¿Puedo vender un "pack" que en realidad son varios productos? | capabilities.md |
| KITCAP10 | ¿Puedo entregar kits parcialmente? | capabilities.md |
| KITCAP11 | ¿El kit funciona en compras (recibir un pack del proveedor como componentes)? | capabilities.md |
| KITCAP12 | ¿El precio del kit es la suma de sus componentes? | capabilities.md |
| KITCAP13 | ¿Puedo poner precio a los componentes dentro del kit? | capabilities.md |
| KITCAP14 | ¿Cómo se factura un kit? | capabilities.md |
| KITCAP15 | ¿Cómo se reparte el coste entre el producto principal y los subproductos? | capabilities.md |
| KITCAP16 | ¿La valoración de inventario es del kit o de los componentes? | capabilities.md |
| KITCAP17 | ¿El margen/coste del kit sale bien en informes? | capabilities.md |
| KITCAP18 | ¿Un kit puede contener otro kit (anidado)? | capabilities.md |
| KITCAP19 | ¿Un kit puede mezclar productos almacenables y servicios? | capabilities.md |
| KITCAP2 | ¿Un kit crea una orden de fabricación (MO)? | capabilities.md |
| KITCAP20 | ¿Los componentes pueden estar en UoM distinta al kit? | capabilities.md |
| KITCAP21 | ¿Puedo tener un kit por variante concreta? | capabilities.md |
| KITCAP22 | ¿Puedo condicionar componentes del kit por atributo/variante? | capabilities.md |
| KITCAP23 | ¿Cuántos componentes puede tener un kit? | capabilities.md |
| KITCAP24 | ¿Un kit puede tener subproductos (by-products)? | capabilities.md |
| KITCAP25 | ¿Se abastecen automáticamente los componentes de un kit vendido? | capabilities.md |
| KITCAP26 | ¿Puedo poner una regla de reabastecimiento (orderpoint) sobre el kit? | capabilities.md |
| KITCAP27 | ¿El MPS planifica kits? | capabilities.md |
| KITCAP28 | ¿Un kit puede ser MTO (fabricar/comprar bajo pedido)? | capabilities.md |
| KITCAP29 | ¿Los kits funcionan en POS? | capabilities.md |
| KITCAP3 | ¿Necesito instalar la app Fabricación para usar kits? | capabilities.md |
| KITCAP30 | ¿Los kits funcionan en eCommerce? | capabilities.md |
| KITCAP31 | ¿Diferencia kit vs "producto combo" (POS/v18)? | capabilities.md |
| KITCAP32 | ¿Diferencia kit vs producto con opciones/accesorios? | capabilities.md |
| KITCAP33 | ¿Puedo cambiar las líneas de un kit ya usado en movimientos? | capabilities.md |
| KITCAP34 | ¿Puedo borrar un kit con MOs en curso? | capabilities.md |
| KITCAP35 | ¿El kit soporta lotes/números de serie? | capabilities.md |
| KITCAP36 | ¿Devoluciones (RMA) de un kit? | capabilities.md |
| KITCAP37 | ¿El coste del kit se ve en la ficha del producto kit? | capabilities.md |
| KITCAP38 | ¿Sigue existiendo `type='phantom'` en v19? | capabilities.md |
| KITCAP39 | ¿Qué se añadió en v19 en `mrp.bom`? | capabilities.md |
| KITCAP4 | ¿Cómo sé si un producto es un kit? | capabilities.md |
| KITCAP40 | ¿Cambió la definición de constraints en v19? | capabilities.md |
| KITCAP41 | ¿`explode`, `_bom_find`, `_skip_bom_line`, `cost_share` cambian en v19? | capabilities.md |
| KITCAP5 | ¿Un kit tiene stock propio? | capabilities.md |
| KITCAP6 | ¿Puedo inventariar un kit como tal (ajuste de stock del kit)? | capabilities.md |
| KITCAP7 | ¿Cuándo se descompone el kit? | capabilities.md |
| KITCAP8 | ¿En el pedido de venta el cliente ve el kit o los componentes? | capabilities.md |
| KITCAP9 | ¿Cómo se calcula la cantidad entregada de un kit? | capabilities.md |
| MCG1 | Enable Manufacturing | config_guide.md |
| MCG10 | Unbuild (Disassembly) | config_guide.md |
| MCG11 | Configure Manufacturing Steps (2/3 step) | config_guide.md |
| MCG2 | Configure Product for Manufacturing | config_guide.md |
| MCG3 | Configure Kit (Phantom BOM) | config_guide.md |
| MCG4 | Create Manufacturing Order | config_guide.md |
| MCG5 | Configure Subcontracting (Basic) | config_guide.md |
| MCG6 | Configure Component Supply to Subcontractor | config_guide.md |
| MCG7 | Configure Multi-Level BOM (Sub-assemblies) | config_guide.md |
| MCG8 | Configure Byproducts | config_guide.md |
| MCG9 | Configure BOM per Warehouse | config_guide.md |
| MG1 | Production location is virtual | gotchas.md |
| MG10 | Production cost = components + operations + extra_cost | gotchas.md |
| MG11 | WIP account entries only with real_time valuation | gotchas.md |
| MG12 | Multi-level BOMs need reordering rules, not just MTO | gotchas.md |
| MG13 | 2-step manufacturing creates pickings BEFORE the MO | gotchas.md |
| MG14 | Subcontractor portal has very limited write access | gotchas.md |
| MG15 | MPS skips phantom BOMs to avoid double procurement | gotchas.md |
| MG16 | Cancelling MO cascades to stock moves, not to POs/SOs | gotchas.md |
| MG17 | Consumption='strict' blocks mismatched quantities | gotchas.md |
| MG18 | Finished product move_dest_ids links to SO delivery | gotchas.md |
| MG19 | No native re-work flow | gotchas.md |
| MG2 | Phantom BOMs explode at move confirmation, not MO creation | gotchas.md |
| MG20 | Analytic distribution propagates from MO uniformly | gotchas.md |
| MG21 | extra_cost does NOT generate a separate journal entry | gotchas.md |
| MG22 | Cost Structure Report requires mrp_account_enterprise | gotchas.md |
| MG23 | Kit (phantom BOM) supports cross-UoM components | gotchas.md |
| MG24 | BOM component qty can include expected shrinkage | gotchas.md |
| MG25 | Procurement MOs are created as SUPERUSER | gotchas.md |
| MG26 | mrp_subcontracting_account splits the receipt credit into component + service lines | gotchas.md |
| MG27 | _bom_find resolves by `sequence`, not by picking_type specificity | gotchas.md |
| MG28 | 3-step (`pbm_sam`) MOs produce to `sam_loc_id`, not Stock | gotchas.md |
| MG29 | MPS replenishment ignores lead/produce delays (`skip_lead_time=True`) | gotchas.md |
| MG3 | _run_manufacture() merges into existing draft MOs | gotchas.md |
| MG30 | Finished product cannot also be a byproduct of its own BoM | gotchas.md |
| MG31 | `button_mark_done` sets un-consumed moves to `done` with qty 0, it does NOT cancel them | gotchas.md |
| MG32 | Backorder pop-up is governed by `picking_type_id.create_backorder`, not by the user | gotchas.md |
| MG33 | Marking an MO done with `qty_producing == 0` triggers mass-produce / auto-fill, not an empty close | gotchas.md |
| MG34 | Subcontracting valuation forces `price_unit` and bypasses automatic raw-move valuation | gotchas.md |
| MG4 | Backorder MOs share procurement_group_id | gotchas.md |
| MG5 | manual_consumption requires operator to confirm qty | gotchas.md |
| MG6 | unit_factor scaling produces rounding issues | gotchas.md |
| MG7 | Subcontract BOMs cannot have operations or byproducts | gotchas.md |
| MG8 | Subcontracting changes move location silently | gotchas.md |
| MG9 | Kit cost distributed via cost_share, not proportionally | gotchas.md |
| MRPCAP1 | ¿BOM multinivel (sub-conjuntos)? | capabilities.md |
| MRPCAP10 | ¿Desechos (scrap)? | capabilities.md |
| MRPCAP11 | ¿Fabricación bajo pedido (MTO desde venta)? | capabilities.md |
| MRPCAP12 | ¿Gestión de cambios de ingeniería / PLM (ECO)? | capabilities.md |
| MRPCAP13 | ¿App de taller (shop floor / tablet)? | capabilities.md |
| MRPCAP14 | ¿Trazabilidad de lotes/serie en producción? | capabilities.md |
| MRPCAP15 | ¿Mantenimiento de máquinas integrado? | capabilities.md |
| MRPCAP16 | ¿Coste real de fabricación (componentes + operaciones)? | capabilities.md |
| MRPCAP17 | ¿Consumo flexible/backflush de componentes? | capabilities.md |
| MRPCAP18 | ¿Reabastecimiento de componentes automático? | capabilities.md |
| MRPCAP19 | ¿Puedo producir en una UoM distinta a la de stock (ej. BoM produce 1 docena)? | capabilities.md |
| MRPCAP2 | ¿Kits / phantom BOM? | capabilities.md |
| MRPCAP20 | ¿Cada línea de componente puede ir en su propia UoM? | capabilities.md |
| MRPCAP21 | ¿Fabricaciones parciales con decimales cuadran los componentes? | capabilities.md |
| MRPCAP3 | ¿BOM por variante? | capabilities.md |
| MRPCAP4 | ¿Órdenes de trabajo con centros de trabajo y rutas? | capabilities.md |
| MRPCAP5 | ¿Capacidad/carga de centros de trabajo? | capabilities.md |
| MRPCAP6 | ¿Subcontratación (fabricación externa)? | capabilities.md |
| MRPCAP7 | ¿Plan maestro de producción (MPS)? | capabilities.md |
| MRPCAP8 | ¿Controles de calidad en fabricación? | capabilities.md |
| MRPCAP9 | ¿Subproductos (by-products)? | capabilities.md |
| VXK1 | Tengo un producto con tallas/colores. ¿Puedo tener una BoM distinta por variante? | integrations.md |
| VXK10 | Tengo un atributo "Grabado" que NO crea variantes. ¿Puede aun así añadir un componente/operación al  | integrations.md |
| VXK11 | ¿Cómo llega ese valor no_variant desde la línea de venta hasta la explosión de la BoM? | integrations.md |
| VXK12 | Un valor no_variant con precio extra, ¿se cobra sin crear variante? | integrations.md |
| VXK13 | Con un catálogo enorme configurable, ¿puedo NO pre-generar variantes y crearlas al vender? | integrations.md |
| VXK14 | Esa variante creada al vender, ¿arrastra la BoM correcta a la MO? | integrations.md |
| VXK15 | ¿El flujo venta→fabricación de una variante configurada es MTO nativo? | integrations.md |
| VXK16 | Si la combinación elegida está excluida, ¿puede llegar a fabricarse? | integrations.md |
| VXK17 | ¿Un kit puede contener variantes concretas (talla M + color Rojo)? | integrations.md |
| VXK18 | ¿Un componente de un kit puede ser a su vez un producto fabricado (con su MO)? | integrations.md |
| VXK19 | ¿Un kit puede contener otro kit que contiene variantes? | integrations.md |
| VXK2 | Si defino una BoM genérica (template) y otra específica de la variante "Rojo", ¿cuál gana al fabrica | integrations.md |
| VXK20 | Al vender un kit de variantes, ¿se reabastece cada variante-componente por separado? | integrations.md |
| VXK21 | ¿Puedo condicionar los componentes de un kit según la variante del propio kit? | integrations.md |
| VXK22 | ¿Puedo calcular el coste de una variante fabricada a partir de su BoM? | integrations.md |
| VXK23 | ¿El coste desde BoM respeta las líneas condicionadas por variante? | integrations.md |
| VXK24 | En un kit, ¿cómo se reparte el coste real entre componentes al vender? | integrations.md |
| VXK25 | ¿El `standard_price` del producto kit refleja la suma de sus componentes? | integrations.md |
| VXK26 | ¿Coste distinto por variante y por compañía a la vez? | integrations.md |
| VXK27 | ¿Cuándo uso variantes y cuándo un kit? | integrations.md |
| VXK28 | Mi configurable genera 5000 combinaciones. ¿Qué evita el colapso? | integrations.md |
| VXK29 | ¿Una BoM matricial (líneas condicionadas) reemplaza a tener N BoMs por variante? | integrations.md |
| VXK3 | ¿Cómo elige Odoo la BoM al crear la MO desde una venta MTO? | integrations.md |
| VXK30 | ¿Un "pack" que el cliente configura (elige 1 de N) es kit o combo? | integrations.md |
| VXK31 | Cambio la BoM de una variante con MOs abiertas. ¿Qué pasa con esas MOs? | integrations.md |
| VXK32 | ¿Puedo cambiar los atributos de un producto ya fabricado/vendido sin romper el historial? | integrations.md |
| VXK33 | Cambio las líneas de un kit ya usado en entregas. ¿Afecta a lo ya explotado? | integrations.md |
| VXK34 | Gestión formal de cambios de BoM/variante (aprobaciones). | integrations.md |
| VXK35 | ¿Puedo trazar lote/serie de los componentes de un kit vendido? | integrations.md |
| VXK36 | ¿El forecast/reabastecimiento distingue por variante? | integrations.md |
| VXK37 | ¿La cantidad entregada de un kit de variantes se calcula bien en la venta? | integrations.md |
| VXK38 | ¿Cambia algo del flujo variante×kit×BoM en v19? | integrations.md |
| VXK4 | ¿Puedo tener varias BoM para la misma variante y elegir según el almacén? | integrations.md |
| VXK5 | ¿Y por compañía (multi-company)? | integrations.md |
| VXK6 | En una BoM compartida por todas las variantes, ¿puedo hacer que un componente solo entre para la tal | integrations.md |
| VXK7 | ¿Puedo condicionar también una operación (ruta) o un subproducto a una variante? | integrations.md |
| VXK8 | Con una sola BoM "matricial" (líneas condicionadas por atributo), ¿evito crear una BoM por variante? | integrations.md |
| VXK9 | Si cambio los valores posibles del atributo, ¿se limpian las condiciones de las líneas? | integrations.md |

## cross-module/payment

| ID | Título | Fichero |
|----|--------|---------|
| PAYCAP1 | ¿Qué pasarelas trae de serie? | capabilities.md |
| PAYCAP10 | ¿Enviar enlace de pago desde backend? | capabilities.md |
| PAYCAP11 | ¿Express checkout (Apple Pay / Google Pay)? | capabilities.md |
| PAYCAP12 | ¿3DS / SCA? | capabilities.md |
| PAYCAP13 | ¿Comisiones del proveedor (fees) reflejadas? | capabilities.md |
| PAYCAP14 | ¿POS y eCommerce comparten este motor? | capabilities.md |
| PAYCAP15 | ¿Anticipo / down payment en el pedido (cobrar % antes)? | capabilities.md |
| PAYCAP16 | ¿Domiciliación SEPA / recibo domiciliado (direct debit)? | capabilities.md |
| PAYCAP17 | ¿Conciliación automática del pago con la factura? | capabilities.md |
| PAYCAP18 | ¿Multi-adquirente por compañía (multi-empresa)? | capabilities.md |
| PAYCAP19 | ¿Reintento de pago fallido (dunning) con token/suscripción? | capabilities.md |
| PAYCAP2 | ¿Guardar tarjeta / tokenización? | capabilities.md |
| PAYCAP20 | ¿El cliente paga varias facturas juntas desde el portal? | capabilities.md |
| PAYCAP3 | ¿Cobro recurrente automático con token (S2S)? | capabilities.md |
| PAYCAP4 | ¿Autorizar y capturar después (captura manual)? | capabilities.md |
| PAYCAP5 | ¿Reembolsar desde Odoo? | capabilities.md |
| PAYCAP6 | ¿Reembolso parcial? | capabilities.md |
| PAYCAP7 | ¿Modo test/sandbox por proveedor? | capabilities.md |
| PAYCAP8 | ¿Límite de importe / restricción por moneda o país? | capabilities.md |
| PAYCAP9 | ¿Pago manual/offline con instrucciones (transferencia)? | capabilities.md |
| PAYG1 | El `operation` no es fiable en draft/pending. | gotchas.md |
| PAYG10 | Las acciones backend usan sudo deliberadamente. | gotchas.md |
| PAYG11 | La transacción fuente se confirma sola cuando los hijos cubren el importe. | gotchas.md |
| PAYG12 | Ningún `_send_*_request` funciona con el provider `disabled`. | gotchas.md |
| PAYG13 | No puedes borrar un provider con xmlid ni cambiarle la compañía si tiene transacciones. | gotchas.md |
| PAYG14 | Habilitar un payment method exige un provider habilitado que lo soporte. | gotchas.md |
| PAYG15 | Sin webhook, el cobro queda "colgado" hasta el cron. | integrations.md |
| PAYG16 | `_post_process` es idempotente por flag, NO por lógica, y el flag se RESETEA. | integrations.md |
| PAYG17 | Pagos agrupados rompen la confirmación. | integrations.md |
| PAYG18 | Conciliación silenciosa por `destination_account_id`. | integrations.md |
| PAYG19 | `validation` no crea payment ni confirma pedido. | integrations.md |
| PAYG2 | Los estados no revierten libremente y un estado ilegal se ignora SIN error. | gotchas.md |
| PAYG20 | Importe negativo → pago outbound. | integrations.md |
| PAYG21 | Facturación automática post-cobro es opcional y asíncrona. | integrations.md |
| PAYG22 | El autor del chatter cambia según contexto. | integrations.md |
| PAYG23 | Captura/void/refund parciales crean tx HIJAS, no modifican la original. | integrations.md |
| PAYG24 | Anular calcula el importe ya capturado por los hijos. | integrations.md |
| PAYG25 | Webhook sin secreto configurado se acepta SIN verificar firma. | gotchas.md |
| PAYG26 | El webhook rechaza eventos de más de 10 min (anti-replay). | gotchas.md |
| PAYG27 | El reembolso de tx en `error` SÍ puede venir de `done` (excepción única a los estados). | gotchas.md |
| PAYG28 | Un refund confirmado dispara el cron de post-proceso al instante. | gotchas.md |
| PAYG29 | `requires_action` (SCA/3DS pendiente) mapea a `draft`, no a `pending`. | gotchas.md |
| PAYG3 | Cambiar el `state` de un provider archiva sus tokens. | gotchas.md |
| PAYG30 | El reembolso parcial disponible se calcula sobre PAGOS confirmados, no sobre tx. | gotchas.md |
| PAYG31 | Los tokens de providers con captura manual quedan EXCLUIDOS del selector de pago. | gotchas.md |
| PAYG32 | Un pago por token que no llega a `done` se CANCELA, no queda pendiente. | gotchas.md |
| PAYG33 | El partner del `account.payment` se fuerza al `commercial_partner_id`. | gotchas.md |
| PAYG34 | Stripe recibe el código del método PRIMARIO en el PaymentIntent, pero el BRAND en el | gotchas.md |
| PAYG35 | El descuento por pronto pago (EPD) solo se aplica si el importe casa EXACTO. | gotchas.md |
| PAYG36 | `_update_landing_route` concatena query params en crudo. | gotchas.md |
| PAYG37 | El refund se localiza por `provider_reference` en `charge.refund.updated`. | gotchas.md |
| PAYG4 | No puedes desarchivar un token con method inactivo o provider disabled. | gotchas.md |
| PAYG5 | `maximum_amount` se compara en la moneda de la COMPAÑÍA, no en la del pago. | gotchas.md |
| PAYG6 | El cron de post-procesado se auto-activa/desactiva según haya providers no-disabled. | gotchas.md |
| PAYG7 | El cron de post-proceso solo reintenta 4 días. | gotchas.md |
| PAYG8 | `amount` se invalida tras create para arreglar el float. | gotchas.md |
| PAYG9 | El estado `authorized` exige que el provider soporte captura manual. | gotchas.md |

## cross-module/product

| ID | Título | Fichero |
|----|--------|---------|
| ANG13 | (analytic): tampoco heredan `analytic_distribution`. Ver `analytic/gotchas.md`. | integrations.md |
| ATTRCAP1 | ¿Los atributos son globales o por producto? | capabilities.md |
| ATTRCAP10 | ¿Puedo tener un atributo que NO genere variantes (solo opción visual)? | capabilities.md |
| ATTRCAP11 | ¿Puedo diferir la creación de variantes a cuando se venden (catálogo enorme)? | capabilities.md |
| ATTRCAP12 | ¿Puedo cambiar el modo de un atributo ya usado en productos? | capabilities.md |
| ATTRCAP13 | ¿Puedo mezclar modos en el mismo producto (un atributo always + otro dynamic)? | capabilities.md |
| ATTRCAP14 | ¿`no_variant` y `dynamic` se pueden combinar en un producto? | capabilities.md |
| ATTRCAP15 | ¿Qué widgets de selección hay para un atributo? | capabilities.md |
| ATTRCAP16 | ¿Puedo mostrar swatches de color? | capabilities.md |
| ATTRCAP17 | ¿Puedo usar imágenes como swatch (no solo color)? | capabilities.md |
| ATTRCAP18 | ¿Puedo cambiar el display_type después de crear el atributo? | capabilities.md |
| ATTRCAP19 | ¿El display_type afecta a las variantes generadas? | capabilities.md |
| ATTRCAP2 | ¿Puedo usar el mismo atributo "Color" en cientos de productos? | capabilities.md |
| ATTRCAP20 | ¿`radio`/`pills`/`select` cambian la lógica de negocio? | capabilities.md |
| ATTRCAP21 | ¿Puedo permitir seleccionar varios valores del mismo atributo a la vez (toppings)? | capabilities.md |
| ATTRCAP22 | ¿Multi-checkbox puede generar variantes? | capabilities.md |
| ATTRCAP23 | ¿El onchange fuerza no_variant al elegir multi? | capabilities.md |
| ATTRCAP24 | ¿El nombre del atributo es traducible? | capabilities.md |
| ATTRCAP25 | ¿Puedo ordenar los atributos en el configurador? | capabilities.md |
| ATTRCAP26 | ¿Reordenar atributos invalida caches? | capabilities.md |
| ATTRCAP27 | ¿El orden de los valores dentro del atributo se respeta? | capabilities.md |
| ATTRCAP28 | ¿Puedo archivar un atributo? | capabilities.md |
| ATTRCAP29 | ¿Puedo borrar un atributo usado en productos? | capabilities.md |
| ATTRCAP3 | ¿Cuántos atributos puede tener un producto? | capabilities.md |
| ATTRCAP30 | ¿Duplicar un atributo copia sus valores? | capabilities.md |
| ATTRCAP31 | ¿`active=False` oculta el atributo sin borrarlo? | capabilities.md |
| ATTRCAP32 | ¿Puedo renombrar un atributo en producción? | capabilities.md |
| ATTRCAP33 | ¿Hay riesgo al cambiar el modo por script/ORM saltándome el aviso? | capabilities.md |
| ATTRCAP34 | ¿El constraint multi/no_variant se valida a nivel BD? | capabilities.md |
| ATTRCAP35 | ¿Los atributos afectan a precios? | capabilities.md |
| ATTRCAP36 | ¿Puedo excluir combinaciones (talla XL no en color Rojo)? | capabilities.md |
| ATTRCAP37 | ¿Los atributos se usan en pricelists por variante? | capabilities.md |
| ATTRCAP38 | ¿Los atributos condicionan líneas de BoM (MRP)? | capabilities.md |
| ATTRCAP39 | ¿Los atributos funcionan en POS? | capabilities.md |
| ATTRCAP4 | ¿Cuántos valores puede tener un atributo? | capabilities.md |
| ATTRCAP40 | ¿Los atributos se usan como filtros/facetas en eCommerce? | capabilities.md |
| ATTRCAP41 | ¿Puedo permitir que el cliente escriba texto libre (grabado)? | capabilities.md |
| ATTRCAP42 | ¿Puedo importar atributos y valores por CSV/Excel? | capabilities.md |
| ATTRCAP43 | ¿Puedo crear atributos por API (XML-RPC/JSON-RPC)? | capabilities.md |
| ATTRCAP44 | ¿Hay un asistente para actualizar precios extra masivamente? | capabilities.md |
| ATTRCAP45 | ¿Cambió la definición de constraints en v19? | capabilities.md |
| ATTRCAP46 | ¿Hay nuevos display_type en v19? | capabilities.md |
| ATTRCAP47 | ¿Cambió alguna acción a readonly en v19? | capabilities.md |
| ATTRCAP48 | ¿`create_variant`, `_create_variant_ids`, `_without_no_variant_attributes` siguen igual en v19? | capabilities.md |
| ATTRCAP5 | ¿Puedo saber en qué productos se usa un atributo? | capabilities.md |
| ATTRCAP6 | ¿Los atributos se ven siempre en la ficha de producto? | capabilities.md |
| ATTRCAP7 | ¿Puedo tener atributos sin activar "Variantes" en ajustes? | capabilities.md |
| ATTRCAP8 | ¿Qué modos de generación de variante existen? | capabilities.md |
| ATTRCAP9 | ¿Cuál es el modo por defecto? | capabilities.md |
| CTG1 | `property_cost_method` vive en categoría, no en producto | gotchas.md |
| CTG10 | `cost_method` y `valuation` en `product.template` son related a categoría | gotchas.md |
| CTG2 | Cambiar `categ_id` de producto con stock dispara reevaluación masiva | gotchas.md |
| CTG3 | `removal_strategy_id` — ubicación gana a categoría | gotchas.md |
| CTG4 | Cuentas resueltas con cascada Producto → Categoría → Padre | gotchas.md |
| CTG5 | `parent_path` SQL prefix matching | gotchas.md |
| CTG6 | 3 categorías master no se pueden borrar | gotchas.md |
| CTG7 | `product_properties_definition` (categoría) define schema dinámico | gotchas.md |
| CTG8 | `product.category._compute_product_count` cuenta toda la rama | gotchas.md |
| CTG9 | Mover categoría con `parent_id` change valida recursión | gotchas.md |
| PAVCAP1 | ¿Puedo cobrar un extra por un valor (ej. talla XXL +5€)? | capabilities.md |
| PAVCAP10 | ¿Dónde se guarda el texto que escribe el cliente? | capabilities.md |
| PAVCAP11 | ¿Un valor is_custom crea una variante nueva por cada texto? | capabilities.md |
| PAVCAP12 | ¿is_custom funciona en cualquier modo de variante? | capabilities.md |
| PAVCAP13 | ¿El nombre del custom value muestra "Atributo: texto"? | capabilities.md |
| PAVCAP14 | ¿Puedo asignar un color HTML a un valor? | capabilities.md |
| PAVCAP15 | ¿Puedo poner una imagen (foto de tela) a un valor? | capabilities.md |
| PAVCAP16 | ¿Hay un color de UI (kanban) distinto del swatch? | capabilities.md |
| PAVCAP17 | ¿El display_type se hereda del atributo al valor? | capabilities.md |
| PAVCAP18 | ¿El nombre del valor es traducible? | capabilities.md |
| PAVCAP19 | ¿Puedo ordenar los valores dentro del atributo? | capabilities.md |
| PAVCAP2 | ¿El extra es global o por producto? | capabilities.md |
| PAVCAP20 | ¿Puedo tener dos valores con el mismo nombre en un atributo? | capabilities.md |
| PAVCAP21 | ¿Puedo mover un valor de un atributo a otro? | capabilities.md |
| PAVCAP22 | ¿Puedo borrar un valor usado en productos? | capabilities.md |
| PAVCAP23 | ¿Qué pasa si borro un valor usado solo en variantes archivadas? | capabilities.md |
| PAVCAP24 | ¿Puedo archivar un valor? | capabilities.md |
| PAVCAP25 | ¿Borrar un atributo borra sus valores? | capabilities.md |
| PAVCAP26 | ¿Duplicar un atributo duplica los enlaces a líneas de los valores? | capabilities.md |
| PAVCAP27 | ¿Puedo añadir un valor nuevo a todos los productos que ya usan el atributo? | capabilities.md |
| PAVCAP28 | ¿Puedo saber en qué productos se usa un valor? | capabilities.md |
| PAVCAP29 | ¿Por qué el campo se llama `ptav_active` y no `active`? | capabilities.md |
| PAVCAP3 | ¿Si cambio el extra global se actualizan los productos existentes? | capabilities.md |
| PAVCAP30 | ¿Puedo cambiar el valor (PAV) de una PTAV existente? | capabilities.md |
| PAVCAP31 | ¿Puedo mover una PTAV a otro producto? | capabilities.md |
| PAVCAP32 | ¿Puedo asignar variantes a una PTAV directamente? | capabilities.md |
| PAVCAP33 | ¿El nombre mostrado de una PTAV incluye siempre el atributo? | capabilities.md |
| PAVCAP34 | ¿Cómo se compone el nombre de la variante? | capabilities.md |
| PAVCAP35 | ¿La unicidad de valor por producto está garantizada? | capabilities.md |
| PAVCAP36 | ¿Se valida que la PTAV pertenezca a los valores de su línea? | capabilities.md |
| PAVCAP37 | ¿Borrar una PTAV usada en pedidos rompe el historial? | capabilities.md |
| PAVCAP38 | ¿Borrar una PTAV de una línea single-value la quita de las variantes? | capabilities.md |
| PAVCAP39 | ¿Puedo excluir combinaciones desde un valor (Rojo no en talla XL)? | capabilities.md |
| PAVCAP4 | ¿Cómo propago un cambio de extra a todas las variantes existentes? | capabilities.md |
| PAVCAP40 | ¿Puedo excluir valores de productos opcionales/accesorios (cross-producto)? | capabilities.md |
| PAVCAP41 | ¿Las exclusiones son bidireccionales? | capabilities.md |
| PAVCAP42 | ¿Una exclusión puede dejar el producto sin variantes? | capabilities.md |
| PAVCAP43 | ¿Cambió el modelo PAV en v19? | capabilities.md |
| PAVCAP44 | ¿Cambió el modelo PTAV en v19? | capabilities.md |
| PAVCAP45 | ¿`price_extra`, `is_custom`, exclusiones, `_get_combination_name` cambian en v19? | capabilities.md |
| PAVCAP5 | ¿El extra puede ser negativo (descuento por valor)? | capabilities.md |
| PAVCAP6 | ¿El extra respeta la moneda del producto? | capabilities.md |
| PAVCAP7 | ¿El extra entra en la pricelist / se recalcula con reglas? | capabilities.md |
| PAVCAP8 | ¿Puedo ver si un valor tiene el precio "desincronizado"? | capabilities.md |
| PAVCAP9 | ¿Puedo dejar que el cliente escriba un valor (grabado, dedicatoria)? | capabilities.md |
| PLSG1 | `applied_on` lexicográfico determina precedencia | gotchas.md |
| PLSG10 | Pricelist por country_group resuelve por país del partner | gotchas.md |
| PLSG11 | `applied_on` se re-asigna automáticamente desde el contenido (NO hay `_onchange_rule_content`) | gotchas.md |
| PLSG12 | `currency_id` per pricelist != currency_id per company | gotchas.md |
| PLSG13 | Regla "based on Cost" / fórmula sobre coste ignora el `price_extra` de variantes | gotchas.md |
| PLSG2 | `min_quantity` en UoM del PRODUCTO, no del SO line | gotchas.md |
| PLSG3 | Pricelist archivada (`active=False`) SIGUE matcheando | gotchas.md |
| PLSG4 | Pricelist congelada en SO line al confirmar | gotchas.md |
| PLSG5 | `fixed_price` se interpreta en `product.uom_id` | gotchas.md |
| PLSG6 | Orden en fórmula es crítico: discount → round → surcharge → margin clamp | gotchas.md |
| PLSG7 | `base_pricelist_id` permite recursión, constraint solo evita auto-ref directa | gotchas.md |
| PLSG8 | `price_extra` (de variantes) se suma al base price ANTES de la regla | gotchas.md |
| PLSG9 | Multi-currency: conversion en `_compute_base_price` con `round=False` | gotchas.md |
| PRG1 | Siempre existe al menos una `product.product` incluso sin atributos | gotchas.md |
| PRG10 | Eliminar template raramente funciona | gotchas.md |
| PRG11 | `product.combo` (nuevo tipo v18) descompone en líneas hijas | gotchas.md |
| PRG12 | `image_1920` del template es un Image propio (image.mixin); es la VARIANTE la que hace fallback al t | gotchas.md |
| PRG13 | `is_storable` (v18+) no es `type='storable'` (v17) | gotchas.md |
| PRG14 | `product_variant_id` (singular) es solo la PRIMERA variante | gotchas.md |
| PRG15 | Multi-company: `company_id=False` significa "global", no "main company" | gotchas.md |
| PRG2 | `_inherits` (no `_inherit`) — campos del template escriben en el template | gotchas.md |
| PRG3 | `standard_price` es `company_dependent` en variant pero computed en template | gotchas.md |
| PRG4 | `weight` / `volume` del template solo funcionan si hay 1 sola variante | gotchas.md |
| PRG5 | `barcode` y `default_code` son **propios** de la variante | gotchas.md |
| PRG6 | Barcode único POR EMPRESA (también vs packaging) | gotchas.md |
| PRG7 | `product.template.list_price` NO aplica pricelist | gotchas.md |
| PRG8 | `lst_price` en variant SÍ incluye `price_extra` pero NO pricelist | gotchas.md |
| PRG9 | `categ_id` arrastra cuentas contables via property fields | gotchas.md |
| SIG1 | `sequence` gana sobre precio en `_select_seller` | gotchas.md |
| SIG2 | `min_qty` evaluado en UoM del vendor | gotchas.md |
| SIG3 | `product_uom` es related a `uom_po_id`, NO editable per-supplierinfo | gotchas.md |
| SIG4 | `product_id=False` aplica a TODAS las variantes | gotchas.md |
| SIG5 | Cambios en supplierinfo NO retroactivos | gotchas.md |
| SIG6 | `delay` (lead time) suma al `date_planned` | gotchas.md |
| SIG7 | `_sanitize_vals` rellena `product_tmpl_id` desde `product_id` pero NO valida coherencia | gotchas.md |
| SIG8 | Multi-company: `company_id=False` significa "todas las companies" | gotchas.md |
| SRVG1 | `service_tracking != 'no'` requiere módulo (`sale_project` o `sale_timesheet`) | gotchas.md |
| SRVG2 | Cambiar `service_tracking` retroactivo no afecta SOs existentes | gotchas.md |
| SRVG3 | `service_policy='delivered_timesheet'` requiere `sale_timesheet` | gotchas.md |
| SRVG4 | `service_policy='delivered_milestones'` crea `sale.order.line.milestone_ids` | gotchas.md |
| SRVG5 | Servicios y BoM — incompatibles en MRP nativo | gotchas.md |
| SRVG6 | `service_tracking='project_only'` crea proyecto completo, NO solo task | gotchas.md |
| SRVG7 | Servicios y stock — generalmente sin `is_storable` | gotchas.md |
| SRVG8 | `invoice_policy='delivery'` en service | gotchas.md |
| UOG1 | Solo conversión dentro de la misma categoría | gotchas.md |
| UOG10 | `uom_po_id` se resetea silenciosamente al cambiar `uom_id` a otra categoría | gotchas.md |
| UOG11 | `product_packaging` (v18) vs UoM — son ortogonales | gotchas.md |
| UOG12 | Conversión vendor → PO line → stock acumula errores | gotchas.md |
| UOG13 | SO line: el domain `category_id` restringe el selector, NO valida al escribir | gotchas.md |
| UOG14 | Multi-UoM en facturas: cambiar UoM SÍ recalcula price_unit (corregido v18) | gotchas.md |
| UOG15 | SII / EDI español: la UoM de la línea va al XML | gotchas.md |
| UOG16 | `decimal.precision` se aplica a `digits='Nombre'`, NO a `digits=(p, s)` | gotchas.md |
| UOG17 | Reducir `decimal.precision` solo AVISA (no está prohibido) | gotchas.md |
| UOG18 | `uom.group_uom` desactivado oculta UoMs pero NO las elimina | gotchas.md |
| UOG19 | POS no permite cambiar UoM en línea ni vender en packaging | gotchas.md |
| UOG2 | `factor` y `factor_inv` parecen redundantes pero NO son lo mismo | gotchas.md |
| UOG20 | UoM de servicios (hour/day) compite con working schedules en HR | gotchas.md |
| UOG21 | Convertir 0 cantidad siempre devuelve 0 sin error | gotchas.md |
| UOG22 | `_check_qty` (helper de packaging) puede devolver qty distinto al input por floats | gotchas.md |
| UOG23 | Cambiar `product_uom` en SO line re-resuelve el precio | gotchas.md |
| UOG24 | v19: la conversión deja de ser "misma categoría" y pasa a ser por árbol `relative_uom_id` | gotchas.md |
| UOG25 | v19: `product_uom` → `product_uom_id` en TODAS las líneas (rename BREAKING) | gotchas.md |
| UOG26 | v19: `sale.order.line.product_packaging_id`/`product_packaging_qty` ELIMINADOS (no reconvertidos) | gotchas.md |
| UOG27 | v19: `stock.move` gana campo NUEVO `packaging_uom_id` (NO es `product_packaging_id` reciclado) | gotchas.md |
| UOG28 | v19: `_compute_price` TAMBIÉN invierte la fórmula y NO valida árbol | gotchas.md |
| UOG29 | v19: `raise_if_failure` en `_compute_quantity` es un parámetro MUERTO | gotchas.md |
| UOG3 | Cambiar `factor` de UoM en uso NO actualiza datos históricos | gotchas.md |
| UOG30 | v19: `relative_uom_id` es `ondelete='cascade'` → borrar una UoM padre borra el subárbol | gotchas.md |
| UOG31 | v19: constraints de UoM cambian por completo | gotchas.md |
| UOG32 | v19: cambiar `product.uom_id` hace swap de NOMBRE 1:1, no conversión por factor | gotchas.md |
| UOG33 | v19: `allowed_uom_ids` limita el selector de UoM en líneas al producto, no al árbol | gotchas.md |
| UOG34 | v19: `product.uom` (modelo nuevo) exige `barcode` obligatorio y único global | gotchas.md |
| UOG35 | v19: muchas `uom.uom` estándar vienen ARCHIVADAS + los M2M aplican `active_test` al leer | gotchas.md |
| UOG4 | `_compute_quantity` por defecto redondea con `rounding_method='UP'` | gotchas.md |
| UOG5 | `_compute_price` NO redondea | gotchas.md |
| UOG6 | Precision `digits=0` en `factor`/`rounding` = NUMERIC sin límite | gotchas.md |
| UOG7 | `rounding` de la UoM vs `decimal.precision('Product Unit of Measure')` son COSAS DISTINTAS | gotchas.md |
| UOG8 | UoMs maestras del módulo `uom` no se pueden borrar | gotchas.md |
| UOG9 | `product_uom_hour` se protege adicionalmente cuando hr_timesheet está instalado | gotchas.md |
| VAG1 | `create_variant` NO se puede cambiar tras usar el atributo | gotchas.md |
| VAG10 | `ptav_product_variant_ids` es readonly — escribir lanza UserError | gotchas.md |
| VAG11 | PTAV `display_name` SIEMPRE prefija "Atributo: Value" (sin escape) | gotchas.md |
| VAG12 | Cambiar `attribute_id` o `product_tmpl_id` de una PTAL/PTAV está prohibido | gotchas.md |
| VAG13 | PTAL archivada se REACTIVA en `create` con misma combinación | gotchas.md |
| VAG14 | `product_template_variant_value_ids` excluye single-value lines | gotchas.md |
| VAG15 | Variantes archivadas siguen vinculadas a SO/PO/stock antiguos | gotchas.md |
| VAG16 | `combination_indices` puede dar falsos negativos en queries directas | gotchas.md |
| VAG17 | Atributos en BoM/MRP — variantes específicas | gotchas.md |
| VAG18 | eCommerce: PAV `image` aparece como swatch solo si display_type compatible | gotchas.md |
| VAG19 | Variantes en POS — limitación severa | gotchas.md |
| VAG2 | Atributo con un solo value se inyecta silenciosamente | gotchas.md |
| VAG20 | is_custom value crea texto libre — pero NO crea variante nueva | gotchas.md |
| VAG3 | `display_type='multi'` fuerza `create_variant='no_variant'` | gotchas.md |
| VAG4 | `dynamic` no crea variantes hasta el SO | gotchas.md |
| VAG5 | Limite hard de 1000 variantes por template (always) | gotchas.md |
| VAG6 | `exclude_for` es direccional pero Odoo valida ambas direcciones | gotchas.md |
| VAG7 | Exclusiones pueden dejar el template sin variantes válidas | gotchas.md |
| VAG8 | `ptav_active` (no `active`) — los views muestran archived por defecto | gotchas.md |
| VAG9 | `default_extra_price` (PAV) NO sincroniza con `price_extra` (PTAV) | gotchas.md |
| VARCAP1 | ¿Cómo crea Odoo las variantes de un producto? | capabilities.md |
| VARCAP10 | ¿Puedo NO pre-generar miles de variantes (configurador)? | capabilities.md |
| VARCAP11 | ¿Cómo se crea una variante bajo demanda? | capabilities.md |
| VARCAP12 | ¿Basta con que UN atributo sea dynamic para diferir todo? | capabilities.md |
| VARCAP13 | ¿`product_variant_count==0` es un bug en dynamic? | capabilities.md |
| VARCAP14 | ¿Puedo detectar por campo si un template es dinámico? (v19) | capabilities.md |
| VARCAP15 | ¿Un atributo no_variant genera product.product? | capabilities.md |
| VARCAP16 | ¿El extra price de un no_variant se aplica igual? | capabilities.md |
| VARCAP17 | ¿Cómo identifica Odoo una combinación única? | capabilities.md |
| VARCAP18 | ¿Hay unicidad a nivel BD de la combinación? | capabilities.md |
| VARCAP19 | ¿Puedo buscar una variante por combinación con `('combination_indices','=',...)`? | capabilities.md |
| VARCAP2 | ¿Siempre existe al menos una variante por producto? | capabilities.md |
| VARCAP20 | ¿Escribir la combinación en la variante refresca caches? | capabilities.md |
| VARCAP21 | ¿Los campos del template se ven en la variante? | capabilities.md |
| VARCAP22 | ¿Escribir un campo de template en una variante afecta a las hermanas? | capabilities.md |
| VARCAP23 | ¿Qué campos son propios de la variante (no del template)? | capabilities.md |
| VARCAP24 | ¿El coste (`standard_price`) puede diferir por variante y por compañía? | capabilities.md |
| VARCAP25 | ¿`weight`/`volume` del template reflejan la variante? | capabilities.md |
| VARCAP26 | ¿Puedo poner precio de venta (`list_price`) distinto por variante? | capabilities.md |
| VARCAP27 | ¿Puedo poner imagen distinta por variante? | capabilities.md |
| VARCAP28 | ¿Referencia interna/código de barras por variante? | capabilities.md |
| VARCAP29 | ¿Se borran o se archivan las variantes que dejan de ser válidas? | capabilities.md |
| VARCAP3 | ¿Puedo forzar la regeneración manual de variantes? | capabilities.md |
| VARCAP30 | ¿Una combinación archivada se reactiva si vuelve a ser válida? | capabilities.md |
| VARCAP31 | ¿Borrar la última variante borra el template? | capabilities.md |
| VARCAP32 | ¿Debo borrar manualmente variantes archivadas? | capabilities.md |
| VARCAP33 | ¿Qué pasa con los "combo items" al desaparecer una variante? | capabilities.md |
| VARCAP34 | ¿Cómo se calcula el precio de una variante? | capabilities.md |
| VARCAP35 | ¿El `lst_price` depende de la UoM del contexto? | capabilities.md |
| VARCAP36 | ¿Qué pasa si una exclusión elimina la última variante posible? | capabilities.md |
| VARCAP37 | ¿Miles de variantes degradan el sistema? | capabilities.md |
| VARCAP38 | ¿`_create_variant_ids` es transaccional/seguro ante fallos? | capabilities.md |
| VARCAP39 | ¿Puedo crear variantes por API sin pasar por atributos? | capabilities.md |
| VARCAP4 | ¿Un atributo con un solo valor crea variantes extra? | capabilities.md |
| VARCAP40 | ¿Cambió el motor `_create_variant_ids` en v19? | capabilities.md |
| VARCAP41 | ¿Qué se añadió en v19 sobre variantes? | capabilities.md |
| VARCAP42 | ¿`combination_indices`, `_inherits`, unicidad, límite 1000 siguen igual en v19? | capabilities.md |
| VARCAP5 | ¿Cómo sé cuántas variantes tiene un template? | capabilities.md |
| VARCAP6 | ¿En modo "always" cuándo se crean las variantes? | capabilities.md |
| VARCAP7 | ¿Hay un límite de variantes generables? | capabilities.md |
| VARCAP8 | ¿Puedo subir el límite de 1000? | capabilities.md |
| VARCAP9 | ¿Qué pasa si añado un valor a un atributo "always" ya en uso? | capabilities.md |

## cross-module/reporting

| ID | Título | Fichero |
|----|--------|---------|
| RPTG1 | wkhtmltopdf usa un WebKit MUY viejo (QtWebKit ~2012) → flexbox/grid/CSS moderno fallan | gotchas.md |
| RPTG10 | DPI bajo → imágenes/logos pixelados o desproporcionados | gotchas.md |
| RPTG11 | `css_margins` cambia QUIÉN controla los márgenes | gotchas.md |
| RPTG12 | El truco `?debug` y `/report/html` para depurar | gotchas.md |
| RPTG13 | wkhtmltopdf returncode 1 NO es error; -11 = falta de memoria | gotchas.md |
| RPTG14 | Saltos de página: `page-break-inside: avoid` SÍ funciona, `break-inside` (moderno) NO | gotchas.md |
| RPTG15 | El split por registro depende de los PDF outlines (`<h1>`) | gotchas.md |
| RPTG16 | Tablas enormes se parten en bloques de 500 filas (cambia la paginación) | gotchas.md |
| RPTG17 | `workers=1` → el PDF nunca se genera (estado `workers`) | gotchas.md |
| RPTG18 | wkhtmltopdf no instalado → UserError solo en runtime, no al crear el informe | gotchas.md |
| RPTG19 | Imágenes/QR vía URL en blanco sin cookie de sesión | gotchas.md |
| RPTG2 | Página en BLANCO con paperformat custom: `page_width`/`page_height` se ignoran si | gotchas.md |
| RPTG20 | `attachment_use` cachea el PDF: cambios al template no se ven | gotchas.md |
| RPTG21 | v19 sigue con wkhtmltopdf — NO migra a Chromium/Playwright | gotchas.md |
| RPTG22 | v19 detecta QT no parcheado en multi-documento | gotchas.md |
| RPTG23 | El PDF no se traduce al idioma del cliente: falta `lang` en el contexto del registro | gotchas.md |
| RPTG24 | "Cannot separate file to save as attachment": faltan `data-oe-model`/`data-oe-id` en el | gotchas.md |
| RPTG25 | Render QWeb es TODO-O-NADA: un `t-out`/`t-field` que peta tumba el PDF ENTERO | gotchas.md |
| RPTG26 | Paper Muncher (v20) | gotchas.md |
| RPTG3 | Header/footer se REPITEN o DESAPARECEN: el truco `subst()` JS | gotchas.md |
| RPTG4 | Header/footer desaparecen aleatoriamente: assets no cargan a tiempo (issue #2083) | gotchas.md |
| RPTG5 | `web.minimal_layout` != `web.external_layout` — confundirlos rompe el PDF | gotchas.md |
| RPTG6 | CSS no aplica en el PDF: falta `t-call-assets` o lo metiste en el bundle equivocado | gotchas.md |
| RPTG7 | Diferencia preview vs PDF por el reset CSS | gotchas.md |
| RPTG8 | `header_spacing` mal calculado → contenido bajo el header o header pisando el body | gotchas.md |
| RPTG9 | `data-report-*` del `<html>` raíz SOBRESCRIBEN el report.paperformat | gotchas.md |

## cross-module/security

| ID | Título | Fichero |
|----|--------|---------|
| SECG1 | Record rules: global=AND, group=OR | gotchas.md |
| SECG10 | Empty `group_id` in ACL = public access (deprecated) | gotchas.md |
| SECG11 | User-type groups are mutually exclusive | gotchas.md |
| SECG12 | `ir.rule` in `noupdate="1"` ignores code changes on upgrade | gotchas.md |
| SECG13 | `check_company=True` validates but doesn't filter the UI | gotchas.md |
| SECG14 | Portal `child_of [commercial_partner_id]` exposes all sibling contacts | gotchas.md |
| SECG15 | `groups='.'` means NO ONE (only sudo) | gotchas.md |
| SECG16 | ACL caches per (uid, mode) — programmatic changes need a cache clear | gotchas.md |
| SECG17 | Multiple ACL rows for the same model+group are OR-ed, not intersected | gotchas.md |
| SECG18 | Per-operation rules: a read-only rule leaves writes unrestricted | gotchas.md |
| SECG19 | Search silently returns fewer rows; direct browse raises | gotchas.md |
| SECG2 | No ACL record = DENIED (not default-allow) | gotchas.md |
| SECG20 | Rule domain uses `company_ids` (switched), not `user.company_ids` | gotchas.md |
| SECG21 | A user with no matching rule (global or group) sees ALL rows | gotchas.md |
| SECG22 | `sudo()` bypasses record rules — including multi-company | gotchas.md |
| SECG23 | `compute_sudo` runs computes as superuser by default for stored AND related fields | gotchas.md |
| SECG24 | `with_user` re-checks everything; `sudo` doesn't — don't confuse them for impersonation | gotchas.md |
| SECG25 | Field `groups=` is OR across the CSV, and ANY group grants it | gotchas.md |
| SECG26 | `share` is computed from group membership, not freely set | gotchas.md |
| SECG27 | `has_group` RAISES if you query a user other than yourself (unless su/internal) | gotchas.md |
| SECG28 | `sudo()` in portal/website controllers must be paired with `access_token` | gotchas.md |
| SECG29 | `groups_id`/`users`/`trans_implied_ids` renamed in v19 | gotchas.md |
| SECG3 | Field groups: SILENT only when fields aren't named — explicit access RAISES | gotchas.md |
| SECG30 | v18 `field.is_accessible` removed in v19 | gotchas.md |
| SECG31 | `with_user(u)` resets `env.su` to False — except for the superuser | gotchas.md |
| SECG32 | `env.company`/`env.companies` validate `allowed_company_ids` — and raise — but NOT under sudo | gotchas.md |
| SECG33 | No `allowed_company_ids` → `env.companies` defaults to ALL the user's companies | gotchas.md |
| SECG34 | v18 group membership is a set-algebra registry (`SetDefinitions`), not raw rel scans | gotchas.md |
| SECG35 | `has_groups` with only `!`-negated terms (no positives) returns True | gotchas.md |
| SECG36 | Record rules cover `_inherits` parents — a child rule silently AND-s the parent's rules | gotchas.md |
| SECG37 | `_compute_domain` group-match uses `groups_id`, while `_get_rules` pre-selects via `_get_group_ids() | gotchas.md |
| SECG4 | `implied_ids` inserts REAL rows in `res_groups_users_rel` | gotchas.md |
| SECG5 | `base.group_no_one` is debug-session-dependent | gotchas.md |
| SECG6 | `sudo()` flips `env.su` but does NOT change `env.uid` | gotchas.md |
| SECG7 | Multi-company rule is NOT automatic from `company_id` | gotchas.md |
| SECG8 | Portal users rely ENTIRELY on record rules | gotchas.md |
| SECG9 | Rule domains cache by (uid, su, model, mode, allowed_company_ids) | gotchas.md |

## ecosystem/inbuilt

| ID | Título | Fichero |
|----|--------|---------|
| BC3 | file → `fiebdc3.job` (queued) → cron processes → creates SO with: | core.md |
| IBCG1 | Enable Measurement System | config_guide.md |
| IBCG10 | Multi-Certification Workflow (Monthly) | config_guide.md |
| IBCG2 | Configure FIEBDC3 Import | config_guide.md |
| IBCG3 | Setup Project Certification | config_guide.md |
| IBCG4 | Create First Certification | config_guide.md |
| IBCG5 | Configure Retention | config_guide.md |
| IBCG6 | Project Margin Analysis Setup | config_guide.md |
| IBCG7 | FIEBDC3 Percentage Map Configuration | config_guide.md |
| IBCG8 | Hierarchical Task Creation from SO | config_guide.md |
| IBCG9 | Cost Category Setup for Construction | config_guide.md |
| IBG1 | `billing_type` (origin/current) is | gotchas.md |
| IBG10 | Cancelling a certification does NOT cancel its linked invoices. Must cancel invoices separately. | gotchas.md |
| IBG11 | When `sale.measurement` records exist on a SOL, `product_uom_qty` is computed from `sum(measurement. | gotchas.md |
| IBG12 | `sale.measurement.partial = units × length × width × height`. If any dimension is 0, partial is 0. C | gotchas.md |
| IBG13 | `project.certification.measurement` is a | gotchas.md |
| IBG14 | FIEBDC3 job runs via | gotchas.md |
| IBG15 | `fiebdc3.percentage.map` uses `safe_eval` with a restricted context. Available variables: `code`, `b | gotchas.md |
| IBG16 | BC3 UOM mapping is done via `uom.uom` name matching. If a BC3 file uses a unit name not mapped in Od | gotchas.md |
| IBG17 | FIEBDC3 import creates products (`product.template`) if no match found by code. Default `type` = | gotchas.md |
| IBG18 | Percentage map `code_pattern` uses exact match first, then wildcard. Pattern `%0*` matches any code  | gotchas.md |
| IBG19 | Failed jobs (`state='failed'`) can be manually retried by setting state back to `draft` then `queued | gotchas.md |
| IBG2 | Origin billing requires `certification_previous_billing_product_id` on the company. If missing → err | gotchas.md |
| IBG20 | FIEBDC3 chunked processing (100 lines) commits per chunk. If import fails mid-way, partial data exis | gotchas.md |
| IBG21 | `sale_indirect_purchase_price_percent` is company-level. All projects in the same company use the sa | gotchas.md |
| IBG22 | Project margin computed fields (30+) use `@api.depends` chains that can be expensive on large projec | gotchas.md |
| IBG23 | `project.cost.summary` aggregates from `account.analytic.line`. Timesheets that are NOT linked to a  | gotchas.md |
| IBG24 | Stock material costs on tasks come from `stock.move` valuation. If valuation method is `standard`, c | gotchas.md |
| IBG25 | `inbuilt_sale_margin_project` extends `project.project` with `sale_order_ids` (One2many). If a proje | gotchas.md |
| IBG26 | The project creation wizard (`inbuilt.project.create`) options `task_chapter` / `task_part` / `task_ | gotchas.md |
| IBG27 | `parent_sol_id` on `project.task` links to the sale order line that generated it. If the SOL is dele | gotchas.md |
| IBG28 | `inbuilt_sale_project` overrides `_compute_show_project_and_task_button()`. If another module also o | gotchas.md |
| IBG29 | Cancelling an | gotchas.md |
| IBG30 | `project.certification._compute_state` treats | gotchas.md |
| IBG31 | Deleting a `sale.order.line` with `certification_line_ids` does NOT cascade. `sale_line_id` is Many2 | gotchas.md |
| IBG32 | `action_send_email` enters `mass_mail` mode for `len(self) > 1`, but does | gotchas.md |
| IBG33 | `project.currency_id` and `sale.order.currency_id` can diverge. Certification uses `project_id.curre | gotchas.md |
| IBG34 | Modifying a certification AFTER creating the invoice does NOT sync to the invoice (one-way build). U | gotchas.md |
| IBG35 | The `Add Lines` wizard rejects mixing SOs with different `fiscal_position_id`, `currency_id`, or `pa | gotchas.md |
| IBG36 | `certified_before_amount` aggregates | gotchas.md |
| IBG37 | `quantity_certified` lower bound is `quantity_certified_before` | gotchas.md |
| IBG38 | `_check_billing_type_not_change_if_certifications_exist` constraint counts | gotchas.md |
| IBG39 | BC3 re-import on an existing SO does | gotchas.md |
| IBG4 | Retention creates a | gotchas.md |
| IBG40 | Wizard rejects re-import if the previous `fiebdc3_job_id` is in state other than `done`/`failed`/`Fa | gotchas.md |
| IBG41 | Auto-created percentage parsers (`_auto_create_percentage_parsers`) skip codes whose parser exists w | gotchas.md |
| IBG42 | FIEBDC3 cron uses `_trigger()` (event-driven), not interval. The daily schedule is fallback only. If | gotchas.md |
| IBG43 | Ephemeral products (`product.template` with `fiebdc3_sale_id`) and BC3-mapped UoMs (`uom.uom.fiebdc3 | gotchas.md |
| IBG44 | When the import reuses an ephemeral product from a PREVIOUS SO (by `default_code` match), the produc | gotchas.md |
| IBG45 | Auto-created UoMs all fall in `uom_categ_unit` (Unit category) with `factor=1.0`. A BC3 file mixing  | gotchas.md |
| IBG46 | `recalculate_price_from_cost=True` does `price_unit = sum(c.so_price)` without currency conversion.  | gotchas.md |
| IBG47 | FIEBDC3 job commits per chunk (default 100 lines, configurable via `ir.config_parameter` `inbuilt_fi | gotchas.md |
| IBG48 | `%` parsers in `split` mode (`{'mode':'split'}`) require | gotchas.md |
| IBG49 | The wizard updates tasks | gotchas.md |
| IBG5 | Certification wizard propagates parent/child selection | gotchas.md |
| IBG50 | Asymmetric write in `_save_task`: | gotchas.md |
| IBG51 | `allocated_hours` is recomputed on each wizard run from `cost_ids.is_time` recursively. Manual time  | gotchas.md |
| IBG52 | Mixing `chapter_subtasks=True` and `chapter_subtasks=False` across re-runs on the same project can l | gotchas.md |
| IBG53 | Cancelling a `sale.order` does NOT propagate to its `project.project`. The project stays active with | gotchas.md |
| IBG54 | Moving a SOL between chapters (changing `parent_id`) IS reflected in the task hierarchy on re-run. B | gotchas.md |
| IBG55 | `inbuilt_sale_project.action_create_project` falls back to `super()` (Odoo standard) when `order_typ | gotchas.md |
| IBG56 | Missing `dest_location_id` or `default_stock_route_id` on a project triggers `UserError` | gotchas.md |
| IBG57 | When the manufacture path creates a `mrp.bom`, it | gotchas.md |
| IBG58 | Phantom BoMs (`type='phantom'`) unlink ALL `stock.warehouse.orderpoint` records for the product (inc | gotchas.md |
| IBG59 | A partida with `manufacture` route AND `cost_ids` of category `is_material`: the material costs go i | gotchas.md |
| IBG6 | `certified_before_amount` aggregates ALL previous certifications (even cancelled ones if they had in | gotchas.md |
| IBG60 | Re-running `Create Project` after removing a `cost_id` from a SOL leaves an orphan `project.stock.mo | gotchas.md |
| IBG61 | A SOL with `product_type='service'` AND `manufacture` route triggers `_create_boms`, which creates a | gotchas.md |
| IBG62 | Service-type SOL children of a manufacture partida are excluded from BoM components (`_get_mrp_compo | gotchas.md |
| IBG63 | Adding `sale.measurement` records to a SOL | gotchas.md |
| IBG64 | Editing dimensions on a `sale.measurement` that is already partially certified recalculates `partial | gotchas.md |
| IBG65 | `sale.measurement` deletion cascades to `project.certification.measurement` (`ondelete='cascade'`) W | gotchas.md |
| IBG66 | `project.certification.measurement` does | gotchas.md |
| IBG67 | Budget vs Real is a | gotchas.md |
| IBG68 | `project.sale_cost_summary_ids` and `project.project_cost_summary_ids` are computed `store=False` | gotchas.md |
| IBG69 | Multi-currency in `sale_cost_summary`: subtotals from different SOs are summed | gotchas.md |
| IBG7 | Invoice presentation `certified_lines` filters lines where `quantity_certified > 0`. Lines with 0 ce | gotchas.md |
| IBG70 | `_process_analytic_vendor_bill` uses `abs(amount)`. A | gotchas.md |
| IBG71 | Cost summary at task level includes only analytic lines with `task_id=this`. Cost summary at project | gotchas.md |
| IBG72 | A timesheet whose `analytic_account_id` differs from the project's `account_id` is | gotchas.md |
| IBG73 | `indaws_account_retention` automates | gotchas.md |
| IBG74 | Modifying `retention_percent` on a posted invoice updates the field but does | gotchas.md |
| IBG75 | `project.certification.retention_percent` propagates to `account.move.retention_percent` only at inv | gotchas.md |
| IBG76 | `inbuilt_sale_measurement._compute_product_uom_qty` forces `qty = sum(measurement.partial)` when mea | gotchas.md |
| IBG77 | `sale_cost_summary_ids` aggregates from SOs in any state except `cancel` — | gotchas.md |
| IBG8 | Certification lines inherit hierarchy from sale order lines via `section.line.mixin`. If the SO line | gotchas.md |
| IBG9 | `certified_percent` is computed from `quantity_certified / product_uom_qty`. If `product_uom_qty = 0 | gotchas.md |

## ecosystem/indaws

| ID | Título | Fichero |
|----|--------|---------|
| IG1 | indaws_base auto_install triggers cascade installation | gotchas.md |
| IG10 | CashKeeper uses direct socket communication | gotchas.md |
| IG11 | indaws_stock_delivery_portal requires partner portal access | gotchas.md |
| IG12 | Order type changes sequence and journal | gotchas.md |
| IG13 | Verifactu sends AFTER posting (does NOT block the post) | gotchas.md |
| IG14 | indaws_api uses PBKDF2-hashed opaque tokens (NOT JWT) that expire | gotchas.md |
| IG15 | Triple discount fields don't appear without the module | gotchas.md |
| IG16 | Composition works on ANY product type, not just services | gotchas.md |
| IG17 | `link_delivered=True` enables cross-type delivery inheritance | gotchas.md |
| IG18 | Mixed VAT pattern: storable + service compositions | gotchas.md |
| IG19 | Parent 0€ line appears on invoice | gotchas.md |
| IG2 | Triple discount is multiplicative, not additive | gotchas.md |
| IG20 | `composition_uuid` namespacing avoids cross-document collision | gotchas.md |
| IG21 | Recursive compositions nest automatically | gotchas.md |
| IG22 | `_service_tracking_blacklist` hides compositions from product selectors | gotchas.md |
| IG23 | Order type RENUMBERS the SO on write | gotchas.md |
| IG24 | Invoice journal forced from order type, not standard flow | gotchas.md |
| IG25 | `sale.order.line.discount` is tier-1, NOT the aggregate (sale≠account) | gotchas.md |
| IG26 | Triple discount adds SQL CHECK constraints — legacy data >100% hard-fails | gotchas.md |
| IG27 | Invoice lines are a stored materialized-path tree | gotchas.md |
| IG28 | `cost_summary_ids` deletes+recreates child records on every recompute | gotchas.md |
| IG29 | Picking↔invoice link is a COMPUTED m2m (no manual relation/join logic) | gotchas.md |
| IG3 | Picking-to-invoice bypasses SO/PO invoice flow | gotchas.md |
| IG30 | Rappel volume is invoice-based and counts refunds negatively | gotchas.md |
| IG31 | `indaws_data_export` monkeypatches ORM methods on ARBITRARY models | gotchas.md |
| IG32 | The import "god class" is `indaws.import`, not `import_record.py` | gotchas.md |
| IG33 | `dynamic_invoice_section` evals stored Python on the company record | gotchas.md |
| IG4 | Rappel settlements generate credit notes automatically | gotchas.md |
| IG5 | indaws_pos_lot_auto_selection auto-selects lots by removal strategy (FIFO/LIFO only — NOT FEFO) | gotchas.md |
| IG6 | indaws_pos_location assigns stock locations to POS order lines (multi-location picking) | gotchas.md |
| IG7 | Product composition ≠ BOM/Kit — but can coexist | gotchas.md |
| IG8 | indaws_data_import uses queue_job (async) | gotchas.md |
| IG9 | Sale approval cron only NOTIFIES — it does NOT auto-process | gotchas.md |
| IGV19 | 1. `_sql_constraints` silently lost in v19 | gotchas.md |

## ecosystem/l10n_es

| ID | Título | Fichero |
|----|--------|---------|
| ESG1 | `l10n_es_type='ignore'` excludes BOTH base AND tax | gotchas.md |
| ESG10 | SII error 1117 triggers automatic IDOtro retry | gotchas.md |
| ESG11 | Vendor bill matching by `ref` is fuzzy | gotchas.md |
| ESG12 | Bizkaia/Gipuzkoa SOAP requires DH cipher downgrade | gotchas.md |
| ESG13 | mod 303 `marca_sepa` of `'0'` (empty) blocks both SEPA and AEAT account | gotchas.md |
| ESG14 | mod 303 cuota_compensar carry-over is manual | gotchas.md |
| ESG15 | Annual regularization (casilla 68) only on 4T/12 | gotchas.md |
| ESG16 | mod 390 exoneration must match all four mod 303 returns | gotchas.md |
| ESG17 | mod 390 needs `mod_390_casilla_667` ref | gotchas.md |
| ESG18 | mod 349 monthly/quarterly switch is irreversible mid-year | gotchas.md |
| ESG19 | mod 130 v18 lives in TWO different modules (CE + Enterprise) | gotchas.md |
| ESG2 | Recargo de equivalencia is detected globally per invoice, not per line | gotchas.md |
| ESG20 | REAGYP compensation is NOT a real tax — it's a refund of farmer's IRPF | gotchas.md |
| ESG21 | ROI (Operador Intracomunitario) registration is NOT in Odoo | gotchas.md |
| ESG22 | `tax_scope` is required for foreign customers (consu vs service) | gotchas.md |
| ESG23 | TBAI chain freezes if previous invoice rejected | gotchas.md |
| ESG24 | TBAI chain_index gaps on send failure | gotchas.md |
| ESG25 | Bizkaia Batuz wraps TBAI inside LROE — different protocol | gotchas.md |
| ESG26 | TBAI freelancer in Bizkaia requires `epigrafe` config parameter | gotchas.md |
| ESG27 | TBAI refund reason `R5` is mutually exclusive with non-simplified | gotchas.md |
| ESG28 | SII strips l10n_es_exempt_reason if l10n_es_type != 'exento' | gotchas.md |
| ESG29 | There is NO TicketBAI module in OCA | gotchas.md |
| ESG3 | SII intracom detection uses mod_303 tags 10/11, not l10n_es_type | gotchas.md |
| ESG30 | Facturae signing happens on canonicalized SubjectMatter, not raw XML | gotchas.md |
| ESG31 | Facturae reason code default '10' (Detalle Operación) is generic | gotchas.md |
| ESG32 | Facturae OCA has FACe submission, Enterprise does NOT | gotchas.md |
| ESG33 | Veri*Factu chain-index cannot be deleted | gotchas.md |
| ESG34 | Veri*Factu has 60-second wait between batches | gotchas.md |
| ESG35 | Prorrata IVA has no Enterprise module | gotchas.md |
| ESG36 | mod 303 OCA result_type computed every save | gotchas.md |
| ESG37 | `l10n_es_aeat.res_partner_aeat` is a hardcoded counterpart | gotchas.md |
| ESG38 | account_tax_balance dependency required by mod 347 OCA | gotchas.md |
| ESG39 | SII OSS clave 17 detection happens on repartition lines | gotchas.md |
| ESG4 | `_l10n_es_get_main_tax_types` excludes retencion AND recargo | gotchas.md |
| ESG40 | Spanish state codes (province) hardcoded in ENTERPRISE | gotchas.md |
| ESG41 | Los registros encadenados NUNCA se borran. | gotchas.md |
| ESG42 | Cancelar una factura Veri\*Factu NO la borra: crea una ANULACIÓN encadenada. | gotchas.md |
| ESG43 | `chain_index` solo se asigna si el JSON se generó sin error. | gotchas.md |
| ESG44 | La secuencia de cadena es `no_gap` → NO hay encadenado en paralelo. | gotchas.md |
| ESG45 | La huella encadena con el documento ANTERIOR; corromper uno invalida toda la cola. | gotchas.md |
| ESG46 | `test_environment` viene activado por DEFECTO (`default=True`). | gotchas.md |
| ESG47 | Sin certificado de scope `verifactu`, falla en creación Y en envío. | gotchas.md |
| ESG48 | El envío es por LOTES con límite de 1000 y espera impuesta por AEAT. | gotchas.md |
| ESG49 | Un timeout de red = posible DUPLICADO en AEAT. | gotchas.md |
| ESG5 | mod 347 threshold uses fiscal year, not declaration period | gotchas.md |
| ESG50 | `state` lo fija la respuesta de la AEAT, no el usuario. | gotchas.md |
| ESG51 | El QR de cotejo apunta a un entorno según test/producción. | gotchas.md |
| ESG52 | El POS genera un documento por TICKET al cobrar, no al facturar. | gotchas.md |
| ESG53 | Un ticket de POS NO se puede Veri\*Factu-cancelar. | gotchas.md |
| ESG54 | El envío hace COMMIT por documento. | gotchas.md |
| ESG55 | Veri\*Factu NO manda emails ni notifica al cliente. | gotchas.md |
| ESG6 | mod 347 EXCLUDES retentions in subquery filtering | gotchas.md |
| ESG7 | Tax-tag conflict between SII and mod 303 | gotchas.md |
| ESG8 | `csv_number` batching prevents mixing A0 and A1 in same envelope | gotchas.md |
| ESG9 | SII `RegistroDuplicado` is silent success | gotchas.md |

## ecosystem/mis_builder

| ID | Título | Fichero |
|----|--------|---------|
| MISG1 | El modo por defecto es `p` (variación del periodo), no el saldo | gotchas.md |
| MISG10 | La analítica se filtra por `analytic_domain` (texto = dominio literal), a nivel instancia Y | gotchas.md |
| MISG11 | `accumulation_method` (SUM default / AVG / NONE) reajusta el KPI al periodo con PRO-RATA | gotchas.md |
| MISG12 | `type` (num/pct/str) y `compare_method` (diff/pct/none, default PCT) cambian formato y | gotchas.md |
| MISG13 | `sumcol`/`cmpcol` NO consultan la BD: combinan otras columnas | gotchas.md |
| MISG14 | `actuals_alt` corre el AEP contra OTRO modelo de move lines (budget, committed…) | gotchas.md |
| MISG15 | `auto_expand_accounts` explota el KPI en una fila por cuenta (pesado) | gotchas.md |
| MISG16 | Multi-compañía: sin `currency_id` explícito, todas las compañías deben compartir moneda o | gotchas.md |
| MISG17 | `smart_end` (default ON) hace DOS queries por saldo final | gotchas.md |
| MISG18 | Un KPI `multi=True` (subKPIs) devuelve un SimpleArray, no un escalar | gotchas.md |
| MISG2 | Selector de cuentas: `%`=comodín (`=like`), coma=lista, `[]`=TODAS, o un dominio | gotchas.md |
| MISG3 | `bali`/`bale` calculan distinto en P&L vs balance según `include_initial_balance` | gotchas.md |
| MISG4 | Modo `u` (P&L no distribuido) = resultado de ejercicios ANTERIORES | gotchas.md |
| MISG5 | En modo inicial/unallocated, saldo CERO = AccountingNone (celda en blanco), NO 0 | gotchas.md |
| MISG6 | `fld` solo con modo `p` y OBLIGA `.fieldname`; los demás fields NO admiten fieldname | gotchas.md |
| MISG7 | `AccountingNone` se comporta como 0 al comparar pero se DISUELVE en aritmética | gotchas.md |
| MISG8 | Los errores de expresión NO explotan: se vuelven `#NAME` / `#DIV/0` / `#ERR` (estilo Excel) | gotchas.md |
| MISG9 | Para conciliar con la contabilidad oficial, deja "posted". | config_guide.md |

## ecosystem/oca-accounting

| ID | Título | Fichero |
|----|--------|---------|
| OAG1 | FX division-by-zero on a 0-amount foreign-currency line | gotchas.md |
| OAG10 | `post_and_reconcile` reconciles both sides with no balance check | gotchas.md |
| OAG11 | Payment-line FX date defaults to `context_today` when unset | gotchas.md |
| OAG12 | Payment-mode journal constraint checks method, not type vs `payment_type` | gotchas.md |
| OAG13 | Mandate reference unique per company, NOT per bank account | gotchas.md |
| OAG14 | Sequence type + `last_debit_date` written only at UPLOAD, not file generation | gotchas.md |
| OAG15 | Changing `partner_bank_id` force-resets sequence to "first" | gotchas.md |
| OAG16 | One-off (OOFF) mandates can never be reused | gotchas.md |
| OAG17 | Daily cron expires mandates by 36-month inactivity | gotchas.md |
| OAG18 | PAIN version dictates GrpHdr layout | gotchas.md |
| OAG19 | SMNDA amendment indicator emitted for FRST when `last_debit_date` exists | gotchas.md |
| OAG2 | Auto-reconcile can fire on a half-computed structure | gotchas.md |
| OAG20 | HARD CONFLICT — excludes native `account_asset` | gotchas.md |
| OAG21 | Asset-posting cron ships DISABLED, no error notification | gotchas.md |
| OAG22 | Final depreciation line forced to `remaining_value`, masking rounding | gotchas.md |
| OAG23 | `prorata` auto-forced True for non-yearly `method_time` | gotchas.md |
| OAG24 | Asset profile swappable until first move posted | gotchas.md |
| OAG25 | Posting an invoice with `asset_profile_id` overwrites an existing `asset_id` | gotchas.md |
| OAG26 | Return move uses only `move_line_ids[0].account_id` | gotchas.md |
| OAG27 | One counterpart reconciled against multiple payments | gotchas.md |
| OAG28 | `action_cancel` relies on the return move being posted | gotchas.md |
| OAG29 | Bank-return fees miscategorized | gotchas.md |
| OAG3 | `_synchronize_to_moves` assumes single liquidity/suspense lines | gotchas.md |
| OAG30 | `target_move` defaults to "posted" — draft entries silently excluded | gotchas.md |
| OAG31 | FX initial balance distorted on multi-currency accounts | gotchas.md |
| OAG32 | Trial balance groups by currency but merges by account | gotchas.md |
| OAG33 | Journal ledger reads tax relations via raw SQL | gotchas.md |
| OAG34 | Trial balance does an O(n) in-memory rebuild to sort partners | gotchas.md |
| OAG35 | Multi-statement CAMT file keeps only the LAST statement's currency/IBAN | gotchas.md |
| OAG36 | ZIP import concatenates all statements into one | gotchas.md |
| OAG37 | Currency mismatch checked only AFTER IBAN journal match | gotchas.md |
| OAG38 | Dropped duplicate's amount is added back to `balance_start` | gotchas.md |
| OAG39 | Missing `account_number` weakens `unique_import_id` | gotchas.md |
| OAG4 | Partial-match clamp flips the sign | gotchas.md |
| OAG40 | Zero-amount CAMT detail splits silently ignored | gotchas.md |
| OAG41 | Cutoff price taken from the LAST invoiced line, not weighted average | gotchas.md |
| OAG42 | Cutoff date filter is timezone-sensitive | gotchas.md |
| OAG43 | `back2draft` + regenerate can duplicate lines | gotchas.md |
| OAG44 | Per-line rounding then summed can imbalance the entry | gotchas.md |
| OAG45 | `analytic_distribution` merge keys on `json.dumps()` string | gotchas.md |
| OAG46 | "posted" policy is a hard block AT posting, not before | gotchas.md |
| OAG47 | "always" policy blocks on every create/write | gotchas.md |
| OAG48 | Only fully-zero (debit AND credit 0) lines bypass the check | gotchas.md |
| OAG5 | `unique_matching=True` silently drops multi-candidate matches | gotchas.md |
| OAG6 | `draft2open` DELETES and recreates all `account.payment` | gotchas.md |
| OAG7 | `generate_payment_file` is a stub returning `(False, False)` | gotchas.md |
| OAG8 | Uploaded orders can't be deleted, and cancel never unlinks | gotchas.md |
| OAG9 | Grouped lines collapse to the FIRST line's destination account | gotchas.md |

## ecosystem/queue_job

| ID | Título | Fichero |
|----|--------|---------|
| QJG1 | `queue_job` DEBE estar en `server_wide_modules` (`--load`) o el runner nunca arranca | gotchas.md |
| QJG10 | El job corre COMO EL USUARIO que lo creó, respetando sus record rules | gotchas.md |
| QJG11 | El contexto se PODA al encolar: solo sobreviven 5 claves por defecto | gotchas.md |
| QJG12 | En cadenas/grafos, `delay()` va en el PADRE top-most | gotchas.md |
| QJG13 | `allow_commit` corre el job en un CURSOR NUEVO (conexión extra) | gotchas.md |
| QJG14 | `queue.job` tiene `_log_access = False` — no hay create_date/write_uid/etc. | gotchas.md |
| QJG15 | No puedes crear/editar `queue.job` por RPC: `create` es `@api.private` y hay campos protegidos | gotchas.md |
| QJG16 | `retry_pattern` es "desde el intento N, pospón M segundos" | gotchas.md |
| QJG17 | La config de la job.function está `ormcache`-ada — editarla fuera de ORM no aplica | gotchas.md |
| QJG18 | Un job FALLIDO postea en su chatter y SUSCRIBE+notifica a todo el grupo "Queue Job Manager" + al | gotchas.md |
| QJG19 | `requeue()` funciona desde FAILED/DONE/CANCELLED (¡puedes re-ejecutar un job DONE!); `button_done`/ | gotchas.md |
| QJG2 | `channels = root:0` desactiva el runner entero | gotchas.md |
| QJG20 | El autovacuum borra por CANAL, según `removal_interval` del canal (default 30 días) | gotchas.md |
| QJG3 | `--stop-after-init` (updates `-u`/`-i`, tests) NO arranca el runner | gotchas.md |
| QJG4 | El runner ejecuta vía HTTP a `localhost:8069` — si no hay worker HTTP alcanzable, nada corre | gotchas.md |
| QJG5 | `QUEUE_JOB__NO_DELAY` (env) o `queue_job__no_delay` (ctx) ejecutan el job INLINE, sin encolar | gotchas.md |
| QJG6 | Los jobs NO se ejecutan dentro de un test | gotchas.md |
| QJG7 | `max_retries=0` (o vacío) = reintentos INFINITOS, no cero | gotchas.md |
| QJG8 | `identity_key` solo deduplica contra jobs en `wait_dependencies/pending/enqueued` | gotchas.md |
| QJG9 | `RetryableJobError` reintenta; `ignore_retry=True` NO incrementa el contador | gotchas.md |

