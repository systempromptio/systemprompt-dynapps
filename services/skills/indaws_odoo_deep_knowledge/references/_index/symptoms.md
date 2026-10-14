# Cross-Module Symptom Index — Odoo 18.0

> Compact lookup: symptom/question → reference file + gotcha ID. **CURADO a mano** (frases-síntoma en lenguaje natural).
> Para el lookup COMPLETO ID → título → fichero (cobertura 100%, todos los dominios, auto-generado) usa [`gotcha_index.md`](gotcha_index.md). Este fichero es el atajo humano por síntoma; aquel es el índice exhaustivo por ID.
> ~240 entries, curated from 790+ gotchas across 25 modules (incl. product/variants/UoM/category/pricelist/supplierinfo/service, fleet).

---

## 1. "Why doesn't X work?" — Things that silently fail

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| `type='product'` filter matches nothing in v18 | stock | STKG1 | functional/stock/gotchas.md |
| MTO on product does nothing after install | stock | STKG21 | functional/stock/gotchas.md |
| Cancelling SO doesn't cancel linked PO/MO | stock | STKG23 | functional/stock/gotchas.md |
| Backorder doesn't trigger new PO/MO | stock | STKG9 | functional/stock/gotchas.md |
| Cross-dock route not auto-created | stock | STKG25 | functional/stock/gotchas.md |
| Standalone payment doesn't mark invoice paid | account | AG4 | functional/account/gotchas.md |
| Writing `payment_state='paid'` has no effect | account | AG5 | functional/account/gotchas.md |
| Multi-currency reconciliation fails | account | AG18 | functional/account/gotchas.md |
| Invoice posting blocked by analytic distribution | account | AG20 | functional/account/gotchas.md |
| Landed costs ignored on Standard Price products | stock | STKG32 | functional/stock/gotchas.md |
| Pricelist discount not applied on subscription | sale | SG23 | functional/sale/gotchas.md |
| Promotion "broken" — actually pricelist-restricted | sale | SG16 | functional/sale/gotchas.md |
| Loyalty reward doesn't match kit components | sale | SG24 | functional/sale/gotchas.md |
| Can't create bill before goods received | purchase | PG5 | functional/purchase/gotchas.md |
| 3-way matching not working with ordered qty policy | purchase | PG6 | functional/purchase/gotchas.md |
| Vendor bill doesn't update PO qty_invoiced | purchase | PG13 | functional/purchase/gotchas.md |
| Service bought "on receipt" blocks billing (qty_received_method manual) | purchase | PG16 | functional/purchase/gotchas.md |
| PO line price won't recompute after partial invoicing | purchase | PG17 | functional/purchase/gotchas.md |
| Adding line to confirmed PO instantly creates receipt move | purchase | PG19 | functional/purchase/gotchas.md |
| Can't decrease PO qty below received (UserError) | purchase | PG20 | functional/purchase/gotchas.md |
| Multi-step receipt: qty_received stays 0 until final move done | purchase | PG21 | functional/purchase/gotchas.md |
| Vendor return doesn't reduce qty_received (to_refund unset) | purchase | PG22 | functional/purchase/gotchas.md |
| Can't cancel PO with posted vendor bills | purchase | PG23 | functional/purchase/gotchas.md |
| PO over-invoiced after migration (invoicing_legacy bills count) | purchase | PG24 | functional/purchase/gotchas.md |
| Duplicate vendor bill warning never triggers (empty ref) | purchase | PG26 | functional/purchase/gotchas.md |
| Editing header date_planned wipes per-line planned dates | purchase | PG27 | functional/purchase/gotchas.md |
| requisition_type 'tender' doesn't exist in v18 (blanket/template only) | purchase | PG28 | functional/purchase/gotchas.md |
| MPS won't procure kit products | mrp | MG15 | cross-module/mrp/gotchas.md |
| Subcontract BOM rejects operations/byproducts | mrp | MG7 | cross-module/mrp/gotchas.md |
| No rework flow in manufacturing | mrp | MG19 | cross-module/mrp/gotchas.md |
| POS FEFO not respected for lot selection | pos | POG16 | functional/pos/gotchas.md |
| POS sales missing from sale.order reports | pos | POG4 | functional/pos/gotchas.md |
| Lead auto-assignment not happening | crm | CG11 | functional/crm/gotchas.md |
| Recurring revenue fields hidden | crm | CG17 | functional/crm/gotchas.md |
| Email alias with followers-only rejects new emails | mail | MCG7 | functional/mail/gotchas.md |
| Unsubscribe URLs broken after DB restore | mailing | MMG11 | functional/mailing/gotchas.md |
| Blacklist/opt-out not enforced in marketing automation | marketing_automation | MAG12 | enterprise/marketing_automation/gotchas.md |
| Marketing event (open/click) doesn't advance branch on stopped campaign | marketing_automation | MAG29 | enterprise/marketing_automation/gotchas.md |
| `_execute_email` raises AccessError when fired as non-marketing user | marketing_automation | MAG33 | enterprise/marketing_automation/gotchas.md |
| Changing campaign target model silently clears dedup + filter | marketing_automation | MAG31 | enterprise/marketing_automation/gotchas.md |
| Test participants still receive emails (is_test only hides stats) | marketing_automation | MAG32 | enterprise/marketing_automation/gotchas.md |
| Social stream feed never updates (no polling cron) | social | SOG22 | enterprise/social/gotchas.md |
| Social account auto-disconnects / reconnects on its own (YouTube) | social | SOG19 | enterprise/social/gotchas.md |
| Social posts to one account fail but post marked "posted" | social | SOG12 | enterprise/social/gotchas.md |
| Can't delete a social account's UTM medium | social | SOG29 | enterprise/social/gotchas.md |
| WhatsApp sends fail — phone formatting issue | whatsapp | WAG2 | enterprise/whatsapp/gotchas.md |
| WhatsApp template stopped working (RED quality / not approved) | whatsapp | WAG10 | enterprise/whatsapp/gotchas.md |
| WhatsApp template edits lost after sync | whatsapp | WAG7 | enterprise/whatsapp/gotchas.md |
| WhatsApp pending/paused template missing from send UI | whatsapp | WAG23 | enterprise/whatsapp/gotchas.md |
| WhatsApp retry "succeeds" but message cancelled (window closed) | whatsapp | WAG12 | enterprise/whatsapp/gotchas.md |
| WhatsApp account gets no inbound (missing app_secret) | whatsapp | WAG25 | enterprise/whatsapp/gotchas.md |
| WhatsApp customer who said "stop" is re-subscribed | whatsapp | WAG14 | enterprise/whatsapp/gotchas.md |
| Posting a normal comment to whatsapp channel doesn't send | whatsapp | WAG28 | enterprise/whatsapp/gotchas.md |
| WhatsApp 24h window expires mid agent conversation | whatsapp | WAG24 | enterprise/whatsapp/gotchas.md |
| Portal user can't write on tasks | project | PRJG2 | functional/project/gotchas.md |
| Timer on non-today creates wrong timesheet | timesheet | TG3 | enterprise/timesheet/gotchas.md |
| Timesheet amount overwritten after manual edit | project | PRJG6 | functional/project/gotchas.md |
| Work entries empty — bridge module missing | hr | HRG30 | functional/hr/gotchas.md |
| Barcode validation_full setting ignored | stock_barcode | BG4 | enterprise/stock_barcode/gotchas.md |
| Multi-barcode scanning silently stops | stock_barcode | BG19 | enterprise/stock_barcode/gotchas.md |
| Auto-plan does nothing — all flexible employees | planning | PLG1 | enterprise/planning/gotchas.md |
| SMS auth for signing fails silently | enterprise_tools | ETG6 | enterprise/enterprise_tools/gotchas.md |
| Sign requests expired — 6-month auto-cancel | enterprise_tools | ETG7 | enterprise/enterprise_tools/gotchas.md |
| Campaign copy has flat activities, not tree | marketing_automation | MAG18 | enterprise/marketing_automation/gotchas.md |
| Instagram post stuck in "posting" state | social | SOG3 | enterprise/social/gotchas.md |

---

## 2. "Where is X?" — Things hidden or renamed

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Fetchmail addon missing — merged into mail | mail | MCG1 | functional/mail/gotchas.md |
| Valuation report requires Developer Mode | stock | STKG34 | functional/stock/gotchas.md |
| Multi-step route options invisible | stock | STKG28 | functional/stock/gotchas.md |
| `mail.catchall.domain` ICP no longer read | mail | MCG19 | functional/mail/gotchas.md |
| Lot properties button hidden by default | stock | STKG64 | functional/stock/gotchas.md |
| Cost Structure Report is Enterprise-only | mrp | MG22 | cross-module/mrp/gotchas.md |
| Technical fields vanish outside debug mode | security | SECG5 | cross-module/security/gotchas.md |
| Draft planning shifts invisible to employees | planning | PLG4 | enterprise/planning/gotchas.md |
| Marketing automation mailings hidden from list | marketing_automation | MAG11 | enterprise/marketing_automation/gotchas.md |
| Warehouse creates 8 picking types (6 archived) | stock | STKG3 | functional/stock/gotchas.md |
| OF confirm: 'No rule has been found to replenish X in Y' → location orphaned (no parent/warehouse) or picking-type default loc stale | stock | STKG75 | functional/stock/gotchas.md |
| `mail.gateway.allowed` has no menu | mail | MCG9 | functional/mail/gotchas.md |
| POS payments not in Accounting Payments view | pos | POG12 | functional/pos/gotchas.md |
| Employee data redirected to public model | hr | HRG2 | functional/hr/gotchas.md |
| `account.account.code` is JSON per-company | account | AG11 | functional/account/gotchas.md |
| Typo in method name: `action_print_packges` | stock_barcode | BG6 | enterprise/stock_barcode/gotchas.md |
| Typo in method: `_generate_timesheeets` | project | PRJG16 | functional/project/gotchas.md |
| Typo: `_check_restrinct_scan_locations` | stock_barcode | BG5 | enterprise/stock_barcode/gotchas.md |

---

## 3. "Why does X happen?" — Unexpected automatic behaviors

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Confirming PO auto-adds vendor to product | purchase | PG3 | functional/purchase/gotchas.md |
| Procurement lines appear on existing draft PO | purchase | PG7 | functional/purchase/gotchas.md |
| Draft MO quantity increases unexpectedly | mrp | MG3 | cross-module/mrp/gotchas.md |
| Unexpected exchange difference journal entries | account | AG10 | functional/account/gotchas.md |
| Fiscal position changes all PO line taxes | purchase | PG15 | functional/purchase/gotchas.md |
| Invoice grouping merges multiple SOs into one | sale | SG7 | functional/sale/gotchas.md |
| Cash rounding adds invisible adjustment lines | account | AG21 | functional/account/gotchas.md |
| Multiple promotions stack — deep discount | sale | SG19 | functional/sale/gotchas.md |
| COW creates silent view copies on edit | website | WG1 | functional/website/gotchas.md |
| Checkout address changes the partner record | website | WG18 | functional/website/gotchas.md |
| Menu without website_id duplicates everywhere | website | WG4 | functional/website/gotchas.md |
| Records visible on site but missing in backend list | website | WG31 | functional/website/gotchas.md |
| View/snippet edit not showing until cache clear | website | WG32 | functional/website/gotchas.md |
| Visitor `_compute_partner_id` int()-crash / wrong partner | website | WG33 | functional/website/gotchas.md |
| Page-view analytics undercount rapid reloads (30min dedup) | website | WG34 | functional/website/gotchas.md |
| Visitor not created / `last_connection` stale on untracked pages | website | WG35 | functional/website/gotchas.md |
| `_get_visitor_from_request` returns None → AttributeError | website | WG36 | functional/website/gotchas.md |
| Frontend JS loads too early/late (assets_frontend split) | website | WG37 | functional/website/gotchas.md |
| CDN URL bug invisible in edit mode, breaks for anon visitors | website | WG38 | functional/website/gotchas.md |
| Message post bulk-links partner to ALL leads | crm | CG7 | functional/crm/gotchas.md |
| Auto-assignment converts leads to opportunities | crm | CG25 | functional/crm/gotchas.md |
| SO revenue update is one-directional (up only) | crm | CG13 | functional/crm/gotchas.md |
| Daily lead quota much lower than assignment_max | crm | CG41 | functional/crm/gotchas.md |
| Marking lead Won moves it backwards in pipeline | crm | CG35 | functional/crm/gotchas.md |
| Won/lost lead can't be converted to opportunity | crm | CG14 | functional/crm/gotchas.md |
| Merge loses data / followers / scalar fields | crm | CG45/CG46 | functional/crm/gotchas.md |
| Salesperson sees other team's unassigned leads | crm | CG47 | functional/crm/gotchas.md |
| Leads from same email domain flagged as duplicates | crm | CG18/CG48 | functional/crm/gotchas.md |
| New Quotation from opp opens partner wizard | crm | CG43 | functional/crm/gotchas.md |
| Web form leads aren't tagged 'Website' medium | crm | CG30 | functional/crm/gotchas.md |
| Web submissions silently switch lead↔opportunity | crm | CG31 | functional/crm/gotchas.md |
| Restoring lost lead wipes the lost reason | crm | CG37 | functional/crm/gotchas.md |
| Salesperson can't delete leads (only archive) | crm | CG16 | functional/crm/gotchas.md |
| `lead.mobile`/`lead.title` write doesn't persist (gone v19) | crm | CG40 | functional/crm/v19_changes.md |
| Email CC subscribes ALL recipients as followers | mail | MCG27 | functional/mail/gotchas.md |
| Rating email auto-changes task/ticket state | mail | MCG29 | functional/mail/gotchas.md |
| Odoo manda demasiados correos / cómo desactivar notificaciones | mail | GUÍA | functional/mail/config_guide.md |
| Notificaciones al email de un usuario: pasar a la campana / inbox | mail | GUÍA | functional/mail/config_guide.md |
| Desactivar notificación concreta por subtipo / modelo / global | mail | GUÍA | functional/mail/config_guide.md |
| Mute nativo no corta el email (solo la UI) | mail | MCG110DC | functional/mail/config_guide.md |
| Snippet assets disabled by weekly cron | website | WG10 | functional/website/gotchas.md |
| Contracts silently change state via cron | hr | HRG12 | functional/hr/gotchas.md |
| Public holidays SPLIT existing leaves | hr | HRG14 | functional/hr/gotchas.md |
| Approved leaves auto-cancelled by accrual cron | hr | HRG23 | functional/hr/gotchas.md |
| Auto-checkout after 24h inflates worked hours | hr | HRG25 | functional/hr/gotchas.md |
| Validation stops OTHER people's timers | timesheet | TG5 | enterprise/timesheet/gotchas.md |
| Timer stop with 0 hours DELETES timesheet | timesheet | TG4 | enterprise/timesheet/gotchas.md |
| Reset-to-draft rolls back lock date significantly | timesheet | TG18 | enterprise/timesheet/gotchas.md |
| Recurrence only fires on LAST task close | project | PRJG18 | functional/project/gotchas.md |
| Disabling billable clears so_line on ALL TS | project | PRJG21 | functional/project/gotchas.md |
| Negative move qty triggers location swap | stock | STKG8 | functional/stock/gotchas.md |
| Return wizard unreserves downstream moves | stock | STKG10 | functional/stock/gotchas.md |
| WO auto-starts next after finishing current one | mrp_workorder | WOG9 | enterprise/mrp_workorder/gotchas.md |
| Duplicate WhatsApp messages silently suppressed | whatsapp | WAG3 | enterprise/whatsapp/gotchas.md |
| FROM address silently replaced by notifications | mailing | MMG19 | functional/mailing/gotchas.md |
| Barcode module overwrites picking type config | stock_barcode | BG10 | enterprise/stock_barcode/gotchas.md |
| Published slot modified — employee sees old version | planning | PLG6 | enterprise/planning/gotchas.md |
| Recurrence edit deletes+recreates future slots | planning | PLG14 | enterprise/planning/gotchas.md |
| `_auto_apply_rewards` runs on every cart update | website | WG22 | functional/website/gotchas.md |

---

## 4. "Performance" — Slow queries, N+1, timeouts

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Chained pricelists multiply computation cost | sale | SG4 | functional/sale/gotchas.md |
| Quants grow as products x locations x lots | stock | STKG60 | functional/stock/gotchas.md |
| FIFO walks SVLs sequentially — slow on big stock | stock | STKG60 | functional/stock/gotchas.md |
| Subordinate computation recursive, not cached | hr | HRG32 | functional/hr/gotchas.md |
| `_get_stock_barcode_data` fetches ALL UoMs | stock_barcode | BG1 | enterprise/stock_barcode/gotchas.md |
| Missing packages limit loads ALL packages | stock_barcode | BG9 | enterprise/stock_barcode/gotchas.md |
| quality_point_ids computed with N+1 queries | mrp_workorder | WOG38 | enterprise/mrp_workorder/gotchas.md |
| Gantt unavailability queries per workcenter | mrp_workorder | WOG39 | enterprise/mrp_workorder/gotchas.md |
| _read_group on analytic_distribution limited | analytic | ANG8 | cross-module/analytic/gotchas.md |
| Multi-plan groupby creates column explosion | analytic | ANG29 | cross-module/analytic/gotchas.md |
| Global time-off creates TS for ALL employees | project | PRJG17 | functional/project/gotchas.md |
| Marketing automation sync cron every 12h | marketing_automation | MAG14 | enterprise/marketing_automation/gotchas.md |
| LinkedIn link preview adds latency to posting | social | SOG9 | enterprise/social/gotchas.md |

---

## 5. "v19 Breaking" — Things that break on migration

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Domain class replaces osv.expression | marketing_automation | MAG21 | enterprise/marketing_automation/gotchas.md |
| `threading.current_thread().testing` changed | marketing_automation | MAG22 | enterprise/marketing_automation/gotchas.md |
| WhatsAppMessage class renamed to WhatsappMessage | whatsapp | WAG20 | enterprise/whatsapp/gotchas.md |
| Sign depends on new 'certificate' module | enterprise_tools | ETG21 | enterprise/enterprise_tools/gotchas.md |
| Sign editor components lazy-loaded in v19 | enterprise_tools | ETG22 | enterprise/enterprise_tools/gotchas.md |
| No SSL cert validation in v18 (fixed in v19) | mailing | MMG21 | functional/mailing/gotchas.md |
| Server action state field added in v19 | marketing_automation | MAG23 | enterprise/marketing_automation/gotchas.md |

---

## 6. "Security" — Access denied, invisible records, sudo needed

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Global record rules are AND, group rules are OR | security | SECG1 | cross-module/security/gotchas.md |
| No ACL record = denied (not default allow) | security | SECG2 | cross-module/security/gotchas.md |
| Field with groups= silently hidden, no error | security | SECG3 | cross-module/security/gotchas.md |
| `sudo()` doesn't change uid — only bypasses checks | security | SECG6 | cross-module/security/gotchas.md |
| Multi-company rule not automatic on company_id | security | SECG7 | cross-module/security/gotchas.md |
| Portal ACL without record rule = sees ALL records | security | SECG8 | cross-module/security/gotchas.md |
| Rule changes need registry clear (cache) | security | SECG9 | cross-module/security/gotchas.md |
| Empty group_id in ACL = public access | security | SECG10 | cross-module/security/gotchas.md |
| `ir.rule` noupdate=1 ignores code changes | security | SECG12 | cross-module/security/gotchas.md |
| Portal rule via child_of exposes all contacts | security | SECG14 | cross-module/security/gotchas.md |
| `groups='.'` means NO ONE can access (only sudo) | security | SECG15 | cross-module/security/gotchas.md |
| Employee barcode lookup uses sudo() | mrp_workorder | WOG32 | enterprise/mrp_workorder/gotchas.md |
| No record rules on mass_mailing — all users see all | mailing | MMG12 | functional/mailing/gotchas.md |
| WhatsApp opt-out shared with SMS blacklist | whatsapp | WAG14 | enterprise/whatsapp/gotchas.md |
| Sign access tokens are security-critical | enterprise_tools | ETG1 | enterprise/enterprise_tools/gotchas.md |
| Shared sign documents bypass role restrictions | enterprise_tools | ETG8 | enterprise/enterprise_tools/gotchas.md |
| SQL injection risk in data cleaning actions | enterprise_tools | ETG13 | enterprise/enterprise_tools/gotchas.md |
| mail.message headers expose model and record ID | mail | MCG11 | functional/mail/gotchas.md |
| Webhook signature failure silently drops messages | whatsapp | WAG8 | enterprise/whatsapp/gotchas.md |
| Orphan employees validatable by ANY approver | timesheet | TG17 | enterprise/timesheet/gotchas.md |
| `check_company` validates but doesn't filter UI | security | SECG13 | cross-module/security/gotchas.md |
| Multiple ACL rows for same model+group are OR-ed (can't tighten by adding) | security | SECG17 | cross-module/security/gotchas.md |
| Granting a group via SQL leaves stale "denied" ACL cache | security | SECG16 | cross-module/security/gotchas.md |
| Read-only `ir.rule` leaves writes unrestricted (per-op rules) | security | SECG18 | cross-module/security/gotchas.md |
| `search` silently omits rows but `browse(id).read()` raises | security | SECG19 | cross-module/security/gotchas.md |
| Rule domain uses switched `company_ids`, not `user.company_ids` | security | SECG20 | cross-module/security/gotchas.md |
| User with no matching rule (global or group) sees ALL rows | security | SECG21 | cross-module/security/gotchas.md |
| `sudo().search([])` leaks rows across ALL companies | security | SECG22 | cross-module/security/gotchas.md |
| Stored computed field leaks restricted data via `compute_sudo` | security | SECG23 | cross-module/security/gotchas.md |
| Impersonating with `sudo()` instead of `with_user` falsifies perms | security | SECG24 | cross-module/security/gotchas.md |
| Field `groups='a,b'` exposes to ANY of the listed groups (OR) | security | SECG25 | cross-module/security/gotchas.md |
| Can't set `share=True` directly — computed from user-type group | security | SECG26 | cross-module/security/gotchas.md |
| Portal/website `sudo()` render without `access_token` check leaks docs | security | SECG28 | cross-module/security/gotchas.md |
| `groups_id`/`users`/`trans_implied_ids` renamed in v19 | security | SECG29 | cross-module/security/v19_changes.md |
| `field.is_accessible` removed in v19 (→ `_has_field_access`) | security | SECG30 | cross-module/security/v19_changes.md |

---

*Generated from 26 gotchas files, 704 total gotchas. Last updated: 2026-04-09.*

## 7. Maintenance — Equipment, requests and MTBF issues

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Archived requests not found with `('active','=',False)` filter | maintenance | MTG1 | functional/maintenance/gotchas.md |
| Serial number duplicate error in multi-company | maintenance | MTG2 | functional/maintenance/gotchas.md |
| New requests start in wrong stage after stage reorder | maintenance | MTG3 | functional/maintenance/gotchas.md |
| Kanban state lost when moving request to new stage | maintenance | MTG4 | functional/maintenance/gotchas.md |
| Recurring maintenance not working on corrective requests | maintenance | MTG6 | functional/maintenance/gotchas.md |
| Owner can't see their own equipment | maintenance | MTG11 | functional/maintenance/gotchas.md |
| Schedule end time overwritten when changing schedule date | maintenance | MTG13 | functional/maintenance/gotchas.md |
| Request assigned to team from wrong company | maintenance | MTG14 | functional/maintenance/gotchas.md |
| No automated reminders or overdue alerts | maintenance | MTG15 | functional/maintenance/gotchas.md |
| `owner_user_id` becomes read-only after HR install | maintenance | MTG18 | functional/maintenance/gotchas.md |
| Technician/team cleared when changing request company | maintenance | MTG27 | functional/maintenance/gotchas.md |
| MTBF/MTTR = 0 for same-day repairs (Date arithmetic) | maintenance | MTG28 | functional/maintenance/gotchas.md |
| Heavy preventive equipment shows MTBF=0 / no next failure | maintenance | MTG29 | functional/maintenance/gotchas.md |
| Equipment maintenance counter includes archived requests | maintenance | MTG30 | functional/maintenance/gotchas.md |
| Maintenance activity lands on wrong user (fallback chain) | maintenance | MTG31 | functional/maintenance/gotchas.md |
| `location` (Char) value lost migrating to v18→v19 | maintenance | MTG24 / MC1 | functional/maintenance/v19_changes.md |
| Preventive cadence (period/next_action_date) gone after v14→v18 upgrade | maintenance | MTG33 | functional/maintenance/gotchas.md |
| Kanban shows every stage/category despite ACL | maintenance | MTG35-36 | functional/maintenance/gotchas.md |
| Email-to-request routes by team not category after v19 | maintenance | MTG25 / MC3 | functional/maintenance/v19_changes.md |

## 7b. Rental — Pricing, stock and lifecycle issues

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Order stuck as rental after removing all rental lines | rental | RNTG1 | enterprise/rental/gotchas.md |
| Rentable product added via normal SO not treated as rental | rental | RNTG3 | enterprise/rental/gotchas.md |
| Pricelist discounts not applying to rental products | rental | RNTG5 | enterprise/rental/gotchas.md |
| Short rental charged more than expected (cheapest-wins) | rental | RNTG6 | enterprise/rental/gotchas.md |
| Cannot use lot-tracked products for rental | rental | RNTG9 | enterprise/rental/gotchas.md |
| Rental product also has subscription → error | rental | RNTG10 | enterprise/rental/gotchas.md |
| Rental returns not generating credit notes | rental | RNTG12 | enterprise/rental/gotchas.md |
| Delay charges stopped working (product deactivated) | rental | RNTG18 | enterprise/rental/gotchas.md |
| Late fee bills full overdue duration ignoring grace window | rental | RNTG19 | enterprise/rental/gotchas.md |
| Changing picking mode breaks existing rental orders | rental | RNTG16 | enterprise/rental/gotchas.md |
| Availability window wrong without sale_stock_renting | rental | RNTG17 | enterprise/rental/gotchas.md |
| Rental revenue never posts COGS / stock valuation | rental | RNTG20 / RNTG23 | enterprise/rental/gotchas.md |
| Picking mode depends on who created the order, not current user | rental | RNTG15 | enterprise/rental/gotchas.md |
| Invoice report shows wrong (doubled) lot quantities | rental | RNTG30 | enterprise/rental/gotchas.md |
| Reserved serial replaced by another at delivery | rental | RNTG31 | enterprise/rental/gotchas.md |
| `available_reserved_lots=False` but it still lets you pick | rental | RNTG28 | enterprise/rental/gotchas.md |
| Writing qty_delivered ignored in picking mode | rental | RNTG32 | enterprise/rental/gotchas.md |
| Validating return picking silently adds a delay/billable line | rental | RNTG33 | enterprise/rental/gotchas.md |
| Rental picking unexpectedly creates backorders | rental | RNTG34 | enterprise/rental/gotchas.md |
| Future-dated rental shows optimistic availability | rental | RNTG37 | enterprise/rental/gotchas.md |
| Forecast off right after early pickup/return | rental | RNTG39 | enterprise/rental/gotchas.md |
| "0-duration" pricing charges a flat fee regardless of period | rental | RNTG40 | enterprise/rental/gotchas.md |
| Overnight/night pricing fields missing (only in v19) | rental | RNTG22 | enterprise/rental/v19_changes.md |
| `_eligible_for_stock_account` rental override is a no-op (v18) | rental | RNTG23 | enterprise/rental/v19_changes.md |
| Gantt/schedule override breaks after v18→v19 upgrade | rental | RNTG27 | enterprise/rental/v19_changes.md |

## 7c. Intercompany — Cross-company transaction issues

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| PO approval fails with currency mismatch error | intercompany | ICG1 | enterprise/intercompany/gotchas.md |
| Product missing from mirrored vendor bill line | intercompany | ICG5 | enterprise/intercompany/gotchas.md |
| Tax mapping differs between source invoice and mirror | intercompany | ICG6 | enterprise/intercompany/gotchas.md |
| Payment terms not carried to mirrored document | intercompany | ICG7 | enterprise/intercompany/gotchas.md |
| Delivery-receipt sync silently stopped working | intercompany | ICG8 | enterprise/intercompany/gotchas.md |
| Intercompany config not visible in settings | intercompany | ICG13 | enterprise/intercompany/gotchas.md |
| Auto-posted bills bypass approval workflow | intercompany | ICG16 | enterprise/intercompany/gotchas.md |
| No salesperson on auto-generated SO | intercompany | ICG17 | enterprise/intercompany/gotchas.md |
| Confirming SO in Co.A fails with Co.B access error (aborts origin) | intercompany | ICG21 | enterprise/intercompany/gotchas.md |
| Co.A invoice and Co.B bill never reconcile against each other | intercompany | ICG22 | enterprise/intercompany/gotchas.md |
| Mirror creation crashes on UoM after v19 upgrade | intercompany | ICG23 | enterprise/intercompany/gotchas.md |
| Rentable PO product becomes plain sale, not rental, in mirror | intercompany | ICG25 | enterprise/intercompany/gotchas.md |
| Empty/wrong intercompany_warehouse_id hard-blocks SO/PO (and source) | intercompany | ICG26 | enterprise/intercompany/gotchas.md |
| Bill totals don't tie out to source invoice despite same unit price | intercompany | ICG28 | enterprise/intercompany/gotchas.md |
| Lot/serial not propagated for company-restricted product on receipt | intercompany | ICG29 | enterprise/intercompany/gotchas.md |

## 7d. POS Blackbox (Belgium FDM) — Certified cash register issues

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Self-compiled blackbox runs but fails tax inspection | pos_blackbox | PBB1 | enterprise/pos_blackbox/gotchas.md |
| Cannot uninstall pos_blackbox_be module | pos_blackbox | PBB2 | enterprise/pos_blackbox/gotchas.md |
| Workstation locked out of uncertified POS config (shared IP) | pos_blackbox | PBB3/PBB30 | enterprise/pos_blackbox/gotchas.md |
| Loyalty/gift card program blocks BE session open | pos_blackbox | PBB4 | enterprise/pos_blackbox/gotchas.md |
| Cash rounding error opening BE session (need 0.05 HALF-UP) | pos_blackbox | PBB6 | enterprise/pos_blackbox/gotchas.md |
| Mobile/online payment or kiosk blocks BE session | pos_blackbox | PBB8 | enterprise/pos_blackbox/gotchas.md |
| Floor sharing across configs blocks session | pos_blackbox | PBB9 | enterprise/pos_blackbox/gotchas.md |
| New employee without INSZ silently blocks session open | pos_blackbox | PBB10 | enterprise/pos_blackbox/gotchas.md |
| Custom field on registered pos.order cannot be written | pos_blackbox | PBB11/PBB27 | enterprise/pos_blackbox/gotchas.md |
| vat_letter cannot be edited even on draft | pos_blackbox | PBB12 | enterprise/pos_blackbox/gotchas.md |
| Registered order cannot be deleted (GDPR erasure) | pos_blackbox | PBB15 | enterprise/pos_blackbox/gotchas.md |
| Group/multi-component tax has no identification_letter | pos_blackbox | PBB16 | enterprise/pos_blackbox/gotchas.md |
| Clocked-in employee blocks whole session close | pos_blackbox | PBB21 | enterprise/pos_blackbox/gotchas.md |
| Fiscal position remapping to unlettered tax blocks session | pos_blackbox | PBB28 | enterprise/pos_blackbox/gotchas.md |
| FDM sequences become company-agnostic after v19 upgrade | pos_blackbox | PBB31 | enterprise/pos_blackbox/gotchas.md |
| WORK IN/OUT orders editable while sales frozen | pos_blackbox | PBB26 | enterprise/pos_blackbox/gotchas.md |

## 7e. Website Sale Subscription — eCommerce subscription checkout issues

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Installing sale_subscription silently changes the live shop | website_sale_subscription | WSSG1 | enterprise/website_sale_subscription/gotchas.md |
| Mixed-plan cart slips through in v18 (no plan_id passed) | website_sale_subscription | WSSG2 | enterprise/website_sale_subscription/gotchas.md |
| Cannot switch plan; must empty cart | website_sale_subscription | WSSG3 | enterprise/website_sale_subscription/gotchas.md |
| Cannot mix buy-once + subscription in cart (v19) | website_sale_subscription | WSSG4 | enterprise/website_sale_subscription/gotchas.md |
| Pricelist strike-through never shows for subscription product | website_sale_subscription | WSSG5 | enterprise/website_sale_subscription/gotchas.md |
| Displayed discount % off by one (floor not round) | website_sale_subscription | WSSG6 | enterprise/website_sale_subscription/gotchas.md |
| Discount % jumps after changing product type | website_sale_subscription | WSSG7 | enterprise/website_sale_subscription/gotchas.md |
| Plan radios disappear on subsequent product pages | website_sale_subscription | WSSG8 | enterprise/website_sale_subscription/gotchas.md |
| Archived plan errors at add-to-cart | website_sale_subscription | WSSG9 | enterprise/website_sale_subscription/gotchas.md |
| Optional subscription product save fails (no common plan) | website_sale_subscription | WSSG11 | enterprise/website_sale_subscription/gotchas.md |
| Digital subscription forces full address at checkout | website_sale_subscription | WSSG12 | enterprise/website_sale_subscription/gotchas.md |
| Bulk partner address update shifts renewal tax amounts | website_sale_subscription | WSSG13 | enterprise/website_sale_subscription/gotchas.md |
| Reopened churned sub silently gets new tax rate | website_sale_subscription | WSSG14 | enterprise/website_sale_subscription/gotchas.md |
| Subscription never auto-bills after partial checkout payment | website_sale_subscription | WSSG16 | enterprise/website_sale_subscription/gotchas.md |
| Reorder/Buy-again button absent for subscription lines | website_sale_subscription | WSSG17 | enterprise/website_sale_subscription/gotchas.md |
| Plan radios stuck greyed after failed add-to-cart (v19) | website_sale_subscription | WSSG18 | enterprise/website_sale_subscription/gotchas.md |

## 8. Repair — Silent failures and unexpected behaviors

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Part consumed from consignment without PO or invoice | repair | RG1 | functional/repair/gotchas.md |
| Repair move not visible in picking list | repair | RG2 | functional/repair/gotchas.md |
| No backorder when partial qty on repair part | repair | RG3 | functional/repair/gotchas.md |
| Product move missing after confirming repair | repair | RG4 | functional/repair/gotchas.md |
| SOL created before repair completion | repair | RG7 | functional/repair/gotchas.md |
| Warranty zeroes ALL part prices | repair | RG8 | functional/repair/gotchas.md |
| MTO not triggered despite product needing purchase | repair | RG11 | functional/repair/gotchas.md |
| No auto PO for consignment parts on confirm | repair | RG12 | functional/repair/gotchas.md |
| BoM explosion runs on every save | repair | RG13 | functional/repair/gotchas.md |
| Parts consumed without valuation (SVL) | repair | RG16 | functional/repair/gotchas.md |
| Zero-qty moves silently cancelled on completion | repair | RG21 | functional/repair/gotchas.md |
| Can't finish repair — "must be under repair" error | repair | RG22 | functional/repair/gotchas.md |
| procurement_group_id crashes after v19 migration | repair | RG24 | functional/repair/gotchas.md |
| Serial product repair quantity silently forced to 1 | repair | RG29 | functional/repair/gotchas.md |
| `repair.move_ids` misses the product-to-repair move | repair | RG30 | functional/repair/gotchas.md |
| All parts auto-picked on completion (no partial consume) | repair | RG31 | functional/repair/gotchas.md |
| Labor/fee line qty_delivered not filled, blocks invoicing | repair | RG32 | functional/repair/gotchas.md |
| Repair invoicing fields (`invoice_id`/`invoice_method`) gone after v16→v17 upgrade | repair | RG34-35 | functional/repair/gotchas.md |
| `operations`/`fees_lines`/`repair.line`/`repair.fee` missing in v17+ | repair | RG34-36 | functional/repair/gotchas.md |
| "Invoice before repair" (b4repair) gate gone after upgrade | repair | RG37 | functional/repair/gotchas.md |

## 9. Account Accountant (Enterprise) — Silent failures and gotchas

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Bank reconciliation widget shows "invalid" but no error message | account_accountant | AAG2 | enterprise/account_accountant/gotchas.md |
| Auto-reconcile CRON misses old transactions (>3 months) | account_accountant | AAG5 | enterprise/account_accountant/gotchas.md |
| Reconciliation model matches wrong entry (first-match-wins) | account_accountant | AAG6 | enterprise/account_accountant/gotchas.md |
| EPD discount not applied despite correct payment date | account_accountant | AAG10 | enterprise/account_accountant/gotchas.md |
| Partner detection assigns wrong partner on bank statement | account_accountant | AAG11 | enterprise/account_accountant/gotchas.md |
| Deferred entry amounts wrong (30-day month normalization) | account_accountant | AAG13 | enterprise/account_accountant/gotchas.md |
| Lock date wizard leaves draft entries stranded | account_accountant | AAG20 | enterprise/account_accountant/gotchas.md |
| Follow-up status search loads ALL partners (slow) | account_accountant | AAG26 | enterprise/account_accountant/gotchas.md |
| Batch payment blocked by single unmatched payment | account_accountant | AAG28 | enterprise/account_accountant/gotchas.md |
| Reconcile models don't inherit from parent company | account_accountant | AAG34 | enterprise/account_accountant/gotchas.md |
| bank.rec.widget REMOVED in v19 — custom modules crash | account_accountant | v19 HIGH | enterprise/account_accountant/v19_changes.md |
| Reconciliation engine rewritten to SQL in v19 | account_accountant | v19 HIGH | enterprise/account_accountant/v19_changes.md |

## 10. Delivery / Carrier Framework

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Carrier rate returns 0 or error on SO | delivery | DG6 | functional/delivery/gotchas.md |
| Free shipping not applied despite threshold met | delivery | DG6 | functional/delivery/gotchas.md |
| Delivery line price is 0 until picking validated | delivery | DG17 | functional/delivery/gotchas.md |
| Weight not computed on picking | delivery | DG11 | functional/delivery/gotchas.md |
| Carrier not available for SO despite configuration | delivery | DG26 | functional/delivery/gotchas.md |
| Return shipment fails (no return label) | delivery | DG20 | functional/delivery/gotchas.md |
| stock.quant.package renamed in v19 | delivery | v19 HIGH | functional/delivery/v19_changes.md |

## 11. Account Asset (Enterprise)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Depreciation amounts don't match fiscal calendar | account_asset | ASG1 | enterprise/account_asset/gotchas.md |
| Prorata date silently resets to fiscal year start | account_asset | ASG2 | enterprise/account_asset/gotchas.md |
| Disposing parent asset fails — children block it | account_asset | ASG10 | enterprise/account_asset/gotchas.md |
| Asset cancellation reverses ALL historical moves | account_asset | ASG14 | enterprise/account_asset/gotchas.md |
| validate() posts ALL future depreciation moves at once | account_asset | ASG17 | enterprise/account_asset/gotchas.md |
| Auto-creation creates N*M assets from invoice | account_asset | ASG19 | enterprise/account_asset/gotchas.md |
| Disposal entry unbalanced — gain/loss account empty drops the line | account_asset | ASG11/ASG31 | enterprise/account_asset/gotchas.md |
| Disposal reverses against wrong account (multi-account origin) | account_asset | ASG32 | enterprise/account_asset/gotchas.md |
| Disposal gain/loss off by already_depreciated_amount_import | account_asset | ASG33 | enterprise/account_asset/gotchas.md |
| Wizard residual differs from posted book value (reversals) | account_asset | ASG34 | enterprise/account_asset/gotchas.md |
| Depreciation move drops partner when multiple origin partners | account_asset | integrations | enterprise/account_asset/integrations.md |
| Dependency changed to accountant in v19 | account_asset | v19 HIGH | enterprise/account_asset/v19_changes.md |

## 12. Account Reports (Enterprise)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Report shows wrong sign (negative when should be positive) | account_reports | ARG1 | enterprise/account_reports/gotchas.md |
| Custom report expression returns 0 unexpectedly | account_reports | ARG3 | enterprise/account_reports/gotchas.md |
| Report very slow with many journals/companies | account_reports | ARG8 | enterprise/account_reports/gotchas.md |
| Tax report amounts don't match tax declarations | account_reports | ARG15 | enterprise/account_reports/gotchas.md |
| Comparison column shows wrong period | account_reports | ARG22 | enterprise/account_reports/gotchas.md |
| PDF export looks different from screen | account_reports | ARG30 | enterprise/account_reports/gotchas.md |
| Tax Return system new in v19 (3000+ lines) | account_reports | v19 NEW | enterprise/account_reports/v19_changes.md |

## 13. Account Budget (Enterprise)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Budget matches ALL lines when analytic not set | account_budget | ABG1 | enterprise/account_budget/gotchas.md |
| Theoretical amount off by one day | account_budget | ABG2 | enterprise/account_budget/gotchas.md |
| Budget report slow on large datasets | account_budget | ABG5 | enterprise/account_budget/gotchas.md |
| Budget upgrade v17→v18 — G/L budgets orphaned (model changed) | account_budget | ABG19 | enterprise/account_budget/legacy_v17.md |
| v17 budget uses crossovered.budget / budgetary positions | account_budget | ABG19 | enterprise/account_budget/legacy_v17.md |
| Duplicated revenue budget becomes expense (budget_type copy=False) | account_budget | ABG21 | enterprise/account_budget/gotchas.md |
| Committed amount missing in v19 | account_budget | v19 HIGH | enterprise/account_budget/v19_changes.md |

## 14. Quality (Enterprise)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Quality check not created on receipt despite control point | quality | QG8 | enterprise/quality/gotchas.md |
| Stock validation not blocked by failed quality check | quality | QG16 | enterprise/quality/gotchas.md |
| Quality check auto-created for wrong product/operation | quality | QG10 | enterprise/quality/gotchas.md |
| SPC control chart shows no data | quality | QG28 | enterprise/quality/gotchas.md |
| Failure rerouting doesn't work on certain operations | quality | QG20 | enterprise/quality/gotchas.md |
| Worksheet not visible on quality check | quality | QG24 | enterprise/quality/gotchas.md |
| lot_id → lot_ids breaks in v19 (M2O to M2M) | quality | v19 HIGH | enterprise/quality/v19_changes.md |

---

## 15. Product / Variants / UoM (cross-module)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| "¿Odoo PUEDE hacer X con atributos/variantes/kits?" (eje consultor) | capabilities | ATTRCAP/PAVCAP/VARCAP | cross-module/product/capabilities.md |
| ¿Puedo usar imágenes como swatch de atributo? (v19 display_type='image') | capabilities | ATTRCAP17 | cross-module/product/capabilities.md |
| ¿Puedo cambiar el modo de creación de variante tras usarlo? (no) | capabilities | ATTRCAP12 | cross-module/product/capabilities.md |
| ¿Atributos visibles sin activar el ajuste "Variantes"? (no, gated) | capabilities | ATTRCAP6 | cross-module/product/capabilities.md |
| Cannot delete `product.template` — silently archived | product | PRG10 | cross-module/product/gotchas.md |
| Writing field on variant changes ALL siblings | product | PRG2 | cross-module/product/gotchas.md |
| Variant `weight`/`volume` returns 0 in template | product | PRG4 | cross-module/product/gotchas.md |
| Cost (`standard_price`) differs between companies for same product | product | PRG3 | cross-module/product/gotchas.md |
| Pricelist not applied — `list_price` shown instead | product | PRG7 | cross-module/product/gotchas.md |
| Barcode import fails partially (duplicate vs packaging) | product | PRG6 | cross-module/product/gotchas.md |
| `type='storable'` filter returns nothing in v18 | product | PRG13 | cross-module/product/gotchas.md |
| Combo SO line doubles totals in reports | product | PRG11 | cross-module/product/gotchas.md |
| `create_variant` cannot be changed (locked after first use) | variants | VAG1 | cross-module/product/gotchas.md |
| Atributo con un solo value se "aplica solo" silenciosamente | variants | VAG2 | cross-module/product/gotchas.md |
| `display_type='multi'` rejects `create_variant=always` | variants | VAG3 | cross-module/product/gotchas.md |
| Dynamic attribute: `product_variant_count == 0` is legitimate | variants | VAG4 | cross-module/product/gotchas.md |
| Limit 1000 variants per template (UserError) | variants | VAG5 | cross-module/product/gotchas.md |
| Exclusion leaves template with no valid variant | variants | VAG7 | cross-module/product/gotchas.md |
| `ptav_active` (not `active`) — domains with `active` return empty | variants | VAG8 | cross-module/product/gotchas.md |
| Changing `default_extra_price` doesn't update PTAVs already created | variants | VAG9 | cross-module/product/gotchas.md |
| Variant configurator missing in POS | variants | VAG19 | cross-module/product/gotchas.md |
| BoM per variant: line not applied when expected | variants | VAG17 | cross-module/product/gotchas.md |
| Custom text input not on product.product (in custom.value model) | variants | VAG20 | cross-module/product/gotchas.md |
| UoM conversion silently returns input (different categories) | uom | UOG1 | cross-module/product/gotchas.md |
| Stock qty changes after changing `factor` of a UoM in use | uom | UOG3 | cross-module/product/gotchas.md |
| Invoice total 0.01 off — `_compute_quantity` rounds UP by default | uom | UOG4 | cross-module/product/gotchas.md |
| `uom_po_id` resets to `uom_id` after changing UoM category | uom | UOG10 | cross-module/product/gotchas.md |
| Triple conversion vendor → PO → stock leaves rounding errors | uom | UOG12 | cross-module/product/gotchas.md |
| Invoice line UoM change does NOT recompute price_unit | uom | UOG14 | cross-module/product/gotchas.md |
| SII rejects invoice due to custom UoM in line | uom | UOG15 | cross-module/product/gotchas.md |
| Cannot reduce `decimal.precision` digits (blocked) | uom | UOG17 | cross-module/product/gotchas.md |
| POS line UoM is readonly — can't sell in packaging | uom | UOG19 | cross-module/product/gotchas.md |
| Day = 8h UoM wrong for non-standard schedules (7.5h) | uom | UOG20 | cross-module/product/gotchas.md |
| `product.packaging` removed in v19 — migrated to `product.uom` | v19 | v19 BREAKING | cross-module/product/v19_changes.md |
| `uom.category` no longer discriminator in v19 (tree-based) | v19 | v19 BREAKING | cross-module/product/v19_changes.md |
| `uom_po_id` removed in v19 | v19 | v19 BREAKING | cross-module/product/v19_changes.md |
| `factor` semantics changed in v19 (absolute vs relative) | v19 | v19 HIGH | cross-module/product/v19_changes.md |
| `rounding` per-UoM removed in v19 (global from decimal.precision) | v19 | v19 HIGH | cross-module/product/v19_changes.md |
| v19: `product_uom` → `product_uom_id` on sale/purchase/supplierinfo lines (AttributeError) | uom/v19 | UOG25 | cross-module/product/gotchas.md |
| v19: `sale.order.line.product_packaging_id`/`_qty` removed entirely | uom/v19 | UOG26 | cross-module/product/gotchas.md |
| v19: `stock.move` new field `packaging_uom_id` (not repurposed product_packaging_id) | uom/v19 | UOG27 | cross-module/product/gotchas.md |
| v19: `_compute_price` inverted + no tree guard → wrong price silently | uom/v19 | UOG28 | cross-module/product/gotchas.md |
| v19: `raise_if_failure` dead param — UoM conversion never raises, returns garbage | uom/v19 | UOG29 | cross-module/product/gotchas.md |
| v19: deleting a parent UoM cascade-deletes the whole subtree (data loss) | uom/v19 | UOG30 | cross-module/product/gotchas.md |
| v19: UoM constraints changed (`_check_factor`, no reference uniqueness, no rounding>0) | uom/v19 | UOG31 | cross-module/product/gotchas.md |
| v19: changing product `uom_id` does 1:1 name swap or blocks (not numeric conversion) | uom/v19 | UOG32 | cross-module/product/gotchas.md |
| v19: `allowed_uom_ids` limits line UoM selector to product's uom_id+uom_ids (not tree) | uom/v19 | UOG33 | cross-module/product/gotchas.md |
| v19: `product.uom` requires unique barcode (GS1 cross-check with products) | uom/v19 | UOG34 | cross-module/product/gotchas.md |
| v19: added a UoM to uom_ids but it doesn't show (uom archived + M2M active_test) | uom/v19 | UOG35 | cross-module/product/gotchas.md |
| Cost method changes when moving product to other category (massive SVLs) | category | CTG2 | cross-module/product/gotchas.md |
| Removal strategy not respecting category (location wins) | category | CTG3 | cross-module/product/gotchas.md |
| Account fallback Product→Category→Parent cascade not configured at root | category | CTG4 | cross-module/product/gotchas.md |
| `cost_method`/`valuation` cannot be edited per-product (only category) | category | CTG10 | cross-module/product/gotchas.md |
| Pricelist `min_quantity` evaluated in product UoM, not SO line UoM | pricelist | PLSG2 | cross-module/product/gotchas.md |
| Archived pricelist items still apply | pricelist | PLSG3 | cross-module/product/gotchas.md |
| Pricelist frozen at SO confirm — doesn't update | pricelist | PLSG4 | cross-module/product/gotchas.md |
| `fixed_price` interpreted in product UoM, not SO line UoM | pricelist | PLSG5 | cross-module/product/gotchas.md |
| Pricelist recursion ciclo A→B→C→A → RecursionError | pricelist | PLSG7 | cross-module/product/gotchas.md |
| Variant `price_extra` applied BEFORE pricelist rule | pricelist | PLSG8 | cross-module/product/gotchas.md |
| New partner with no country gets "wrong" pricelist | pricelist | PLSG10 | cross-module/product/gotchas.md |
| `_onchange_rule_content` re-assigns `applied_on` automatically | pricelist | PLSG11 | cross-module/product/gotchas.md |
| Vendor selection picks more expensive (sequence wins over price) | supplierinfo | SIG1 | cross-module/product/gotchas.md |
| `min_qty` excludes vendor unexpectedly when changing PO UoM | supplierinfo | SIG2 | cross-module/product/gotchas.md |
| Cannot configure supplierinfo with different UoM than `uom_po_id` | supplierinfo | SIG3 | cross-module/product/gotchas.md |
| Service product config doesn't change after toggling `service_tracking` | service | SRVG2 | cross-module/product/gotchas.md |
| `service_policy='delivered_timesheet'` requires `sale_timesheet` | service | SRVG3 | cross-module/product/gotchas.md |
| Milestones don't sum 100% → line never fully invoiced | service | SRVG4 | cross-module/product/gotchas.md |
| Cannot create BoM for `type='service'` product | service | SRVG5 | cross-module/product/gotchas.md |
| `free_qty` ≠ `virtual_available` (reservas excluidas vs no) | stock qty | core | cross-module/product/core.md |
| `qty_available` returns 0 for `type='service'` or non-storable consu | stock qty | core | cross-module/product/core.md |
| `nbr_reordering_rules` doesn't sum across warehouses | stock qty | core | cross-module/product/core.md |
| FSM project save fails with cryptic SQL error (missing company_id) | FSM | FSMG1+STKG5 | enterprise/fsm/gotchas.md |
| FSM task dates wiped silently on partial date edit | FSM | FSMG2 | enterprise/fsm/gotchas.md |
| FSM partner phone change overwrites customer master data (v18) | FSM | FSMG1 | enterprise/fsm/gotchas.md |
| Timer running on two devices, only one logs hours | FSM | FSMG11 | enterprise/fsm/gotchas.md |
| Timesheet shows 10 min for a 2-min timer (rounding silently inflates) | FSM | FSMG10 | enterprise/fsm/gotchas.md |
| FSM "Mark as Done" button disappeared (timer running) | FSM | FSMG7 | enterprise/fsm/gotchas.md |
| Two FSM tasks created from same SO line (double creation) | FSM/sale | FSMG22 | enterprise/fsm/gotchas.md |
| Material "+" button silently does nothing (locked SO) | FSM/sale | FSMG25 | enterprise/fsm/gotchas.md |
| Materials marked delivered without `industry_fsm_stock` (no real picking) | FSM/sale | FSMG28 | enterprise/fsm/gotchas.md |
| Partial pick on FSM task — remaining qty gone, no backorder | FSM/stock | FSMG31 | enterprise/fsm/gotchas.md |
| Cannot decrease material qty below delivered amount | FSM/stock | FSMG32 | enterprise/fsm/gotchas.md |
| Lot tracking wizard cancels intermediate moves (2-step delivery + warehouse mismatch) | FSM/stock | FSMG34 | enterprise/fsm/gotchas.md |
| Serial-tracked product creates N SOLs instead of 1 with qty N | FSM/stock | FSMG35 | enterprise/fsm/gotchas.md |
| Worksheet PDF section blank (template never form-saved) | FSM/report | FSMG38 | enterprise/fsm/gotchas.md |
| "Send Report" button hidden (no timesheet logged) | FSM/report | FSMG43 | enterprise/fsm/gotchas.md |
| Stage SMS doesn't send (partner has phone but no mobile) | FSM/sms | FSMG42 | enterprise/fsm/gotchas.md |
| Map view tiles silently fail (no Google API key) | FSM/mobile | FSMG40 | enterprise/fsm/gotchas.md |
| Signature drifts on iOS Safari < 15 | FSM/mobile | FSMG39 | enterprise/fsm/gotchas.md |
| Helpdesk → FSM convert opens empty project (no FSM project in team's company) | FSM/helpdesk | FSMG51 | enterprise/fsm/gotchas.md |
| Subscription renewal creates FSM task off-cycle | FSM/subscription | FSMG52 | enterprise/fsm/gotchas.md |
| FSM project losing Documents config after `is_fsm=True` toggle | FSM/documents | FSMG53 | enterprise/fsm/gotchas.md |
| v19 upgrade fails: `partner_phone` field gone, `expression.OR` gone | FSM/v19 | FSMG57+58 | enterprise/fsm/v19_changes.md |
| Worksheet Studio field rename → blank in PDF until form re-saved | FSM/worksheet | FSMG17 | enterprise/fsm/gotchas.md |
| Ex-owner still edits document after access set to 'none' | documents (EE) | DOG1 | enterprise/documents/gotchas.md |
| Sensitive doc leaks to ALL internal users | documents (EE) | DOG2 | enterprise/documents/gotchas.md |
| Doc accessible via link though sharing never set on it | documents (EE) | DOG3 | enterprise/documents/gotchas.md |
| Document disappears after switching company in UI | documents (EE) | DOG5 | enterprise/documents/gotchas.md |
| "Invalid token" on share/sign-up link sent earlier | documents (EE) | DOG8 | enterprise/documents/gotchas.md |
| Can't generate share-and-signup link (B2B-only instance) | documents (EE) | DOG9 | enterprise/documents/gotchas.md |
| Archived document still occupies storage / not deleted | documents (EE) | DOG10 | enterprise/documents/gotchas.md |
| Deleting a folder orphans 1000s of files (no archive) | documents (EE) | DOG12 | enterprise/documents/gotchas.md |
| Shortcut shows "Inaccessible" thumbnail | documents (EE) | DOG13 | enterprise/documents/gotchas.md |
| Email with N PDFs creates N separate documents | documents (EE) | DOG16 | enterprise/documents/gotchas.md |
| Bulk URL import stalls (synchronous OG metadata fetch) | documents (EE) | DOG18 | enterprise/documents/gotchas.md |
| Document name keeps old filename after replacing attachment | documents (EE) | DOG21 | enterprise/documents/gotchas.md |
| Company B can't create tag with same name as Company A | documents (EE) | DOG22 | enterprise/documents/gotchas.md |
| Can't request same document from two people | documents (EE) | DOG24 | enterprise/documents/gotchas.md |
| Documents kanban slow with many users (compute cache) | documents (EE) | DOG28 | enterprise/documents/gotchas.md |
| Side panel slow on deep folder trees | documents (EE) | DOG29 | enterprise/documents/gotchas.md |
| Digitized invoice created empty (no IAP credits, no error) | documents/account (EE) | DOG33 | enterprise/documents/gotchas.md |
| Can't map two folders to the same accounting journal | documents/account (EE) | DOG34 | enterprise/documents/gotchas.md |
| Signed documents never purged by autovacuum (pile up) | documents/sign (EE) | DOG35 | enterprise/documents/gotchas.md |
| "Sign directly" fails on multiple selected documents | documents/sign (EE) | DOG36 | enterprise/documents/gotchas.md |
| Project folder archived (not deleted) when last project removed | documents/project (EE) | DOG37 | enterprise/documents/gotchas.md |
| Cross-company folder assignment raises ValidationError | documents/project+hr (EE) | DOG38 | enterprise/documents/gotchas.md |
| Attachments stop becoming documents after folder removed (silent) | documents bridges (EE) | DOG39 | enterprise/documents/gotchas.md |
| Spreadsheet read-only though editor would allow editing | documents/spreadsheet (EE) | DOG40 | enterprise/documents/gotchas.md |
| Installing documents+FSM disables Documents on FSM tasks | documents/fsm (EE) | DOG41 | enterprise/documents/gotchas.md |
| v17→v18 upgrade: documents.folder/share/workflow.rule models gone | documents (EE) | core | enterprise/documents/core.md |
| Pinning embedded action on view-only folder raises AccessError (v19) | documents (EE) | DOG32 | enterprise/documents/gotchas.md |
| Peppol UUIDs misassigned to invoices on batched send | EDI/Peppol | EDIG1 | functional/edi/gotchas.md |
| `peppol_move_state='processing'` stuck forever (IAP 702) | EDI/Peppol | EDIG12 | functional/edi/gotchas.md |
| Token expired but other DB also tries to refresh → invalid signature | EDI/Peppol | EDIG4 | functional/edi/gotchas.md |
| Incoming Peppol invoices land in wrong purchase journal | EDI/Peppol | EDIG5 | functional/edi/gotchas.md |
| Imported UBL invoice has 0% VAT silently | EDI/UBL | EDIG13 | functional/edi/gotchas.md |
| Imported XML doesn't create partner | EDI/UBL | EDIG14 | functional/edi/gotchas.md |
| UBL credit note treated as positive invoice | EDI/UBL | EDIG15 | functional/edi/gotchas.md |
| UoM dropped silently on import | EDI/UBL | EDIG16 | functional/edi/gotchas.md |
| Spanish IGIC not mapped to UBL L/M code | EDI/UBL | EDIG17 | functional/edi/gotchas.md |
| Fixed taxes (Recupel) round-trip lost between systems | EDI/UBL | EDIG18 | functional/edi/gotchas.md |
| XRechnung rejected for `BuyerReference='N/A'` | EDI/UBL | EDIG19 | functional/edi/gotchas.md |
| `[UBL-SR-48]` line has multiple tax categories | EDI/UBL | EDIG21 | functional/edi/gotchas.md |
| 64 MB UBL XML rejected by receiver | EDI/UBL | EDIG27 | functional/edi/gotchas.md |
| EDI cancel silently reverts posted invoice to draft+cancel | EDI/framework | EDIG44 | functional/edi/gotchas.md |
| EDI doc stuck in `to_send`, cron never retries (blocking_level=error) | EDI/framework | EDIG45 | functional/edi/gotchas.md |
| One bad invoice blocks whole EDI batch (savepoint rollback) | EDI/framework | EDIG46 | functional/edi/gotchas.md |
| "Document being sent by another process" on cloned DB | EDI/framework | EDIG47 | functional/edi/gotchas.md |
| `edi_state` stays False for offline/XML formats | EDI/framework | EDIG48 | functional/edi/gotchas.md |
| No `account.edi.document` row for UBL/CII/Facturae invoice | EDI/framework | EDIG51 | functional/edi/gotchas.md |
| Partner can't have both Facturae and UBL as primary format | EDI/send | EDIG53 | functional/edi/gotchas.md |
| peppol done but SII still to_send (states not synced) | EDI/send | EDIG54 | functional/edi/gotchas.md |
| SII error 3000: recargo without main sujeto tax | EDI/l10n_es | EDIG55 | functional/edi/gotchas.md |
| Facturae silently skipped for non-ES country partner | EDI/l10n_es | EDIG57 | functional/edi/gotchas.md |
| Intrastat shows all transactions with NULL transaction code | Intrastat | EDIG33 | functional/edi/gotchas.md |
| Intrastat report excludes UK trade post-Brexit cutoff | Intrastat | EDIG31 | functional/edi/gotchas.md |
| Intrastat triangular trade goes unreported | Intrastat | EDIG34 | functional/edi/gotchas.md |
| v19 upgrade: `account_peppol_proxy_state` filter breaks | EDI/v19 | EDIG39 | functional/edi/v19_changes.md |
| v19 upgrade: `account_peppol_advanced_fields` DEPRECATED — don't use | EDI/v19 | EDIG40 | functional/edi/v19_changes.md |
| v19 upgrade: `account_intrastat_services` reference fails | EDI/v19 | EDIG41 | enterprise/intrastat/v19_changes.md |
| Intrastat all default transaction codes expired (none active in base) | Intrastat (EE) | INTR1 | enterprise/intrastat/gotchas.md |
| Intrastat service lines excluded from goods declaration | Intrastat (EE) | INTR3 | enterprise/intrastat/gotchas.md |
| Intrastat domestic sales wrongly excluded / cross-border filter | Intrastat (EE) | INTR6 | enterprise/intrastat/gotchas.md |
| Intrastat weight 0 default emits missing-weight warning | Intrastat (EE) | INTR9 | enterprise/intrastat/gotchas.md |
| Intrastat multi-warehouse region KeyError 'invoice_id' | Intrastat (EE) | INTR12 | enterprise/intrastat/gotchas.md |
| Vendor bill stuck in 'exception' (no PO line / manual bill) | 3-way match (EE) | A3M2 | enterprise/account_3way_match/gotchas.md |
| 3-way match FX exception even when prices match (today's rate) | 3-way match (EE) | A3M3 | enterprise/account_3way_match/gotchas.md |
| force_release_to_pay not set via ORM (onchange-only) | 3-way match (EE) | A3M4 | enterprise/account_3way_match/gotchas.md |
| All bills 'exception' after module install (default value) | 3-way match (EE) | A3M1 | enterprise/account_3way_match/gotchas.md |
| Disallowed category code clash across companies (unique global) | Disallowed (EE) | ADE1 | enterprise/account_disallowed/gotchas.md |
| Disallowed category cleared switching account internal_group | Disallowed (EE) | ADE3 | enterprise/account_disallowed/gotchas.md |
| Disallowed report rate differs from category current_rate badge | Disallowed (EE) | ADE16 | enterprise/account_disallowed/gotchas.md |
| v19: account_disallowed_expenses renamed account_fiscal_categories | Disallowed/v19 | ADE4 | enterprise/account_disallowed/v19_changes.md |
| Consolidation removed in v18+ (orphan tables on upgrade) | Consolidation (EE,v17) | ACG1 | enterprise/account_consolidation/gotchas.md |
| Consolidation missing/zero FX rate returns amount unchanged (no warning) | Consolidation (EE,v17) | ACG10 | enterprise/account_consolidation/gotchas.md |
| Consolidation regenerating journals wipes manual edits | Consolidation (EE,v17) | ACG3 | enterprise/account_consolidation/gotchas.md |
| Consolidation closed period blocks late adjustment journals | Consolidation (EE,v17) | ACG4 | enterprise/account_consolidation/gotchas.md |
| Consolidation hist accounts explode journal line count | Consolidation (EE,v17) | ACG28 | enterprise/account_consolidation/gotchas.md |
| 3-way match value 'no' is overloaded (paid OR nothing received) | 3-way match (EE) | A3M5 | enterprise/account_3way_match/gotchas.md |
| 3-way match mixed PO line statuses → exception (not OR) | 3-way match (EE) | A3M8 | enterprise/account_3way_match/gotchas.md |
| Disallowed report counts draft moves with all-entries filter | Disallowed (EE) | ADE17 | enterprise/account_disallowed/gotchas.md |
| Disallowed rate is whole-number Float (/100 in report) | Disallowed (EE) | ADE9 | enterprise/account_disallowed/gotchas.md |
| IoT box "drivers reverted" after reboot | IoT | IOTG3 | enterprise/iot/gotchas.md |
| POS offline shows printer offline | IoT | IOTG4 | enterprise/iot/gotchas.md |
| Belgian blackbox locks after network change | IoT | IOTG11 | enterprise/iot/gotchas.md |
| IoT box appears offline but is up (cert expiry) | IoT | IOTG13 | enterprise/iot/gotchas.md |
| Print confirmation timeout under 10 s on large jobs | IoT | IOTG20 | enterprise/iot/gotchas.md |
| IoT box pairing token expired during onboarding | IoT | IOTG17 | enterprise/iot/gotchas.md |
| v19 upgrade: Ingenico/Worldline driver missing | IoT/v19 | IOTG19 | enterprise/iot/v19_changes.md |
| IoT "empareja pero no imprime" (orden de config) | IoT | — | enterprise/iot/config_guide.md §5 |
| IoT "a mí imprime, al compañero no" (localStorage) | IoT | IOTG33 | enterprise/iot/config_guide.md §2 |
| IoT config en 4 capas / dónde toca cada ajuste | IoT | — | enterprise/iot/config_guide.md §0-4 |
| IoT taxonomía de fallos por configuración | IoT | — | enterprise/iot/config_guide.md §6 |
| IoT flujos cross-module (POS/MRP/Quality/delivery/report/bus) | IoT | — | enterprise/iot/integrations.md |
| ¿Cuándo aparece una impresora en el panel de Odoo? | IoT | — | enterprise/iot/integrations.md §1 + core.md §3 |
| Keyboard layout no refresca tras re-setup | IoT | IOTG24 | enterprise/iot/config_guide.md §1 |
| VoIP ringtone silent on first inbound call | VoIP | VOIPG2 | enterprise/voip/gotchas.md |
| VoIP click-to-call doesn't match `0034...` numbers | VoIP | VOIPG12 | enterprise/voip/gotchas.md |
| VoIP DTMF not received by IVR (RFC 2833 vs SIP INFO) | VoIP | VOIPG10 | enterprise/voip/gotchas.md |
| Two Odoo tabs → wrong tab rings | VoIP | VOIPG8 | enterprise/voip/gotchas.md |
| Mobile tab in background → softphone frozen | VoIP | VOIPG9 | enterprise/voip/gotchas.md |
| WSS blocked by corporate firewall | VoIP | VOIPG4 | enterprise/voip/gotchas.md |
| `voip_secret` plaintext in DB backup | VoIP | VOIPG1 | enterprise/voip/gotchas.md |
| Spreadsheet cell shows `LOADING_ERROR` indefinitely | Spreadsheet | SPRG1 | functional/spreadsheet/gotchas.md |
| Spreadsheet large list >10k rows lags UI | Spreadsheet | SPRG1 | functional/spreadsheet/gotchas.md |
| `=10.1 + 0.2` returns 10.29999... not 10.3 | Spreadsheet | SPRG3 | functional/spreadsheet/gotchas.md |
| Spreadsheet formula breaks for German user (`,` vs `;`) | Spreadsheet | SPRG4 | functional/spreadsheet/gotchas.md |
| Custom o-spreadsheet plugin not loaded | Spreadsheet | SPRG5 | functional/spreadsheet/gotchas.md |
| `=ODOO.DEBIT` returns 0 for posted draft invoice | Spreadsheet | SPRG13 | functional/spreadsheet/gotchas.md |
| Renaming source-model field breaks spreadsheet silently | Spreadsheet | SPRG14 | functional/spreadsheet/gotchas.md |
| Shared spreadsheet shows `#ERROR` cells (ACL on source model) | Spreadsheet | SPRG15 | functional/spreadsheet/gotchas.md |
| Pivot cardinality explosion — UI freezes | Spreadsheet | SPRG17 | functional/spreadsheet/gotchas.md |
| Pivot date filter shifts by ±1 day (timezone) | Spreadsheet | SPRG18 | functional/spreadsheet/gotchas.md |
| Filter on same field as pivot domain returns no records (AND-merge) | Spreadsheet/filters | SPRG22 | functional/spreadsheet/gotchas.md |
| Chart doesn't update after global filter change | Spreadsheet/filters | SPRG23 | functional/spreadsheet/gotchas.md |
| Filter labels not retranslated after language switch | Spreadsheet/filters | SPRG24 | functional/spreadsheet/gotchas.md |
| Built-in dashboard customisations lost on `-u` | Spreadsheet/dashboards | SPRG26 | functional/spreadsheet/gotchas.md |
| Public dashboard shows empty cells (public user ACL) | Spreadsheet/dashboards | SPRG27 | functional/spreadsheet/gotchas.md |
| Sample dashboard masks empty database | Spreadsheet/dashboards | SPRG28 | functional/spreadsheet/gotchas.md |
| Image upload >25 MB silently fails | Spreadsheet/editor | SPRG30 | functional/spreadsheet/gotchas.md |
| Concurrent edit rejected — second user must rebase | Spreadsheet/documents | SPRG32 | functional/spreadsheet/gotchas.md |
| Revision log grows unbounded | Spreadsheet/documents | SPRG33 | functional/spreadsheet/gotchas.md |
| Read-only shared spreadsheet still allows full XLSX download | Spreadsheet/share | SPRG36 | functional/spreadsheet/gotchas.md |
| Public spreadsheet with live data returns error page | Spreadsheet/share | SPRG38 | functional/spreadsheet/gotchas.md |
| v19 upgrade: custom UI plugin broken | Spreadsheet/v19 | SPRG41 | functional/spreadsheet/v19_changes.md |
| v19 upgrade: `spreadsheet.dependencies` asset key fails | Spreadsheet/v19 | SPRG43 | functional/spreadsheet/v19_changes.md |
| v19 upgrade: `spreadsheet_dashboard_hr_contract` missing | Spreadsheet/v19 | SPRG44 | functional/spreadsheet/v19_changes.md |
| `=ODOO.LIST` fetches thousands of rows from one high-index formula | Spreadsheet | SPRG45 | functional/spreadsheet/gotchas.md |
| Spreadsheet cell stuck on loading placeholder (screenshot/print too early) | Spreadsheet | SPRG46 | functional/spreadsheet/gotchas.md |
| Spreadsheet cell never errors but never resolves (10 s notify timeout, hung RPC) | Spreadsheet | SPRG47 | functional/spreadsheet/gotchas.md |
| Rapid filter changes: intermediate data-source loads discarded (KeepLast) | Spreadsheet | SPRG48 | functional/spreadsheet/gotchas.md |
| "Last refreshed" timestamp wrong after a failed data-source load | Spreadsheet | SPRG49 | functional/spreadsheet/gotchas.md |
| `=ODOO.BALANCE` returns 0 (draft moves excluded), looks like empty account | Spreadsheet/account | SPRG50 | functional/spreadsheet/gotchas.md |
| `=ODOO.BALANCE` mixes cumulative and period figures (BS vs P&L date window) | Spreadsheet/account | SPRG51 | functional/spreadsheet/gotchas.md |
| `=ODOO.BALANCE("4%")` catches unintended accounts (LIKE prefix) | Spreadsheet/account | SPRG52 | functional/spreadsheet/gotchas.md |
| Concurrent collaborative edit rejected, no merge (parent-unique index) | Spreadsheet/collab (EE) | SPRGE1 | enterprise/spreadsheet/gotchas.md |
| Cannot branch spreadsheet history (one child per parent) | Spreadsheet/collab (EE) | SPRGE2 | enterprise/spreadsheet/gotchas.md |
| Revision table grows unbounded (snapshot needs 2 h idle + a writer) | Spreadsheet/collab (EE) | SPRGE3 | enterprise/spreadsheet/gotchas.md |
| Snapshot raises ValueError on revision-id mismatch | Spreadsheet/collab (EE) | SPRGE4 | enterprise/spreadsheet/gotchas.md |
| Spreadsheet time-travel history gone (365-day revision GC) | Spreadsheet/collab (EE) | SPRGE5 | enterprise/spreadsheet/gotchas.md |
| Cannot edit a frozen spreadsheet (AccessError, locked at 3 layers) | Spreadsheet/frozen (EE) | SPRGE6 | enterprise/spreadsheet/gotchas.md |
| Public share with live Odoo data returns error page | Spreadsheet/share (EE) | SPRGE7 | enterprise/spreadsheet/gotchas.md |
| Read-only token viewer can still download full JSON + XLSX | Spreadsheet/share (EE) | SPRGE8 | enterprise/spreadsheet/gotchas.md |
| Portal/share user cannot get edit role on a spreadsheet | Spreadsheet/share (EE) | SPRGE9 | enterprise/spreadsheet/gotchas.md |
| Dashboard visible (group_ids) but cells empty (no source-model ACL) | Spreadsheet/dashboards (EE) | SPRGE10 | enterprise/spreadsheet/gotchas.md |
| Built-in dashboard edit lost on `-u`; all managers edit all dashboards | Spreadsheet/dashboards (EE) | SPRGE11 | enterprise/spreadsheet/gotchas.md |
| XLSX import fails (1000-file / 50 MB unzipped cap) | Spreadsheet/import (EE) | SPRGE12 | enterprise/spreadsheet/gotchas.md |
| Editor image upload silently fails (413, no client guard) | Spreadsheet/editor (EE) | SPRGE13 | enterprise/spreadsheet/gotchas.md |
| Historial de revisiones desaparece al escribir datos por código (preserve_spreadsheet_revisions) | Spreadsheet/collab (EE) | SPRGE14 | enterprise/spreadsheet/gotchas.md |
| Restaurar versión anterior descarta revisiones equivocadas (por id, no timestamp) | Spreadsheet/collab (EE) | SPRGE15 | enterprise/spreadsheet/gotchas.md |
| Override de spreadsheet.mixin roto en v19 / `_get_spreadsheet_snapshot` devuelve string | Spreadsheet/collab (EE) | SPRGE18 | enterprise/spreadsheet/gotchas.md |
| Comentarios de celda desaparecen al duplicar/copiar la hoja | Spreadsheet/editor (EE) | SPRGE19 | enterprise/spreadsheet/gotchas.md |
| XLSX importado pierde macros / propiedades del documento (SUPPORTED_PATHS) | Spreadsheet/import (EE) | SPRGE23 | enterprise/spreadsheet/gotchas.md |
| Tiles del selector de spreadsheet en blanco para algunos usuarios / tras restore | Spreadsheet/editor (EE) | SPRGE26 | enterprise/spreadsheet/gotchas.md |
| Amazon `refresh_token` visible in DB backup | Amazon | AMZG1 | enterprise/sale_amazon/gotchas.md |
| Amazon sync silently stops (refresh token revoked) | Amazon | AMZG2 | enterprise/sale_amazon/gotchas.md |
| Amazon SP-API returns 404/401 (wrong region) | Amazon | AMZG3 | enterprise/sale_amazon/gotchas.md |
| Amazon 429 rate limit → orders missing from sync | Amazon | AMZG6 | enterprise/sale_amazon/gotchas.md |
| Anonymous Amazon partners flooding DB | Amazon | AMZG10 | enterprise/sale_amazon/gotchas.md |
| Unknown SKU silently maps to "Amazon Sales" generic product | Amazon | AMZG11 | enterprise/sale_amazon/gotchas.md |
| Amazon invoice total ≠ Amazon settlement (tax recomputed) | Amazon | AMZG12 | enterprise/sale_amazon/gotchas.md |
| Multi-currency: USD order shows wrong EUR amount months later | Amazon | AMZG14 | enterprise/sale_amazon/gotchas.md |
| Amazon refund not reflected in Odoo | Amazon | AMZG15 | enterprise/sale_amazon/gotchas.md |
| FBA inventory in Odoo wrong (no API sync) | Amazon/FBA | AMZG22 | enterprise/sale_amazon/gotchas.md |
| FBA orders have no picking record | Amazon/FBA | AMZG23 | enterprise/sale_amazon/gotchas.md |
| Pan-EU FBA: cannot track which FC fulfilled order | Amazon/FBA | AMZG24 | enterprise/sale_amazon/gotchas.md |
| FBM late ship-confirmation penalised by Amazon scorecard | Amazon/FBM | AMZG25 | enterprise/sale_amazon/gotchas.md |
| Wrong Amazon tracking number cannot be fixed | Amazon/FBM | AMZG26 | enterprise/sale_amazon/gotchas.md |
| FBM partial-line shipment UserError | Amazon/FBM | AMZG27 | enterprise/sale_amazon/gotchas.md |
| Amazon return not auto-created in Odoo | Amazon/returns | AMZG28 | enterprise/sale_amazon/gotchas.md |
| `amazon.offer` JSON cache stale (channel changed at Amazon) | Amazon | AMZG34 | enterprise/sale_amazon/gotchas.md |
| New marketplace not visible in account | Amazon | AMZG36 | enterprise/sale_amazon/gotchas.md |
| AvaTax: bridge doesn't fire because non-AvaTax FP explicit | Amazon/bridges | AMZG39 | enterprise/sale_amazon/gotchas.md |
| Spanish simplified invoice forced for high-value Amazon order | Amazon/l10n_es | AMZG40 | enterprise/sale_amazon/gotchas.md |
| v19 upgrade: `partner.amazon_email` field missing | Amazon/v19 | AMZG43 | enterprise/sale_amazon/v19_changes.md |
| v19 upgrade: `tax_id` → `tax_ids` rename breaks custom flows | Amazon/v19 | AMZG46 | enterprise/sale_amazon/v19_changes.md |

## 14. inBuilt — Construction vertical (presupuesto → obra → certificación)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Real cost (e.g., machine rental) doesn't update the budget | inBuilt | IBG67 | ecosystem/inbuilt/gotchas.md |
| Cancelling intermediate certification breaks `certified_before` of later ones | inBuilt | IBG29 | ecosystem/inbuilt/gotchas.md |
| Draft invoice blocks editing the certification | inBuilt | IBG30 | ecosystem/inbuilt/gotchas.md |
| Deleting a SOL with certifications leaves orphan certification lines | inBuilt | IBG31 | ecosystem/inbuilt/gotchas.md |
| Bulk-sending certifications doesn't preload the email template | inBuilt | IBG32 | ecosystem/inbuilt/gotchas.md |
| Project currency ≠ SO currency: certification invoice wrong | inBuilt | IBG33 | ecosystem/inbuilt/gotchas.md |
| Editing certification after invoice creation doesn't sync | inBuilt | IBG34 | ecosystem/inbuilt/gotchas.md |
| Wizard rejects mixing SOs with different FP/currency/partner_invoice_id | inBuilt | IBG35 | ecosystem/inbuilt/gotchas.md |
| `billing_type` locked forever once any (even cancelled) certification exists | inBuilt | IBG38 | ecosystem/inbuilt/gotchas.md |
| BC3 re-import keeps old orphan lines not present in new file | inBuilt | IBG39 | ecosystem/inbuilt/gotchas.md |
| Cannot re-import BC3 while previous job is running | inBuilt | IBG40 | ecosystem/inbuilt/gotchas.md |
| Disabled `%` parser silently ignored in subsequent imports | inBuilt | IBG41 | ecosystem/inbuilt/gotchas.md |
| FIEBDC3 jobs stuck in queued — cron disabled, no watchdog | inBuilt | IBG42 | ecosystem/inbuilt/gotchas.md |
| Ephemeral BC3 products/UoMs collide across companies | inBuilt | IBG43 | ecosystem/inbuilt/gotchas.md |
| Deleting first SO that created an ephemeral product breaks second SO | inBuilt | IBG44 | ecosystem/inbuilt/gotchas.md |
| Auto-created UoMs all in Unit category — conversion broken | inBuilt | IBG45 | ecosystem/inbuilt/gotchas.md |
| `recalculate_price_from_cost` doesn't convert currencies | inBuilt | IBG46 | ecosystem/inbuilt/gotchas.md |
| FIEBDC3 import failure leaves partial data (no rollback) | inBuilt | IBG47 | ecosystem/inbuilt/gotchas.md |
| `%` split-mode parsers need 4-decimal precision | inBuilt | IBG48 | ecosystem/inbuilt/gotchas.md |
| Create Project re-run overwrites task name but NOT description | inBuilt | IBG50 | ecosystem/inbuilt/gotchas.md |
| Manual task allocated_hours overwritten by wizard re-run | inBuilt | IBG51 | ecosystem/inbuilt/gotchas.md |
| Cancelled SO doesn't cancel its project (manual cleanup) | inBuilt | IBG53 | ecosystem/inbuilt/gotchas.md |
| Missing `dest_location_id`/`default_stock_route_id` fails mid-flow | inBuilt | IBG56 | ecosystem/inbuilt/gotchas.md |
| Pre-existing BoM overwritten silently on Create Project | inBuilt | IBG57 | ecosystem/inbuilt/gotchas.md |
| Phantom BoM destroys orderpoints, no restore | inBuilt | IBG58 | ecosystem/inbuilt/gotchas.md |
| Service product with manufacture route creates broken BoM | inBuilt | IBG61 | ecosystem/inbuilt/gotchas.md |
| Orphan `project.stock.move` after removing cost from SOL | inBuilt | IBG60 | ecosystem/inbuilt/gotchas.md |
| Measurements added after certification creation don't sync | inBuilt | IBG63 | ecosystem/inbuilt/gotchas.md |
| Editing certified measurement dimensions leaves qty > partial | inBuilt | IBG64 | ecosystem/inbuilt/gotchas.md |
| Deleting certified measurement cascades — audit lost | inBuilt | IBG65 | ecosystem/inbuilt/gotchas.md |
| `project.certification.measurement` lacks multi-company `ir.rule` | inBuilt | IBG66 | ecosystem/inbuilt/gotchas.md |
| Multi-currency cost summary sums raw subtotals (latent bug) | inBuilt | IBG69 | ecosystem/inbuilt/gotchas.md |
| Vendor credit notes count as positive cost — refunds invisible | inBuilt | IBG70 | ecosystem/inbuilt/gotchas.md |
| Project cost summary double-counts task-level costs (no dedup) | inBuilt | IBG71 | ecosystem/inbuilt/gotchas.md |
| Timesheet with mismatched analytic account invisible in summary | inBuilt | IBG72 | ecosystem/inbuilt/gotchas.md |
| Retention never auto-released to customer | inBuilt | IBG73 | ecosystem/inbuilt/gotchas.md |
| Changing `retention_percent` on posted invoice doesn't update retention move | inBuilt | IBG74 | ecosystem/inbuilt/gotchas.md |
| Certification `retention_percent` changes don't reach existing invoice | inBuilt | IBG75 | ecosystem/inbuilt/gotchas.md |
| Manual `product_uom_qty` edit reverted by measurement compute | inBuilt | IBG76 | ecosystem/inbuilt/gotchas.md |
| DRAFT presupuestos inflate `sale_amount_total` of the project | inBuilt | IBG77 | ecosystem/inbuilt/gotchas.md |
| Approval request stuck in `pending` though approval_minimum reached | approvals | APPG4 | enterprise/approvals/gotchas.md |
| Cambiar `category_id` borra approvers manuales del request | approvals | APPG1 | enterprise/approvals/gotchas.md |
| Manager aprueba siempre primero aunque otros tengan sequence=1 | approvals | APPG2 | enterprise/approvals/gotchas.md |
| `approval_minimum` mayor que approvers no falla — se reduce silenciosamente | approvals | APPG6 | enterprise/approvals/gotchas.md |
| `status='waiting'` no aparece nunca aunque hay sequencing | approvals | APPG7 | enterprise/approvals/gotchas.md |
| `manager_approval='required'` bloquea confirm sin parent_id | approvals | APPG11 | enterprise/approvals/gotchas.md |
| Approval `action_cancel` no cancela las POs ya creadas | approvals | APPP10 | enterprise/approvals/gotchas.md |
| "Create RFQ's" no genera ajuste tras editar líneas aprobadas | approvals | APPP2/APPP3 | enterprise/approvals/gotchas.md |
| 2 líneas mismo product + vendor + warehouse distinto → 2 POs | approvals | APPS3 | enterprise/approvals/gotchas.md |
| Product line aparece roja en list view — falta vendor | approvals | APPP9 | enterprise/approvals/gotchas.md |
| `_select_seller` cambia el vendor al cambiar qty | approvals | APPP8 | enterprise/approvals/gotchas.md |
| Multi-vendor en distintas currencies genera POs separadas | approvals | APPP4 | enterprise/approvals/gotchas.md |
| Activities del approval aprobado/rechazado dejan traza, cancelado no | approvals | APPG14 | enterprise/approvals/gotchas.md |
| v19: AccessError al aprobar tu propio request siendo owner+approver | approvals | APPG-v19-5 | enterprise/approvals/v19_changes.md |
| v19: `uom_po_id` → `uom_id` rompe overrides de approvals_purchase | approvals | APPP-v19-1 | enterprise/approvals/v19_changes.md |
| v19: `_prepare_purchase_order_line` cambió signature (seller → vendor) | approvals | APPP-v19-2 | enterprise/approvals/v19_changes.md |
| v19: `_get_purchase_orders_domain` retorna `Domain` no lista | approvals | APPP-v19-3 | enterprise/approvals/v19_changes.md |
| v19: `_ensure_can_approve()` eliminado — overrides custom rompen | approvals | APPG-v19-2 | enterprise/approvals/v19_changes.md |
| SO confirmado con picking 'assigned' tiene qty_delivered=0 e invoice_status='no' | sale | SAS1 | functional/sale/integrations.md |
| Cambiar product_uom_qty en SO 'sale' genera moves duplicados en draft | sale | SAS2 | functional/sale/integrations.md |
| Rounding en UoM conversions evita procurement aunque visualmente cambia qty | sale | SAS3 | functional/sale/integrations.md |
| MTO route con warehouse distinto al SO rompe picking sync | sale | SAS4 | functional/sale/integrations.md |
| picking_policy='one' genera 2 pickings si una línea agotada | sale | SAS5 | functional/sale/integrations.md |
| date_planned cae en pasado con customer_lead/security_lead mal config | sale | SAS6 | functional/sale/integrations.md |
| RMA parcial pendiente de validación infla qty_to_invoice | sale | SAS7 | functional/sale/integrations.md |
| task_global_project sin product.project_id rompe SO.confirm sin rollback | sale | SAP2 | functional/sale/integrations.md |
| SO con SOLs de tipos service distintos comparten project incoherentemente | sale | SAP3 | functional/sale/integrations.md |
| Cambiar qty SOL auto-sobrescribe task.allocated_hours, invalida burndown | sale | SAP4 | functional/sale/integrations.md |
| milestone.quantity_percentage default 1 → 3 milestones triplican qty_delivered | sale | SAP6 | functional/sale/integrations.md |
| Sub-tasks de project_template no heredan sale_line_id (filtro parent_id) | sale | SAP7 | functional/sale/integrations.md |
| picking_type.analytic_costs=False por default → moves no reinvoiced | sale | SAP9 | functional/sale/integrations.md |
| Phantom BOM multi-warehouse sin coordinación de moves | sale | SAM1 | functional/sale/integrations.md |
| Kit con sub-BOM no genera moves para subassemblies (explode 1-level) | sale | SAM4 | functional/sale/integrations.md |
| Dropship kit con mix MTS → qty_delivered all-or-nothing | sale | SAM5 | functional/sale/integrations.md |
| Subir qty en SO con PO done crea nueva PO para delta | sale | SAPU3 | functional/sale/integrations.md |
| Currency conversion PO usa rate del día create PO, no del SO | sale | SAPU5 | functional/sale/integrations.md |
| Cancel SO con PO recibida deja activity huérfana | sale | SAPU7 | functional/sale/integrations.md |
| recompute_delivery_price flag set pero no auto-recompute | sale | SAD1 | functional/sale/integrations.md |
| free_over recompute oscila precio con cambios de qty | sale | SAD2 | functional/sale/integrations.md |
| zip_prefix regex no escapa metachars (`.` literal interpretado) | sale | SAD5 | functional/sale/integrations.md |
| Cambiar invoice_policy post-confirm NO redispara qty_to_invoice | sale | SAG10 | functional/sale/integrations.md |
| Down payment rounding deltas asumen receivable/product/tax lines | sale | SAG12 | functional/sale/integrations.md |
| SO locked se puede facturar (locked NO bloquea _create_invoices) | sale | SAG14 | functional/sale/integrations.md |
| Multiple SOs agrupados duplican "Down Payment Section" en invoice | sale | SAG16 | functional/sale/integrations.md |
| fiscal_position change no retrigger tax recompute en SOLs no facturadas | sale | SAG17 | functional/sale/integrations.md |
| qty_delivered analytic NO filtra por accrual date — expense retroactivo salta | sale | SAG18 | functional/sale/integrations.md |
| analytic_distribution multi-company sin filtro company_id → wrong account | sale | SAG19 | functional/sale/integrations.md |
| stock.move NO hereda analytic_distribution de SOL | sale | SAG20 | functional/sale/integrations.md |
| purchase_price no recompute si product.standard_price cambia post-confirm | sale | SAMA1 | functional/sale/integrations.md |
| margin_percent 0/0 retorna False en edge cases (list_price=0+disc=100%) | sale | SAMA2 | functional/sale/integrations.md |
| MO cost actual NO propagado a SOL.margin (scrap/desviación invisibles) | sale | SAMA5 | functional/sale/integrations.md |
| Multiple expenses al mismo SO → 10 SOLs sin agrupación | sale | SAE4 | functional/sale/integrations.md |
| External tax API fail wipea ALL tax_ids sin rollback | sale | SAX1 | functional/sale/integrations.md |
| Down payments excluidos de external tax → mixed tax sources en invoice | sale | SAX4 | functional/sale/integrations.md |
| Commission calculada sobre amount_total al confirm, no qty_to_invoice | sale | SAC1 | functional/sale/integrations.md |
| Refund SO NO reversa commission posteada | sale | SAC4 | functional/sale/integrations.md |
| intrastat_country_id NULL si partner_shipping sin country | sale | SAI1 | functional/sale/integrations.md |
| v19: product_uom → product_uom_id rename rompe customs sale | sale | v19 | functional/sale/integrations.md |
| v19: tax_id → tax_ids (M2M) rompe customs sale | sale | v19 | functional/sale/integrations.md |
| v19: sale_purchase_stock refactor stock_reference_ids rompe _rollup customs | sale | v19 | functional/sale/integrations.md |
| v19: sale_async_emails REMOVIDO (mergeado en core) | sale | v19 | functional/sale/integrations.md |
| Cambiar company_id en PO draft no actualiza picking_type_id si era incoming | purchase | PUS2 | functional/purchase/integrations.md |
| reception_steps=3 cancel PO cascade y backorders no se cancelan | purchase | PUS3, PUS7 | functional/purchase/integrations.md |
| effective_date filtra location_dest=supplier — dropship distorsiona métrica | purchase | PUS5 | functional/purchase/integrations.md |
| Invoice-before-receipt con multi-currency rounding ±0.01 | purchase | PUS6, PUAC17 | functional/purchase/integrations.md |
| Bajar product_qty con qty_invoiced > new qty imposible decrementar | purchase | PUS8 | functional/purchase/integrations.md |
| RTV con mucho nesting pierde purchase_line_id | purchase | PUS9 | functional/purchase/integrations.md |
| No existe consignment/owner_id en purchase_stock v18 | purchase | PUS10 | functional/purchase/integrations.md |
| Kit phantom PO qty_received bottleneck por componente más lento | purchase | PUM2 | functional/purchase/integrations.md |
| BOM cost_share suma 100% per variant — no validación automática | purchase | PUM3 | functional/purchase/integrations.md |
| Kit dropship + componentes buy = 2 pickings con effective_date confuso | purchase | PUM6 | functional/purchase/integrations.md |
| Dropship a customer genérico choca serialization tracking | purchase | PUD1 | functional/purchase/integrations.md |
| Dropship return a warehouse no incrementa qty_received | purchase | PUD2 | functional/purchase/integrations.md |
| Intercompany sin pricelist currency = PO currency → UserError | purchase | PUIC1 | functional/purchase/integrations.md |
| Intercompany taxes NO se transfieren PO→SO automático | purchase | PUIC6 | functional/purchase/integrations.md |
| purchase_method NO es invoice_policy (confusión común) | purchase | PUAC1 | functional/purchase/integrations.md |
| qty_to_invoice=0 antes de confirmar PO aunque haya qty_received | purchase | PUAC2 | functional/purchase/integrations.md |
| TOLERANCE 0.02 en bill matching — diff >2% rompe match | purchase | PUAC4 | functional/purchase/integrations.md |
| Currency conversion bill usa today rate, no PO date_order | purchase | PUAC6 | functional/purchase/integrations.md |
| Autocomplete purchase_id borra invoice_line_ids manuales (force_write) | purchase | PUAC8 | functional/purchase/integrations.md |
| 3-way match price check strict sin tolerance config | purchase | PU3W1 | functional/purchase/integrations.md |
| 3-way match AML sin purchase_line_id = siempre exception | purchase | PU3W3 | functional/purchase/integrations.md |
| Analytic distribution silent si no rule match | purchase | PUAN1 | functional/purchase/integrations.md |
| SVL analytic_distribution solo en valuation=real_time | purchase | PUAN4 | functional/purchase/integrations.md |
| PD cantidad capped a qty_received — resto ignorado silently | purchase | PUAC11 | functional/purchase/integrations.md |
| History replay con >500 moves+invoices timeout | purchase | PUAC16 | functional/purchase/integrations.md |
| Intrastat country.intrastat=False silent (sin warning) | purchase | PUI1 | functional/purchase/integrations.md |
| UBL BIS3 NO soporta downpayments | purchase | PUED3 | functional/purchase/integrations.md |
| Cancel requisition borra supplier.info ANTES de validar POs confirmadas | purchase | PUQ1 | functional/purchase/integrations.md |
| purchase.order.group con ≤1 PO se borra dejando POs huérfanas | purchase | PUQ2 | functional/purchase/integrations.md |
| Requisition tender permite price=0 (vs blanket order que valida) | purchase | PUQ3 | functional/purchase/integrations.md |
| UoM conversion blanket→PO con factor fraccionario pierde decimales | purchase | PUQ4 | functional/purchase/integrations.md |
| Supplier sequence priority IMPLICIT en requisition_stock | purchase | PUQS1 | functional/purchase/integrations.md |
| Picking_type missing en requisition → procurement falla silently | purchase | PUQS4 | functional/purchase/integrations.md |
| sale_purchase reusa primer draft PO sin validar currency/company | purchase | PUSA1 | functional/purchase/integrations.md |
| Cancel SO solo notifica POs de service_to_purchase=True | purchase | PUSA3 | functional/purchase/integrations.md |
| Approvals_purchase seller_id two-pass fallback sin warning | purchase | PUAP1 | functional/purchase/integrations.md |
| Approvals_purchase búsqueda PO domain por currency mismatch | purchase | PUAP2 | functional/purchase/integrations.md |
| product_matrix apply 2x mismo combo → 2 líneas idénticas | purchase | PUPM1 | functional/purchase/integrations.md |
| v19: state 'done' ELIMINADO — usar locked Boolean | purchase | v19 | functional/purchase/integrations.md |
| v19: button_done() REMOVIDO — usar button_lock() | purchase | v19 | functional/purchase/integrations.md |
| v19: mail_reminder/reception_* fields REMOVIDOS de purchase.order | purchase | v19 | functional/purchase/integrations.md |
| v19: product_uom→product_uom_id, taxes_id→tax_ids, notes→note en purchase | purchase | v19 | functional/purchase/integrations.md |
| v19: onchange_partner_id_warning REMOVIDO — usar purchase_warning_text computed | purchase | v19 | functional/purchase/integrations.md |
| v19: write() override de purchase.order ELIMINADO — partner_vals en create() | purchase | v19 | functional/purchase/integrations.md |
| v19: purchase_stock usa stock_reference_ids M2M (legacy _rollup_move_origs) | purchase | v19 | functional/purchase/integrations.md |
| v19: purchase_async_emails REMOVIDO (mergeado en core) | purchase | v19 | functional/purchase/integrations.md |
| Payment sin outstanding_account_id → UserError al crear | account | ACP1 | functional/account/integrations.md |
| Cambiar amount/fecha en payment posted no sincroniza el move | account | ACP2 | functional/account/integrations.md |
| origin_payment_id huérfano si payment cancelado tras posted | account | ACP8 | functional/account/integrations.md |
| Bank rec widget transient — cambios perdidos sin validar | account | ACBW1 | functional/account/integrations.md |
| Abrir bank rec widget 2x sin validar duplica líneas | account | ACBW2 | functional/account/integrations.md |
| reconcile.model regex sin escape mete metachars literales | account | ACR1 | functional/account/integrations.md |
| payment_tolerance_param=50 con type=percentage permite ±50% | account | ACR3 | functional/account/integrations.md |
| Draft CABA moves se UNLINK en v19 (no reverse como v18) | account | ACPR4 | functional/account/integrations.md |
| price_include es computed, cambiar company politicy daña facturas viejas | account | ACT1 | functional/account/integrations.md |
| Mezclar price-included + price-excluded en MISMA línea → tax wrong | account | ACT3 | functional/account/integrations.md |
| Reverse charge detection silenciosa si factor no es exacto [-100, 100] | account | ACT5 | functional/account/integrations.md |
| Tax mapping fiscal_position cross-company no validado | account | ACT8 | functional/account/integrations.md |
| Tax mapping NO retro-update cuando cambias fiscal_position post-set | account | ACFP2 | functional/account/integrations.md |
| vat_required no valida partner.vat realmente (silent reverse charge) | account | ACFP3 | functional/account/integrations.md |
| AvaTax wipe ALL tax_ids antes de API call — falla → sin rollback | account | ACAV1 | functional/account/integrations.md |
| AvaTax tax_date no recalcula si cambias invoice_date post-post | account | ACAV2 | functional/account/integrations.md |
| l10n_es recargo sin pareja sujeto principal → SII error 3000 | account | ACSII1 | functional/account/integrations.md |
| l10n_es DUA mapping incorrecto → SII trata como factura normal | account | ACSII3 | functional/account/integrations.md |
| Intra-community EU detection falla con multi-country supplier/buyer | account | ACSII4 | functional/account/integrations.md |
| Peppol cert expira en proxy IAP — sin retry automático | account | ACPP1 | functional/account/integrations.md |
| EDI document.state='to_send' stuck si web service timeout | account | ACED2 | functional/account/integrations.md |
| EUROPEAN_ECONOMIC_AREA_COUNTRY_CODES hardcoded — nuevos EU rompen | account | ACUB4 | functional/account/integrations.md |
| Asset auto-create requiere move.line.name set | account | ACAS1 | functional/account/integrations.md |
| Asset disposal bloquea si children draft/open con value_residual>0 | account | ACAS3 | functional/account/integrations.md |
| Asset disposal_date <= lock_date → UserError sin exception bypass | account | ACAS4 | functional/account/integrations.md |
| Reversal depreciation sin moves posteriores rompe balance | account | ACAS6 | functional/account/integrations.md |
| hard_lock_date IRREVERSIBLE — sin bypass | account | ACLO1 | functional/account/integrations.md |
| bypass_lock_check requiere objeto BYPASS_LOCK_CHECK exacto (no True) | account | ACLO4 | functional/account/integrations.md |
| Auto-adjust date concurrente puede romper sequence correlativa | account | ACLO5 | functional/account/integrations.md |
| Lock exception end_datetime UTC — server tz diferente expira inesperado | account | ACLO6 | functional/account/integrations.md |
| Reconciliation multi-company SOLO branches (parent/child), no hermanos | account | ACMC1 | functional/account/integrations.md |
| Auto-transfer crea moves DRAFT, no postea — transferencia no ejecuta | account | ACAT1 | functional/account/integrations.md |
| Auto-transfer lines no pueden tener tax_ids (constraint hard) | account | ACAT2 | functional/account/integrations.md |
| Consolidation account mapping MANUAL — nuevas cuentas no aparecen auto | account | ACCS1 | functional/account/integrations.md |
| Disallowed expenses category resetea al cambiar internal_group | account | ACDE2 | functional/account/integrations.md |
| v19: tax_tag_invert REMOVIDO — usar extra_tax_data Json | account | v19 | functional/account/integrations.md |
| v19: product_uom_category_id REMOVIDO — usar allowed_uom_ids computed | account | v19 | functional/account/integrations.md |
| v19: filtro deprecated REMOVIDO de todos domains — cuentas viejas visibles | account | v19 | functional/account/integrations.md |
| v19: account.payment._compute_partner_id REMOVIDO — campo simple ahora | account | v19 | functional/account/integrations.md |
| v19: account_sepa → account_iso20022 (rebrand dependency) | account | v19 | functional/account/integrations.md |
| v19: read_group → formatted_read_group (signature nueva totalmente) | account | v19 | functional/account/integrations.md |
| v19: _sql_constraints → models.Constraint, create_index → models.Index | account | v19 | functional/account/integrations.md |
| v19: expression.AND/OR → Domain &/| (TODOS los modules account_*) | account | v19 | functional/account/integrations.md |
| v19: line_subsection nuevo display_type — l10n_es filters deben actualizar | account | v19 | functional/account/integrations.md |
| v19: account.document.import.mixin centraliza EDI import (568 líneas) | account | v19 | functional/account/integrations.md |
| v19: invoice_reference_type opción 'none' REMOVIDA | account | v19 | functional/account/integrations.md |
| v19: alias_id (M2O) → alias_name (Char) en journal — type change | account | v19 | functional/account/integrations.md |
| Slot en timezone visitante muestra hora incorrecta (slot usa appointment_tz) | appointment | APG1 | enterprise/appointment/gotchas.md |
| min_schedule_hours buffer se salta con reference_date futuro | appointment | APG2 | enterprise/appointment/gotchas.md |
| Custom slots siguen abiertos durante el evento (filtro por end_datetime) | appointment | APG3 | enterprise/appointment/gotchas.md |
| Slot con weekday='0' silenciosamente ignorado (usar '7' para domingo) | appointment | APG4 | enterprise/appointment/gotchas.md |
| active=False en calendar.event auto-cancela appointment (y viceversa) | appointment | APG5 | enterprise/appointment/gotchas.md |
| Quitar appointment_type_id de event borra historial booked/attended/no_show | appointment | APG6 | enterprise/appointment/gotchas.md |
| Re-guardar evento booked sin cambios NO reenvía email invitación | appointment | APG7 | enterprise/appointment/gotchas.md |
| Cancel vía portal token no elimina (active=False) → DB siempre crece | appointment | APG8 | enterprise/appointment/gotchas.md |
| Resource no-shareable reserva capacidad completa aunque pidan menos | appointment | APG9 | enterprise/appointment/gotchas.md |
| linked_resource_ids simétrico: leer solo source_resource_ids puede fallar | appointment | APG10 | enterprise/appointment/gotchas.md |
| resource shareable=True permite doble booking mismo slot (sin DB lock) | appointment | APG11 | enterprise/appointment/gotchas.md |
| staff_user_ids excluye portal users (domain share=False) | appointment | APG12 | enterprise/appointment/gotchas.md |
| v18 assign_method 3 valores vs v19 assignment_method 2 valores + flags | appointment | APG13 | enterprise/appointment/gotchas.md |
| v18 manual_confirmation vs v19 auto_confirm — lógica invertida | appointment | APG14 | enterprise/appointment/gotchas.md |
| short_code acepta solo [A-Za-z0-9_-] — acentos/utm params → error | appointment | APG15 | enterprise/appointment/gotchas.md |
| invite staff_user_ids obsoletos (borrado de type) → silently desaparecen | appointment | APG16 | enterprise/appointment/gotchas.md |
| invite multi-type sin priority order (orden por sequence global) | appointment | APG17 | enterprise/appointment/gotchas.md |
| access_token lazy-generated: eventos viejos sin token hasta primer acceso | appointment | APG18 | enterprise/appointment/gotchas.md |
| videocall_redirection fallback nulo si event_videocall_source vaciado | appointment | APG19 | enterprise/appointment/gotchas.md |
| Cambiar reminder_ids en type NO propaga a eventos ya existentes | appointment | APG20 | enterprise/appointment/gotchas.md |
| appointment.type sin company_id: todos los types visibles en multi-company | appointment | APG21 | enterprise/appointment/gotchas.md |
| _slots_fill_users_availability lee TODO el calendario de cada staff user | appointment | APG22 | enterprise/appointment/gotchas.md |
| _get_appointment_slots bucle Python puro — lento con max_schedule_days>90 | appointment | APG23 | enterprise/appointment/gotchas.md |
| v19: appointment.question M2O → M2M reusable (cascade delete eliminado) | appointment | APG24 | enterprise/appointment/gotchas.md |
| v19: frontend assets movidos de js/ a interactions/ — imports deben cambiar | appointment | APG25 | enterprise/appointment/gotchas.md |
| Slot end_hour=0 significa medianoche del día siguiente (24:00), no 00:00 mismo día | appointment | APG83 | enterprise/appointment/gotchas.md |
| end_hour por debajo de start_hour se reescribe silenciosamente a start+duration | appointment | APG84 | enterprise/appointment/gotchas.md |
| ValidationError en slot al subir appointment_duration del type (no cabe la duración) | appointment | APG85 | enterprise/appointment/gotchas.md |
| Quitar usuario del type vacía restrict_to_user_ids del slot (queda libre para todos) | appointment | APG86 | enterprise/appointment/gotchas.md |
| slot_type computado de category: no se pueden mezclar unique+recurring en un type | appointment | APG87 | enterprise/appointment/gotchas.md |
| Resource sequence decide qué recurso/combo se reserva primero (lowest first) | appointment | APG89 | enterprise/appointment/gotchas.md |
| appointment.resource.company_id default False → recurso visible cross-company | appointment | APG90 | enterprise/appointment/gotchas.md |
| Booking line capacity_reserved=0 → no consume capacidad pero aparece booked (ghost) | appointment | APG91 | enterprise/appointment/gotchas.md |
| Portal no deja cancelar dentro de min_cancellation_hours (default 1h, "no time left") | appointment | APG92 | enterprise/appointment/gotchas.md |
| Slot válido en grid pero rechazado al submit (recurring exige duración exacta) | appointment | APG93 | enterprise/appointment/gotchas.md |
| allow_guests + payment step → los guests se pierden al convertir booking→event | appointment | APG94 | enterprise/appointment/gotchas.md |
| Appointment con pago: capacity NO reservada hasta pago confirmado (race) | appointment | APG28P | enterprise/appointment/integrations.md |
| Appointment booking pendiente sin invoice posted queda huérfano | appointment | APG29P | enterprise/appointment/integrations.md |
| Refund de appointment pagado NO libera capacidad (manual workaround) | appointment | APG32P | enterprise/appointment/integrations.md |
| Guest_ids en booking se PIERDEN al convertir a calendar.event | appointment | APG33P | enterprise/appointment/integrations.md |
| appointment_crm lead se crea en company del organizer, no del visitor | appointment | APG35C | enterprise/appointment/integrations.md |
| appointment_crm multi-partner event solo convierte primer partner [:1] | appointment | APG38C | enterprise/appointment/integrations.md |
| appointment_crm GDPR risk: emails attendees en lead.description searchable | appointment | APG37C | enterprise/appointment/integrations.md |
| appointment_crm + pre-linked invite.opportunity_id sobrescritos por auto-create | appointment | APG39C | enterprise/appointment/integrations.md |
| appointment_sms phone no es required → SMS falla silenciosamente | appointment | APG42S | enterprise/appointment/integrations.md |
| appointment_sms language hardcoded por company, no por visitor.lang | appointment | APG45S | enterprise/appointment/integrations.md |
| appointment_sms anonymous partner sin mobile → SMS imposible | appointment | APG46S | enterprise/appointment/integrations.md |
| website_appointment.copy_data() fuerza is_published=False (sorpresa) | appointment | APG54W | enterprise/appointment/integrations.md |
| Operator selection skip si avatars_display='no' (sin feedback admin) | appointment | APG50W | enterprise/appointment/integrations.md |
| appointment_hr flexible_hours=True desactiva work_hours_activated | appointment | APG57H | enterprise/appointment/integrations.md |
| appointment_hr on_leave_partner_ids no recompute auto, invalidate manual | appointment | APG58H | enterprise/appointment/integrations.md |
| appointment_hr work_hours_activated invisible si schedule_based_on=resources | appointment | APG62H | enterprise/appointment/integrations.md |
| appointment_hr_recruitment applicant_id solo propaga via context, no URL | appointment | APG63HR | enterprise/appointment/integrations.md |
| appointment_hr_recruitment answers NO se copian a applicant.description | appointment | APG65HR | enterprise/appointment/integrations.md |
| appointment_google_calendar is_google_calendar_synced no chequea token expiry | appointment | APG67GC | enterprise/appointment/integrations.md |
| appointment_google_calendar videocall_redirection pre-sync sin Meet link | appointment | APG68GC | enterprise/appointment/integrations.md |
| appointment_google_calendar cambio event_videocall_source no limpia events viejos | appointment | APG70GC | enterprise/appointment/integrations.md |
| appointment_google_reserve cron interval=9999months — nunca corre auto | appointment | APG74GR | enterprise/appointment/integrations.md |
| appointment_google_reserve party_size ignora linked_resource_ids combinables | appointment | APG76GR | enterprise/appointment/integrations.md |
| appointment_google_reserve unlink Odoo NO notifica Google cancel | appointment | APG78GR | enterprise/appointment/integrations.md |
| appointment_microsoft_calendar sync NO bidireccional para answers/resources | appointment | APG80MC | enterprise/appointment/integrations.md |
| appointment_microsoft_calendar multi-tenant rompe sync entre staff users | appointment | APG82MC | enterprise/appointment/integrations.md |

---

## 16. POS — bridges (stock/account/restaurant/blackbox/payment/self_order/preparation_display/iot/l10n_es)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Picking POS queda en draft sin notificar al cajero | pos | POG19S | functional/pos/integrations.md |
| Diferencia en cash close no aparece en P&L | pos | POG31A | functional/pos/integrations.md |
| QR codes de mesas dejan de funcionar tras admin save | pos | POG69SO | functional/pos/integrations.md |
| No se puede cobrar ticket > 3000€ sin invoice en España | pos | POG92ES | functional/pos/integrations.md |
| Loyalty bloqueado en POS Bélgica (blackbox) | pos | POG53BB | functional/pos/integrations.md |
| qty_delivered de SO no se actualiza tras pago POS | pos | POG36SL | functional/pos/integrations.md |
| Adyen notification perdida en split payment | pos | POG60P | functional/pos/integrations.md |
| Item duplicado en KDS tras cambio de nota | pos | POG79PD | functional/pos/integrations.md |
| IoT device se desasocia al cambiar use_payment_terminal | pos | POG66P / POG88IO | functional/pos/integrations.md |
| Refund VeriFactu sin reason rechazado en sync | pos | POG93ES | functional/pos/integrations.md |
| Floor no se puede modificar con session abierta en multi-pos | pos | POG47R | functional/pos/integrations.md |
| Urban Piper orders se pierden si session está cerrada al webhook | pos | POG108UP | functional/pos/integrations.md |
| pos_iot_six (LAN direct) duplica con pos_six legacy en v18 | pos | POG65P | functional/pos/integrations.md |
| pos_iot fragmentado en v19 (iot_base CE + iot EE) | pos | v19 | functional/pos/integrations.md |

---

## 17. Mail / Discuss — mixin + integrations (chatter/template/alias/gateway/composer)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Portal users postean en records que no deberían | mail | MCG32T | functional/mail/integrations.md |
| `tracking_disable=True` rompe log de creación | mail | MCG40T | functional/mail/integrations.md |
| Imports masivos no generan activities | mail | MCG42AM | functional/mail/integrations.md |
| Template en lang inesperado (fallback a user.lang en mass) | mail | MCG50TM | functional/mail/integrations.md |
| Mass_mail no genera audit trail en chatter | mail | MCG95CP | functional/mail/integrations.md |
| Loop detection no protege models sin `_primary_email` | mail | MCG77GW / MCG39T | functional/mail/integrations.md |
| Muted member sigue recibiendo emails | mail | MCG110DC | functional/mail/integrations.md |
| Attachments composer huérfanos tras abandonar wizard | mail | MCG119AT | functional/mail/integrations.md |
| Cada respuesta del cliente "arrastra" todos los adjuntos previos | mail | MCG126AT | functional/mail/integrations.md |
| Multi-company catchall sirve mail desde company B con headers de A | mail | MC bridge | functional/mail/integrations.md |
| `alias_force_thread_id` con catchall cruza threads multi-empresa | mail | MCG alias | functional/mail/integrations.md |

---

## 18. Subscription — bridges (sale/account/payment/CRM/portal/MRR/POS/website)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Factura recurrente duplicada / draft huérfano | subscription | SUBG35A | enterprise/subscription/integrations.md |
| `payment_exception` no se borra automáticamente | subscription | SUBG43P | enterprise/subscription/integrations.md |
| Churn no aparece en MRR reports (cron usa mail_notrack=True) | subscription | SUBG65M | enterprise/subscription/integrations.md |
| Reminder spam tras cron parado (cron_call_shift compensa días) | subscription | SUBG47P | enterprise/subscription/integrations.md |
| Upsell genera entregas duplicadas (parent + upsell pickings) | subscription | SUBG30SL / SUBG76S | enterprise/subscription/integrations.md |
| POS pago no genera invoice contable (avanza next_invoice_date sin move) | subscription | SUBG82POS | enterprise/subscription/integrations.md |
| `_get_subscription_delta` usa `2_transfer` que no existe (bug latente) | subscription | SUBG69M | enterprise/subscription/integrations.md |
| v19 elimina `sale_order_alert.py` y `sale_order_option.py` (breaking) | subscription | v19 | enterprise/subscription/integrations.md |

### 18b. Subscription — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Facturar por uso / metered / consumo variable? | NO | SUBCAP1 | enterprise/subscription/capabilities.md |
| ¿Periodo de prueba gratuito (free trial)? | DESARROLLO (workaround) | SUBCAP2 | enterprise/subscription/capabilities.md |
| ¿Precios distintos mensual vs anual? | NATIVO | SUBCAP3 | enterprise/subscription/capabilities.md |
| ¿Descuentos / cupones en suscripción? | NATIVO | SUBCAP4 | enterprise/subscription/capabilities.md |
| ¿Suscripciones multi-moneda? | NATIVO | SUBCAP5 | enterprise/subscription/capabilities.md |
| ¿Facturas consolidadas por cliente? | CONFIG | SUBCAP6 | enterprise/subscription/capabilities.md |
| ¿Cobro automático con tarjeta/token? | NATIVO | SUBCAP7 | enterprise/subscription/capabilities.md |
| ¿Reintentos de pago fallido y dunning? | NATIVO | SUBCAP8 | enterprise/subscription/capabilities.md |
| ¿Prorrateo en upgrade/downgrade mid-periodo? | NATIVO | SUBCAP9 | enterprise/subscription/capabilities.md |
| ¿Upsell (añadir productos a sub viva)? | NATIVO | SUBCAP10 | enterprise/subscription/capabilities.md |
| ¿Contratos con fin / auto-cierre vs renovación? | NATIVO | SUBCAP11 | enterprise/subscription/capabilities.md |
| ¿Reactivación auto al pagar factura pendiente? | NATIVO | SUBCAP12 | enterprise/subscription/capabilities.md |
| ¿Cliente pausa/reanuda solo? | CONFIG | SUBCAP13 | enterprise/subscription/capabilities.md |
| ¿Cliente cancela solo (con opciones)? | CONFIG | SUBCAP14 | enterprise/subscription/capabilities.md |
| ¿Cliente cambia de plan desde portal? | CONFIG | SUBCAP15 | enterprise/subscription/capabilities.md |
| ¿Analítica MRR/ARR, churn, expansion? | NATIVO | SUBCAP16 | enterprise/subscription/capabilities.md |
| ¿Alertas automáticas (riesgo churn)? | NATIVO | SUBCAP17 | enterprise/subscription/capabilities.md |
| ¿Plantillas de cotización de suscripción? | NATIVO | SUBCAP18 | enterprise/subscription/capabilities.md |
| ¿Productos físicos enviados cada periodo? | NATIVO (módulo) | SUBCAP19 | enterprise/subscription/capabilities.md |
| ¿Vender suscripciones en web y POS? | NATIVO (módulos) | SUBCAP20 | enterprise/subscription/capabilities.md |

---

### 18c. Documents — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Capturar email entrante como documento (alias → workspace)? | NATIVO | DOCAP1 | enterprise/documents/capabilities.md |
| ¿Acciones/flujos automáticos sobre documentos (workspace actions)? | NATIVO | DOCAP2 | enterprise/documents/capabilities.md |
| ¿Versionado de ficheros (histórico de versiones)? | NATIVO | DOCAP3 | enterprise/documents/capabilities.md |
| ¿Solicitar un documento a alguien (request/placeholder)? | NATIVO | DOCAP4 | enterprise/documents/capabilities.md |
| ¿Carpetas/workspaces anidados? | NATIVO | DOCAP5 | enterprise/documents/capabilities.md |
| ¿Tags y reglas automáticas de etiquetado? | CONFIG | DOCAP6 | enterprise/documents/capabilities.md |
| ¿Buscar dentro del contenido del documento (full-text)? | NATIVO | DOCAP7 | enterprise/documents/capabilities.md |
| ¿Split / merge de PDF desde la interfaz? | NATIVO | DOCAP8 | enterprise/documents/capabilities.md |
| ¿Dónde se almacenan los ficheros y limpieza automática? | NATIVO | DOCAP9 | enterprise/documents/capabilities.md |
| ¿Generar un documento desde un registro de Odoo (mixin)? | NATIVO | DOCAP10 | enterprise/documents/capabilities.md |
| ¿Convertir un PDF en factura de proveedor? | NATIVO (módulo documents_account) | DOCAP11 | enterprise/documents/capabilities.md |
| ¿OCR/digitalización automática del PDF? | CONFIG (IAP de pago) | DOCAP12 | enterprise/documents/capabilities.md |
| ¿Pedir firma electrónica desde un documento? | NATIVO (módulo documents_sign) | DOCAP13 | enterprise/documents/capabilities.md |
| ¿Crear una hoja de cálculo nativa dentro de Documents? | NATIVO (módulo documents_spreadsheet) | DOCAP14 | enterprise/documents/capabilities.md |
| ¿Vincular documentos a proyecto/tareas (workspace auto)? | NATIVO (módulo documents_project) | DOCAP15 | enterprise/documents/capabilities.md |
| ¿Documents integrado con Field Service (FSM)? | NO (el puente lo desactiva) | DOCAP16 | enterprise/documents/capabilities.md |

---

### 18d. Planning — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Turnos por empleado y por rol? | NATIVO | PLANCAP1 | enterprise/planning/capabilities.md |
| ¿Turnos recurrentes / plantillas / copiar semana? | NATIVO | PLANCAP2 | enterprise/planning/capabilities.md |
| ¿Publicar el planning y notificar empleados? | NATIVO | PLANCAP3 | enterprise/planning/capabilities.md |
| ¿Open shifts que el empleado coge (self-assign)? | NATIVO | PLANCAP4 | enterprise/planning/capabilities.md |
| ¿Respetar horario laboral y ausencias? | NATIVO (ausencias con planning_holidays) | PLANCAP5 | enterprise/planning/capabilities.md |
| ¿Vender horas y planificarlas? | CONFIG (sale_planning) | PLANCAP6 | enterprise/planning/capabilities.md |
| ¿Forecast / planificar contra proyecto? | CONFIG (project_forecast, nivel proyecto) | PLANCAP7 | enterprise/planning/capabilities.md |
| ¿Asignar según skills/competencias? | CONFIG (filtro, no auto-match) | PLANCAP8 | enterprise/planning/capabilities.md |
| ¿Planning integrado con Field Service? | NO | PLANCAP9 | enterprise/planning/capabilities.md |
| ¿El empleado ve su planning (portal/self)? | NATIVO | PLANCAP10 | enterprise/planning/capabilities.md |
| ¿% de asignación / horas asignadas vs disponibles? | NATIVO | PLANCAP11 | enterprise/planning/capabilities.md |
| ¿Respetar fechas de contrato al planificar? | CONFIG (planning_contract) | PLANCAP12 | enterprise/planning/capabilities.md |

---

### 18e. Sign — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Firma de PDF con campos arrastrables? | NATIVO | SIGNCAP1 | enterprise/sign/capabilities.md |
| ¿Múltiples firmantes con orden y roles? | NATIVO | SIGNCAP2 | enterprise/sign/capabilities.md |
| ¿Plantillas reutilizables de documento? | NATIVO | SIGNCAP3 | enterprise/sign/capabilities.md |
| ¿Enviar a firmar a externos sin cuenta (token)? | NATIVO | SIGNCAP4 | enterprise/sign/capabilities.md |
| ¿Autenticación del firmante (SMS / identidad)? | NATIVO (solo SMS en v18) | SIGNCAP5 | enterprise/sign/capabilities.md |
| ¿Audit trail / certificado / log inalterable? | NATIVO | SIGNCAP6 | enterprise/sign/capabilities.md |
| ¿Recordatorios automáticos y caducidad? | NATIVO | SIGNCAP7 | enterprise/sign/capabilities.md |
| ¿Firma en persona / in situ? | NATIVO | SIGNCAP8 | enterprise/sign/capabilities.md |
| ¿Firmar contratos de RRHH en el onboarding? | NATIVO (hr_contract_sign) | SIGNCAP9 | enterprise/sign/capabilities.md |
| ¿Campos autocompletados desde el partner? | NATIVO | SIGNCAP10 | enterprise/sign/capabilities.md |
| ¿Pago + firma combinados? | NO | SIGNCAP11 | enterprise/sign/capabilities.md |
| ¿Pedir firma desde Documents? | NATIVO (documents_sign) | SIGNCAP12 | enterprise/sign/capabilities.md |

---

### 18f. Rental — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Alquilar con periodo (pickup/return)? | NATIVO | RENTCAP1 | enterprise/rental/capabilities.md |
| ¿Tarifas escalonadas por duración? | NATIVO | RENTCAP2 | enterprise/rental/capabilities.md |
| ¿Fianza / depósito de seguridad? | NO | RENTCAP3 | enterprise/rental/capabilities.md |
| ¿Penalización por devolución tardía? | NATIVO | RENTCAP4 | enterprise/rental/capabilities.md |
| ¿Stock real: pickup/return generan movimientos? | CONFIG (sale_stock_renting) | RENTCAP5 | enterprise/rental/capabilities.md |
| ¿Disponibilidad / evitar overbooking? | NATIVO | RENTCAP6 | enterprise/rental/capabilities.md |
| ¿Vender y alquilar en el mismo pedido? | NATIVO | RENTCAP7 | enterprise/rental/capabilities.md |
| ¿Alquiler desde eCommerce/web? | CONFIG (website_sale_renting) | RENTCAP8 | enterprise/rental/capabilities.md |
| ¿Facturación del alquiler? | NATIVO | RENTCAP9 | enterprise/rental/capabilities.md |
| ¿Estado del flujo y cantidades recogida/devuelta? | NATIVO | RENTCAP10 | enterprise/rental/capabilities.md |
| ¿Informe/análisis de alquiler? | NATIVO | RENTCAP11 | enterprise/rental/capabilities.md |
| ¿Alquiler con tracking por número de serie? | NATIVO | RENTCAP12 | enterprise/rental/capabilities.md |

---

### 18g. Approvals — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Tipos de aprobación configurables? | NATIVO | APPRCAP1 | enterprise/approvals/capabilities.md |
| ¿Mínimo de aprobaciones + múltiples aprobadores? | NATIVO | APPRCAP2 | enterprise/approvals/capabilities.md |
| ¿Aprobadores por usuario o por grupo/rol? | NATIVO (solo usuario + manager) | APPRCAP3 | enterprise/approvals/capabilities.md |
| ¿Flujo secuencial vs paralelo? | CONFIG | APPRCAP4 | enterprise/approvals/capabilities.md |
| ¿Campos activables por tipo (importe/partner/producto…)? | NATIVO | APPRCAP5 | enterprise/approvals/capabilities.md |
| ¿Adjuntar documentos obligatorios? | NATIVO | APPRCAP6 | enterprise/approvals/capabilities.md |
| ¿Una aprobación genera pedido de compra? | NATIVO (approvals_purchase) | APPRCAP7 | enterprise/approvals/capabilities.md |
| ¿Notificaciones/actividades a aprobadores? | NATIVO | APPRCAP8 | enterprise/approvals/capabilities.md |
| ¿Routing condicional por umbral de importe? | NO | APPRCAP9 | enterprise/approvals/capabilities.md |
| ¿Estados del flujo y permisos? | NATIVO | APPRCAP10 | enterprise/approvals/capabilities.md |

---

### 18h. Quality — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Puntos de control por producto/categoría/operación? | NATIVO | QCAP1 | enterprise/quality/capabilities.md |
| ¿Tipos de control (passfail/medida/foto/worksheet)? | NATIVO | QCAP2 | enterprise/quality/capabilities.md |
| ¿Controles en recepción/entrega/fabricación? | NATIVO | QCAP3 | enterprise/quality/capabilities.md |
| ¿Alertas de calidad con etapas y equipo? | NATIVO | QCAP4 | enterprise/quality/capabilities.md |
| ¿Controles en OF y por orden de trabajo? | NATIVO (quality_mrp/_workorder) | QCAP5 | enterprise/quality/capabilities.md |
| ¿Hojas de trabajo configurables como control? | NATIVO (quality_control_worksheet) | QCAP6 | enterprise/quality/capabilities.md |
| ¿Medición automática vía IoT? | NATIVO (quality_iot) | QCAP7 | enterprise/quality/capabilities.md |
| ¿Frecuencia/periodicidad del control? | CONFIG | QCAP8 | enterprise/quality/capabilities.md |
| ¿Un control pendiente bloquea el flujo? | NATIVO | QCAP9 | enterprise/quality/capabilities.md |
| ¿Equipos de calidad y asignación? | NATIVO | QCAP10 | enterprise/quality/capabilities.md |

---

### 18i. Knowledge — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Artículos jerárquicos tipo wiki? | NATIVO | KNCAP1 | enterprise/knowledge/capabilities.md |
| ¿Permisos por artículo y miembros? | NATIVO | KNCAP2 | enterprise/knowledge/capabilities.md |
| ¿Workspace/privado/compartido con herencia? | NATIVO | KNCAP3 | enterprise/knowledge/capabilities.md |
| ¿Plantillas de artículo? | NATIVO | KNCAP4 | enterprise/knowledge/capabilities.md |
| ¿Items embebidos (vistas Odoo)? | NATIVO | KNCAP5 | enterprise/knowledge/capabilities.md |
| ¿Versionado/historial? | NATIVO | KNCAP6 | enterprise/knowledge/capabilities.md |
| ¿Favoritos con orden personal? | NATIVO | KNCAP7 | enterprise/knowledge/capabilities.md |
| ¿Papelera con borrado diferido? | NATIVO | KNCAP8 | enterprise/knowledge/capabilities.md |
| ¿Publicar artículo en la web? | NATIVO (website_knowledge) | KNCAP9 | enterprise/knowledge/capabilities.md |
| ¿Propiedades personalizadas? | NATIVO | KNCAP10 | enterprise/knowledge/capabilities.md |

---

### 18j. Appointment — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Tipos de cita configurables? | NATIVO | APPTCAP1 | enterprise/appointment/capabilities.md |
| ¿Disponibilidad por usuario según calendario? | NATIVO (horario laboral vía appointment_hr) | APPTCAP2 | enterprise/appointment/capabilities.md |
| ¿Citas con recursos (salas/mesas)? | NATIVO | APPTCAP3 | enterprise/appointment/capabilities.md |
| ¿Reserva pública desde web sin login? | NATIVO | APPTCAP4 | enterprise/appointment/capabilities.md |
| ¿Capacidad por slot / reservas múltiples? | NATIVO (modo recursos) | APPTCAP5 | enterprise/appointment/capabilities.md |
| ¿Antelación mínima y ventana máxima? | NATIVO | APPTCAP6 | enterprise/appointment/capabilities.md |
| ¿Preguntas personalizadas al reservar? | NATIVO | APPTCAP7 | enterprise/appointment/capabilities.md |
| ¿Asignación staff elegido vs automático? | NATIVO | APPTCAP8 | enterprise/appointment/capabilities.md |
| ¿Enlace de videollamada al confirmar? | NATIVO (solo Discuss en base) | APPTCAP9 | enterprise/appointment/capabilities.md |
| ¿Recordatorios y confirmación automática? | NATIVO | APPTCAP10 | enterprise/appointment/capabilities.md |
| ¿Pago al reservar? | NO (módulo aparte) | APPTCAP11 | enterprise/appointment/capabilities.md |
| ¿Sync calendario externo (Google/Microsoft)? | NO (módulo aparte) | APPTCAP12 | enterprise/appointment/capabilities.md |

---

### 18k. Marketing Automation — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Campañas multi-paso con flujo? | NATIVO | MACAP1 | enterprise/marketing_automation/capabilities.md |
| ¿Disparadores (al entrar/tras tiempo/tras acción)? | NATIVO | MACAP2 | enterprise/marketing_automation/capabilities.md |
| ¿Target cualquier modelo con dominio? | NATIVO | MACAP3 | enterprise/marketing_automation/capabilities.md |
| ¿Acciones email/SMS/WhatsApp/server action? | NATIVO (SMS/WA puentes) | MACAP4 | enterprise/marketing_automation/capabilities.md |
| ¿Ramas condicionales según interacción? | NATIVO | MACAP5 | enterprise/marketing_automation/capabilities.md |
| ¿A/B testing de la campaña? | NO | MACAP6 | enterprise/marketing_automation/capabilities.md |
| ¿Métricas por actividad? | NATIVO | MACAP7 | enterprise/marketing_automation/capabilities.md |
| ¿Participantes únicos / no reprocesar? | NATIVO | MACAP8 | enterprise/marketing_automation/capabilities.md |
| ¿Probar la campaña antes de lanzar? | NATIVO | MACAP9 | enterprise/marketing_automation/capabilities.md |
| ¿Cron que ejecuta actividades programadas? | NATIVO | MACAP10 | enterprise/marketing_automation/capabilities.md |

---

### 18l. Social Marketing — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Cuentas de redes y media? | NATIVO | SOCAP1 | enterprise/social/capabilities.md |
| ¿Publicar en varias redes a la vez? | NATIVO | SOCAP2 | enterprise/social/capabilities.md |
| ¿Programar publicaciones? | NATIVO | SOCAP3 | enterprise/social/capabilities.md |
| ¿Bandeja de interacciones/comentarios? | NATIVO | SOCAP4 | enterprise/social/capabilities.md |
| ¿Responder comentarios desde Odoo? | NATIVO | SOCAP5 | enterprise/social/capabilities.md |
| ¿Métricas/insights por cuenta y post? | NATIVO | SOCAP6 | enterprise/social/capabilities.md |
| ¿Crear lead desde interacción social? | NATIVO (social_crm) | SOCAP7 | enterprise/social/capabilities.md |
| ¿Push notifications web? | NATIVO (social_push_notifications) | SOCAP8 | enterprise/social/capabilities.md |
| ¿Qué redes soporta? | NATIVO (FB/IG/LinkedIn/X/YouTube) | SOCAP9 | enterprise/social/capabilities.md |
| ¿Tracking UTM de enlaces? | NATIVO | SOCAP10 | enterprise/social/capabilities.md |

---

### 18m. WhatsApp — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Envío vía WhatsApp Business API (Meta)? | NATIVO | WACAP1 | enterprise/whatsapp/capabilities.md |
| ¿Plantillas aprobadas con variables? | NATIVO | WACAP2 | enterprise/whatsapp/capabilities.md |
| ¿Conversación bidireccional (recepción)? | NATIVO | WACAP3 | enterprise/whatsapp/capabilities.md |
| ¿Disparar plantilla desde un registro? | NATIVO | WACAP4 | enterprise/whatsapp/capabilities.md |
| ¿Integración con Discuss? | NATIVO | WACAP5 | enterprise/whatsapp/capabilities.md |
| ¿Ventana de servicio de 24h? | NATIVO | WACAP6 | enterprise/whatsapp/capabilities.md |
| ¿Botones/respuestas interactivas? | NATIVO | WACAP7 | enterprise/whatsapp/capabilities.md |
| ¿Adjuntos/multimedia? | NATIVO | WACAP8 | enterprise/whatsapp/capabilities.md |
| ¿Multi-cuenta / varios números? | NATIVO | WACAP9 | enterprise/whatsapp/capabilities.md |
| ¿Envío masivo por WhatsApp? | PARCIAL (solo marketing_automation) | WACAP10 | enterprise/whatsapp/capabilities.md |

---

### 18n. Email Marketing (mass_mailing) — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Editor de email drag-and-drop? | NATIVO | MMCAP1 | functional/mailing/capabilities.md |
| ¿Listas de correo y contactos? | NATIVO | MMCAP2 | functional/mailing/capabilities.md |
| ¿Segmentar por dominio sobre cualquier modelo? | NATIVO | MMCAP3 | functional/mailing/capabilities.md |
| ¿A/B testing de campañas? | NATIVO | MMCAP4 | functional/mailing/capabilities.md |
| ¿Métricas sent/opened/clicked/bounced? | NATIVO | MMCAP5 | functional/mailing/capabilities.md |
| ¿Unsubscribe y lista negra? | NATIVO | MMCAP6 | functional/mailing/capabilities.md |
| ¿Programar el envío? | NATIVO | MMCAP7 | functional/mailing/capabilities.md |
| ¿Envío por SMS? | NATIVO (mass_mailing_sms) | MMCAP8 | functional/mailing/capabilities.md |
| ¿Tracking de clics con UTM? | NATIVO | MMCAP9 | functional/mailing/capabilities.md |
| ¿Throttling / límite diario? | PARCIAL (lotes sí, diario no en CE) | MMCAP10 | functional/mailing/capabilities.md |

---

### 18o. Studio — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Crear modelos y campos sin código? | NATIVO | STUCAP1 | enterprise/studio/capabilities.md |
| ¿Editar vistas (form/list/kanban) arrastrando? | NATIVO | STUCAP2 | enterprise/studio/capabilities.md |
| ¿Crear menús y acciones? | NATIVO | STUCAP3 | enterprise/studio/capabilities.md |
| ¿Crear automatizaciones sin código? | NATIVO | STUCAP4 | enterprise/studio/capabilities.md |
| ¿Editor de informes QWeb? | NATIVO | STUCAP5 | enterprise/studio/capabilities.md |
| ¿Reglas de aprobación en botones? | NATIVO | STUCAP6 | enterprise/studio/capabilities.md |
| ¿Exportar la personalización como módulo? | NATIVO | STUCAP7 | enterprise/studio/capabilities.md |
| ¿Campos computados con Python desde Studio? | PARCIAL | STUCAP8 | enterprise/studio/capabilities.md |
| ¿Editar la pantalla de inicio / apps? | NATIVO | STUCAP9 | enterprise/studio/capabilities.md |
| ¿Qué NO puede (solo escribe datos, nunca Python real)? | LÍMITE | STUCAP10 | enterprise/studio/capabilities.md |

---

### 18p. Timesheet — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Registrar horas por tarea/proyecto? | NATIVO (CE) | TCAP1 | enterprise/timesheet/capabilities.md |
| ¿Vista grid editable por celda? | NATIVO (EE timesheet_grid) | TCAP2 | enterprise/timesheet/capabilities.md |
| ¿Temporizador start/stop? | NATIVO (EE) | TCAP3 | enterprise/timesheet/capabilities.md |
| ¿Facturar timesheets en un pedido? | NATIVO (sale_timesheet) | TCAP4 | enterprise/timesheet/capabilities.md |
| ¿Aprobación/validación de partes? | NATIVO (EE) | TCAP5 | enterprise/timesheet/capabilities.md |
| ¿Redondeo y duración mínima? | NATIVO (EE) | TCAP6 | enterprise/timesheet/capabilities.md |
| ¿Asistencia vs horas imputadas? | NATIVO (hr_timesheet_attendance) | TCAP7 | enterprise/timesheet/capabilities.md |
| ¿Registro desde portal? | CONSULTA sí, alta NO | TCAP8 | enterprise/timesheet/capabilities.md |
| ¿Rentabilidad por proyecto? | NATIVO (sale_timesheet) | TCAP9 | enterprise/timesheet/capabilities.md |
| ¿Control de horas vs planificado/vendido? | NATIVO (informativo) | TCAP10 | enterprise/timesheet/capabilities.md |

---

### 18q. Payment (motor de pago) — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Qué pasarelas trae de serie? | NATIVO (17 providers CE) | PAYCAP1 | cross-module/payment/capabilities.md |
| ¿Guardar tarjeta / tokenización? | NATIVO | PAYCAP2 | cross-module/payment/capabilities.md |
| ¿Cobro recurrente automático con token (S2S)? | NATIVO | PAYCAP3 | cross-module/payment/capabilities.md |
| ¿Autorizar y capturar después? | NATIVO | PAYCAP4 | cross-module/payment/capabilities.md |
| ¿Reembolsar desde Odoo? | NATIVO | PAYCAP5 | cross-module/payment/capabilities.md |
| ¿Reembolso parcial? | NATIVO | PAYCAP6 | cross-module/payment/capabilities.md |
| ¿Modo test/sandbox por proveedor? | NATIVO | PAYCAP7 | cross-module/payment/capabilities.md |
| ¿Límite de importe / moneda / país? | NATIVO | PAYCAP8 | cross-module/payment/capabilities.md |
| ¿Pago manual/offline con instrucciones? | NATIVO (payment_custom) | PAYCAP9 | cross-module/payment/capabilities.md |
| ¿Enviar enlace de pago desde backend? | NATIVO | PAYCAP10 | cross-module/payment/capabilities.md |
| ¿Express checkout (Apple/Google Pay)? | NATIVO | PAYCAP11 | cross-module/payment/capabilities.md |
| ¿3DS / SCA? | NATIVO (delegado al provider) | PAYCAP12 | cross-module/payment/capabilities.md |
| ¿Comisiones del proveedor (fees) reflejadas? | NO | PAYCAP13 | cross-module/payment/capabilities.md |
| ¿POS y eCommerce comparten motor? | NO | PAYCAP14 | cross-module/payment/capabilities.md |

---

### 18r. Delivery / envíos — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Métodos de envío configurables? | NATIVO | DELCAP1 | functional/delivery/capabilities.md |
| ¿Tarifas por reglas (peso/precio/cantidad)? | NATIVO | DELCAP2 | functional/delivery/capabilities.md |
| ¿Tarifa fija como línea del pedido? | NATIVO | DELCAP3 | functional/delivery/capabilities.md |
| ¿Envío gratis a partir de un importe? | NATIVO | DELCAP4 | functional/delivery/capabilities.md |
| ¿Calcular precio real contra el transportista? | NATIVO (EE) | DELCAP5 | functional/delivery/capabilities.md |
| ¿Generar etiqueta de envío vía API? | NATIVO (EE) | DELCAP6 | functional/delivery/capabilities.md |
| ¿Número y URL de seguimiento? | NATIVO | DELCAP7 | functional/delivery/capabilities.md |
| ¿Qué transportistas trae de serie? | NATIVO (fedex/ups/dhl/sendcloud/mondialrelay…) | DELCAP8 | functional/delivery/capabilities.md |
| ¿Restringir método por país/zona/CP? | NATIVO | DELCAP9 | functional/delivery/capabilities.md |
| ¿Elegir método de envío en el checkout web? | NATIVO | DELCAP10 | functional/delivery/capabilities.md |
| ¿Cancelar un envío ya creado? | NATIVO (EE) | DELCAP11 | functional/delivery/capabilities.md |
| ¿Multi-paquete / peso y dimensiones? | NATIVO | DELCAP12 | functional/delivery/capabilities.md |
| ¿Puntos de recogida (pickup points)? | NATIVO | DELCAP13 | functional/delivery/capabilities.md |
| ¿Margen coste vs cobrado? | CONFIG | DELCAP14 | functional/delivery/capabilities.md |

---

### 18s. Loyalty / promociones — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Qué tipos de programa hay? | NATIVO (8 tipos) | LOYCAP1 | cross-module/loyalty/capabilities.md |
| ¿Descuentos % o fijo, pedido/productos/más barato? | NATIVO | LOYCAP2 | cross-module/loyalty/capabilities.md |
| ¿Compra X llévate Y? | NATIVO | LOYCAP3 | cross-module/loyalty/capabilities.md |
| ¿Cupones con código y generación masiva? | NATIVO | LOYCAP4 | cross-module/loyalty/capabilities.md |
| ¿Tarjetas regalo (compra y canje)? | NATIVO | LOYCAP5 | cross-module/loyalty/capabilities.md |
| ¿Monedero / ewallet? | NATIVO | LOYCAP6 | cross-module/loyalty/capabilities.md |
| ¿Puntos por dinero/unidad/pedido y canje? | NATIVO | LOYCAP7 | cross-module/loyalty/capabilities.md |
| ¿Caducidad de puntos/cupones? | NATIVO | LOYCAP8 | cross-module/loyalty/capabilities.md |
| ¿Restricción por producto/importe/fechas/pricelist/web? | NATIVO | LOYCAP9 | cross-module/loyalty/capabilities.md |
| ¿Solo con código vs automática? | NATIVO | LOYCAP10 | cross-module/loyalty/capabilities.md |
| ¿Recompensa de envío gratis? | CONFIG (sale_loyalty_delivery) | LOYCAP11 | cross-module/loyalty/capabilities.md |
| ¿Aplicar en eCommerce y POS? | NATIVO | LOYCAP12 | cross-module/loyalty/capabilities.md |
| ¿Cupones de siguiente pedido? | NATIVO | LOYCAP13 | cross-module/loyalty/capabilities.md |
| ¿Combinar varios programas en un pedido? | NATIVO | LOYCAP14 | cross-module/loyalty/capabilities.md |

---

### 18t. Expenses / gastos — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Registrar gasto con producto/importe/impuestos? | NATIVO | EXPCAP1 | functional/expense/capabilities.md |
| ¿Agrupar en informe y enviar a aprobar? | NATIVO | EXPCAP2 | functional/expense/capabilities.md |
| ¿Reembolsable (empleado) vs tarjeta de empresa? | NATIVO | EXPCAP3 | functional/expense/capabilities.md |
| ¿Reembolsar al empleado y contabilizar? | NATIVO | EXPCAP4 | functional/expense/capabilities.md |
| ¿Escanear ticket con OCR? | NATIVO (EE, IAP de pago) | EXPCAP5 | functional/expense/capabilities.md |
| ¿Crear gasto por email con el ticket? | CONFIG | EXPCAP6 | functional/expense/capabilities.md |
| ¿Refacturar el gasto al cliente? | NATIVO (sale_expense) | EXPCAP7 | functional/expense/capabilities.md |
| ¿Imputar a proyecto / analítica? | NATIVO | EXPCAP8 | functional/expense/capabilities.md |
| ¿Reembolso vía nómina? | NATIVO (EE) | EXPCAP9 | functional/expense/capabilities.md |
| ¿Gastos en otra moneda? | NATIVO | EXPCAP10 | functional/expense/capabilities.md |
| ¿Kilometraje / dietas? | NO | EXPCAP11 | functional/expense/capabilities.md |
| ¿Aprobación en dos pasos (manager + contabilidad)? | CONFIG | EXPCAP12 | functional/expense/capabilities.md |
| ¿Varios justificantes / verlos en Documents? | NATIVO | EXPCAP13 | functional/expense/capabilities.md |
| ¿Límite de importe por categoría? | NO | EXPCAP14 | functional/expense/capabilities.md |

---

## 19quater. Expenses — flujo de gasto (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Escribo el state del gasto/informe y no cambia | EXPG1 | functional/expense/gotchas.md |
| Un mismo "enviar" me creó dos informes distintos | EXPG3 | functional/expense/gotchas.md |
| Gasto de empresa aparece "pagado" pero el banco no está conciliado | EXPG12 | functional/expense/gotchas.md |
| No puedo refusar/cancelar un informe con asiento posteado | EXPG13 | functional/expense/gotchas.md |
| El refacturado al cliente no se genera (falta analítica) | EXPG19 | functional/expense/integrations.md |
| Resetear un informe refacturado descuadra las cantidades de la SO | EXPG20 | functional/expense/integrations.md |
| El OCR no se dispara al adjuntar el ticket | EXPG21 | functional/expense/integrations.md |
| No puedo cambiar el estado de un gasto a mano, lo escribo y no pasa nada | EXPG2 | functional/expense/gotchas.md |
| Metí el importe con impuestos incluidos pero el impuesto sale mal calculado | EXPG4 | functional/expense/gotchas.md |
| No me deja reportar un gasto con importe 0 o sin producto | EXPG5 | functional/expense/gotchas.md |
| Error al poner total 0 en una línea de gasto ya en un informe | EXPG6 | functional/expense/gotchas.md |
| Puse un tipo de cambio manual y no se recalcula al guardar | EXPG7 | functional/expense/gotchas.md |
| El reembolso al empleado sale en moneda de la empresa, no en la del gasto | EXPG8 | functional/expense/gotchas.md |
| No me deja borrar un gasto aprobado o un informe contabilizado | EXPG9 | functional/expense/gotchas.md |
| Un usuario normal no puede aprobar/validar gastos (error) | EXPG10 | functional/expense/gotchas.md |
| No puedo quitar líneas de un informe que no está en borrador | EXPG11 | functional/expense/gotchas.md |
| Al refusar un gasto da error "linked to a posted journal entry" | EXPG14 | functional/expense/gotchas.md |
| No puedo marcar un producto como gasto (no consumible/servicio o no comprable) | EXPG14 | functional/expense/gotchas.md |
| Cambié la moneda del gasto y me pisó el total en moneda de empresa | EXPG25 | functional/expense/gotchas.md |
| Con tipo de cambio manual el impuesto en moneda de empresa no cuadra | EXPG26 | functional/expense/gotchas.md |
| No me deja editar impuestos/analítica/cuenta del gasto aunque tengo permisos | EXPG27 | functional/expense/gotchas.md |
| Cambié el empleado de una línea y cambió el de todo el informe | EXPG28 | functional/expense/gotchas.md |
| Guardé un gasto sin analítica y solo da error al aprobar el informe | EXPG29 | functional/expense/gotchas.md |
| No me deja borrar una cuenta analítica usada en algún gasto | EXPG30 | functional/expense/gotchas.md |
| No puedo adjuntar a un gasto ya enviado, me pide resetear a borrador | EXPG31 | functional/expense/gotchas.md |
| Moví una línea de gasto y los adjuntos no se mueven | EXPG32 | functional/expense/gotchas.md |
| No veo/edito la fecha contable del gasto sin permisos contables | EXPG33 | functional/expense/gotchas.md |
| El precio unitario del gasto no se recalcula tras enviarlo | EXPG35 | functional/expense/gotchas.md |
| Reescribí moneda/total/fecha y se perdió mi tipo de cambio manual | EXPG36 | functional/expense/gotchas.md |
| El informe no pasa a "done"/pagado aunque lo aprobé | EXPG37 | functional/expense/gotchas.md |
| No puedo editar el pago de un gasto pagado por la empresa | EXPG16 | functional/expense/integrations.md |
| No me deja borrar solo algunos asientos/pagos de un informe | EXPG17 | functional/expense/integrations.md |
| Cancelé el asiento del gasto y el gasto sigue activo | EXPG18 | functional/expense/integrations.md |
| No puedo quitar un informe de gastos de una nómina ya validada | EXPG23 | functional/expense/integrations.md |
| v19: módulo roto — `hr.expense.sheet` / `sheet_id` ya no existen | — | functional/expense/v19_changes.md |

---

### 18u. Events / eventos — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Evento con varias entradas y aforo? | NATIVO | EVTCAP1 | functional/event/capabilities.md |
| ¿Inscripción online desde la web? | NATIVO (website_event) | EVTCAP2 | functional/event/capabilities.md |
| ¿Vender entradas de pago? | NATIVO (event_sale) | EVTCAP3 | functional/event/capabilities.md |
| ¿Entradas gratis con confirmación automática? | NATIVO | EVTCAP4 | functional/event/capabilities.md |
| ¿Agenda / ponencias / tracks? | NATIVO (website_event_track) | EVTCAP5 | functional/event/capabilities.md |
| ¿Call for proposals? | NATIVO | EVTCAP6 | functional/event/capabilities.md |
| ¿Stands de patrocinadores? | NATIVO (event_booth) | EVTCAP7 | functional/event/capabilities.md |
| ¿Eventos online / salas de vídeo? | NATIVO (website_event_meet/jitsi) | EVTCAP8 | functional/event/capabilities.md |
| ¿Check-in con código de barras / badge? | NATIVO | EVTCAP9 | functional/event/capabilities.md |
| ¿Emails automáticos programados? | NATIVO | EVTCAP10 | functional/event/capabilities.md |
| ¿Preguntas personalizadas en el registro? | NATIVO | EVTCAP11 | functional/event/capabilities.md |
| ¿Generar lead desde inscripciones? | NATIVO (event_crm) | EVTCAP12 | functional/event/capabilities.md |
| ¿Vender/registrar entradas en POS? | NATIVO (pos_event) | EVTCAP13 | functional/event/capabilities.md |
| ¿Plantilla reutilizable y eventos recurrentes? | PLANTILLA sí / RECURRENCIA NO | EVTCAP14 | functional/event/capabilities.md |

---

## 19quinquies. Events — inscripción/aforo/comunicación (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| El asistente aparece confirmado nada más crearlo (no en borrador) | EVTG1 | functional/event/gotchas.md |
| Los asientos disponibles no cuadran (draft/cancel no restan) | EVTG3 | functional/event/gotchas.md |
| Edité la plantilla del tipo de evento y los eventos existentes no cambian | EVTG10 | functional/event/gotchas.md |
| El recordatorio pre-evento no se envió (el evento ya había pasado) | EVTG7 | functional/event/gotchas.md |
| Confirmé varias SO a la vez y las inscripciones no quedaron en borrador | EVTG15 | functional/event/integrations.md |
| Bajé la cantidad en el carrito y se cancelaron asistentes | EVTG17 | functional/event/integrations.md |
| Reservé un stand y se cancelaron otros pedidos del mismo stand | EVTG21 | functional/event/integrations.md |
| Confirmo una venta de evento y no me deja rellenar los datos del asistente | EVTG16 | functional/event/integrations.md |
| El registro web de entradas gratis no aparece al buscar por pedido de venta | EVTG16 | functional/event/integrations.md |
| La registration del evento web sale sin partner / sin cliente | EVTG18 | functional/event/integrations.md |
| El patrocinador no se crea aunque venda el stand | EVTG22 | functional/event/integrations.md |
| Las escrituras de inscripciones van lentas con POS instalado | EVTG23 | functional/event/integrations.md |
| Me llega el email de cambio de etapa al crear la propuesta de charla | EVTG24 | functional/event/integrations.md |
| Una charla sin publicar aparece en la agenda del evento | EVTG24 | functional/event/integrations.md |
| Busco el flag auto_confirm y ya no existe (auto-confirma por defecto) | EVTG2 | functional/event/gotchas.md |
| El asistente que vino cuenta en "usados", no en "reservados" | EVTG4 | functional/event/gotchas.md |
| El aforo del evento y la suma de aforos de tickets no coinciden | EVTG5 | functional/event/gotchas.md |
| El email after_sub se programa desde la fecha de creación del EVENTO | EVTG6 | functional/event/gotchas.md |
| El scheduler after_sub nunca queda "sent", sigue siempre en running | EVTG8 | functional/event/gotchas.md |
| Al confirmar un registro se disparan los emails en caliente (como superuser) | EVTG9 | functional/event/gotchas.md |
| Los eventos terminados se mueven solos a la etapa "End" | EVTG11 | functional/event/gotchas.md |
| No me deja cambiar el tipo de pregunta ni borrar un ticket con registros | EVTG12 | functional/event/gotchas.md |
| El check-in por código de barras confirma a "done" automáticamente | EVTG13 | functional/event/gotchas.md |
| La fecha de asistencia no se actualiza si ya tenía valor | EVTG14 | functional/event/gotchas.md |
| v19: módulo roto por slots / `event.question.event_id` ahora M2many | — | functional/event/v19_changes.md |

---

### 18v. eLearning / cursos — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Cursos con vídeo/PDF/artículo/quiz? | NATIVO | SLDCAP1 | functional/slides/capabilities.md |
| ¿Vídeo YouTube/Drive/Vimeo? | NATIVO | SLDCAP2 | functional/slides/capabilities.md |
| ¿Inscripción pública/invitación/pago? | CONFIG | SLDCAP3 | functional/slides/capabilities.md |
| ¿Vender cursos? | CONFIG (website_sale_slides) | SLDCAP4 | functional/slides/capabilities.md |
| ¿Quiz con puntuación? | NATIVO | SLDCAP5 | functional/slides/capabilities.md |
| ¿Certificación con certificado descargable? | CONFIG (website_slides_survey) | SLDCAP6 | functional/slides/capabilities.md |
| ¿Gamificación: karma/rangos/insignias? | NATIVO | SLDCAP7 | functional/slides/capabilities.md |
| ¿Progreso y % completado por alumno? | NATIVO | SLDCAP8 | functional/slides/capabilities.md |
| ¿Prerrequisitos entre cursos? | NATIVO | SLDCAP9 | functional/slides/capabilities.md |
| ¿Foro por curso? | CONFIG (website_slides_forum) | SLDCAP10 | functional/slides/capabilities.md |
| ¿Reseñas / valoración? | NATIVO | SLDCAP11 | functional/slides/capabilities.md |
| ¿Competencias del empleado al completar? | PARCIAL (al CV, no skill) | SLDCAP12 | functional/slides/capabilities.md |
| ¿Visibilidad y publicación? | NATIVO | SLDCAP13 | functional/slides/capabilities.md |
| ¿Duración por lección? | NATIVO | SLDCAP14 | functional/slides/capabilities.md |

---

## 19sexies. eLearning — cursos/certificación/karma (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| El curso no se marca completado / completar es automático | SLDG1 | functional/slides/gotchas.md |
| Despubliqué un slide y los alumnos ya terminados no cambian | SLDG2 | functional/slides/gotchas.md |
| Repetir el quiz no sube más karma (decreciente por intento) | SLDG4 | functional/slides/gotchas.md |
| Un invitado ve el curso pero no accede al contenido | SLDG8 | functional/slides/gotchas.md |
| Suspendí la certificación y me echó del curso | SLDG17 | functional/slides/integrations.md |
| Aprobé la certificación pero no me dieron insignia | SLDG15 | functional/slides/integrations.md |
| No puedo borrar el survey de certificación | SLDG22 | functional/slides/integrations.md |
| El karma de un alumno baja al despublicar un curso o quitarle slides | SLDG3 | functional/slides/gotchas.md |
| No me deja marcar un slide con preguntas como completado a mano | SLDG5 | functional/slides/gotchas.md |
| Error al marcar visto/completado un slide sin estar inscrito | SLDG6 | functional/slides/gotchas.md |
| No puedo poner un curso solo-asistentes con auto-inscripción abierta | SLDG7 | functional/slides/gotchas.md |
| Desaparecen invitaciones a cursos sin que nadie las borre | SLDG9 | functional/slides/gotchas.md |
| Al publicar un slide se recalcula el progreso de todos y se envían emails | SLDG10 | functional/slides/gotchas.md |
| Las categorías salen siempre publicadas y no cuentan en el progreso | SLDG11 | functional/slides/gotchas.md |
| Al re-inscribir a un alumno conserva su progreso y no vuelve a ganar karma | SLDG12 | functional/slides/gotchas.md |
| No me deja dejar más de una reseña en un curso o me cuesta karma | SLDG13 | functional/slides/gotchas.md |
| No puedo auto-inscribir miembros en un curso de invitación | SLDG14 | functional/slides/gotchas.md |
| Los intentos de certificación de un alumno re-inscrito salen mal contados | SLDG18 | functional/slides/integrations.md |
| Al cambiar la publicación del producto el curso de pago queda inconsistente | SLDG20 | functional/slides/integrations.md |
| Un curso de pago se añade al carrito aunque las reglas lo prohibirían | SLDG21 | functional/slides/integrations.md |
| Al cambiar el foro del curso quedan foros huérfanos | SLDG23 | functional/slides/integrations.md |
| No se crea la línea de CV del empleado al completar un curso | SLDG24 | functional/slides/integrations.md |
| v19: módulo roto por `_filter_add_members(target_partners,...)` | — | functional/slides/v19_changes.md |

---

### 18w. Recruitment / selección — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Publicar ofertas con nº de vacantes? | NATIVO | RECCAP1 | functional/recruitment/capabilities.md |
| ¿Portal de empleo web y candidatura online? | NATIVO (website_hr_recruitment) | RECCAP2 | functional/recruitment/capabilities.md |
| ¿Pipeline kanban con etapas? | NATIVO | RECCAP3 | functional/recruitment/capabilities.md |
| ¿Candidatura por email (alias)? | NATIVO | RECCAP4 | functional/recruitment/capabilities.md |
| ¿Formularios de entrevista? | NATIVO (hr_recruitment_survey) | RECCAP5 | functional/recruitment/capabilities.md |
| ¿Agendar entrevista con el candidato? | NATIVO (EE appointment) | RECCAP6 | functional/recruitment/capabilities.md |
| ¿OCR del CV? | NATIVO (EE extract) | RECCAP7 | functional/recruitment/capabilities.md |
| ¿Motivos de rechazo con email automático? | NATIVO | RECCAP8 | functional/recruitment/capabilities.md |
| ¿Crear el empleado al contratar? | NATIVO | RECCAP9 | functional/recruitment/capabilities.md |
| ¿Enviar oferta/contrato a firmar? | NATIVO (EE sign) | RECCAP10 | functional/recruitment/capabilities.md |
| ¿Programa de referidos con puntos? | NATIVO (EE hr_referral) | RECCAP11 | functional/recruitment/capabilities.md |
| ¿Banco de talento / reutilizar candidatos? | PARCIAL (talent pool real en v19) | RECCAP12 | functional/recruitment/capabilities.md |
| ¿Origen de la candidatura (UTM)? | NATIVO | RECCAP13 | functional/recruitment/capabilities.md |
| ¿Competencias/skills del candidato? | NATIVO (hr_recruitment_skills) | RECCAP14 | functional/recruitment/capabilities.md |

---

## 19septies. Recruitment — candidaturas/pipeline/contratación (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Edité el email del candidato y cambió en todas sus candidaturas | RECG1 | functional/recruitment/gotchas.md |
| El candidato vuelve a gris al moverlo de columna (kanban_state) | RECG2 | functional/recruitment/gotchas.md |
| Saqué a alguien de "Contratado" y se borró la fecha de contratación | RECG3 | functional/recruitment/gotchas.md |
| Desarchivé un rechazado y se "des-rechazó" volviendo al inicio | RECG6 | functional/recruitment/gotchas.md |
| No encuentro el CV en el applicant (cuelga del candidato) | RECG15 | functional/recruitment/integrations.md |
| El OCR no se valida hasta llegar a la etapa de contratado | RECG16 | functional/recruitment/integrations.md |
| El puesto no se lista en la web (website_published vs is_published) | RECG17 | functional/recruitment/integrations.md |
| Rechacé un candidato y rechazó también a otros candidatos parecidos | RECG8 | functional/recruitment/gotchas.md |
| Un candidato rechazado sigue como rechazado aunque lo archive | RECG4 | functional/recruitment/gotchas.md |
| Rechacé una candidatura, desapareció del pipeline pero los datos siguen | RECG5 | functional/recruitment/gotchas.md |
| Varios emails del mismo candidato se agrupan bajo una sola ficha | RECG7 | functional/recruitment/gotchas.md |
| Las respuestas por email del candidato van al alias del puesto | RECG9 | functional/recruitment/gotchas.md |
| Me avisa al quitar la marca de contratación de un stage con candidatos | RECG10 | functional/recruitment/gotchas.md |
| No encuentro la opción de publicar el puesto en la web | RECG11 | functional/recruitment/gotchas.md |
| No puedo editar la evaluación del candidato (es media de sus candidaturas) | RECG12 | functional/recruitment/gotchas.md |
| Al desarchivar o mover una candidatura no se envía el email de stage | RECG13 | functional/recruitment/gotchas.md |
| Al crear la candidatura se duplica el CV y a veces da error de compañía | RECG14 | functional/recruitment/gotchas.md |
| El formulario de entrevista no cambia por candidato y crea contactos duplicados | RECG18 | functional/recruitment/integrations.md |
| Las firmas de la oferta no se propagan al empleado al contratar | RECG19 | functional/recruitment/integrations.md |
| Activé Documents pero los CVs siguen yendo a adjuntos normales | RECG20 | functional/recruitment/integrations.md |
| Cambié el referrer y se recalcularon todos los puntos de referido | RECG21 | functional/recruitment/integrations.md |
| El candidato entrado por email aparece sin email o sin autor | RECG22 | functional/recruitment/integrations.md |
| El alias por fuente falla si la compañía no tiene dominio de alias | RECG23 | functional/recruitment/integrations.md |
| v19: módulo roto — `hr.candidate` / `candidate_id` eliminados | — | functional/recruitment/v19_changes.md |

---

### 18x. Landed Costs / costes en destino — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Repartir flete/aduana/seguro sobre el coste de productos? | NATIVO | LCCAP1 | cross-module/landed_costs/capabilities.md |
| ¿Métodos de reparto (igual/cantidad/peso/volumen/coste)? | NATIVO | LCCAP2 | cross-module/landed_costs/capabilities.md |
| ¿Crear el coste desde la factura del transportista? | NATIVO | LCCAP3 | cross-module/landed_costs/capabilities.md |
| ¿Aplicar a recepciones de compra? | NATIVO | LCCAP4 | cross-module/landed_costs/capabilities.md |
| ¿Aplicar a órdenes de fabricación? | NATIVO (mrp_landed_costs) | LCCAP5 | cross-module/landed_costs/capabilities.md |
| ¿Funciona con valoración manual o requiere FIFO/AVCO? | CONFIG | LCCAP6 | cross-module/landed_costs/capabilities.md |
| ¿Ajusta SVL y asiento contable? | NATIVO | LCCAP7 | cross-module/landed_costs/capabilities.md |
| ¿Revertir/cancelar un landed cost validado? | NO (LC negativo) | LCCAP8 | cross-module/landed_costs/capabilities.md |
| ¿Coste en destino en subcontratación? | NATIVO (módulo) | LCCAP9 | cross-module/landed_costs/capabilities.md |
| ¿Peso/volumen del producto para el reparto? | NATIVO | LCCAP10 | cross-module/landed_costs/capabilities.md |

---

### 18y. Maintenance — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Equipos con categorías? | NATIVO | MTCAP1 | functional/maintenance/capabilities.md |
| ¿Correctivo y preventivo? | NATIVO | MTCAP2 | functional/maintenance/capabilities.md |
| ¿Preventivo recurrente? | NATIVO (por copy, no cron) | MTCAP3 | functional/maintenance/capabilities.md |
| ¿Equipos de mantenimiento y kanban? | NATIVO | MTCAP4 | functional/maintenance/capabilities.md |
| ¿KPIs MTBF/MTTR? | NATIVO | MTCAP5 | functional/maintenance/capabilities.md |
| ¿Equipo ligado a centro MRP? | NATIVO (EE mrp_maintenance) | MTCAP6 | functional/maintenance/capabilities.md |
| ¿Solicitud desde fabricación? | NATIVO (EE) | MTCAP7 | functional/maintenance/capabilities.md |
| ¿Asignar equipo a empleado/dpto? | NATIVO (hr_maintenance) | MTCAP8 | functional/maintenance/capabilities.md |
| ¿Coste del mantenimiento? | PARCIAL (solo por equipo) | MTCAP9 | functional/maintenance/capabilities.md |
| ¿Subequipos / ubicación / serie? | PARCIAL (sin subequipos) | MTCAP10 | functional/maintenance/capabilities.md |

---

### 18z. Repair — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Orden de reparación con estados? | NATIVO | REPCAP1 | functional/repair/capabilities.md |
| ¿Piezas add/remove/recycle? | NATIVO | REPCAP2 | functional/repair/capabilities.md |
| ¿Reparación bajo garantía? | NATIVO | REPCAP3 | functional/repair/capabilities.md |
| ¿Facturar la reparación? | NATIVO (solo vía SO) | REPCAP4 | functional/repair/capabilities.md |
| ¿Reparación desde helpdesk? | NATIVO (EE) | REPCAP5 | functional/repair/capabilities.md |
| ¿Reparación desde devolución (RMA)? | NO | REPCAP6 | functional/repair/capabilities.md |
| ¿Controles de calidad? | NATIVO (EE quality_repair) | REPCAP7 | functional/repair/capabilities.md |
| ¿Consumir/producir contra stock? | NATIVO | REPCAP8 | functional/repair/capabilities.md |
| ¿Producto con nº de serie/lote? | NATIVO | REPCAP9 | functional/repair/capabilities.md |
| ¿Vincular a fabricación? | NATIVO (mrp_repair) | REPCAP10 | functional/repair/capabilities.md |

---

### 18aa. Fleet — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Vehículos con marca/modelo? | NATIVO | FLCAP1 | functional/fleet/capabilities.md |
| ¿Contratos con recordatorio de renovación? | NATIVO | FLCAP2 | functional/fleet/capabilities.md |
| ¿Servicios y odómetro? | NATIVO | FLCAP3 | functional/fleet/capabilities.md |
| ¿Conductor e historial? | NATIVO | FLCAP4 | functional/fleet/capabilities.md |
| ¿Costes en contabilidad? | NATIVO (account_fleet) | FLCAP5 | functional/fleet/capabilities.md |
| ¿Amortizar como activo? | NATIVO (EE account_asset_fleet) | FLCAP6 | functional/fleet/capabilities.md |
| ¿Recordatorios de vencimiento? | PARCIAL (contrato; ITV/seguro no dedicado) | FLCAP7 | functional/fleet/capabilities.md |
| ¿Combustible / consumo? | NO | FLCAP8 | functional/fleet/capabilities.md |
| ¿Documentos del vehículo? | NATIVO (EE documents_fleet) | FLCAP9 | functional/fleet/capabilities.md |
| ¿Vincular a empleado / valor en nómina? | VÍNCULO sí / NÓMINA no | FLCAP10 | functional/fleet/capabilities.md |

---

### 18ab. Account Assets — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Activos fijos con amortización? | NATIVO | ASCAP1 | enterprise/account_asset/capabilities.md |
| ¿Métodos lineal/degresivo/mixto? | NATIVO | ASCAP2 | enterprise/account_asset/capabilities.md |
| ¿Generar el activo desde cuenta/factura? | NATIVO | ASCAP3 | enterprise/account_asset/capabilities.md |
| ¿Tabla de asientos de amortización? | NATIVO | ASCAP4 | enterprise/account_asset/capabilities.md |
| ¿Venta/disposición con pérdida/ganancia? | NATIVO | ASCAP5 | enterprise/account_asset/capabilities.md |
| ¿Modificar/revalorizar el activo? | NATIVO | ASCAP6 | enterprise/account_asset/capabilities.md |
| ¿Prorrateo prorata temporis? | NATIVO | ASCAP7 | enterprise/account_asset/capabilities.md |
| ¿Duración en meses/años? | NATIVO | ASCAP8 | enterprise/account_asset/capabilities.md |
| ¿Ingresos/gastos diferidos con el mismo motor? | NATIVO | ASCAP9 | enterprise/account_asset/capabilities.md |
| ¿Pausar/reanudar la amortización? | NATIVO | ASCAP10 | enterprise/account_asset/capabilities.md |

---

### 18ac. Survey — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Tipos de pregunta variados? | NATIVO | SVCAP1 | functional/survey/capabilities.md |
| ¿Modos encuesta/quiz/evaluación/live? | NATIVO | SVCAP2 | functional/survey/capabilities.md |
| ¿Puntuación y nota de aprobado? | NATIVO | SVCAP3 | functional/survey/capabilities.md |
| ¿Paginación configurable? | NATIVO | SVCAP4 | functional/survey/capabilities.md |
| ¿Tiempo límite? | NATIVO | SVCAP5 | functional/survey/capabilities.md |
| ¿Aleatorizar preguntas? | NATIVO (por sección) | SVCAP6 | functional/survey/capabilities.md |
| ¿Público y por invitación con token? | NATIVO | SVCAP7 | functional/survey/capabilities.md |
| ¿Estadísticas agregadas? | NATIVO | SVCAP8 | functional/survey/capabilities.md |
| ¿Certificado descargable al aprobar? | NATIVO | SVCAP9 | functional/survey/capabilities.md |
| ¿Lógica condicional entre preguntas? | NATIVO | SVCAP10 | functional/survey/capabilities.md |
| ¿Reintentos limitados? | NATIVO | SVCAP11 | functional/survey/capabilities.md |
| ¿Preguntas obligatorias? | NATIVO | SVCAP12 | functional/survey/capabilities.md |

---

### 18ad. Follow-up / recobro — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Niveles de seguimiento configurables? | NATIVO | AFCAP1 | enterprise/account_followup/capabilities.md |
| ¿Recordatorio escalado (email/SMS/carta)? | NATIVO (parcial) | AFCAP2 | enterprise/account_followup/capabilities.md |
| ¿Cron de envío automático? | NATIVO | AFCAP3 | enterprise/account_followup/capabilities.md |
| ¿Estado de seguimiento por cliente? | NATIVO | AFCAP4 | enterprise/account_followup/capabilities.md |
| ¿Plantilla por nivel? | NATIVO | AFCAP5 | enterprise/account_followup/capabilities.md |
| ¿Carta PDF de recobro? | NATIVO | AFCAP6 | enterprise/account_followup/capabilities.md |
| ¿Recargos/intereses por demora? | NO | AFCAP7 | enterprise/account_followup/capabilities.md |
| ¿Responsable del recobro? | NATIVO | AFCAP8 | enterprise/account_followup/capabilities.md |
| ¿Excluir/posponer facturas? | CONFIG (parcial) | AFCAP9 | enterprise/account_followup/capabilities.md |
| ¿Informe de vencidos por cliente? | NATIVO | AFCAP10 | enterprise/account_followup/capabilities.md |

---

### 18ae. Account Budget — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Presupuestos por cuenta analítica? | NATIVO | ABCAP1 | enterprise/account_budget/capabilities.md |
| ¿Líneas por periodo? | NATIVO | ABCAP2 | enterprise/account_budget/capabilities.md |
| ¿Presupuestado vs real conseguido? | NATIVO | ABCAP3 | enterprise/account_budget/capabilities.md |
| ¿% de cumplimiento? | NATIVO | ABCAP4 | enterprise/account_budget/capabilities.md |
| ¿Presupuestar por varios planes/ejes? | NATIVO | ABCAP5 | enterprise/account_budget/capabilities.md |
| ¿Comprometido vs real? | NATIVO | ABCAP6 | enterprise/account_budget/capabilities.md |
| ¿Presupuesto de compras? | NATIVO (integrado) | ABCAP7 | enterprise/account_budget/capabilities.md |
| ¿Estados del presupuesto? | NATIVO | ABCAP8 | enterprise/account_budget/capabilities.md |
| ¿Generar líneas por mes/trimestre? | NATIVO | ABCAP9 | enterprise/account_budget/capabilities.md |
| ¿Bloqueo al superar el presupuesto? | NO (solo señalización) | ABCAP10 | enterprise/account_budget/capabilities.md |

---

### 18af. Intercompany — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Auto-crear PO en compañía B desde SO en A? | NATIVO | ICCAP1 | enterprise/intercompany/capabilities.md |
| ¿Factura/bill espejo entre compañías? | NATIVO | ICCAP2 | enterprise/intercompany/capabilities.md |
| ¿Movimiento de stock entre compañías? | NATIVO (sincroniza) | ICCAP3 | enterprise/intercompany/capabilities.md |
| ¿Reglas por compañía? | PARCIAL (rule_type eliminado) | ICCAP4 | enterprise/intercompany/capabilities.md |
| ¿Validación automática vs borrador? | NATIVO (document_state) | ICCAP5 | enterprise/intercompany/capabilities.md |
| ¿Qué documentos se sincronizan? | SO↔PO, factura↔bill | ICCAP6 | enterprise/intercompany/capabilities.md |
| ¿Almacén destino por compañía? | NATIVO | ICCAP7 | enterprise/intercompany/capabilities.md |
| ¿Multi-moneda PO→SO? | NO (bloqueado) | ICCAP8 | enterprise/intercompany/capabilities.md |
| ¿Trazabilidad del documento intercompañía? | NATIVO | ICCAP9 | enterprise/intercompany/capabilities.md |
| ¿Detección de compañía por el partner? | NATIVO | ICCAP10 | enterprise/intercompany/capabilities.md |

---

### 18ag. Web Subscriptions — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Vender suscripciones desde la web? | NATIVO | WSSCAP1 | enterprise/website_sale_subscription/capabilities.md |
| ¿Mostrar plan y precio recurrente? | NATIVO | WSSCAP2 | enterprise/website_sale_subscription/capabilities.md |
| ¿El checkout crea la suscripción? | NATIVO | WSSCAP3 | enterprise/website_sale_subscription/capabilities.md |
| ¿Pago recurrente con token desde web? | NATIVO | WSSCAP4 | enterprise/website_sale_subscription/capabilities.md |
| ¿Portal: ver/gestionar/cancelar? | NATIVO | WSSCAP5 | enterprise/website_sale_subscription/capabilities.md |
| ¿Combinar recurrente y normal en el pedido? | SÍ (no planes distintos) | WSSCAP6 | enterprise/website_sale_subscription/capabilities.md |
| ¿Periodo de prueba desde la web? | NO | WSSCAP7 | enterprise/website_sale_subscription/capabilities.md |
| ¿Renovación/upsell desde el portal? | PARCIAL | WSSCAP8 | enterprise/website_sale_subscription/capabilities.md |
| ¿Elegir variante de plan (mensual/anual)? | NATIVO | WSSCAP9 | enterprise/website_sale_subscription/capabilities.md |
| ¿Validar 1 plan por carrito? | NATIVO | WSSCAP10 | enterprise/website_sale_subscription/capabilities.md |

---

### 18ah. Amazon Connector — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Conector con Amazon (SP-API)? | NATIVO | AMZCAP1 | enterprise/sale_amazon/capabilities.md |
| ¿Importar pedidos como SO? | NATIVO | AMZCAP2 | enterprise/sale_amazon/capabilities.md |
| ¿FBA vs FBM con flujos distintos? | NATIVO | AMZCAP3 | enterprise/sale_amazon/capabilities.md |
| ¿Crear contacto desde el pedido? | NATIVO | AMZCAP4 | enterprise/sale_amazon/capabilities.md |
| ¿Importar comisiones/fees como gasto? | NO | AMZCAP5 | enterprise/sale_amazon/capabilities.md |
| ¿Multi-marketplace / países? | NATIVO | AMZCAP6 | enterprise/sale_amazon/capabilities.md |
| ¿Confirmar el envío hacia Amazon? | NATIVO | AMZCAP7 | enterprise/sale_amazon/capabilities.md |
| ¿Mapeo de productos por SKU? | NATIVO | AMZCAP8 | enterprise/sale_amazon/capabilities.md |
| ¿Cron de pedidos y cancelaciones? | NATIVO | AMZCAP9 | enterprise/sale_amazon/capabilities.md |
| ¿Publicar stock hacia Amazon (FBM)? | NATIVO | AMZCAP10 | enterprise/sale_amazon/capabilities.md |

---

### 18ai. Bank Statement Import — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Importar extractos QIF/OFX/CSV/CAMT? | NATIVO | BSICAP1 | enterprise/bank_statement_import/capabilities.md |
| ¿Mapeo de columnas para CSV? | CONFIG | BSICAP2 | enterprise/bank_statement_import/capabilities.md |
| ¿Crear statement + líneas automáticamente? | NATIVO | BSICAP3 | enterprise/bank_statement_import/capabilities.md |
| ¿Detección de duplicados? | NATIVO | BSICAP4 | enterprise/bank_statement_import/capabilities.md |
| ¿Detectar el diario por la cuenta del fichero? | NATIVO | BSICAP5 | enterprise/bank_statement_import/capabilities.md |
| ¿Multi-divisa en las líneas? | CONFIG (solo CAMT) | BSICAP6 | enterprise/bank_statement_import/capabilities.md |
| ¿Importar saldo inicial/final? | NATIVO | BSICAP7 | enterprise/bank_statement_import/capabilities.md |
| ¿OCR de extracto PDF/imagen? | NO (módulo aparte) | BSICAP8 | enterprise/bank_statement_import/capabilities.md |
| ¿Sincronización bancaria automática? | NO (módulo aparte) | BSICAP9 | enterprise/bank_statement_import/capabilities.md |
| ¿Conciliación tras importar? | NATIVO | BSICAP10 | enterprise/bank_statement_import/capabilities.md |

---

### 18aj. 3-Way Match — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Conciliación a 3 bandas PO↔recepción↔factura? | NATIVO | A3MCAP1 | enterprise/account_3way_match/capabilities.md |
| ¿Estado "Should Be Paid"? | NATIVO | A3MCAP2 | enterprise/account_3way_match/capabilities.md |
| ¿Override manual del estado? | NATIVO | A3MCAP3 | enterprise/account_3way_match/capabilities.md |
| ¿Considera PO y recepción? | NATIVO | A3MCAP4 | enterprise/account_3way_match/capabilities.md |
| ¿Aviso visual en la factura? | NATIVO (parcial) | A3MCAP5 | enterprise/account_3way_match/capabilities.md |
| ¿Requiere purchase? | NATIVO | A3MCAP6 | enterprise/account_3way_match/capabilities.md |
| ¿Bloquea el pago o solo informa? | NO (solo informa) | A3MCAP7 | enterprise/account_3way_match/capabilities.md |
| ¿Recepción parcial? | NATIVO | A3MCAP8 | enterprise/account_3way_match/capabilities.md |
| ¿Se recalcula al validar recepción/factura? | NATIVO | A3MCAP9 | enterprise/account_3way_match/capabilities.md |
| ¿Configurable por compañía o por factura? | POR FACTURA | A3MCAP10 | enterprise/account_3way_match/capabilities.md |

---

### 18ak. Intrastat — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Declaración Intrastat? | NATIVO | INTRCAP1 | enterprise/intrastat/capabilities.md |
| ¿Códigos de mercancía? | NATIVO | INTRCAP2 | enterprise/intrastat/capabilities.md |
| ¿Tipo de transacción / naturaleza? | NATIVO | INTRCAP3 | enterprise/intrastat/capabilities.md |
| ¿País de origen, peso, valor? | NATIVO | INTRCAP4 | enterprise/intrastat/capabilities.md |
| ¿Aplica a ventas y compras? | NATIVO | INTRCAP5 | enterprise/intrastat/capabilities.md |
| ¿Datos desde movimientos de stock? | NATIVO | INTRCAP6 | enterprise/intrastat/capabilities.md |
| ¿Región/provincia? | NATIVO | INTRCAP7 | enterprise/intrastat/capabilities.md |
| ¿Modo de transporte? | NATIVO | INTRCAP8 | enterprise/intrastat/capabilities.md |
| ¿Exportar el fichero para la autoridad? | PARCIAL (oficial por l10n) | INTRCAP9 | enterprise/intrastat/capabilities.md |
| ¿Incoterm en la declaración? | NATIVO | INTRCAP10 | enterprise/intrastat/capabilities.md |

---

### 18al. ISO20022 / SEPA — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Generar fichero ISO20022/SEPA (pain.001)? | NATIVO | ISOCAP1 | enterprise/account_iso20022/capabilities.md |
| ¿Lote de pagos? | NATIVO | ISOCAP2 | enterprise/account_iso20022/capabilities.md |
| ¿SEPA Credit Transfer como método? | NATIVO | ISOCAP3 | enterprise/account_iso20022/capabilities.md |
| ¿Varias versiones de pain.001? | NATIVO | ISOCAP4 | enterprise/account_iso20022/capabilities.md |
| ¿Validación IBAN/BIC? | NATIVO | ISOCAP5 | enterprise/account_iso20022/capabilities.md |
| ¿Descargar el XML? | NATIVO | ISOCAP6 | enterprise/account_iso20022/capabilities.md |
| ¿Por diario bancario de la compañía? | NATIVO | ISOCAP7 | enterprise/account_iso20022/capabilities.md |
| ¿Multi-divisa en el fichero? | PARCIAL (sepa_ct fuerza EUR) | ISOCAP8 | enterprise/account_iso20022/capabilities.md |
| ¿Domiciliación SEPA (SDD)? | NO (módulo aparte) | ISOCAP9 | enterprise/account_iso20022/capabilities.md |
| ¿Propósito / referencia estructurada? | NATIVO | ISOCAP10 | enterprise/account_iso20022/capabilities.md |

---

### 18am. Frontdesk — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Estaciones de recepción configurables? | NATIVO | FDCAP1 | enterprise/frontdesk/capabilities.md |
| ¿Registro de visitantes con estados? | NATIVO | FDCAP2 | enterprise/frontdesk/capabilities.md |
| ¿Quiosco web público para check-in? | NATIVO | FDCAP3 | enterprise/frontdesk/capabilities.md |
| ¿Notificar al anfitrión al llegar? | NATIVO | FDCAP4 | enterprise/frontdesk/capabilities.md |
| ¿Campos del visitante configurables? | NATIVO | FDCAP5 | enterprise/frontdesk/capabilities.md |
| ¿Anfitrión (empleados a visitar)? | NATIVO | FDCAP6 | enterprise/frontdesk/capabilities.md |
| ¿Ofrecer bebida? | NATIVO | FDCAP7 | enterprise/frontdesk/capabilities.md |
| ¿Personalización visual del quiosco? | NATIVO | FDCAP8 | enterprise/frontdesk/capabilities.md |
| ¿Check-out / registro de salida? | NATIVO | FDCAP9 | enterprise/frontdesk/capabilities.md |
| ¿Planificar visitas con antelación? | NATIVO | FDCAP10 | enterprise/frontdesk/capabilities.md |

---

### 18an. Calendar — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Recordatorios por email y notificación popup? | NATIVO | CALCAP1 | functional/calendar/capabilities.md |
| ¿Recurrencia diaria/semanal/mensual/anual (count/fecha/forever)? | NATIVO | CALCAP2 | functional/calendar/capabilities.md |
| ¿Mensual "2º lunes" vs "día 15"? | NATIVO | CALCAP3 | functional/calendar/capabilities.md |
| ¿Exportar invitación .ics? | NATIVO (vobject) | CALCAP4 | functional/calendar/capabilities.md |
| ¿Videollamada integrada (Discuss)? | NATIVO | CALCAP5 | functional/calendar/capabilities.md |
| ¿Eventos privados que otros no ven? | NATIVO | CALCAP6 | functional/calendar/capabilities.md |
| ¿Aceptar/Rechazar/Tentativo en el chatter? | NATIVO | CALCAP7 | functional/calendar/capabilities.md |
| ¿Aplicar la respuesta a toda la serie / futuros? | NATIVO | CALCAP8 | functional/calendar/capabilities.md |
| ¿Bloquear todos los emails del calendario? | CONFIG (calendar.block_mail) | CALCAP9 | functional/calendar/capabilities.md |
| ¿Recordatorio SMS nativo en CE? | NO (es EE: calendar_sms) | CALCAP10 | functional/calendar/capabilities.md |

---

### 18ao. Payroll — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Reglas salariales personalizadas en Python? | NATIVO | PYCAP1 | enterprise/hr_payroll/capabilities.md |
| ¿Generar el asiento contable de la nómina? | NATIVO (requiere hr_payroll_account) | PYCAP2 | enterprise/hr_payroll/capabilities.md |
| ¿Registrar el pago al banco del empleado? | CONFIG | PYCAP3 | enterprise/hr_payroll/capabilities.md |
| ¿Distintas frecuencias de pago? | NATIVO | PYCAP4 | enterprise/hr_payroll/capabilities.md |
| ¿Nóminas en lote? | NATIVO | PYCAP5 | enterprise/hr_payroll/capabilities.md |
| ¿Nómina rectificativa / refund? | NATIVO | PYCAP6 | enterprise/hr_payroll/capabilities.md |
| ¿Embargos/deducciones recurrentes (salary attachments)? | NATIVO | PYCAP7 | enterprise/hr_payroll/capabilities.md |
| ¿Acumulado anual (Year-To-Date)? | CONFIG (ytd_computation) | PYCAP8 | enterprise/hr_payroll/capabilities.md |
| ¿Tablas paramétricas versionadas por fecha (IRPF)? | NATIVO | PYCAP9 | enterprise/hr_payroll/capabilities.md |
| ¿Trae las reglas salariales españolas listas? | NO (las aporta l10n_*_hr_payroll) | PYCAP10 | enterprise/hr_payroll/capabilities.md |

---

### 18ap. Appraisal — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Crear evaluaciones automáticamente por antigüedad? | NATIVO (cron + plan) | APRCAP1 | enterprise/hr_appraisal/capabilities.md |
| ¿Ocultar el feedback del manager hasta publicarlo? | NATIVO | APRCAP2 | enterprise/hr_appraisal/capabilities.md |
| ¿Un empleado puede pedir su propia evaluación? | NATIVO | APRCAP3 | enterprise/hr_appraisal/capabilities.md |
| ¿Vincular reuniones de calendario a la evaluación? | NATIVO | APRCAP4 | enterprise/hr_appraisal/capabilities.md |
| ¿Calcular la fecha de entrevista final desde las reuniones? | NATIVO | APRCAP5 | enterprise/hr_appraisal/capabilities.md |
| ¿Rating final no visible al empleado? | NATIVO | APRCAP6 | enterprise/hr_appraisal/capabilities.md |
| ¿Propiedades dinámicas por departamento? | CONFIG | APRCAP7 | enterprise/hr_appraisal/capabilities.md |
| ¿Objetivos con progreso, deadline y tags? | NATIVO | APRCAP8 | enterprise/hr_appraisal/capabilities.md |
| ¿Avisar de evaluaciones duplicadas? | NATIVO | APRCAP9 | enterprise/hr_appraisal/capabilities.md |
| ¿Notificar a varios managers a la vez? | NATIVO | APRCAP10 | enterprise/hr_appraisal/capabilities.md |

---

### 18aq. Live Chat — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Enrutar al operador por idioma del visitante? | NATIVO | LIVCAP1 | functional/im_livechat/capabilities.md |
| ¿Reconectar al visitante con su operador anterior? | NATIVO | LIVCAP2 | functional/im_livechat/capabilities.md |
| ¿Equilibrar carga entre operadores? | NATIVO | LIVCAP3 | functional/im_livechat/capabilities.md |
| ¿Alias público del operador distinto de su nombre? | CONFIG | LIVCAP4 | functional/im_livechat/capabilities.md |
| ¿El chatbot crea un partner desde el email recogido? | NATIVO | LIVCAP5 | functional/im_livechat/capabilities.md |
| ¿Enviar la transcripción del chat por email? | NATIVO | LIVCAP6 | functional/im_livechat/capabilities.md |
| ¿Enviar al visitante el historial de páginas? | NATIVO | LIVCAP7 | functional/im_livechat/capabilities.md |
| ¿Reiniciar el chatbot en la misma sesión? | NATIVO | LIVCAP8 | functional/im_livechat/capabilities.md |
| ¿Medir satisfacción y duración de las sesiones? | NATIVO | LIVCAP9 | functional/im_livechat/capabilities.md |
| ¿Integrar el widget en webs externas? | NATIVO | LIVCAP10 | functional/im_livechat/capabilities.md |

---

### 18ar. PLM — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Crear una ECO desde una Orden de Fabricación? | NATIVO | PLMCAP1 | enterprise/mrp_plm/capabilities.md |
| ¿Generar una BoM nueva desde los cambios reales de una MO? | NATIVO | PLMCAP2 | enterprise/mrp_plm/capabilities.md |
| ¿Crear ECOs por email? | NATIVO (alias) | PLMCAP3 | enterprise/mrp_plm/capabilities.md |
| ¿Aprobar/rechazar una ECO? | NATIVO | PLMCAP4 | enterprise/mrp_plm/capabilities.md |
| ¿Comparar dos revisiones de BoM (diff de líneas/byproducts)? | NATIVO | PLMCAP5 | enterprise/mrp_plm/capabilities.md |
| ¿Ver el diff de cambios de routing? | NATIVO | PLMCAP6 | enterprise/mrp_plm/capabilities.md |
| ¿Aplicar los cambios sin incrementar la versión? | CONFIG (will_update_version) | PLMCAP7 | enterprise/mrp_plm/capabilities.md |
| ¿Adjuntar documentos a la ECO que se propaguen al producto? | NATIVO | PLMCAP8 | enterprise/mrp_plm/capabilities.md |
| ¿Mover la ECO a una etapa final tras aplicar? | NATIVO | PLMCAP9 | enterprise/mrp_plm/capabilities.md |
| ¿Ver desde un producto cuántas ECOs tiene? | NATIVO | PLMCAP10 | enterprise/mrp_plm/capabilities.md |

---

### 18as. Forum — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Limitar a una respuesta por pregunta? | CONFIG (flag; enforcement frontend) | FORCAP1 | functional/forum/capabilities.md |
| ¿Validación/moderación de posts nuevos? | NATIVO | FORCAP2 | functional/forum/capabilities.md |
| ¿Notificar a moderadores de posts pendientes? | NATIVO | FORCAP3 | functional/forum/capabilities.md |
| ¿Convertir comentario↔respuesta? | NATIVO | FORCAP4 | functional/forum/capabilities.md |
| ¿Posts relacionados por similitud de tags? | NATIVO | FORCAP5 | functional/forum/capabilities.md |
| ¿Microdata SEO (schema.org QAPage)? | NATIVO | FORCAP6 | functional/forum/capabilities.md |
| ¿Foros privados por grupo? | NATIVO/CONFIG | FORCAP7 | functional/forum/capabilities.md |
| ¿Flag de posts ofensivos por usuarios? | NATIVO | FORCAP8 | functional/forum/capabilities.md |

---

### 18at. Blog — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Filtrado público de posts por fecha de publicación? | NATIVO | BLGCAP1 | functional/blog/capabilities.md |
| ¿Búsqueda full-text integrada? | NATIVO | BLGCAP2 | functional/blog/capabilities.md |
| ¿Agregar tags por blog con frecuencia mínima? | NATIVO | BLGCAP3 | functional/blog/capabilities.md |
| ¿Acceso "smart" para usuarios externos? | NATIVO | BLGCAP4 | functional/blog/capabilities.md |
| ¿Notificar a followers al publicar? | NATIVO | BLGCAP5 | functional/blog/capabilities.md |
| ¿Teaser manual override? | NATIVO | BLGCAP6 | functional/blog/capabilities.md |
| ¿Cover con resize on-the-fly? | NATIVO | BLGCAP7 | functional/blog/capabilities.md |
| ¿Comentarios públicos? | NATIVO | BLGCAP8 | functional/blog/capabilities.md |

---

### 18au. Membership — ¿Odoo puede…? (capacidades, eje consultor)

| Pregunta del cliente | Veredicto | ID | File |
|----------------------|-----------|----|------|
| ¿Vender membresías como productos de factura? | NATIVO | MEMCAP1 | functional/membership/capabilities.md |
| ¿Facturar miembros en lote? | NATIVO | MEMCAP2 | functional/membership/capabilities.md |
| ¿Membresías gratuitas? | NATIVO | MEMCAP3 | functional/membership/capabilities.md |
| ¿Membresías asociadas/familiares? | NATIVO | MEMCAP4 | functional/membership/capabilities.md |
| ¿Seguimiento automático del ciclo de vida? | NATIVO | MEMCAP5 | functional/membership/capabilities.md |
| ¿Informe de análisis de membresías? | NATIVO | MEMCAP6 | functional/membership/capabilities.md |
| ¿Precio negociado por socio? | NATIVO | MEMCAP7 | functional/membership/capabilities.md |
| ¿Directorio público de socios? | CONFIG (website_membership) | MEMCAP8 | functional/membership/capabilities.md |
| ⚠️ ¿Existe en Odoo 19? | NO — módulo eliminado en v19 | — | functional/membership/v19_changes.md |

---

## Forum (`website_forum`) — síntomas

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| No puedo crear una pregunta: salta AccessError por falta de karma | FORG1 | functional/forum/gotchas.md |
| Mi pregunta no aparece publicada, se queda pendiente de validación | FORG2 | functional/forum/gotchas.md |
| No me dan el karma de preguntar hasta que validan el post | FORG2 | functional/forum/gotchas.md |
| No puedo responder a una pregunta porque está cerrada o borrada (UserError) | FORG3 | functional/forum/gotchas.md |
| No me deja votar mi propio post | FORG4 | functional/forum/gotchas.md |
| No puedo cambiar el voto de otro usuario | FORG5 | functional/forum/gotchas.md |
| No puedo hacer downvote (o upvote): me falta karma para votar | FORG6 | functional/forum/gotchas.md |
| Al quitar la aceptación de una respuesta se resta karma al autor | FORG7 | functional/forum/gotchas.md |
| Al borrar una respuesta aceptada pierdo karma yo y el autor | FORG7 | functional/forum/gotchas.md |
| Cerrar un post como spam/ofensivo me hunde el karma (×10 si era mi primer post) | FORG8 | functional/forum/gotchas.md |
| Reabrir un post ofensivo me devuelve el karma que perdí | FORG9 | functional/forum/gotchas.md |
| No puedo marcar un post como ofensivo: necesito karma de moderación | FORG10 | functional/forum/gotchas.md |
| No puedo editar el post de otro usuario por falta de karma | FORG11 | functional/forum/gotchas.md |
| Al guardar el post salta AccessError porque metí una imagen o enlace sin karma | FORG12 | functional/forum/gotchas.md |
| Mis enlaces salen con rel="nofollow" | FORG12 | functional/forum/gotchas.md |
| No puedo comentar un post por falta de karma | FORG13 | functional/forum/gotchas.md |
| No puedo crear un tag nuevo: AccessError por falta de karma | FORG14 | functional/forum/gotchas.md |
| Al editar un post se notifica a todos los seguidores | FORG16 | functional/forum/integrations.md |
| No me aparece la biografía del usuario en el foro | FORG18 | functional/forum/integrations.md |
| Los seguidores de un tag reciben notificación de cada pregunta nueva | FORG19 | functional/forum/integrations.md |

## Blog (`website_blog`) — síntomas

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Al republicar un post conserva la fecha de la primera publicación, no la actual | BLGG1 | functional/blog/gotchas.md |
| La fecha visible del post (post_date) no coincide con published_date | BLGG2 | functional/blog/gotchas.md |
| Un post con fecha futura no aparece en la web | BLGG2 | functional/blog/gotchas.md |
| Las visitas del blog se cuentan de más al borrar la cookie / no pasan por el ORM | BLGG3 | functional/blog/gotchas.md |
| Al archivar un blog se archivan todos sus posts | BLGG4 | functional/blog/gotchas.md |
| Al archivar un post se despublica solo | BLGG5 | functional/blog/gotchas.md |
| Cambiar el color de un tag toca el write_date de todos sus posts | BLGG6 | functional/blog/gotchas.md |
| El teaser sale truncado a 200 caracteres y pierde imágenes/listas | BLGG7 | functional/blog/gotchas.md |
| Al duplicar un post nace despublicado y con 0 visitas | BLGG8 | functional/blog/gotchas.md |
| La imagen og:image al compartir sale corrupta | BLGG9 | functional/blog/gotchas.md |
| Un post publicado aparece como no publicado en este website (pertenece a otro) | BLGG10 | functional/blog/gotchas.md |
| La meta description SEO sale vacía aunque el post tiene contenido | BLGG11 | functional/blog/gotchas.md |
| Las respuestas al email de publicación entran como notas internas, no notifican | BLGG13 | functional/blog/integrations.md |

## Membership (`membership`) — síntomas

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| No puedo fijar a mano el estado de membresía (membership_state) | MEMG1 | functional/membership/gotchas.md |
| El socio sale como "invoiced" y no como "paid" aunque la factura está validada | MEMG2 | functional/membership/gotchas.md |
| Una membresía caducada sin pagar sale como "none" en vez de "old" | MEMG3 | functional/membership/gotchas.md |
| Marco al socio como free pero sigue saliendo "paid" | MEMG4 | functional/membership/gotchas.md |
| No puedo facturar una membresía a un socio marcado como free (UserError) | MEMG5 | functional/membership/gotchas.md |
| Un socio asociado ignora su propio estado y hereda el del padre | MEMG6 | functional/membership/gotchas.md |
| Salta ValidationError por ciclo de socios asociados | MEMG7 | functional/membership/gotchas.md |
| Una membresía caducada no pasa a "old" hasta que algo la recalcula (el cron no la toca) | MEMG8 | functional/membership/gotchas.md |
| Cancelé una membresía editando la línea y no cambió de estado (hay que revertir la factura) | MEMG10 | functional/membership/gotchas.md |
| Al editar una línea de factura se crea una membresía duplicada inesperada | MEMG11 | functional/membership/gotchas.md |
| Con varias líneas solapadas el estado depende del orden, no de la "mejor" línea | MEMG12 | functional/membership/gotchas.md |
| El date_from de la membresía se recorta a la fecha de factura | MEMG13 | functional/membership/gotchas.md |
| Al borrar la línea de factura desaparece el histórico de membresía | MEMG14 | functional/membership/gotchas.md |
| Vender una membresía por presupuesto no crea la línea hasta facturar | MEMG18 | functional/membership/integrations.md |

---

## 19octies. Landed Costs — reparto/valoración (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| No puedo cancelar un landed cost ya validado | LCG1 | cross-module/landed_costs/gotchas.md |
| El landed cost no afecta a productos de coste estándar | LCG3 | cross-module/landed_costs/gotchas.md |
| Validé el LC pero no generó asiento (valoración manual) | LCG4 | cross-module/landed_costs/gotchas.md |
| El coste del producto no subió todo lo esperado (stock ya vendido) | LCG5 | cross-module/landed_costs/gotchas.md |
| "equal" reparte raro (por nº de líneas, no por importe) | LCG13 | cross-module/landed_costs/gotchas.md |
| El botón "Create Landed Costs" no aparece en la factura | LCG15 | cross-module/landed_costs/integrations.md |
| En fabricación el coste va al producto acabado, no a componentes | LCG16 | cross-module/landed_costs/integrations.md |
| No puedo validar el landed cost, no aparece como pendiente | LCG2 | cross-module/landed_costs/gotchas.md |
| La mercancía ya vendida no sube de coste al añadir un landed cost | LCG6 | cross-module/landed_costs/gotchas.md |
| No me deja validar el landed cost porque las líneas no cuadran | LCG7 | cross-module/landed_costs/gotchas.md |
| Perdí los ajustes manuales del landed cost al recomputar | LCG8 | cross-module/landed_costs/gotchas.md |
| El landed cost por lote reparte el coste lote a lote | LCG9 | cross-module/landed_costs/gotchas.md |
| El landed cost no cambia el precio de la recepción si viene de factura | LCG10 | cross-module/landed_costs/gotchas.md |
| El landed cost no reconcilia el stock_input con la factura | LCG11 | cross-module/landed_costs/gotchas.md |
| No me deja quitar landed_cost_ok ni cambiar el tipo del producto | LCG12 | cross-module/landed_costs/gotchas.md |
| En dropshipping el landed cost va a gasto en vez de a valoración | LCG14 | cross-module/landed_costs/gotchas.md |
| La reconciliación factura-LC no salta sin anglosajón o con factura en borrador | LCG17 | cross-module/landed_costs/integrations.md |
| El landed cost creado desde factura no duplica el coste de la recepción | LCG18 | cross-module/landed_costs/integrations.md |
| La subcontratación reparte el LC por el picking, no por la orden de fabricación | LCG19 | cross-module/landed_costs/integrations.md |
| No me deja desactivar/cambiar un producto de landed cost ya usado | LCG20 | cross-module/landed_costs/integrations.md |
| v19: módulo roto — SVL propias / reconcile_landed_cost eliminados | — | cross-module/landed_costs/v19_changes.md |

---

## 19ter. Loyalty — promos/cupones/puntos (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Reconfiguré el tipo de programa y perdí sus reglas/recompensas | LOYG1 | cross-module/loyalty/gotchas.md |
| Una tarjeta de fidelización no me deja poner caducidad | LOYG6 | cross-module/loyalty/gotchas.md |
| Duplico un pedido y las promociones no se copian | LOYG15 | cross-module/loyalty/integrations.md |
| Cancelo un pedido y el cupón de "siguiente pedido" no se regenera | LOYG18 | cross-module/loyalty/integrations.md |
| El cupón generado no se puede canjear en el mismo pedido | LOYG19 | cross-module/loyalty/integrations.md |
| El envío cuenta para el mínimo del programa / recibe descuento | LOYG20 | cross-module/loyalty/integrations.md |
| El código del cupón choca con el de un programa (unicidad) | LOYG5 | cross-module/loyalty/gotchas.md |
| El descuento se recalcula con impuestos price_include según el cliente | LOYG2 | cross-module/loyalty/gotchas.md |
| Me da error reward_point_split en programa loyalty o ewallet | LOYG4 | cross-module/loyalty/gotchas.md |
| Se crea un email automático al crear una tarjeta con partner | LOYG7 | cross-module/loyalty/gotchas.md |
| Al subir puntos solo llega el email del hito más alto | LOYG8 | cross-module/loyalty/gotchas.md |
| Aparece un producto de descuento oculto por cada recompensa | LOYG9 | cross-module/loyalty/gotchas.md |
| Al archivar el programa se archivan también reglas y recompensas | LOYG10 | cross-module/loyalty/gotchas.md |
| No me deja borrar un programa de fidelización activo | LOYG11 | cross-module/loyalty/gotchas.md |
| Los trigger_product_ids desaparecen al guardar el programa | LOYG12 | cross-module/loyalty/gotchas.md |
| El cálculo de productos descontables va lentísimo en catálogos grandes | LOYG13 | cross-module/loyalty/gotchas.md |
| Las reglas de gift card/ewallet se crean sin productos | LOYG14 | cross-module/loyalty/gotchas.md |
| Me da error al añadir una tarifa de otra divisa al programa | LOYG25 | cross-module/loyalty/gotchas.md |
| La divisa del programa cambió sola al cambiar la compañía | LOYG26 | cross-module/loyalty/gotchas.md |
| Con per_point gasto menos puntos de los nominales | LOYG29 | cross-module/loyalty/gotchas.md |
| El descuento "cheapest" no cuadra con impuestos no incluidos | LOYG30 | cross-module/loyalty/gotchas.md |
| No puedo usar un producto combo como recompensa | LOYG31 | cross-module/loyalty/gotchas.md |
| El importe canjeado de la gift card cambia según el cliente | LOYG32 | cross-module/loyalty/gotchas.md |
| Un cupón caducado entre el pago y la confirmación sigue válido | LOYG33 | cross-module/loyalty/gotchas.md |
| Odoo no aplica el programa con mejor descuento sino por secuencia | LOYG34 | cross-module/loyalty/gotchas.md |
| Al recrear recompensas sale reward_type discount por defecto | LOYG35 | cross-module/loyalty/gotchas.md |
| Gift/ewallet descuenta también sobre impuestos fijos y envío | LOYG36 | cross-module/loyalty/gotchas.md |
| Un cupón future nominativo da error "only on future orders" al canjear | LOYG37 | cross-module/loyalty/gotchas.md |
| No puedo borrar una recompensa o tarjeta referenciada por una línea | LOYG16 | cross-module/loyalty/integrations.md |
| Los puntos del cupón solo se mueven cuando el pedido pasa a sale | LOYG21 | cross-module/loyalty/integrations.md |
| Los programas nominativos no se cargan para usuarios anónimos en web | LOYG22 | cross-module/loyalty/integrations.md |
| Pongo qty 0 en una recompensa web y no se vuelve a aplicar sola | LOYG23 | cross-module/loyalty/integrations.md |
| Solo se aplica una recompensa de envío gratis a la vez | LOYG24 | cross-module/loyalty/integrations.md |
| v19: módulo roto por `loyalty.reward.tax_ids` / `toggle_active` eliminados | — | cross-module/loyalty/v19_changes.md |

---

## 19bis. Payment — flujo de cobro (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| El pago online se confirma pero el pedido no, hasta que pasa el cron | PAYG15 | cross-module/payment/integrations.md |
| Pago correcto pero el pedido no se confirma (varias SO en una transacción) | PAYG17 | cross-module/payment/integrations.md |
| El payment queda sin conciliar con la factura | PAYG18 | cross-module/payment/integrations.md |
| Validación de tarjeta no genera asiento contable | PAYG19 | cross-module/payment/integrations.md |
| Al reconfigurar un proveedor los clientes pierden sus tarjetas guardadas | PAYG3 | cross-module/payment/gotchas.md |
| "not possible to reserve/capture" — estado authorized sin soporte de captura manual | PAYG9 | cross-module/payment/gotchas.md |
| El reembolso no aparece en la transacción original | PAYG23 | cross-module/payment/integrations.md |
| Ramificar lógica por tipo de operación con tx en draft/pending da resultados erróneos | PAYG1 | cross-module/payment/gotchas.md |
| Cambio el estado de una transacción a uno ilegal y Odoo lo ignora sin error | PAYG2 | cross-module/payment/gotchas.md |
| No consigo desarchivar una tarjeta guardada (método inactivo o provider off) | PAYG4 | cross-module/payment/gotchas.md |
| El límite de importe máximo del provider se compara mal en otra moneda | PAYG5 | cross-module/payment/gotchas.md |
| El cron de post-procesado de pagos no se ejecuta | PAYG6 | cross-module/payment/gotchas.md |
| Una transacción PayPal de más de 4 días nunca se vuelve a procesar | PAYG7 | cross-module/payment/gotchas.md |
| El importe de la transacción cambia (decimales raros) tras crearla | PAYG8 | cross-module/payment/gotchas.md |
| No puedo capturar/anular/reembolsar con el provider deshabilitado | PAYG10 | cross-module/payment/gotchas.md |
| La transacción padre se completa sola cuando las hijas cubren el importe | PAYG11 | cross-module/payment/gotchas.md |
| Ningún envío de petición al provider funciona si está deshabilitado | PAYG12 | cross-module/payment/gotchas.md |
| No me deja borrar un provider ni cambiarle la compañía (tiene transacciones) | PAYG13 | cross-module/payment/gotchas.md |
| No puedo habilitar un método porque ningún provider habilitado lo soporta | PAYG14 | cross-module/payment/gotchas.md |
| Llega un webhook de Stripe sin secreto y se procesa sin verificar la firma | PAYG25 | cross-module/payment/gotchas.md |
| Un reenvío tardío de webhook de Stripe se descarta aunque la firma sea válida | PAYG26 | cross-module/payment/gotchas.md |
| Un reembolso pasa de done a error sin fallar (al revés que el resto) | PAYG27 | cross-module/payment/gotchas.md |
| El account.payment del reembolso tarda hasta que corre el cron | PAYG28 | cross-module/payment/gotchas.md |
| Una transacción esperando 3DS/SCA aparece en draft y no en pending | PAYG29 | cross-module/payment/gotchas.md |
| El importe disponible para reembolsar no cuadra con las tx de refund | PAYG30 | cross-module/payment/gotchas.md |
| No aparecen tarjetas guardadas para cobrar offline con captura manual | PAYG31 | cross-module/payment/gotchas.md |
| Un pago por tarjeta guardada que no se confirma acaba cancelado | PAYG32 | cross-module/payment/gotchas.md |
| El account.payment cuelga de la matriz aunque pagó un contacto hijo | PAYG33 | cross-module/payment/gotchas.md |
| El descuento por pronto pago no se aplica si el importe no casa exacto | PAYG35 | cross-module/payment/gotchas.md |
| La URL de retorno se rompe (doble query string) si ya traía parámetros | PAYG36 | cross-module/payment/gotchas.md |
| Un reembolso de Stripe falla al emparejar por referencia duplicada o vacía | PAYG37 | cross-module/payment/gotchas.md |
| Una tx que pasa de pending a done se vuelve a post-procesar entera | PAYG16 | cross-module/payment/integrations.md |
| Un importe negativo genera un pago de salida en vez de entrada | PAYG20 | cross-module/payment/integrations.md |
| La factura no se genera tras el cobro o el email tarda en salir | PAYG21 | cross-module/payment/integrations.md |
| El autor del chatter cambia según si el pago viene de backend o web | PAYG22 | cross-module/payment/integrations.md |
| Al anular tras captura parcial solo se libera el importe restante | PAYG24 | cross-module/payment/integrations.md |
| v19: módulo de acquirer roto (`_handle_notification_data` ya no existe) | — | cross-module/payment/v19_changes.md |

---

## 19. Timesheet — bridges (hr/project/analytic/sale/expense/grid/helpdesk/fsm)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| "Timesheets must be created with an active employee in the selected companies" | timesheet | TG27HR | enterprise/timesheet/integrations.md |
| "Only a Timesheets Approver or Manager is allowed to modify a validated entry" en FSM close | timesheet | TG69FSM / TG55GR | enterprise/timesheet/integrations.md |
| "Cannot modify timesheets that are already invoiced" tras cancelar invoice | timesheet | TG43SL / TG42SL | enterprise/timesheet/integrations.md |
| "X analytic plan(s) required on the project Y" | timesheet | TG38AA / TG85AD | enterprise/timesheet/integrations.md |
| Timesheets de leave validados borrados sin warning al refuse leave | timesheet | TG75HOL | enterprise/timesheet/integrations.md |
| `timesheet_invoice_type='non_billable'` en helpdesk ticket billable (sin bridge sale) | timesheet | TG66HD | enterprise/timesheet/integrations.md |
| Cross-link 7-step task_rate vs employee_rate ya en FSM | timesheet | FSMG21-30 | enterprise/fsm/extensions_sale.md |
| El coste del parte sale 0 / no editable a mano | core | TG21C | enterprise/timesheet/gotchas.md |
| No deja crear partes de ex-empleado / proyecto cerrado | core | TG23C | enterprise/timesheet/gotchas.md |
| Horas al doble/mitad (UoM `project_time_mode_id`) | core | TG20C | enterprise/timesheet/gotchas.md |
| Parte facturado no deja editar/borrar | facturación | TG31C | enterprise/timesheet/gotchas.md |
| Escribir 0 en celda del grid no hace nada | grid | TG35C | enterprise/timesheet/gotchas.md |

---

## 20. Planning — bridges (hr/project_forecast/fsm/sale/timesheet/contract/holidays/skills)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Slot en FSM project decorativo (no genera task, no sincroniza) | planning | PLG33FSM | enterprise/planning/integrations.md |
| Cancelar SOL borra slots published en cascada silenciosa | planning | PLG39SL | enterprise/planning/integrations.md |
| Auto-plan excluye empleados con calendario flexible silentemente | planning | PLG40SL | enterprise/planning/integrations.md |
| `planning_contract` eliminado en v19 (lógica absorbida con hr.version) | planning | PLG58CT | enterprise/planning/integrations.md |
| `effective_hours` match por fecha completa → overlap double-count | planning | PLG46TS | enterprise/planning/integrations.md |
| Leave no cancela slots automáticamente (allocated_hours=0 pero slot visible) | planning | PLG64HOL | enterprise/planning/integrations.md |
| Turnos recurrentes dejan de generarse (lazy 6 meses / cron sin nextcall) | recurrencia | PLG91/PLG92 | enterprise/planning/gotchas.md |
| Editar turno recurrente publicado deja duplicados | recurrencia | PLG96 | enterprise/planning/gotchas.md |
| Horas asignadas cambian al asignar recurso a un open shift | horas | PLG95 | enterprise/planning/gotchas.md |
| Autoasignación con solape / al 200% | self-assign | PLG98 | enterprise/planning/gotchas.md |
| Confirmar SO crea turnos sin fecha en Planning | sale_planning | PLG102/PLG103 | enterprise/planning/gotchas.md |

---

## 20b. Kits / phantom BoM — capacidades (eje consultor)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| "¿Odoo PUEDE hacer X con kits/packs?" (eje consultor) | capabilities | KITCAP1-41 | cross-module/mrp/capabilities.md |
| Flujo fabricante: variante↔BoM↔kit↔MO↔coste↔venta (preguntas de flujo) | flow | VXK1-38 | cross-module/mrp/integrations.md |
| ¿Qué BoM se aplica a cada variante? (variante-específica gana) | flow | VXK1/VXK2 | cross-module/mrp/integrations.md |
| Componente/operación de BoM condicionado por variante o atributo | flow | VXK6/VXK7 | cross-module/mrp/integrations.md |
| Atributo no_variant que añade componente/operación al fabricar | flow | VXK10/VXK11 | cross-module/mrp/integrations.md |
| Coste de variante calculado desde su BoM (Compute Price from BoM) | flow | VXK22 | cross-module/mrp/integrations.md |
| Cambiar BoM con MO abierta → is_outdated_bom | flow | VXK31 | cross-module/mrp/integrations.md |
| Variantes vs kit vs BoM matricial (decisión de diseño) | flow | VXK27-29 | cross-module/mrp/integrations.md |
| ¿El precio del kit es la suma de componentes? (NO — es el del producto kit) | capabilities | KITCAP12 | cross-module/mrp/capabilities.md |
| ¿Un kit tiene stock propio? (no, calculado desde componentes) | capabilities | KITCAP5 | cross-module/mrp/capabilities.md |
| ¿Puedo poner orderpoint sobre un kit? (no, bloqueado) | capabilities | KITCAP26 | cross-module/mrp/capabilities.md |
| Diferencia kit vs combo vs producto opcional/accesorio | capabilities | KITCAP31/32 | cross-module/mrp/capabilities.md |
| v19: batch_size / enable_batch_size en mrp.bom | capabilities | KITCAP39 | cross-module/mrp/capabilities.md |

## 21. MRP Workorder — bridges (mrp/hr/quality/barcode/iot/maintenance/plm/expiry/tablet)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| PIN vacío en hr.employee permite login con cualquier badge (auth bypass) | mrp_workorder | WOG54H | enterprise/mrp_workorder/integrations.md |
| Pedales IoT dejan de responder sin feedback en tablet | mrp_workorder | WOG72IO | enterprise/mrp_workorder/integrations.md |
| Lot SN creado pero label no impreso (auto_print_generated_mrp_lot) | mrp_workorder | WOG67B | enterprise/mrp_workorder/integrations.md |
| Lot expirado de byproduct registrado sin warning (compliance gap) | mrp_workorder | WOG95EX | enterprise/mrp_workorder/integrations.md |
| Maintenance request raise en create pero solo activity en write (inconsistente) | mrp_workorder | WOG76MN | enterprise/mrp_workorder/integrations.md |
| Coste analítico cambia retroactivamente al modificar hourly_cost mid-shift | mrp_workorder | WOG49M / WOG88A / WOG91A | enterprise/mrp_workorder/integrations.md |
| v19: `employee_costs_hour` snapshot resuelve cost retroactivo | mrp_workorder | v19 | enterprise/mrp_workorder/integrations.md |

---

## 22. Quality (Enterprise) — bridges extras (mrp/purchase/stock/fsm-gap/maintenance-gap/iot/batch/alert/worksheet)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Validate button hidden en stock picking con quality checks | quality | QG47P | enterprise/quality/integrations.md |
| Quality checks remain after PO cancel (no cascade) | quality | QG50P | enterprise/quality/integrations.md |
| Random frequency check inconsistente entre picking y backorder | quality | QG52P / QG78BT | enterprise/quality/integrations.md |
| "You still need to do the quality checks!" sin botón visible | quality | QG54S / QG57S | enterprise/quality/integrations.md |
| "Cannot perform a quality check if qty_producing is zero" (v18) | quality | QG42M | enterprise/quality/integrations.md |
| Quality alert via email: subject→title pisado por sequence | quality | QG85AL / QG86AL | enterprise/quality/integrations.md |
| No existen bridges oficiales quality_industry_fsm ni quality_maintenance | quality | QG61-69 | enterprise/quality/integrations.md |

---

## 23. Account Followup — bridges (account/partner/mail/portal/CRM/cron/SMS/multi-company)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Asimetría `followup.line` parent_of vs AML child_of confunde multi-company | account_followup | AFG73MC | enterprise/account_followup/integrations.md |
| Cache `res_partner_all_followup` cruza datos entre companies | account_followup | AFG54P / AFG74MC | enterprise/account_followup/integrations.md |
| Cron no commitea per-partner: failure parcial pero emails enviados | account_followup | AFG84C | enterprise/account_followup/integrations.md |
| Hard limit `[:1000]` partners/company/run con backlog silencioso | account_followup | AFG81C | enterprise/account_followup/integrations.md |
| "Most delayed AML" = mayor amount_residual, no mayor delay temporal | account_followup | AFG67CRM | enterprise/account_followup/integrations.md |
| `search_count([('active','=','True')])` bug type coercion latente | account_followup | AFG62PT | enterprise/account_followup/integrations.md |

---

## 24. Account Reports — bridges (AML/analytic/tax_tags/multi-company/audit/variants/return/FX/export/external)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| `sum_if_pos` da 0 con balances positivos (sign policy sobre total agregado) | account_reports | ARG38A | enterprise/account_reports/integrations.md |
| Tax closing entry no balancea (tax_group sin tax_receivable/payable_account) | account_reports | ARG79RT | enterprise/account_reports/integrations.md |
| `use_in_tax_closing=False` excluye tax del cierre pero no del report | account_reports | ARG84RT | enterprise/account_reports/integrations.md |
| Audit drill-down de `account_codes` con sufijo D/C muestra líneas no contribuyentes | account_reports | ARG67AU | enterprise/account_reports/integrations.md |
| Balance Sheet consolidado multi-currency con números mezclados (LEFT JOIN COALESCE rate=1) | account_reports | ARG87FX | enterprise/account_reports/integrations.md |
| Filtro analytic en Balance Sheet no cambia números (solo drill-down) | account_reports | ARG47AN | enterprise/account_reports/integrations.md |
| Modelo 303 muestra tax pero no aparece en closing entry | account_reports | ARG84RT | enterprise/account_reports/integrations.md |

---

## 25. Stock Barcode — bridges (stock/mrp/pos/quality/bus/iot/inventory/batch/delivery)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| `qty_done` setea `picked=True` automáticamente vía inverse (efecto inesperado server) | stock_barcode | BG22S | enterprise/stock_barcode/integrations.md |
| No live sync entre operarios simultáneos en mismo picking (último save gana) | stock_barcode | BG46BUS | enterprise/stock_barcode/integrations.md |
| `barcode_max_time_between_keys_in_ms` bajo → barcode como keyboard normal | stock_barcode | BG50IO | enterprise/stock_barcode/integrations.md |
| `barcode_write` usa formato `[[1,id,vals],[0,0,vals]]` no estándar Odoo API | stock_barcode | BG54IA | enterprise/stock_barcode/integrations.md |
| GS1 AI 31xx weight NO se convierte de UoM (errores kg vs g masivos) | stock_barcode | BG65DEL | enterprise/stock_barcode/integrations.md |
| Quality check con `barcode_trigger=True` ignora líneas no-picked (wizard vacío) | stock_barcode | BG40Q | enterprise/stock_barcode/integrations.md |
| v19: `stock.quant.package`→`stock.package`, `product.uom` separado de `uom.uom` | stock_barcode | v19 | enterprise/stock_barcode/integrations.md |
| v19: nuevo `epc_encoder.py` con endpoints SGTIN para tags RFID | stock_barcode | v19 NEW | enterprise/stock_barcode/integrations.md |

---

## 25b. Studio (Enterprise) — customizations silently fail / break on upgrade

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Studio view change disappeared after module upgrade (no error) | studio | STUG8 | enterprise/studio/gotchas.md |
| Can't add a field to this model ("doesn't support adding fields") | studio | STUG1 | enterprise/studio/gotchas.md |
| Studio field is `x_active` not `x_studio_active` (archive) | studio | STUG2 | enterprise/studio/gotchas.md |
| Binary Studio field created a `_filename` field I didn't ask for | studio | STUG4 | enterprise/studio/gotchas.md |
| Removing a selection value wiped records to False | studio | STUG5 | enterprise/studio/gotchas.md |
| Can't change a related field's path in Studio | studio | STUG6 | enterprise/studio/gotchas.md |
| Studio view overrides my module's view inherit | studio | STUG9 | enterprise/studio/gotchas.md |
| Group visibility set in Studio isn't enforced as security | studio | STUG10 | enterprise/studio/gotchas.md |
| "Only groups with an external ID can be used here" | studio | STUG12 | enterprise/studio/gotchas.md |
| Approval rule on custom model sends no notifications | studio | STUG13 | enterprise/studio/gotchas.md |
| Webhook automation returns 200 but nothing happens | studio | STUG14 | enterprise/studio/gotchas.md |
| Can't `import` in a Studio code server action | studio | STUG16 | enterprise/studio/gotchas.md |
| Orphan `x_*` models persist after uninstall | studio | STUG17 | enterprise/studio/gotchas.md |
| Base data fix didn't reach a Studio-edited record (noupdate) | studio | STUG18 | enterprise/studio/gotchas.md |
| Exported Studio module installs empty (no records) | studio | STUG19 | enterprise/studio/gotchas.md |
| Some fields missing from Studio export / warnings.txt | studio | STUG20 | enterprise/studio/gotchas.md |
| Studio export won't install on Community | studio | STUG21 | enterprise/studio/gotchas.md |
| Slow filter on a Studio field (no index) | studio | STUG22 | enterprise/studio/gotchas.md |
| `__` rejected in custom field name | studio | STUG24 | enterprise/studio/gotchas.md |
| v19: custom kanban `t-name="kanban-box"` breaks | studio | STUG25 | enterprise/studio/v19_changes.md |
| v19: `/web_studio/*` routes need `jsonrpc`, expression→Domain | studio | v19 | enterprise/studio/v19_changes.md |

---

## 26. Playbooks — el flujo end-to-end se rompe a mitad de la cadena

> Cuando el síntoma es "el proceso completo no llega al final" (no un módulo aislado), entra por aquí: te da la **etapa** donde se corta y el playbook con el mapa de toda la cadena.

### Order-to-Cash (O2C) — `playbooks/order_to_cash.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| "No items are available to invoice" tras confirmar SO | facturación | SG2 | playbooks/order_to_cash.md |
| Factura sigue impagada pese al cobro registrado en banco | pago/conciliación | AG4 | playbooks/order_to_cash.md |
| Posteo factura falla "distribution must be 100%" sin indicar plan | posteo | AG34/ANG4 | playbooks/order_to_cash.md |
| Cliente con facturas vencidas nunca recibe recordatorio | followup | AFG11/AFG13 | playbooks/order_to_cash.md |
| Picking se queda en `confirmed` sin reservar stock | entrega | STKG18 | playbooks/order_to_cash.md |
| Tras confirmar el SO NO aparece ninguna entrega (producto service / sin ruta de salida) | entrega | — | playbooks/order_to_cash.md (etapa 2-3) |

### Procure-to-Pay (P2P) — `playbooks/procure_to_pay.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Servicio comprado nunca facturable (qty_received manual) | control recepción | PG16 | playbooks/procure_to_pay.md |
| No puedo crear factura aunque ya recibí la mercancía | control recepción | PG5/PUAC2 | playbooks/procure_to_pay.md |
| Diferencia de céntimo bloquea el pago (3-way match) | matching | PU3W1 | playbooks/procure_to_pay.md |

### Make-to-Order (MTO) — `playbooks/make_to_order.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| SO confirmado pero no se crea MO ni picking | lanzamiento procurement | STKG21 | playbooks/make_to_order.md |
| MO MTO se queda en `draft` sin auto-confirmar | creación MO | STKG67 | playbooks/make_to_order.md |
| Cancelo el SO pero la MO sigue activa (stock huérfano) | cancelación | STKG23/MG16 | playbooks/make_to_order.md |
| Producto terminado entra a coste 0/irreal o sin WIP | valoración MO | MG10/MG11 | playbooks/make_to_order.md |
| COGS aparece antes de facturar (o nunca) | valoración | AG35 | playbooks/make_to_order.md |

### Subscription lifecycle (SUB) — `playbooks/subscription_lifecycle.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Dos facturas pendientes para la misma suscripción | cron facturación | SUBG2/SUBG35A | playbooks/subscription_lifecycle.md |
| Suscripción congelada, el cron nunca la refactura | pago | SUBG8/SUBG43P | playbooks/subscription_lifecycle.md |
| Dashboard MRR no refleja los churns reales | churn/MRR | SUBG65M | playbooks/subscription_lifecycle.md |
| Renovación tardía contada como churn+new (rompe NRR) | renovación | SUBG4/SUBG66M | playbooks/subscription_lifecycle.md |

### POS reconciliation (POR) — `playbooks/pos_reconciliation.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| "You cannot close the POS when orders are still in draft" | cierre sesión | POG15 | playbooks/pos_reconciliation.md |
| "Please define a Loss/Profit Account" al cerrar | cash control | POG5 | playbooks/pos_reconciliation.md |
| Receivable POS abierto pese al cobro (aparece en aging) | conciliación | AG2/AG30 | playbooks/pos_reconciliation.md |
| Pago cash no aparece en Contabilidad → Pagos (sí el de tarjeta) | conciliación | POG12 | playbooks/pos_reconciliation.md |
| Ventas POS invisibles en informes analíticos | analítica | ANG22/ANG23 | playbooks/pos_reconciliation.md |

### FSM end-to-end (FSM) — `playbooks/fsm_end_to_end.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| 5/10 uds entregadas, el resto desaparece sin backorder | validate/stock | FSMG31 | playbooks/fsm_end_to_end.md |
| Reporte muestra 0 horas facturables antes de cerrar | pricing employee_rate | FSMG30 | playbooks/fsm_end_to_end.md |
| Tarea en garantía sin material cierra sin SO (no facturable) | cierre | FSMG62 | playbooks/fsm_end_to_end.md |
| Override de cierre no entra en la super-chain | cierre | FSMG61 | playbooks/fsm_end_to_end.md |
| ¿FSM funciona offline / sin conexión? (no — PWA-only, sin app nativa) | mobile/offline | FSMG63 | enterprise/fsm/gotchas.md |

### eCommerce flow (ECO) — `playbooks/ecommerce_flow.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Pago OK pero pedido sigue en `sent` (no confirma) | confirmación | ECG22 | playbooks/ecommerce_flow.md |
| Pedido pagado y confirmado pero SIN factura | facturación | ECG5 | playbooks/ecommerce_flow.md |
| Oversell: se vende más stock del disponible | carrito→confirm | ECG1 | playbooks/ecommerce_flow.md |
| Pagar factura vieja reactiva una sub cancelada | renovación | SUBG19 | playbooks/ecommerce_flow.md |

### Intercompany (ICO) — `playbooks/intercompany.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| `UserError` de currency al aprobar PO/SO espejo | PO→SO | ICG1 | playbooks/intercompany.md |
| No se crea documento espejo, sin error | resolución partner | ICG12/ICG4 | playbooks/intercompany.md |
| `UserError` access rights al confirmar (aborta el origen) | creación espejo | ICG21 | playbooks/intercompany.md |
| Facturas A y bill B nunca se concilian entre sí | contable | ICG22 | playbooks/intercompany.md |

### Project profitability (PRJ) — `playbooks/project_profitability.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Margen inflado, coste de horas = 0 | timesheets | ANG20/TG44SL | playbooks/project_profitability.md |
| Coste de compra/gasto no aparece en el proyecto | compras/gastos | ANG9/ANG5 | playbooks/project_profitability.md |
| Coste contado dos veces en el margen | facturación | ANG21 | playbooks/project_profitability.md |
| Tras confirmar SO no hay cuenta analítica → nada se imputa | arranque | ANG19/TG19PR | playbooks/project_profitability.md |

### Returns / RMA (RMA) — `playbooks/returns_rma.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Factura sigue "no pagada" tras crear la nota de crédito | credit note | AG4/AG30 | playbooks/returns_rma.md |
| Doble abono: credit note total pese a stock ya devuelto | refund | AG12 | playbooks/returns_rma.md |
| Tras el return `qty_delivered` no baja (SO sigue entregada) | return picking | STKG36 | playbooks/returns_rma.md |
| Pieza de reparación desaparece sin asiento ni COGS | repair | RG1/RG16 | playbooks/returns_rma.md |

### Event sale to attendance (EVT-SALE) — `playbooks/event_sale_to_attendance.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Pagué la entrada pero el asistente sigue en "no confirmado" / no puedo check-in | confirmación/pago | EVTG15/EVTG1 | playbooks/event_sale_to_attendance.md |
| Pago agrupado (varias SO) y ninguna inscripción se confirma | confirmación SO | EVTG15 | playbooks/event_sale_to_attendance.md |
| Bajé la cantidad en el carrito y se cancelaron asistentes | carrito | EVTG17 | playbooks/event_sale_to_attendance.md |
| No se genera el lead de la inscripción | lead | EVTG19 | playbooks/event_sale_to_attendance.md |

### Expense to reimbursement (EXP-REIMB) — `playbooks/expense_to_reimbursement.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Un solo "enviar" gastos me creó dos informes | sheet | EXPG3 | playbooks/expense_to_reimbursement.md |
| Gasto de empresa aparece "pagado" pero el banco no está conciliado | post/pago | EXPG12 | playbooks/expense_to_reimbursement.md |
| Reembolso por nómina falla (regla EXPENSES) | nómina | EXPG22 | playbooks/expense_to_reimbursement.md |
| El gasto no se refactura al cliente | refacturación | EXPG19 | playbooks/expense_to_reimbursement.md |

### eLearning paid + certification (ELC) — `playbooks/elearning_certification.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Compré el curso pero no me inscribió | inscripción | SLDG19 | playbooks/elearning_certification.md |
| Aprobé la certificación pero no me dieron insignia | badge | SLDG15/SLDG16 | playbooks/elearning_certification.md |
| Suspendí la certificación y me echó del curso | fallo | SLDG17 | playbooks/elearning_certification.md |

### Loyalty earn & redeem (LOY-ER) — `playbooks/loyalty_earn_redeem.md`
| Symptom | Etapa | Gotcha ID | File |
|---------|-------|-----------|------|
| Gané puntos pero no se reflejan hasta confirmar el pedido | ganar | LOYG17 | playbooks/loyalty_earn_redeem.md |
| El cupón descuenta menos de su valor / gasta menos puntos | reclamar | LOYG27/LOYG28 | playbooks/loyalty_earn_redeem.md |
| Cancelé el pedido y el cupón "next order" no se regeneró | cancelar | LOYG18 | playbooks/loyalty_earn_redeem.md |
| En POS el pago se bloquea por cupón inválido | POS | — | playbooks/loyalty_earn_redeem.md |

---

## 27. Bridges web / CRM / helpdesk (remediación FASE 3)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Web form mis-atribuye partner por email+phone | website | WG26 | functional/website/gotchas.md |
| Web form crea lead vs opportunity según `team.use_leads` | website | WG27 | functional/website/gotchas.md |
| Helpdesk crea res.partner duplicado por email exacto (sin dedup) | website | WG29 | functional/website/gotchas.md |
| Factura multi-website pierde `website_id` al agrupar | website | WG30 | functional/website/gotchas.md |
| Enrich cron solo procesa leads frescos (no reintenta antiguos) | crm | CG27 | functional/crm/gotchas.md |
| Lead mining se detiene en seco (créditos IAP / caps 200/5) | crm | CG29 | functional/crm/gotchas.md |
| Web form deriva tipo lead/opp de `team.use_leads` | crm | CG31 | functional/crm/gotchas.md |
| `helpdesk_stock`/`_account`/`_repair` NO son auto_install (hay que activarlos) | helpdesk | HDG21 | enterprise/helpdesk/gotchas.md |
| Return desde ticket exige picking `done` + product_id | helpdesk | HDG22 | enterprise/helpdesk/gotchas.md |
| `helpdesk_sale` NO crea SO (solo enlaza sale_order_id) | helpdesk | HDG23 | enterprise/helpdesk/gotchas.md |
| Credit note desde ticket asume 1 sola SO line | helpdesk | HDG24 | enterprise/helpdesk/gotchas.md |
| FSM desde ticket necesita `fsm_project_id` configurado | helpdesk | HDG26 | enterprise/helpdesk/gotchas.md |
| Alias mail→ticket descarta partner_id cross-company | helpdesk | HDG27 | enterprise/helpdesk/gotchas.md |

---

## 27b. Helpdesk core (SLA / lifecycle / asignación / canales) — **EE**

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| SLA no se dispara / no aplica al ticket (priority no coincide exacto) | helpdesk | HDG5 | enterprise/helpdesk/gotchas.md |
| SLA con varias tags matchea con UNA sola (OR, no AND) | helpdesk | HDG6 | enterprise/helpdesk/gotchas.md |
| Deadline SLA absurdo / 24h = 3 días (working hours, no calendar) | helpdesk | HDG1 | enterprise/helpdesk/gotchas.md |
| SLA metrics todas a False / deadline = create_date (falta resource_calendar_id) | helpdesk | HDG18 | enterprise/helpdesk/gotchas.md |
| Tiempo "en espera" no congela el SLA (stage tracking desactivado) | helpdesk | HDG2 | enterprise/helpdesk/gotchas.md |
| `sla_reached` no significa "alcanzado" sino "alcanzado pronto" (exceeded_hours<0) | helpdesk | HDG3 | enterprise/helpdesk/gotchas.md |
| Reach de SLA en stage alto marca también los stages bajos (cascada) | helpdesk | HDG19 | enterprise/helpdesk/gotchas.md |
| Estado "Blocked/Ready" desaparece al cambiar de stage (kanban_state reset) | helpdesk | HDG7 | enterprise/helpdesk/gotchas.md |
| `assign_hours` no refleja al asignado actual (assign_date solo 1ª vez) | helpdesk | HDG8 | enterprise/helpdesk/gotchas.md |
| Reabrir ticket pierde close_date original | helpdesk | HDG9 | enterprise/helpdesk/gotchas.md |
| Tickets se cierran al crearse (primer stage está folded) | helpdesk | HDG31 | enterprise/helpdesk/gotchas.md |
| Asignación automática no se re-dispara al cambiar de team | helpdesk | HDG4 | enterprise/helpdesk/gotchas.md |
| "Balanced" cuenta tickets solo del team actual | helpdesk | HDG12 | enterprise/helpdesk/gotchas.md |
| Auto-close no cierra (mide write_date, nota interna lo resetea) | helpdesk | HDG13 | enterprise/helpdesk/gotchas.md |
| Auto-close no hace nada (falta to_stage_id) | helpdesk | HDG14 | enterprise/helpdesk/gotchas.md |
| Cliente no recibe email de stage (actor es el cliente / sin partner_email) | helpdesk | HDG15 | enterprise/helpdesk/gotchas.md |
| first_response_hours no se calcula (sin calendar / ticket cerrado) | helpdesk | HDG16 | enterprise/helpdesk/gotchas.md |
| Renombrar/foldear un stage afecta a TODOS los teams (M2M shared) | helpdesk | HDG38 | enterprise/helpdesk/gotchas.md |
| Archivar un stage archiva todos sus tickets | helpdesk | HDG39 | enterprise/helpdesk/gotchas.md |
| ticket_ref repetido entre compañías (secuencia per-company) | helpdesk | HDG33 | enterprise/helpdesk/gotchas.md |
| Ticket livechat/chatbot: historial completo va a `description` | helpdesk | — | enterprise/helpdesk/integrations.md |
| v19: `_determine_user_to_assign` cambia firma (count_per_team) + assign_method 'tags' | helpdesk | — | enterprise/helpdesk/v19_changes.md |
| v17→v18: `helpdesk.ticket.type` / `ticket_type_id` eliminado | helpdesk | — | enterprise/helpdesk/v19_changes.md |

---

## 28. Bridges remediados FASE 3 punto 2 (stock/hr/project/repair/marketing/mailing/sign/rental/maintenance)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| `carrier_id` no existe en `stock.picking` / ImportError al instalar | stock | STKG68 | functional/stock/gotchas.md |
| POS no respeta ruta MTO/dropship (picking al pagar) | stock | STKG69 | functional/stock/gotchas.md |
| Reparación no descuenta stock hasta cerrarla | stock | STKG70 | functional/stock/gotchas.md |
| Transferencia interna entre ubicaciones valoradas no genera SVL | stock | STKG72 | functional/stock/gotchas.md |
| Buscar `hr.timesheet` no encuentra nada (es account.analytic.line) | hr | HRG39 | functional/hr/gotchas.md |
| Payslip sin asiento contable y sin error (lo crea hr_payroll_account) | hr | HRG41 | functional/hr/gotchas.md |
| Escribir `planning.slot.employee_id` se revierte (deriva de resource_id) | hr | HRG42 | functional/hr/gotchas.md |
| `hr_contract` no existe en v19 (fusionado en hr.version) | hr | HRG (v19) | functional/hr/gotchas.md |
| Smart-button de POs en proyecto cuenta de más | project | PRJG34 | functional/project/gotchas.md |
| Poner proyecto en un PO cambia la distribución analítica de las líneas | project | PRJG35 | functional/project/gotchas.md |
| Gasto no aparece en el margen del proyecto (solo posted/done) | project | PRJG36 | functional/project/gotchas.md |
| Proyecto desde plantilla pierde los roles (project.role v19) | project | PRJG37 | functional/project/gotchas.md |
| Completar/cancelar repair no escribe en el ticket helpdesk | repair | RG26 | functional/repair/gotchas.md |
| Cambiar producto/lote de la RO pierde quality checks hechos | repair | RG27 | functional/repair/gotchas.md |
| Cambié activity email→SMS y perdí el mailing (mass_mailing_id computed) | marketing | MAG24 | enterprise/marketing_automation/gotchas.md |
| Clicks del mailing no atribuidos a la campaña UTM | marketing | MAG25 | enterprise/marketing_automation/gotchas.md |
| Aperturas infladas: open sin abrir (add_click dispara set_opened) | mailing | MMG24 | functional/mailing/gotchas.md |
| Dominio sobre `mailing.trace.mailing_id` devuelve vacío (usa mass_mailing_id) | mailing | MMG25 | functional/mailing/gotchas.md |
| Mailing en cola no se envía nunca (depende del cron, interval 1 day) | mailing | MMG27 | functional/mailing/gotchas.md |
| PDF firmado no valida criptográficamente en v18 (stamp ReportLab, X.509 solo v19) | sign | SNG17 | enterprise/sign/gotchas.md |
| Link de firma expira antes/después al migrar (48h v18 → 360h v19) | sign | SNG18 | enterprise/sign/gotchas.md |
| No encuentro las `sign.request` de un contrato por reference_doc (HR usa M2M) | sign | SNG20 | enterprise/sign/gotchas.md |
| Picking de devolución de alquiler no disponible hasta validar el pickup | rental | RNTG24 | enterprise/rental/gotchas.md |
| Pedido mixto sale+rental genera pickings mezclados (force-merge) | rental | RNTG26 | enterprise/rental/gotchas.md |
| Owner de equipo cambia solo al editar (hr_maintenance owner='other') | maintenance | MTG19 | functional/maintenance/gotchas.md |
| Programé mantenimiento de máquina pero siguen planificando MOs encima | maintenance | MTG22 | functional/maintenance/gotchas.md |
| Tras migrar v19 desaparece ubicación del equipo / serial no matchea | maintenance | MTG24 | functional/maintenance/gotchas.md |

---

## 29. Cross-module enhanced (analytic / mrp / product)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Devolví un albarán y desaparecieron las líneas analíticas (netean a 0 → unlink) | analytic | ANG34 | cross-module/analytic/gotchas.md |
| Importe analítico del picking no cuadra con el coste FIFO/AVCO real | analytic | ANG35 | cross-module/analytic/gotchas.md |
| Movimiento de stock (dropship/back-order/manual) sin analítica | analytic | ANG36 | cross-module/analytic/gotchas.md |
| Tras migrar v19 mi override de `_condition_to_sql` deja de filtrar | analytic | ANG38 | cross-module/analytic/gotchas.md |
| `create_uid` de la MO es admin, no el comercial que confirmó el SO | mrp | MG25 | cross-module/mrp/gotchas.md |
| Coste de subcontratista en 2-3 líneas de asiento o va a diferencia de precio | mrp | MG26 | cross-module/mrp/gotchas.md |
| Odoo elige el BOM genérico en vez del específico del almacén (orden por sequence) | mrp | MG27 | cross-module/mrp/gotchas.md |
| Terminé la MO pero el producto no está en stock (3-step pbm_sam) | mrp | MG28 | cross-module/mrp/gotchas.md |
| MO/PO del MPS salen tarde / no restan lead time | mrp | MG29 | cross-module/mrp/gotchas.md |
| Cambio de UoM en factura no recalcula precio pero en pedido sí | product/uom | UOG23 | cross-module/product/gotchas.md |
| v19: conversión UoM falla / `category_id` no existe en uom.uom (árbol relative_uom_id) | product/uom | UOG24 | cross-module/product/gotchas.md |
| Pricelist "100% del coste" da el mismo precio a todas las variantes (ignora price_extra) | pricelist | PLSG13 | cross-module/product/gotchas.md |

---

## 30. FASE 4 — banca / compliance (bank_statement_import, iso20022, saft, extract, loans)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Transacciones duplicadas tras importar el fichero en el journal equivocado | bank_statement_import | BSIG1 | enterprise/bank_statement_import/gotchas.md |
| "You already have imported that file" aunque hay líneas nuevas | bank_statement_import | BSIG2 | enterprise/bank_statement_import/gotchas.md |
| CAMT multi-IBAN/multi-divisa se importa a medias | bank_statement_import | BSIG5 | enterprise/bank_statement_import/gotchas.md |
| Norma 43 española (N43/AEB43) NO soportada por EE → requiere OCA | bank_statement_import | BSIG21 | enterprise/bank_statement_import/gotchas.md |
| Importar CAMT aborta "No exchange rate was found to convert an amount" | bank_statement_import | BSIG23 | enterprise/bank_statement_import/gotchas.md |
| Línea de banco no cuadra con factura por unos céntimos (gastos bancarios embebidos en CAMT) | bank_statement_import | BSIG24 | enterprise/bank_statement_import/gotchas.md |
| Líneas CAMT duplicadas de un banco que no rellena AcctSvcrRef (NOTPROVIDED/ceros) | bank_statement_import | BSIG25 | enterprise/bank_statement_import/gotchas.md |
| No aparece SEPA Credit Transfer al registrar el pago (moneda no-SEPA) | account_iso20022 | ISOG1 | enterprise/account_iso20022/gotchas.md |
| XML pasa en Odoo pero el banco lo rechaza (no hay validación XSD) | account_iso20022 | ISOG3 | enterprise/account_iso20022/gotchas.md |
| Banco rechaza por falta de BIC (NOTPROVIDED silencioso) | account_iso20022 | ISOG4 | enterprise/account_iso20022/gotchas.md |
| "Your company must have a creditor identifier" (SDD) | account_iso20022 | ISOG17 | enterprise/account_iso20022/gotchas.md |
| Normas AEB 34.14/19.14 (registro fijo) NO nativas → OCA/l10n_es | account_iso20022 | ISOG | enterprise/account_iso20022/gotchas.md |
| SAF-T NO aplica a España (usa SII + VeriFactu/TBAI) | account_saft | SAFTG1 | enterprise/account_saft/gotchas.md |
| "SAF-T is only compatible with one column group" | account_saft | SAFTG4 | enterprise/account_saft/gotchas.md |
| XML SAF-T con xmlns vacío / versión undefined → rechazado | account_saft | SAFTG6 | enterprise/account_saft/gotchas.md |
| Faltan importes de impuesto en TaxInformation de ciertas líneas (signo cuota≠base) | account_saft | SAFTG17 | enterprise/account_saft/gotchas.md |
| La base imponible del XML no coincide con `tax_base_amount` de la línea | account_saft | SAFTG18 | enterprise/account_saft/gotchas.md |
| `account_extract` es glue vacío (lógica en account_invoice_extract) | account_extract | AEXG1 | enterprise/account_extract/gotchas.md |
| Cada factura consume crédito IAP sin pedirlo (auto_send) | account_extract | AEXG3 | enterprise/account_extract/gotchas.md |
| PDF de factura sale a extract.api.odoo.com (no OCR local — RGPD) | account_extract | AEXG2 | enterprise/account_extract/gotchas.md |
| OCR no sobreescribe campos rellenados a mano | account_extract | AEXG6 | enterprise/account_extract/gotchas.md |
| Cuotas futuras del préstamo no se contabilizan solas | account_loans | LOANG1 | enterprise/account_loans/gotchas.md |
| Asiento del préstamo no va contra banco (572) — conciliación manual | account_loans | LOANG3 | enterprise/account_loans/gotchas.md |
| "amount should equal sum of principals" por redondeos del banco | account_loans | LOANG9 | enterprise/account_loans/gotchas.md |
| Reclasificación LT→ST hardcoded a 12 cuotas | account_loans | LOANG11 | enterprise/account_loans/gotchas.md |
| Cerrar préstamo no revierte pagos ya posteados (sólo borra drafts) | account_loans | LOANG21 | enterprise/account_loans/gotchas.md |
| interest_rate=0 genera amortización lineal, no annuity de interés 0 | account_loans | LOANG22 | enterprise/account_loans/gotchas.md |
| first_payment_date ignorado en modo end_of_month | account_loans | LOANG23 | enterprise/account_loans/gotchas.md |
| Reclasif LT→ST no lleva analítica/tax_tags (sólo corte de balance) | account_loans | LOANG24 | enterprise/account_loans/gotchas.md |
| UserError: SAF-T solo compatible con un column group | saft export | SAFTG4 | enterprise/account_saft/gotchas.md |
| Import SAF-T NO/AT/RO: KeyError namespace / no encuentra nodos | saft import | SAFTG28 | enterprise/account_saft/gotchas.md |
| RO: export SAF-T falla por CUI/VAT inválido o producto sin referencia | saft RO | SAFTG24 | enterprise/account_saft/gotchas.md |
| AT: "accounts can not be mapped to chart given in SAF-T" | saft AT | SAFTG26 | enterprise/account_saft/gotchas.md |
| Contactos del XML SAF-T salen con `FirstName=NotUsed` | saft export | SAFTG21 | enterprise/account_saft/gotchas.md |

---

## 31. FASE 4 — admin tools (data_cleaning / data_merge / data_recycle, frontdesk)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Registros desaparecen de la BD tras el cron (recycle borra, no archiva, por defecto) | data_tools | DTG3 | enterprise/data_tools/gotchas.md |
| Duplicados con mismo email no se fusionan / % similitud arbitrario | data_tools | DTG14 | enterprise/data_tools/gotchas.md |
| Registros se fusionan de madrugada con FKs reapuntadas sin aviso (merge automático irreversible) | data_tools | DTG17 | enterprise/data_tools/gotchas.md |
| Nombres de contacto quedan pegados sin espacios (`trim_all`) | data_tools | DTG10 | enterprise/data_tools/gotchas.md |
| Match `accent` falla sin extensión PG `unaccent` | data_tools | DTG16 | enterprise/data_tools/gotchas.md |
| `data_merge` no es módulo propio (modelos viven en data_cleaning EE) | data_tools | DTG | enterprise/data_tools/gotchas.md |
| Merge configurado como archive pero los duplicados se borran (modelo sin campo active) | data_tools | DTG35 | enterprise/data_tools/gotchas.md |
| Grupo de duplicados no se fusiona y no hay error ("master record does not exist" en log) | data_tools | DTG36 | enterprise/data_tools/gotchas.md |
| Merge revienta con error SQL al reapuntar M2M (rama M2M sin savepoint) | data_tools | DTG34 | enterprise/data_tools/gotchas.md |
| Desactivé notificaciones pero recepcionistas siguen recibiendo Discuss | frontdesk | FDG2 | enterprise/frontdesk/gotchas.md |
| Host con visita asignada no recibe aviso (falta `host_selection`) | frontdesk | FDG3 | enterprise/frontdesk/gotchas.md |
| QR de check-in caduca ("QR Code Expired", 1h / cambio de día) | frontdesk | FDG6 | enterprise/frontdesk/gotchas.md |
| Host concreto no recibe email/SMS (sin work_email/work_phone) | frontdesk | FDG4 | enterprise/frontdesk/gotchas.md |
| Plantilla de email de frontdesk sale vacía (debe ser sobre frontdesk.visitor) | frontdesk | FDG5 | enterprise/frontdesk/gotchas.md |
| Notificaciones se disparan en write(state=checked_in), no en action_check_in (import/server action notifica) | frontdesk | FDG1 | enterprise/frontdesk/gotchas.md |
| El kiosk me desloguea del backend al abrirlo (session.logout) | frontdesk | FDG7 | enterprise/frontdesk/gotchas.md |
| Quick check-in no encuentra al planned visitor (falta check_in previsto / fuera de ±45min) | frontdesk | FDG15 | enterprise/frontdesk/gotchas.md |
| Write masivo de estado notifica bebidas de visitors que no las pidieron (self.drink_ids global) | frontdesk | FDG16 | enterprise/frontdesk/gotchas.md |
| Botón "served" no cambia estado ni hora (solo served=True) | frontdesk | FDG17 | enterprise/frontdesk/gotchas.md |
| duration muestra 1.0 fantasma en visitas sin check-out (default, store) | frontdesk | FDG18 | enterprise/frontdesk/gotchas.md |
| Host de otra company asignable en backend pero no en kiosk (filtro company solo en kiosk) | frontdesk | FDG19 | enterprise/frontdesk/gotchas.md |
| create con state=checked_in NO notifica ni rellena check_in en v18 (solo write) | frontdesk | FDG20 | enterprise/frontdesk/gotchas.md |
| Cambiar visitor_properties_definition de la estación deja properties huérfanas en visitas viejas | frontdesk | FDG21 | enterprise/frontdesk/gotchas.md |
| No puedo crear un drink sin notify_user_ids (required) y ese user recibe Discuss siempre | frontdesk | FDG22 | enterprise/frontdesk/gotchas.md |

---

## 32. ir.cron + QWeb/PDF reporting (eval gap-fill)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Cron activo y fecha pasada pero NO se ejecuta (nextcall congelado, sin log) | cron | CRONG1 | cross-module/cron/gotchas.md |
| La DB no se procesa en este nodo aunque funciona en otro (`--max-cron-threads`/lista DBs) | cron | CRONG2 | cross-module/cron/gotchas.md |
| Cron pasó a `active=False` solo ("deactivated after failing 5 times") | cron | CRONG6 | cross-module/cron/gotchas.md |
| nextcall avanza pero lastcall no se mueve / datos nunca cambian | cron | CRONG7 | cross-module/cron/gotchas.md |
| Llamé a `._trigger()` y no pasó nada | cron | CRONG9 | cross-module/cron/gotchas.md |
| "Record cannot be modified right now" en un cron que no corre (lock) | cron | CRONG13 | cross-module/cron/gotchas.md |
| Tras downtime los crons no recuperan ejecuciones perdidas (no hay catch-up; `numbercall`/`doall` ya no existen) | cron | CRONG8 | cross-module/cron/gotchas.md |
| El cron no se dispara dentro de un test / `_trigger()` no ejecuta en test | cron | CRONG17 | cross-module/cron/gotchas.md |
| `Invalid field 'numbercall'/'doall' on model 'ir.cron'` al instalar/migrar módulo | cron | CRONG18 | cross-module/cron/gotchas.md |
| Automatización por fecha (time-based) se ejecuta tarde (hasta el intervalo del cron compartido) | cron/base_automation | CRONG19 | cross-module/cron/gotchas.md |
| Todas las automatizaciones por tiempo dejan de correr a la vez (cron base_automation borrado/inactivo) | cron/base_automation | CRONG20 | cross-module/cron/gotchas.md |
| Email saliente atascado en "outgoing" / correo entrante no se importa | cron/mail | CRONG21 | cross-module/cron/integrations.md |
| Jobs `with_delay()` (OCA queue_job) atascados en PENDING para siempre (jobrunner ≠ cron) | cron/queue_job | CRONG22 | cross-module/cron/gotchas.md |
| queue_job no procesa (capacity 0 / URL HTTP callback inalcanzable) | cron/queue_job | CRONG23 | cross-module/cron/gotchas.md |
| ¿Qué cron mueve X? (email queue, fetchmail, scheduler, auto-post, subs, abandoned cart, autovacuum) | cron | §7 índice | cross-module/cron/integrations.md |
| PDF renderiza distinto que el preview del navegador (flex/grid fallan en wkhtmltopdf/WebKit) | reporting | RPTG1 | cross-module/reporting/gotchas.md |
| Informe sale en BLANCO con paperformat custom (márgenes/header_spacing) | reporting | RPTG2 | cross-module/reporting/gotchas.md |
| Header/footer se repiten o desaparecen en el PDF | reporting | RPTG3 | cross-module/reporting/gotchas.md |
| CSS custom no aplica en el PDF (sin estilo — falta `t-call-assets`/bundle) | reporting | RPTG6 | cross-module/reporting/gotchas.md |
| PDF colgado/timeout al imprimir / "unpatched QT" multi-documento | reporting | RPTG17/RPTG22 | cross-module/reporting/gotchas.md |
| Salto de página parte una fila/bloque a mitad | reporting | RPTG14 | cross-module/reporting/gotchas.md |
| PDF no se traduce al idioma del cliente (sale en idioma del usuario) | reporting | RPTG23 | cross-module/reporting/gotchas.md |
| Imágenes/QR en blanco en PDF de email/cron (sin cookie de sesión) | reporting | RPTG19 | cross-module/reporting/gotchas.md |
| Editas el SQL de un report de análisis y "no cambia nada" (vista PG vieja) | reporting | RPTG-SQL1 | cross-module/reporting/gotchas.md |
| "column must appear in the GROUP BY clause" al extender sale.report/etc | reporting | RPTG-SQL2 | cross-module/reporting/gotchas.md |
| Report de análisis: filas duplicadas/perdidas (id colisiona) | reporting | RPTG-SQL3 | cross-module/reporting/gotchas.md |
| Report `_table_query` lee datos viejos en la misma transacción (falta `_depends`) | reporting | RPTG-SQL4 | cross-module/reporting/gotchas.md |
| `read_group(lazy=True)` solo agrupa por el primer campo | reporting | RPTG-BI1 | cross-module/reporting/gotchas.md |
| KeyError `__count` con un solo groupby lazy (es `<field>_count`) | reporting | RPTG-BI2 | cross-module/reporting/gotchas.md |
| Campo computed sin store=True no aparece como medida en pivot/graph | reporting | RPTG-BI3 | cross-module/reporting/gotchas.md |
| `group_operator` deprecado → usar `aggregator` (v18) | reporting | RPTG-BI4 | cross-module/reporting/gotchas.md |
| Pivot SUMA un precio unitario/% y da número absurdo (falta aggregator='avg') | reporting | RPTG-BI5 | cross-module/reporting/gotchas.md |
| Pivot grande tarda al "expandir todo" (N read_groups) | reporting | RPTG-BI7 | cross-module/reporting/gotchas.md |
| account.report existe en BD pero no calcula (instancia solo-CE, motor en EE) | reporting | RPTG-AR1 | cross-module/reporting/gotchas.md |
| account.report engine `domain` da IntegrityError (subformula requerida) | reporting | RPTG-AR2 | cross-module/reporting/gotchas.md |
| v19: read_group deprecada → `_read_group`/`formatted_read_group` (sin `lazy`, args reordenados) | reporting | v19 | cross-module/reporting/v19_changes.md |
| ORM precompute / @api.depends / related store=True / métodos compute deprecados en v19 | engine (delegado) | — | skill `odoo-version-api-reference` (v18-base + diffs/v19) |

## 33. Comunicaciones — mailing / voip / discuss_bus (refuerzo L8)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Reporte "Sent" vs "Delivered" del mailing sale invertido (label≠value) | mailing | MMG29 | functional/mailing/gotchas.md |
| Filtro `trace_status='sent'` devuelve los "Delivered", no los "Sent" | mailing | MMG29 | functional/mailing/gotchas.md |
| Override de `add_click` registra clic pero la traza no pasa a "Opened" | mailing | MMG30 | functional/mailing/gotchas.md |
| Mailing masivo commitea cada 50 emails, no cada 1000 | mailing | MMG32 | functional/mailing/gotchas.md |
| Mailing en `sending` tras crash se reintenta solo en el siguiente cron | mailing | MMG34 | functional/mailing/gotchas.md |
| Mailing se renderiza en el idioma del OWNER del mailing, no del que lanza el cron | mailing | MMG35 | functional/mailing/gotchas.md |
| Pongo `opt_out_reason_id` para analítica y el contacto queda dado de baja | mailing | MMG33 | functional/mailing/gotchas.md |
| Email "huérfano" (sin partner) acaba en blacklist solo por conteo de bounces | mailing | MMG4 | functional/mailing/gotchas.md |
| El mailing no añade Return-Path (es la capa SMTP/alias domain) | mailing | MMG36 | functional/mailing/gotchas.md |
| Un user normal se cambió su propio `voip_secret`/proveedor SIP | VoIP | VOIPG27 | enterprise/voip/gotchas.md |
| `voip.provider` sin company_id visible para todos los usuarios | VoIP | VOIPG28 | enterprise/voip/gotchas.md |
| Staging no marca llamadas (neutralize.sql fuerza demo mode) | VoIP | VOIPG30 | enterprise/voip/gotchas.md |
| voip_ai v19 GUARDA el audio crudo `.ogg` (riesgo GDPR) | VoIP | VOIPG22/26 | enterprise/voip/gotchas.md |
| Auto-reject (SIP 488) vs decline manual (603) confundidos | VoIP | VOIPG24 | enterprise/voip/gotchas.md |
| Red con NAT estricto pierde candidatos ICE (cap 1s) | VoIP | VOIPG31 | enterprise/voip/gotchas.md |
| Leí bus_bus tras `_sendone()` y está vacío (defer hasta commit) | discuss_bus | BUSG1 | cross-module/discuss_bus/gotchas.md |
| Handler JS espera el message en la raíz pero llega `{type,payload}` | discuss_bus | BUSG27 | cross-module/discuss_bus/gotchas.md |
| Notificaciones bus no llegan con pgbouncer transaction mode | discuss_bus | BUSG28 | cross-module/discuss_bus/gotchas.md |
| Tabla `bus_bus` crece sin parar (autovacuum/cron parado) | discuss_bus | BUSG29 | cross-module/discuss_bus/gotchas.md |
| Segundo `subscribe()` borra los canales del primero | discuss_bus | BUSG6 | cross-module/discuss_bus/gotchas.md |
| Deploy que sube versión NO provoca tormenta de reconexiones (clean close) | discuss_bus | BUSG30 | cross-module/discuss_bus/gotchas.md |
| Conteo de "conexiones websocket" no es por tab (SharedWorker) | discuss_bus | BUSG31 | cross-module/discuss_bus/gotchas.md |
| `_bus_send` revienta con `ensure_one()` si `_bus_channel` devuelve multi/empty | discuss_bus | BUSG26 | cross-module/discuss_bus/gotchas.md |
| Longpolling devuelve datos pero el usuario aparece offline a los 65s | discuss_bus | BUSG12 | cross-module/discuss_bus/gotchas.md |
| v19: `bus.presence` eliminado, `im_status` ya no en bus res.partner/res.users | discuss_bus | v19 | cross-module/discuss_bus/v19_changes.md |
| Lead enrichment consume créditos IAP en cada lead nuevo (auto) | lead-intel | ETG34 | enterprise/enterprise_tools/gotchas.md |
| Desactivar el cron no para el auto-enrich (dos switches) | lead-intel | ETG35 | enterprise/enterprise_tools/gotchas.md |
| Lead enriquecido con datos viejos (enrich no sobreescribe) | lead-intel | ETG36 | enterprise/enterprise_tools/gotchas.md |
| Website reveal no crea leads (logueado / sin GeoIP) | lead-intel | ETG40 | enterprise/enterprise_tools/gotchas.md |

## Sign (Enterprise)

| Symptom | Section | Gotcha ID | File |
|---|---|---|---|
| No puedo editar los campos del template de firma (ni como admin) | sign | SNG1 | enterprise/sign/gotchas.md |
| No me deja borrar el template de firma | sign | SNG1 | enterprise/sign/gotchas.md |
| "You cannot reassign this signatory" al cambiar el firmante | sign | SNG2 | enterprise/sign/gotchas.md |
| "Signed documents cannot be deleted for legal reasons" | sign | SNG3 | enterprise/sign/gotchas.md |
| No puedo borrar ni modificar registros de sign.log | sign | SNG3 | enterprise/sign/gotchas.md |
| Tras cancelar, los links de descarga del PDF firmado dan 404 | sign | SNG4 | enterprise/sign/gotchas.md |
| Archivar una request en curso la cancela e invalida los tokens | sign | SNG5 | enterprise/sign/gotchas.md |
| Documento firmado sin el código SMS que se suponía obligatorio | sign | SNG6 | enterprise/sign/gotchas.md |
| El link de firma usa el dominio equivocado en multi-website | sign | SNG7 | enterprise/sign/gotchas.md |
| "Link expirado" pero la request sigue válida | sign | SNG8 | enterprise/sign/gotchas.md |
| Migrar v18→v19 alarga la vida del link de 48h a 360h | sign | SNG8 | enterprise/sign/gotchas.md |
| Importar requests con validez antigua las marca expiradas al instante | sign | SNG9 | enterprise/sign/gotchas.md |
| "Malformed expression" al configurar auto_field del tipo de campo | sign | SNG10 | enterprise/sign/gotchas.md |
| Se firmó sin pedir el OTP / código SMS predecible o sin caducar | sign OTP | SNG18 | enterprise/sign/gotchas.md |
| PDF firmado en blanco / pide password (PDF cifrado) | sign | SNG23 | enterprise/sign/gotchas.md |
| El 2º firmante nunca recibe el email (orden de firma secuencial) | sign orden | SNG25 | enterprise/sign/gotchas.md |
| auto_field no autorrellena desde el reference_doc (solo partner) | sign | SNG10 | enterprise/sign/gotchas.md |
| Error de selección con auth_method='itsme' | sign | SNG11 | enterprise/sign/gotchas.md |
| AccessError al leer sign.request.item.value sin sudo | sign | SNG12 | enterprise/sign/gotchas.md |
| Firma completada pero no aparece nota ni PDF en el record enlazado | sign | SNG13 | enterprise/sign/gotchas.md |
| Filtro state='refused' no devuelve nada en v18 | sign | SNG14 | enterprise/sign/gotchas.md |
| Las requests compartidas expiradas se acumulan y no se borran | sign | SNG15 | enterprise/sign/gotchas.md |
| Un sign user no ve IP/geolocalización en el audit log | sign | SNG16 | enterprise/sign/gotchas.md |
| El PDF firmado no tiene firma criptográfica X.509 (v18) | sign | core §5 | enterprise/sign/core.md |
| No encuentro firmas de un empleado por reference_doc | sign | integrations | enterprise/sign/integrations.md |
| Artículo "privado" aparece como "Compartido" sin tocarlo | knowledge (EE) | KNG1 | enterprise/knowledge/gotchas.md |
| Bajar un member a 'read' no quita el write a un usuario interno | knowledge (EE) | KNG2 | enterprise/knowledge/gotchas.md |
| Member con 'No access' degrada a un interno que tenía write | knowledge (EE) | KNG2 | enterprise/knowledge/gotchas.md |
| Cambiar permiso del padre recalcula todo el subárbol (lento) | knowledge (EE) | KNG6 | enterprise/knowledge/gotchas.md |
| Error al guardar artículo "debe tener un editor" / no puedo quitar el último write | knowledge (EE) | KNG5 | enterprise/knowledge/gotchas.md |
| Borrar una página de categoría borró todos sus hijos | knowledge (EE) | KNG7 | enterprise/knowledge/gotchas.md |
| Mover un artículo le cambió el permiso (public/private) | knowledge (EE) | KNG8 | enterprise/knowledge/gotchas.md |
| No puedo mover un artículo bajo un Article Item | knowledge (EE) | KNG9 | enterprise/knowledge/gotchas.md |
| Item pierde su stage al cambiar de padre | knowledge (EE) | KNG11 | enterprise/knowledge/gotchas.md |
| Propiedades del item se pierden al reparentar | knowledge (EE) | KNG12 | enterprise/knowledge/gotchas.md |
| `to_delete=True` por RPC lanza error de constraint | knowledge (EE) | KNG17 | enterprise/knowledge/gotchas.md |
| La papelera no se vacía entera de golpe (100/tick) | knowledge (EE) | KNG20 | enterprise/knowledge/gotchas.md |
| `mapped('body')` dispara N+1 / list view rápida pero detalle lento | knowledge (EE) | KNG22 | enterprise/knowledge/gotchas.md |
| Ediciones simultáneas pierden caracteres sin aviso | knowledge (EE) | KNG23 | enterprise/knowledge/gotchas.md |
| Cualquiera con lectura puede comentar el artículo | knowledge (EE) | KNG26 | enterprise/knowledge/gotchas.md |
| Favoritos siguen apuntando a artículos en papelera/archivados | knowledge (EE) | KNG29 | enterprise/knowledge/gotchas.md |
| El cuerpo del artículo no se traduce (multilingüe) | knowledge (EE) | KNG32 | enterprise/knowledge/gotchas.md |
| Link de invitación de portal dejó de funcionar tras recrear el member | knowledge (EE) | KNG33 | enterprise/knowledge/gotchas.md |
| Publicar un artículo en web expuso todo su subárbol | knowledge (EE) | KNG34 | enterprise/knowledge/gotchas.md |
| Import `web_editor.tools.handle_history_divergence` rompe en v19 | knowledge (EE) | KNG35 | enterprise/knowledge/v19_changes.md |
| Embedded view muestra placeholder de error (modelo/acción borrada) | knowledge (EE) | core §embedded | enterprise/knowledge/core.md |



---

## 34. Fleet — vehicles, drivers, contracts, odometer, costs

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Leasing contract expired and nobody got a renewal / no new contract created | fleet | FLG1 | functional/fleet/gotchas.md |
| Recurring contract cost never appears in the GL / journal entries | fleet | FLG2 | functional/fleet/gotchas.md |
| Odometer reading looks wrong after switching km↔miles | fleet | FLG3 | functional/fleet/gotchas.md |
| "Odometer value cannot be lower than the previous one" UserError | fleet | FLG4 | functional/fleet/gotchas.md |
| Group-by on odometer value returns max instead of sum | fleet | FLG5 | functional/fleet/gotchas.md |
| Current odometer stuck on an old too-high reading | fleet | FLG6 | functional/fleet/gotchas.md |
| Service created with odometer 0 has no odometer row | fleet | FLG7 | functional/fleet/gotchas.md |
| Future-dated contract shows as Running, not New | fleet | FLG8 | functional/fleet/gotchas.md |
| Changing model_id wiped my per-vehicle CO2/doors/color overrides | fleet | FLG9 | functional/fleet/gotchas.md |
| Model swap left stale spec values on the vehicle | fleet | FLG10 | functional/fleet/gotchas.md |
| Driver history shows two drivers overlapping on one vehicle | fleet | FLG11 | functional/fleet/gotchas.md |
| Old service log still shows previous driver as purchaser | fleet | FLG12 | functional/fleet/gotchas.md |
| Cost dashboard inflated by not-yet-performed services | fleet | FLG13 | functional/fleet/gotchas.md |
| Yearly cost spikes on one month in the cost report | fleet | FLG14 | functional/fleet/gotchas.md |
| Driver waiting for a car not flagged for payroll | fleet | FLG15 | functional/fleet/gotchas.md |
| Driver visible to manager but not to the driver user | fleet | FLG17 | functional/fleet/gotchas.md |
| Duplicating a vehicle state raises unique-constraint error | fleet | FLG18 | functional/fleet/gotchas.md |
| `delay_alert_contract` param not found searching `fleet.*` | fleet | FLG19 | functional/fleet/gotchas.md |
| Smart-button count differs from records shown on click | fleet | FLG21 | functional/fleet/gotchas.md |
| Fleet manager sees bill_count = 0 despite real bills | fleet | FLG22 | functional/fleet/gotchas.md |
| Restored an archived vehicle but contracts/services stay hidden | fleet | FLG23 | functional/fleet/gotchas.md |
| Bulk future-driver assign set wrong plan_to_change flag (mixed car/bike) | fleet | FLG24 | functional/fleet/gotchas.md |
| "Contracts to renew" filter hides already-expired contracts | fleet | FLG25 | functional/fleet/gotchas.md |
| Overdue contract dropped off the overdue list after creating replacement | fleet | FLG26 | functional/fleet/gotchas.md |
| contract_state on vehicle flips between recomputes | fleet | FLG27 | functional/fleet/gotchas.md |
| Editing a contract's expiration didn't refresh vehicle renewal flags | fleet | FLG28 | functional/fleet/gotchas.md |
| CO2 column blank in grouped list; odometer "total" is a max | fleet | FLG29 | functional/fleet/gotchas.md |
| Fleet manager edited a partner flag they shouldn't be able to (v18 sudo) | fleet | FLG30 | functional/fleet/gotchas.md |
| Two odometer rows created same day; lower correction ignored | fleet | FLG31 | functional/fleet/gotchas.md |
| Contract "Driver" rewrites historically but service "Driver" stays frozen | fleet | FLG32 | functional/fleet/gotchas.md |
| Removing vehicle_id from a posted bill deleted the cost log | fleet (account_fleet) | FLE-A | functional/fleet/enterprise.md |
| Same bill produces different fleet cost depending on installed modules | fleet (account_asset_fleet) | FLE-B | functional/fleet/enterprise.md |
| Manual JE on car account skips disallowance / vehicle not required | fleet (disallowed) | FLE-C | functional/fleet/enterprise.md |
| Inspection PDFs vanish from My Documents after driver change | fleet (documents_fleet) | FLE-D | functional/fleet/enterprise.md |
| Wrong company's employee bound to assignation log (multi-company) | fleet (hr_fleet) | FLE-E | functional/fleet/enterprise.md |
| User can't edit own mobility card from public profile | fleet (hr_fleet) | FLE-F | functional/fleet/enterprise.md |
| Reinstall order flipped service amounts (account_fleet vs asset) | fleet (MRO) | FLE-G | functional/fleet/enterprise.md |
| Bought a car via Purchase but it didn't appear in Fleet | fleet (purchase) | integ §3 | functional/fleet/integrations.md |
| Changing employee work address silently reassigned their car | fleet (hr_fleet) | integ §1 | functional/fleet/integrations.md |
| Delivery batch driver didn't update when vehicle driver changed | fleet (stock_fleet) | integ §4 | functional/fleet/integrations.md |
| v19: code referencing `first_contract_date` breaks | fleet/v19 | v19 §1 | functional/fleet/v19_changes.md |
| v19: `partner.plan_to_change_car` no longer exists | fleet/v19 | v19 §2 | functional/fleet/v19_changes.md |
| v19: model_year now a Selection string, integer code breaks | fleet/v19 | v19 §3 | functional/fleet/v19_changes.md |
| v19: override of `_compute_model_fields` no longer fires | fleet/v19 | v19 §8 | functional/fleet/v19_changes.md |
| v19: fleet_group_user no longer narrowed to own car (rules removed) | fleet/v19 | v19 §16 | functional/fleet/v19_changes.md |
| Lunch wallet went negative / refund with no accounting trail | lunch | LUG1 | functional/lunch/gotchas.md |
| Lunch order balance check passes despite negative wallet | lunch | LUG2 | functional/lunch/gotchas.md |
| Two identical lunch orders created but only one row with qty 2 | lunch | LUG4 | functional/lunch/gotchas.md |
| v19: writing toppings on a sent/confirmed lunch order merges/deactivates it | lunch | LUG5 | functional/lunch/gotchas.md |
| Can't delete an ordered/sent/confirmed lunch order from UI | lunch | LUG6 | functional/lunch/gotchas.md |
| Reorder added item after supplier deadline | lunch | LUG7 | functional/lunch/gotchas.md |
| Deleting supplier cron breaks supplier / module update doesn't recreate it | lunch | LUG11 | functional/lunch/gotchas.md |
| Supplier auto-email never sent the same day it was created | lunch | LUG12 | functional/lunch/gotchas.md |
| Toggling a supplier weekday cancelled next week's orders | lunch | LUG13 | functional/lunch/gotchas.md |
| Lunch "12 PM" send time behaves as end of day, not noon | lunch | LUG22 | functional/lunch/gotchas.md |
| Supplier auto-email silently never arrives (no error) | lunch | LUG23 | functional/lunch/gotchas.md |
| Changing supplier weekdays didn't resync the auto-email cron | lunch | LUG24 | functional/lunch/gotchas.md |
| Survey max score wrong after switching question type | survey | SVG1 | functional/survey/gotchas.md |
| Survey scores below 0% / negative penalty shows as 0% | survey | SVG2 | functional/survey/gotchas.md |
| Scored choice question always returns 100% | survey | SVG5 | functional/survey/gotchas.md |
| Raising pass threshold didn't fail past attempts | survey | SVG4 | functional/survey/gotchas.md |
| Conditional question stopped showing after reordering questions | survey | SVG6 | functional/survey/gotchas.md |
| Certification threshold unreachable because of a hidden conditional | survey | SVG7 | functional/survey/gotchas.md |
| Testing a survey didn't burn an attempt / attempt-limit bypass | survey | SVG10 | functional/survey/gotchas.md |
| Duplicated certification survey stopped issuing badges | survey | SVG19/SVG31 | functional/survey/gotchas.md |
| Editing questions during a live session corrupted attendees | survey | SVG16/SVG28 | functional/survey/gotchas.md |
| Survey time limit not enforced when JS disabled | survey | SVG22 | functional/survey/gotchas.md |
| Can't build a 0-100 scale question (capped at 10) | survey | SVG29 | functional/survey/gotchas.md |
| Survey always requires 80% to pass | survey | SVG30 | functional/survey/gotchas.md |
| Interview form cambia para todos los candidatos al cambiar el del puesto | survey | SVG34 | functional/survey/integrations.md |
| Línea de certificación no aparece en el resume del empleado | survey | SVG38 | functional/survey/integrations.md |
| Certificado de curso no llega / alumno expulsado tras suspender | survey | SVG42 | functional/survey/integrations.md |
| Badge de certificación no se otorga al aprobar | survey | SVG48 | functional/survey/integrations.md |
| Survey duplicado deja de emitir badge (copy=False + UNIQUE) | survey | SVG49 | functional/survey/integrations.md |
| Empleado evaluado no puede ver resultados de su 360º (EE) | survey | SVG53 | functional/survey/integrations.md |
| No deja activar certificación / scoring obligatorio | survey | SUCG3 | functional/survey/config_guide.md |
| Límite de intentos no se aplica en encuesta pública | survey | SUCG6 | functional/survey/config_guide.md |
| Error al combinar "volver atrás" con scoring por página | survey | SUCG2 | functional/survey/config_guide.md |
| Pregunta condicional no aparece / trigger ignorado | survey | SUCG4 | functional/survey/config_guide.md |
| Saltos/condiciones se rompen al duplicar la encuesta | survey | SUCG4 | functional/survey/config_guide.md |
| Web lead with no country never routed to a partner | partnership | PAG4 | functional/partnership/gotchas.md |
| Same lead routes to different partner each call | partnership | PAG5 | functional/partnership/gotchas.md |
| Partner with grade still never receives leads (weight 0) | partnership | PAG6 | functional/partnership/gotchas.md |
| v19: UserError assigning pricelist to a graded partner | partnership | PAG7 | functional/partnership/gotchas.md |
| v19: confirming SO upgraded the wrong (parent) partner's grade | partnership | PAG12 | functional/partnership/gotchas.md |
| v19: cart mixing two grade products fails on confirm | partnership | PAG13 | functional/partnership/gotchas.md |
| v19: product grade_id does nothing on sale | partnership | PAG14 | functional/partnership/gotchas.md |
| v18 grade-to-pricelist custom code breaks in v19 (field renamed) | partnership | PAG15 | functional/partnership/gotchas.md |
| v19 grade pricelists missing — only website_crm_partner_assign installed | partnership | PAG16 | functional/partnership/gotchas.md |
| Partner that declined a lead by mistake excluded from future routing | partnership | PAG19 | functional/partnership/gotchas.md |
| Portal partner gets AccessError editing assigned lead | partnership | PAG10 | functional/partnership/gotchas.md |

## Ecosystem — Indaws suite (instalar/migrar/codificar sobre indaws_core)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| Instalé `sale`/`account` y se instaló media suite indaws sola | indaws | IG1 | ecosystem/indaws/gotchas.md |
| No puedo instalar Odoo vanilla — los indaws_* se cuelan por auto_install | indaws | IG1 | ecosystem/indaws/integrations.md |
| Cambié el order type y el SO se renumeró solo | indaws | IG23 | ecosystem/indaws/gotchas.md |
| La factura sale con un journal que no esperaba (lo fuerza el order type) | indaws | IG24 | ecosystem/indaws/gotchas.md |
| `sale.order.line.discount` no es el descuento efectivo (es el tramo 1) | indaws | IG25 | ecosystem/indaws/gotchas.md |
| En factura el `discount` sí es el agregado pero en SO no — incoherencia | indaws | IG25 | ecosystem/indaws/integrations.md |
| Tres descuentos del 10% no dan 30% (multiplicativo) | indaws | IG2 | ecosystem/indaws/gotchas.md |
| Import de datos con discount >100% peta en BD (SQL CHECK) | indaws | IG26 | ecosystem/indaws/gotchas.md |
| Raw SQL/bulk write corrompió la jerarquía de líneas de factura (parent_path) | indaws | IG27 | ecosystem/indaws/gotchas.md |
| `cost_summary_ids` lento y sus IDs cambian en cada recompute | indaws | IG28 | ecosystem/indaws/gotchas.md |
| No veo el desglose de coste/margen (falta `group_sale_advanced_costs`) | indaws | IG28 | ecosystem/indaws/integrations.md |
| Factura desde picking no aparece en `sale.order.invoice_ids` | indaws | IG3/IG29 | ecosystem/indaws/gotchas.md |
| Rompí `invoice_stock_move_id` y se perdió el link picking↔factura | indaws | IG29 | ecosystem/indaws/integrations.md |
| El rappel no cuenta una factura (estaba en borrador) / las notas de crédito restan | indaws | IG30 | ecosystem/indaws/gotchas.md |
| Settlement de rappel generó una nota de crédito inesperada | indaws | IG4/IG30 | ecosystem/indaws/gotchas.md |
| Guardar un res.partner dispara algo raro (export monkeypatch) | indaws | IG31 | ecosystem/indaws/integrations.md |
| Busco `import_record.py` y no existe (god class = `indaws.import`) | indaws | IG32 | ecosystem/indaws/gotchas.md |
| Imports async atascados (falta runner OCA queue_job) | indaws | IG8/IG32 | ecosystem/indaws/gotchas.md |
| Borré un scheduler de import y desapareció su ir.cron | indaws | IG32 | ecosystem/indaws/integrations.md |
| `dynamic_invoice_section` ejecuta Python arbitrario en la company | indaws | IG33 | ecosystem/indaws/gotchas.md |
| Composición de producto no es BOM pero coexisten en el mismo producto | indaws | IG7 | ecosystem/indaws/gotchas.md |
| POS sí respeta FEFO al instalar lot_auto_selection | indaws | IG5 | ecosystem/indaws/gotchas.md |
| v19: módulo indaws v18 no existe en v19 (~84 pendientes) | indaws/v19 | IGV19-3 | ecosystem/indaws/v19_changes.md |
| v19: `_sql_constraints` se pierde en silencio (UNIQUE/CHECK) | indaws/v19 | IGV19-1 | ecosystem/indaws/v19_changes.md |
| v19: tras migrar, `sale.order.line.discount` cambia de significado (→ agregado) | indaws/v19 | IGV19-5 | ecosystem/indaws/v19_changes.md |
| v19: schedulers import/export no se migran solos al nuevo `indaws_dataflow` | indaws/v19 | IGV19-4 | ecosystem/indaws/v19_changes.md |

## Ecosystem — OCA accounting (account-*, bank-payment, l10n-spain)

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| `account_asset_management` no instala / choca con assets Enterprise | oca-accounting | OAG20 | ecosystem/oca-accounting/gotchas.md |
| Las amortizaciones no se contabilizan solas (cron disabled de fábrica) | oca-accounting | OAG21 | ecosystem/oca-accounting/gotchas.md |
| `account_reconcile_oca` y `account_accountant` se pisan (mismo widget) | oca-accounting | integ §3 | ecosystem/oca-accounting/integrations.md |
| Conciliación OCA peta con ZeroDivisionError (línea FX importe 0) | oca-accounting | OAG1 | ecosystem/oca-accounting/gotchas.md |
| La regla de conciliación "no funciona" y no da error (unique_matching) | oca-accounting | OAG5 | ecosystem/oca-accounting/gotchas.md |
| SEPA DD: el banco rechaza por FRST duplicado tras reintentar upload | oca-accounting | OAG14 | ecosystem/oca-accounting/gotchas.md |
| Fichero SEPA inválido para el banco (versión PAIN incorrecta) | oca-accounting | OAG18 | ecosystem/oca-accounting/gotchas.md |
| Un mandato SEPA "murió" solo (cron de expiración a 36 meses) | oca-accounting | OAG17 | ecosystem/oca-accounting/gotchas.md |
| Re-confirmar la orden de pago borró los account.payment que metí a mano | oca-accounting | OAG6 | ecosystem/oca-accounting/gotchas.md |
| `generate_payment_file` no genera nada (falta el módulo SEPA correcto) | oca-accounting | OAG7 | ecosystem/oca-accounting/gotchas.md |
| Orden de pago agrupada usa la cuenta destino de la primera línea | oca-accounting | OAG9 | ecosystem/oca-accounting/gotchas.md |
| Devolución de pago: factura sigue "pagada" tras el bounce | oca-accounting | OAG28 | ecosystem/oca-accounting/gotchas.md |
| El General Ledger no muestra los asientos en borrador (target=posted) | oca-accounting | OAG30 | ecosystem/oca-accounting/gotchas.md |
| Balance/Trial Balance multi-divisa con números mezclados | oca-accounting | OAG31/32 | ecosystem/oca-accounting/gotchas.md |
| CAMT multi-cuenta importa con la divisa/IBAN equivocados (la última) | oca-accounting | OAG35 | ecosystem/oca-accounting/gotchas.md |
| `account_analytic_required` bloquea AL postear (rollback), no antes | oca-accounting | OAG46 | ecosystem/oca-accounting/gotchas.md |
| Accrual/cutoff con precio del último abono, no la media | oca-accounting | OAG41 | ecosystem/oca-accounting/gotchas.md |
| ¿OCA o Indaws para SII/Verifactu? (Indaws hace bridge SOBRE OCA) | oca-accounting | integ §4/§7 | ecosystem/oca-accounting/integrations.md |
| v19: cliente español pierde mod303/SII/Verifactu/Facturae (no portados) | oca-accounting/v19 | — | ecosystem/oca-accounting/v19_changes.md |
| v19: CE pierde el widget de conciliación (`account_reconcile_oca` GONE) | oca-accounting/v19 | — | ecosystem/oca-accounting/v19_changes.md |

## Ecosystem — Spain Veri\*Factu (`l10n_es_edi_verifactu` CE + POS)

Núcleo en `ecosystem/l10n_es/verifactu.md`; gotchas ESG41-55 en `ecosystem/l10n_es/gotchas.md`.

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| No puedo borrar un registro Veri\*Factu (UserError al unlink) | l10n_es/verifactu | ESG41 | ecosystem/l10n_es/gotchas.md |
| Cancelar una factura Veri\*Factu no la elimina, crea una anulación | l10n_es/verifactu | ESG42 | ecosystem/l10n_es/gotchas.md |
| Hueco / salto en la secuencia de cadena (`chain_index`) | l10n_es/verifactu | ESG43 | ecosystem/l10n_es/gotchas.md |
| Contención / bloqueo al postear varias facturas a la vez (no_gap) | l10n_es/verifactu | ESG44 | ecosystem/l10n_es/gotchas.md |
| La AEAT rechaza en cascada toda la cola de registros | l10n_es/verifactu | ESG45 | ecosystem/l10n_es/gotchas.md |
| Veri\*Factu "funciona" pero no presenta nada real (test_environment default) | l10n_es/verifactu | ESG46 | ecosystem/l10n_es/gotchas.md |
| Postear factura falla sin certificado de scope verifactu | l10n_es/verifactu | ESG47 | ecosystem/l10n_es/gotchas.md |
| El registro no es inmediato al postear (envío por lotes + espera AEAT) | l10n_es/verifactu | ESG48 | ecosystem/l10n_es/gotchas.md |
| Timeout de red → posible duplicado en AEAT, no reenviar a mano | l10n_es/verifactu | ESG49 | ecosystem/l10n_es/gotchas.md |
| `registered_with_errors`: ¿está presentado o hay que reintentar? | l10n_es/verifactu | ESG50 | ecosystem/l10n_es/gotchas.md |
| El QR del ticket no valida en la web de la AEAT (era entorno test) | l10n_es/verifactu | ESG51 | ecosystem/l10n_es/gotchas.md |
| POS: un documento encadenado por cada ticket cobrado | l10n_es/verifactu | ESG52 | ecosystem/l10n_es/gotchas.md |
| No puedo cancelar/anular un ticket de POS Veri\*Factu | l10n_es/verifactu | ESG53 | ecosystem/l10n_es/gotchas.md |
| Lote a medias deja facturas ya presentadas (commit por documento) | l10n_es/verifactu | ESG54 | ecosystem/l10n_es/gotchas.md |
| Veri\*Factu no envía la factura ni justificante al cliente | l10n_es/verifactu | ESG55 | ecosystem/l10n_es/gotchas.md |

## Calendar (`calendar`) — síntomas

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Al guardar una ocurrencia recurrente salta "Unable to save the recurrence with This Event" | CALG1 | functional/calendar/gotchas.md |
| Una integración escribe campos rrule y se queda en self_only / falla | CALG2 | functional/calendar/gotchas.md |
| Editar el evento entero borra y recrea toda la serie (mucho ruido a Google/Outlook) | CALG3 | functional/calendar/gotchas.md |
| Cambiar la hora de una sola ocurrencia la desvincula de la serie | CALG4 | functional/calendar/gotchas.md |
| Editar "eventos futuros" corta la serie y crea una recurrencia nueva | CALG5 | functional/calendar/gotchas.md |
| Cambiar la hora pone a todos los asistentes en needsAction | CALG6 | functional/calendar/gotchas.md |
| Si otro usuario mueve la reunión, el organizador aparece como needsAction | CALG7 | functional/calendar/gotchas.md |
| La alarma de un evento recurrente no se vuelve a programar | CALG8 | functional/calendar/gotchas.md |
| No me llega el recordatorio de una alarma en el pasado | CALG9 | functional/calendar/gotchas.md |
| Eventos duplicados o desplazados al cambiar la hora (horario de verano/DST) | CALG10 | functional/calendar/gotchas.md |
| Al duplicar un evento se pierden los asistentes | CALG11 | functional/calendar/gotchas.md |
| Borro el evento base y la serie no desaparece | CALG12 | functional/calendar/gotchas.md |
| Cada ocurrencia tiene un enlace de videollamada distinto | CALG13 | functional/calendar/gotchas.md |
| Veo "Busy" en lugar del nombre de un evento de otro / no puedo editar eventos privados ajenos | CALG14 | functional/calendar/gotchas.md |
| Una actividad de tipo reunión no sincroniza con el evento de calendario | CALG15 | functional/calendar/integrations.md |
| Las invitaciones no incluyen el .ics o no me llega copia a mí mismo | CALG18 | functional/calendar/integrations.md |
| El enlace público de calendario por token no abre / sesión no coincide | CALG20 | functional/calendar/integrations.md |

## Payroll (`hr_payroll`) — síntomas

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Recalcular la nómina pierde las líneas editadas a mano | PYG1 | enterprise/hr_payroll/gotchas.md |
| Marqué la nómina como editada pero compute_sheet me la pisa igual | PYG2 | enterprise/hr_payroll/gotchas.md |
| Cambio los inputs y las líneas no se recalculan solas | PYG3 | enterprise/hr_payroll/gotchas.md |
| Una regla salarial no aplica por un typo en el código (no da error) | PYG4 | enterprise/hr_payroll/gotchas.md |
| El error de una regla salarial sale como UserError y oculta el traceback real | PYG5 | enterprise/hr_payroll/gotchas.md |
| Cambié un parámetro de regla por SQL y la nómina usa el valor antiguo (caché) | PYG6 | enterprise/hr_payroll/gotchas.md |
| Qué valor de parámetro aplica para una fecha dada / no hay date_to | PYG7 | enterprise/hr_payroll/gotchas.md |
| No puedo cancelar una nómina en estado done | PYG8 | enterprise/hr_payroll/gotchas.md |
| Al pagar, el embargo/salary attachment se descuenta de forma inesperada | PYG9 | enterprise/hr_payroll/gotchas.md |
| Dos inputs con el mismo código se suman en uno solo | PYG10 | enterprise/hr_payroll/gotchas.md |
| date_from/date_to de la nómina se recortan al fin del contrato | PYG11 | enterprise/hr_payroll/gotchas.md |
| Sumas históricas de nómina ignoran las slips en verify | PYG13 | enterprise/hr_payroll/gotchas.md |
| No puedo confirmar la nómina porque el contrato está fuera de periodo o cancelado | PYG14 | enterprise/hr_payroll/gotchas.md |
| El cálculo de nómina se bloquea por huecos en las work entries (undefined slots) | PYG17 | enterprise/hr_payroll/integrations.md |
| Las ausencias no se difieren bien a la nómina del mes siguiente | PYG18 | enterprise/hr_payroll/integrations.md |
| Los embargos del empleado no aparecen como inputs en la nómina | PYG20 | enterprise/hr_payroll/integrations.md |

## Appraisal (`hr_appraisal`) — síntomas

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| El cron crea evaluaciones directamente en pending y manda los mails sin avisar | APRG1 | enterprise/hr_appraisal/gotchas.md |
| next_appraisal_date no se calcula si el empleado ya tiene una evaluación en curso | APRG2 | enterprise/hr_appraisal/gotchas.md |
| No me deja poner la fecha de próxima evaluación en el pasado | APRG3 | enterprise/hr_appraisal/gotchas.md |
| El mensaje del plan de evaluación solo se postea una vez | APRG4 | enterprise/hr_appraisal/gotchas.md |
| El feedback aparece como "Unpublished" pero el campo no está protegido por permisos | APRG5 | enterprise/hr_appraisal/gotchas.md |
| El empleado no ve la nota privada ni el rating final de su evaluación | APRG6 | enterprise/hr_appraisal/gotchas.md |
| Un empleado no puede publicar el feedback del manager | APRG7 | enterprise/hr_appraisal/gotchas.md |
| Volver a pending re-oculta los feedbacks que ya estaban publicados | APRG8 | enterprise/hr_appraisal/gotchas.md |
| No puedo borrar una evaluación que no esté en new/cancel | APRG9 | enterprise/hr_appraisal/gotchas.md |
| Cancelar una evaluación borra reuniones y actividades | APRG10 | enterprise/hr_appraisal/gotchas.md |
| Archivar al empleado no cancela sus evaluaciones abiertas | APRG11 | enterprise/hr_appraisal/gotchas.md |
| La baja del empleado no canceló las evaluaciones futuras (faltó marcar el check) | APRG12 | enterprise/hr_appraisal/gotchas.md |
| No puedo borrar un empleado que es manager de un objetivo/goal | APRG13 | enterprise/hr_appraisal/gotchas.md |
| No me deja borrar las plantillas de mail de solicitud de evaluación | APRG14 | enterprise/hr_appraisal/gotchas.md |
| Al completar la encuesta 360 no se actualiza el contador / no postea | APRG16 | enterprise/hr_appraisal/integrations.md |
| Las skills del empleado no se vuelcan al cerrar la evaluación | APRG17 | enterprise/hr_appraisal/integrations.md |

## Live Chat (`im_livechat`) — síntomas

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Gana una regla de livechat que no es la más específica (manda el sequence) | LIVG1 | functional/im_livechat/gotchas.md |
| Una regla con país siempre gana sobre una sin país aunque tenga peor sequence | LIVG2 | functional/im_livechat/gotchas.md |
| No se abre el chat porque no hay ningún operador online | LIVG3 | functional/im_livechat/gotchas.md |
| El operador está dado de alta pero no cuenta porque está offline | LIVG4 | functional/im_livechat/gotchas.md |
| El chatbot no salta porque hay un operador disponible | LIVG5 | functional/im_livechat/gotchas.md |
| El chatbot no aparece y la regla se ignora (script inactivo o sin pasos) | LIVG6 | functional/im_livechat/gotchas.md |
| El paso forward_operator no encuentra a nadie y el script sigue sin avisar | LIVG7 | functional/im_livechat/gotchas.md |
| Desaparecen sesiones de livechat vacías (autovacuum) | LIVG8 | functional/im_livechat/gotchas.md |
| El visitante puede sobrescribir la valoración del chat (solo un rating por sesión) | LIVG10 | functional/im_livechat/gotchas.md |
| El matching por país no funciona para visitantes anónimos (falta GeoIP) | LIVG11 | functional/im_livechat/gotchas.md |
| Los mensajes de bienvenida del bot no quedan registrados hasta la primera interacción | LIVG12 | functional/im_livechat/gotchas.md |
| Un paso question_selection sin respuestas rompe el flujo / ValidationError | LIVG13 | functional/im_livechat/gotchas.md |
| La respuesta de email/teléfono del bot no se guarda en el partner | LIVG14 | functional/im_livechat/gotchas.md |
| En el chat livechat aparece el nick del operador en vez de su nombre real | LIVG16 | functional/im_livechat/integrations.md |
| El chatbot no crea el lead/oportunidad de CRM | LIVG18 | functional/im_livechat/integrations.md |
| No existe create_ticket de helpdesk desde chat en CE (solo Enterprise) | LIVG19 | functional/im_livechat/integrations.md |

## PLM (`mrp_plm`) — síntomas

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Al aplicar la ECO la BoM anterior se desactiva sola | PLMG1 | enterprise/mrp_plm/gotchas.md |
| Mi ECO saltó a rebase porque un compañero aplicó la suya | PLMG2 | enterprise/mrp_plm/gotchas.md |
| La etapa no bloquea aunque tenga aprobaciones (eran optional/comment) | PLMG3 | enterprise/mrp_plm/gotchas.md |
| La ECO se da por aprobada aunque el aprobador opcional no hizo nada | PLMG4 | enterprise/mrp_plm/gotchas.md |
| Al cambiar de etapa se pierden las aprobaciones pendientes | PLMG5 | enterprise/mrp_plm/gotchas.md |
| No me deja saltar una etapa bloqueante intermedia (UserError) | PLMG6 | enterprise/mrp_plm/gotchas.md |
| Puedo retroceder de etapa sin que valide las aprobaciones | PLMG7 | enterprise/mrp_plm/gotchas.md |
| Editar la BoM activa con una ECO abierta genera líneas de rebase | PLMG8 | enterprise/mrp_plm/gotchas.md |
| La ECO se marca en conflicto al diferir la línea de rebase de la nueva BoM | PLMG9 | enterprise/mrp_plm/gotchas.md |
| El número de versión de la BoM cambia al marcar/desmarcar will_update_version | PLMG10 | enterprise/mrp_plm/gotchas.md |
| Una ECO de tipo producto ignora todos los cambios de BoM | PLMG11 | enterprise/mrp_plm/gotchas.md |
| Las MOs iniciadas o hechas se quedan con la BoM vieja tras la ECO | PLMG12 | enterprise/mrp_plm/gotchas.md |
| No se ofrece actualizar la BoM en la MO porque la BoM sigue activa | PLMG13 | enterprise/mrp_plm/gotchas.md |
| Comentar en el chatter aprueba mi aprobación tipo comment sin botón | PLMG14 | enterprise/mrp_plm/gotchas.md |
| Los cambios de quality checks no se trackean en la ECO (hook vacío) | PLMG17 | enterprise/mrp_plm/integrations.md |
| Al crear aprobaciones se genera una actividad por cada usuario requerido | PLMG20 | enterprise/mrp_plm/integrations.md |

---

## queue_job (OCA) — jobs asíncronos (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| Los jobs se quedan en `pending`/`enqueued` para siempre, sin error | QJG1 | ecosystem/queue_job/gotchas.md |
| El import/export de Indaws no avanza / se cuelga a mitad (motor async mudo) | QJG1 | ecosystem/queue_job/integrations.md |
| Instalé queue_job pero el runner no arranca (falta en server_wide_modules/--load) | QJG1 | ecosystem/queue_job/gotchas.md |
| El log dice "jobrunner NOT started, root channel capacity 0" | QJG2 | ecosystem/queue_job/gotchas.md |
| Jobs encolados durante un `-u`/`-i` (update) no se ejecutan | QJG3 | ecosystem/queue_job/gotchas.md |
| Jobs `enqueued` que no pasan a `started` (runner no alcanza el HTTP de Odoo) | QJG4 | ecosystem/queue_job/gotchas.md |
| `with_delay` ejecuta síncrono / no crea el job (QUEUE_JOB__NO_DELAY o ctx) | QJG5 | ecosystem/queue_job/gotchas.md |
| Los jobs no se ejecutan dentro de un test | QJG6 | ecosystem/queue_job/gotchas.md |
| Puse max_retries=0 y reintenta infinito | QJG7 | ecosystem/queue_job/gotchas.md |
| identity_key no evita duplicados (el primero ya estaba started) | QJG8 | ecosystem/queue_job/gotchas.md |
| El job hace algo distinto que la llamada directa (perdió claves de contexto) | QJG11 | ecosystem/queue_job/gotchas.md |
| En una cadena de jobs, el padre nunca se ejecuta (delay() en el hijo) | QJG12 | ecosystem/queue_job/gotchas.md |
| El job falla por permisos (corre como el usuario creador, no sudo) | QJG10 | ecosystem/queue_job/gotchas.md |
| Cambié la config de la job.function por SQL y no aplica (ormcache) | QJG17 | ecosystem/queue_job/gotchas.md |
| No puedo ordenar/filtrar queue.job por create_date (no existe, _log_access=False) | QJG14 | ecosystem/queue_job/gotchas.md |
| AccessError al crear/editar queue.job por RPC | QJG15 | ecosystem/queue_job/gotchas.md |
| Los managers reciben emails en cada job fallido | QJG18 | ecosystem/queue_job/gotchas.md |
| La tabla queue_job crece sin control / autovacuum no limpia | QJG20 | ecosystem/queue_job/gotchas.md |
| No puedo forzar a done/cancel un job atascado en `started` | QJG19 | ecosystem/queue_job/gotchas.md |
| En v19 el import/export con _patch_method rompe | v19 §1 | ecosystem/queue_job/v19_changes.md |
| En v19 el runjob no resuelve la BD tras un proxy (header X-Odoo-Database) | v19 §5 | ecosystem/queue_job/v19_changes.md |

---

## mis_builder (OCA) — reporting financiero (síntomas)

| Symptom | Gotcha ID | File |
|---------|-----------|------|
| El KPI muestra la variación del periodo en vez del saldo acumulado | MISG1 | ecosystem/mis_builder/gotchas.md |
| `bal[70]` no da el saldo que espero (es variación, no saldo) | MISG1 | ecosystem/mis_builder/gotchas.md |
| `bal[]` suma TODAS las cuentas sin querer | MISG2 | ecosystem/mis_builder/gotchas.md |
| El comodín de cuenta no filtra como espero (%=like, sin %=exacto) | MISG2 | ecosystem/mis_builder/gotchas.md |
| Un gasto/ingreso (bale/bali) acumula solo el año, no toda la vida de la cuenta | MISG3 | ecosystem/mis_builder/gotchas.md |
| El resultado de ejercicios anteriores no sale (modo u) | MISG4 | ecosystem/mis_builder/gotchas.md |
| Una celda sale en blanco donde esperaba 0.00 (saldo inicial cero) | MISG5 | ecosystem/mis_builder/gotchas.md |
| `fld`/campo custom da UserError (solo modo p + .fieldname) | MISG6 | ecosystem/mis_builder/gotchas.md |
| Un ratio sale #DIV/0 cuando el denominador no tiene datos | MISG7 | ecosystem/mis_builder/gotchas.md |
| Un KPI muestra #NAME / #DIV/0 / #ERR sin traceback | MISG8 | ecosystem/mis_builder/gotchas.md |
| El informe incluye borradores / no cuadra con contabilidad (target_move) | MISG9 | ecosystem/mis_builder/gotchas.md |
| El filtro analítico no aplica / no sé dónde ponerlo | MISG10 | ecosystem/mis_builder/gotchas.md |
| Los saldos salen raros en la columna de comparación (accumulation SUM vs AVG) | MISG11 | ecosystem/mis_builder/gotchas.md |
| La comparación sale en % cuando quería diferencia absoluta | MISG12 | ecosystem/mis_builder/gotchas.md |
| Una columna sumcol/cmpcol sale vacía | MISG13 | ecosystem/mis_builder/gotchas.md |
| Columna actuals_alt: UserError "field does not exist in the model" | MISG14 | ecosystem/mis_builder/gotchas.md |
| El informe genera demasiadas filas / va lento (auto-expand por cuenta) | MISG15 | ecosystem/mis_builder/gotchas.md |
| UserError "all companies must have the same currency" en informe cross-company | MISG16 | ecosystem/mis_builder/gotchas.md |
| Un KPI multi/subKPI da resultados raros al referenciarlo | MISG18 | ecosystem/mis_builder/gotchas.md |
| En v19 un override del AEP rompe (expression.OR/AND → Domain.OR/AND) | v19 §1 | ecosystem/mis_builder/v19_changes.md |

## Añadidos Fase 2a (2026-07-14): síntomas que kb.py keyword no rankea a top-6

| Symptom | Section | Gotcha ID | File |
|---------|---------|-----------|------|
| configuré una regla de almacenaje pero el destino del movimiento no cambia, solo el de … | stock | STKG2 | functional/stock/gotchas.md |
| necesito permisos diferentes para iniciativas y para oportunidades de venta | crm | CG1 | functional/crm/gotchas.md |
| en multicompañía el flag de verifactu del TPV de qué compañía depende | pos | POG100ES | functional/pos/gotchas.md |
| Reusé la misma referencia de factura de proveedor para dos proveedores distintos y la r… | l10n_es | ESG11 | ecosystem/l10n_es/gotchas.md |
| Hice una transferencia interna entre dos almacenes valorados y no se generó ningún asie… | stock | STKG29 | functional/stock/gotchas.md |
| Configuré una regla de reabastecimiento pero me da error al crear una segunda para el m… | stock | STKG57 | functional/stock/gotchas.md |
| Un empleado pide vacaciones de lunes a domingo pero el sistema solo le descuenta 5 días… | hr | HRG13 | functional/hr/gotchas.md |
| Necesito descontar mercancía rota o caducada directamente desde la pantalla de la caja … | pos | POG17 | functional/pos/gotchas.md |
| Archivé un método de pago guardado de un cliente y ahora varias de sus suscripciones de… | sale | SG13 | functional/sale/gotchas.md |
