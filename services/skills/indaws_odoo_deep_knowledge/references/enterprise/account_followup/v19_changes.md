# account_followup — v19 Changes vs v18

Source: `diff -r odoo_enterprise/18/account_followup/ odoo_enterprise/19/account_followup/`.

## Manifest

- v19 adds `'author': 'Odoo S.A.'`.
- New data file: `data/mail_template.xml` — externalised mail template definitions previously inline in `account_followup_data.xml`.
- New view file: `views/account_move_views.xml` — adds followup-related fields to invoice form (e.g. `no_followup` exclusion flag).

## Model Class Names

- `FollowupLine` → `Account_FollowupFollowupLine` (Odoo v19 enforces `<App>_<Model>` PEP8-style class naming convention via auto-renaming).

## SQL Constraints → models.Constraint

```python
# v18
_sql_constraints = [
    ('days_uniq', 'unique(company_id, delay)', '...'),
    ('uniq_name', 'unique(company_id, name)', '...'),
]

# v19
_days_uniq = models.Constraint('unique(company_id, delay)', "...")
_uniq_name = models.Constraint('unique(company_id, name)', "...")
```

## API: read_group → _read_group

`copy_data` rewritten to use `_read_group` with new tuple-unpacking signature:
```python
# v18
for row in self.read_group(domain, fields=['company_id', 'delay:max'], groupby='company_id'):
    row['company_id'][0]: row['delay']

# v19
for company, delay_max in self._read_group(domain, groupby=['company_id'], aggregates=['delay:max']):
    company.id: delay_max
```

## Field Labels

| Field | v18 | v19 |
|-------|-----|-----|
| `send_email` | "Send Email" | "Email" |
| `send_sms` | "Send SMS Message" | "SMS" |

## res.partner Removed Fields/Properties

- `type` Selection no longer extended with `('followup', 'Follow-up Address')`. Property `_complete_name_displayed_types` override removed.
- Followup contacts now ONLY billing address (`address_get(['invoice'])`). Impact: existing partners with `type='followup'` children become orphaned post-upgrade.

## res.partner New Fields

- `total_overdue_followup` Monetary — overdue receivable EXCLUDING `aml.no_followup=True` lines. Used in dashboard counters.
- New action `action_open_partner_followup_journal_items` — opens followup_report's journal items view filtered by partner.

## res.partner Computed Fields

- `_compute_total_due` now also reads `aml.no_followup`. Group-by adds `no_followup`. New aggregate `receivable_overdue_followup_data`.
- `_search_status` returns `Domain.FALSE` instead of `[]` when value is empty. Returns `NotImplemented` for non-`in` operators.
- `_search_followup_line` uses new `Domain` API: `Domain('name', operator, value) & Domain('company_id', 'parent_of', ...)`.
- `_get_unreconciled_aml_domain` returns `Domain.AND([...])` instead of plain list.

## _get_followup_responsible Signature Change

```python
# v18
def _get_followup_responsible(self):
    most_delayed_aml = self._included_unreconciled_aml_max_followup().get('most_delayed_aml')
    candidates = [
        (responsible_type == 'salesperson' and most_delayed_aml and most_delayed_aml.move_id.invoice_user_id),
        ...
    ]

# v19
def _get_followup_responsible(self, multiple_responsible=False):
    all_aml_responsible = self.unreconciled_aml_ids.move_id.invoice_user_id
    max_amount_aml = max(self.unreconciled_aml_ids..., default=..., key=lambda l: l.amount_residual)
    candidates = [
        (responsible_type == 'salesperson' and (
            all_aml_responsible if multiple_responsible else max_amount_aml.move_id.invoice_user_id
        )),
        ...
    ]
```

`_included_unreconciled_aml_max_followup` removed entirely. New flow inlined.

## SMS

`sms_number = to_send_partner.mobile or to_send_partner.phone` (v18) → `sms_number = to_send_partner.phone` (v19, mobile field unified into phone).

## Mail

- `_render_field` signature: keyword args (`res_ids=`, `render_fields=`, `allow_suggested=`, `find_or_create_partners=True`) instead of positional.
- `message_post`: `mail_notify_author` moved from context to direct kwarg `notify_author=True`.

## Account Move Line

- `aml.no_followup` boolean (NEW in v19) — exclude this AML from followup chain. Impact: per-line "skip dunning" flag, replacing per-partner only logic.

## Demo Data

`demo/account_followup_demo.xml` updated; numerous translations (`i18n/*.po`) refreshed. Not behavioural.

## Migration Notes

1. **type='followup' children orphaned**: pre-migration script must remap them to billing address (set `type='contact'` and update partner's `address_id` if needed).
2. **SQL constraints**: custom modules inheriting `account_followup.followup.line` and adding their own `_sql_constraints` must convert to `models.Constraint`.
3. **Override `_get_followup_responsible`**: must accept `multiple_responsible=False` kwarg.
4. **`_included_unreconciled_aml_max_followup` removed**: any custom code calling it breaks. Replace with `max(unreconciled_aml_ids, key=lambda l: l.amount_residual)`.
5. **`mobile` field**: any module joining mobile vs phone lookup must consolidate to phone.
6. **read_group**: callers using v18 dict-style result must migrate to tuple unpacking.
