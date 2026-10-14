<!-- verified: v18.0 @ 2026-06 -->
# BaseModel — Advanced Methods (Odoo 18.0)

Continuation of `orm-models.md`. These methods are used less frequently — for access control, cache management, data import/export, and metadata.

## Access Control

### check_access

```python
def check_access(self, operation: str) -> None
```

Verifies the current user can perform `operation` ('read', 'write', 'create', 'unlink') on all records in self. Raises `AccessError` if denied. Combines ACL + record rules in one call.

### has_access

```python
def has_access(self, operation: str) -> bool
```

Like `check_access` but returns `True`/`False` instead of raising.

### check_access_rights

```python
@api.model
def check_access_rights(self, operation, raise_exception=True) -> bool
```

Checks `ir.model.access` records only (ACL layer). Does NOT check record rules.

> **Cross-version (v16/v17 ↔ v18+) — important for exposure/MCP code.**
> `check_access` and `has_access` are the **v18+** API. In **v16/v17** they do not
> exist; the idiom there is `check_access_rights(operation, raise_exception=False)`
> for a non-raising bool probe (and `check_access_rights(operation)` to raise). In
> v18 `check_access_rights` is **deprecated** and its boolean form is gone in v19 —
> using it leaks deprecation warnings (and can raise where you expected a bool).
> Map by **API family**, not per version:
> | Intent | v16 / v17 | v18 / v19 |
> |--------|-----------|-----------|
> | non-raising read probe (bool) | `model.check_access_rights("read", raise_exception=False)` | `model.has_access("read")` |
> | gate (raise on denial) | `model.check_access_rights("read")` | `model.check_access("read")` |
>
> To branch deterministically, detect the running Odoo version with
> `from odoo import release` → `release.version_info[0]` (int major, e.g. `18`) or
> `release.version` (string like `"18.0"`). Keep one branch per family, not per
> point release.

### check_access_rule

```python
def check_access_rule(self, operation) -> None
```

Checks `ir.rule` records only (record rules layer). Assumes ACL already passed.

### check_field_access_rights

```python
@api.model
def check_field_access_rights(self, operation, field_names) -> list
```

Filters field names by field-level `groups=` attribute. Returns list of accessible field names.

---

## Cache & Flush

### flush_model

```python
def flush_model(self, fnames=None)
```

Writes pending computations and updates to DB for ALL records of self's model. Call before raw SQL that reads the same table.

### flush_recordset

```python
def flush_recordset(self, fnames=None)
```

Like `flush_model` but only for records in self. More targeted, better performance.

### invalidate_model

```python
def invalidate_model(self, fnames=None, flush=True)
```

Clears cache for ALL records of self's model. Call after raw SQL that modifies the table.

### invalidate_recordset

```python
def invalidate_recordset(self, fnames=None, flush=True)
```

Clears cache only for records in self. More targeted.

### fetch

```python
def fetch(self, field_names)
```

Ensures given fields are loaded in cache for records in self. Triggers prefetch for PREFETCH_MAX (1000) records.

### modified

```python
def modified(self, fnames, create=False, before=False)
```

Signals that fields have been/will be modified. Triggers recomputation graph traversal. Rarely called directly — the ORM calls it from `write()`.

---

## Data Import/Export

### export_data

```python
def export_data(self, fields_to_export) -> dict
```

Exports fields for records in self. Returns `{'datas': [[value, ...], ...]}`. Supports dotted paths (`partner_id/name`) and `/id` for external IDs.

### load

```python
@api.model
def load(self, fields, data) -> dict
```

Imports data matrix. Returns `{'ids': [...], 'messages': [...]}`. Used by the import wizard. Handles external IDs, relation resolution, and batch creation.

---

## Metadata

### fields_get

```python
def fields_get(self, allfields=None, attributes=None) -> dict
```

Returns field definitions as `{field_name: {type, string, help, ...}}`. Used by the web client to build views dynamically.

### get_metadata

```python
def get_metadata(self) -> list[dict]
```

Returns creation/modification info: `create_uid`, `create_date`, `write_uid`, `write_date`, `xmlid`, `noupdate`.

### update

```python
def update(self, values: dict)
```

Assigns values to fields via Python attribute assignment (triggers onchange in Form context). Different from `write()` which goes directly to DB.

## Translations & external-id helpers

Métodos periféricos pero parte de la superficie pública del ORM (v18 `odoo/models.py`).

### get_external_id
```python
def get_external_id(self) -> dict  # {record_id: 'module.xmlid'}  (models.py:6091)
```
Devuelve los XML IDs (external ids) de los registros; crea uno `__export__` si no existe.

### get_field_translations / update_field_translations
```python
def get_field_translations(self, field_name, langs=None)      # models.py:4012 → (translations, context)
def update_field_translations(self, field_name, translations, source_lang=None)  # models.py:3865
```
Leer/escribir las traducciones de un campo `translate=True` por idioma (i18n avanzado, sin pasar por .po).

### copy_translations
```python
def copy_translations(self, new, excluded=())  # models.py:5896
```
Copia las traducciones de `self` al registro `new` (lo usa `copy()`).

### get_property_definition
```python
def get_property_definition(self, full_name)  # models.py:3291
```
Devuelve la definición de un campo `Properties` por su nombre completo.
