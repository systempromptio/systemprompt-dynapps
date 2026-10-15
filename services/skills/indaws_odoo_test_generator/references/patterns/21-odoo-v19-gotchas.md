# Pattern 21 — Odoo v19 Test Gotchas

**Document ID:** PATTERN-021
**Load when:** Target version is v19

Real-world pitfalls discovered while writing tests for Odoo 19.0 modules.

---

## 21.1 `safe_eval()` Removed `nocopy` Parameter

**Symptom:** `TypeError: safe_eval() got an unexpected keyword argument 'nocopy'`

In v19, `odoo.tools.safe_eval.safe_eval()` signature changed to:

```python
def safe_eval(expr, /, context=None, *, mode="eval", filename=None)
```

The `nocopy` parameter was removed. Code migrated from v18 that uses `nocopy=True`
will fail silently (wrapped in try/except) or raise.

**Fix:** Remove `nocopy=True` from all `safe_eval()` calls.

**Test pattern:**
```python
def test_code_execution(self):
    """Test that python_code executes without safe_eval errors."""
    collector = self.env['my.model'].create({
        'python_code': "value = [{'a': 1}]",
    })
    # This will catch nocopy errors if still present
    result = collector.execute_code()
    self.assertTrue(result)
```

---

## 21.2 Stored Computed Fields Recompute on Cache Invalidation

**Symptom:** Writing directly to DB + `invalidate_recordset()` doesn't stick — the
field recomputes from source fields when accessed.

**Wrong approach:**
```python
def test_unsupported_extension(self):
    imp = self._create_import(filename='test.csv')
    # This WON'T work — field_extension recomputes from filename
    self.env.cr.execute(
        "UPDATE my_table SET file_extension = 'xyz' WHERE id = %s",
        (imp.id,),
    )
    imp.invalidate_recordset(['file_extension'])
    # FAILS: imp.file_extension is still 'csv' (recomputed)
```

**Correct approach:** Either change the source field too, or test the constraint
directly:
```python
def test_unsupported_extension_rejected(self):
    """Test constraint rejects unsupported extensions."""
    with self.assertRaises(ValidationError):
        self._create_import(filename='test.pdf')
```

---

## 21.3 Locale Affects Float Parsing in Tests

**Symptom:** `_parse_float_value('123.45')` returns `12345.0` instead of `123.45`.

Odoo uses the user's language locale for `csv_thousands_sep` and `csv_decimal_point`.
In Spanish locale (`es_ES`), `.` is the thousands separator and `,` is decimal.

**Wrong:**
```python
def test_parse_float(self):
    # FAILS in es_ES: treats '.' as thousands sep → 12345.0
    self.assertEqual(self.imp._parse_float_value('123.45'), 123.45)
```

**Correct:**
```python
def test_parse_float(self):
    # Always set separators explicitly in tests
    self.imp.csv_thousands_sep = ','
    self.imp.csv_decimal_point = '.'
    self.assertEqual(self.imp._parse_float_value('123.45'), 123.45)
```

**Rule:** Always configure locale-dependent parameters explicitly in test setup.
Never rely on the default locale of the test database.

---

## 21.4 Model `__slots__` Blocks Dynamic Attributes

**Symptom:** `AttributeError: 'my.model' object has no attribute '_my_buffer'`

Odoo models use `__slots__`, so you can't set arbitrary attributes on model instances.
Methods that use `self._buffer = value` work during normal execution (often via
`__getattr__` hacks) but fail in isolated test contexts.

**Wrong:**
```python
def test_skip_rows(self):
    # FAILS: _register_res_id tries self._res_id_buffer = buf
    filtered = imp.skip_sha1_rows(rows)
```

**Correct — mock the problematic method:**
```python
from unittest.mock import patch

def test_skip_rows(self):
    with patch.object(type(imp), '_register_res_id'):
        filtered = imp.skip_sha1_rows(rows)
    self.assertEqual(len(filtered), expected)
```

**Rule:** When testing methods that call internal buffer/state methods using dynamic
attributes, mock those methods with `patch.object(type(record), 'method')`.

---

## 21.5 Form Class Rejects Invisible Fields

**Symptom:** `AssertionError: can't write on invisible field 'field_name'`

In v19, the `Form` test class strictly enforces field visibility. If a field is
`invisible="1"` in the view (or conditionally invisible), you cannot set it via
`form.field_name = value`.

**Wrong:**
```python
def test_form(self):
    with Form(self.env['my.model']) as f:
        f.filename = 'test.csv'  # FAILS if filename is invisible
```

**Correct — use the model directly or set via visible fields:**
```python
def test_form(self):
    # Option A: Test the model directly (no Form)
    record = self.env['my.model'].create({'filename': 'test.csv'})

    # Option B: Set via a visible field that triggers onchange
    with Form(self.env['my.model']) as f:
        f.file = base64.b64encode(b'content')  # visible field
        # filename is set by onchange
```

**Rule:** Before using Form, check the view XML to verify which fields are visible.
Use `Form` only for testing user-facing interactions (onchanges, defaults).

---

## 21.6 Form Class Rejects Fields Not in View

**Symptom:** `AssertionError: 'field_name' was not found in the view`

The `Form` class only allows setting fields that are present in the default form
view. Fields not in the view cannot be set.

**Wrong:**
```python
with Form(self.env['my.model']) as f:
    f.company_id = self.env.company  # Not in view → error
```

**Correct:**
```python
# Set via context or model create
record = self.env['my.model'].with_context(
    default_company_id=self.env.company.id
).create({...})

# Or specify a view that includes the field
with Form(self.env['my.model'], view='module.view_with_company') as f:
    f.company_id = self.env.company
```

---

## 21.7 Unique Constraints Survive Between Test Runs

**Symptom:** `IntegrityError: duplicate key value violates unique constraint`
on test data that should be fresh.

If a previous test run created records with unique names and the transaction
was committed (e.g., via `SingleTransactionCase` or a bug), those records persist.

**Prevention:**
```python
def test_unique_constraint(self):
    # Use unique names per test run
    import uuid
    name = f'Test {uuid.uuid4().hex[:8]}'
    record = self.env['my.model'].create({'name': name})
    with self.assertRaises(Exception), self.cr.savepoint():
        self.env['my.model'].create({'name': name})
```

Or use `self.cr.savepoint()` to ensure the constraint test doesn't pollute state.

---

## 21.8 Imports from `common.py` Don't Propagate to Test Files

**Symptom:** `NameError: name 'new_test_user' is not defined`

If `common.py` imports `new_test_user` and your test class inherits from
`CommonClass`, the import is NOT available in the test file's namespace.
Each test file must explicitly import everything it uses.

**Wrong:**
```python
# test_security.py
from .common import MyCommon  # MyCommon uses new_test_user internally

class TestSecurity(MyCommon):
    def test_multicompany(self):
        user = new_test_user(...)  # NameError!
```

**Correct:**
```python
# test_security.py
from odoo.tests import new_test_user, tagged
from .common import MyCommon

class TestSecurity(MyCommon):
    def test_multicompany(self):
        user = new_test_user(...)  # OK
```

**Rule:** Always add explicit imports in each test file for `new_test_user`,
`tagged`, `Form`, `Command`, and any other utility you use directly —
even if the common base class already imports them.

---

## 21.9 Multi-Company Tests Need Explicit `company_ids` on Users

**Symptom:** User from company B can still see records from company A, or
record rules don't filter as expected.

When creating test users for multi-company isolation tests, you must set
both `company_id` (default company) AND `company_ids` (allowed companies).
If `company_ids` includes both companies, the record rule
`('company_id', 'in', company_ids)` will match both.

**Wrong:**
```python
user_b = new_test_user(cls.env, login='user_b',
    company_id=company_b.id,
    # company_ids not set → inherits all companies
)
```

**Correct:**
```python
user_b = new_test_user(cls.env, login='user_b',
    company_id=company_b.id,
    company_ids=[company_b.id],  # ONLY company B
)
```

**Rule:** For isolation tests, always restrict `company_ids` to a single
company. For cross-company access tests, explicitly list the allowed set.

---

## 21.10 Form Class Rejects `groups=` Fields in Single-Company

**Symptom:** `AssertionError: 'company_id' was not found in the view` or
`AssertionError: can't write on invisible field 'company_id'`

Fields with `groups="base.group_multi_company"` are invisible in single-company
databases. The `Form` class rejects writes on them.

**Wrong:**
```python
with Form(self.env['my.model']) as f:
    f.company_id = self.env.company  # FAILS in single-company
```

**Correct — rely on default, or set via create:**
```python
# company_id has default=lambda self: self.env.company
with Form(self.env['my.model']) as f:
    f.template_id = self.template  # company_id set by default
```

---

## 21.11 `Approx()` Signature Changed in v19

**Symptom:** `TypeError: Approx.__init__() missing 2 required positional arguments: 'rounding' and 'decorate'`

In v19, `odoo.tests.common.Approx` requires `rounding` and `decorate` as
mandatory arguments. It no longer works as a simple float wrapper.

**Wrong:**
```python
from odoo.tests.common import Approx
self.assertEqual(record.amount, Approx(3.14))
```

**Correct — use assertAlmostEqual:**
```python
self.assertAlmostEqual(record.amount, 3.14, places=2)
```

---

## 21.12 Manual `create_uid` Doesn't Trigger Record Rules

**Symptom:** Setting `record.create_uid = user` via ORM or SQL doesn't make
the record visible to that user's record rules.

Record rules are evaluated at query time, so changing `create_uid` DOES affect
`search()` results. However, the record was created in a transaction as admin,
so the user may not have had permission to create it. The correct approach:

**Wrong:**
```python
imp = self._create_import()  # Created as admin
imp.create_uid = self.test_user  # Bypasses create ACL check
visible = Model.with_user(self.test_user).search(...)
```

**Correct — create with the correct user:**
```python
imp = self.env['my.model'].with_user(self.test_user).create({...})
visible = self.env['my.model'].with_user(self.test_user).search(...)
```

If the user doesn't have create permission, use SQL to set create_uid:
```python
imp = self._create_import()  # Created as admin
self.cr.execute(
    "UPDATE my_table SET create_uid = %s WHERE id = %s",
    (self.test_user.id, imp.id),
)
imp.invalidate_recordset()
```

---

## 21.13 `privilege_id` Groups + `new_test_user` ACL Resolution

**Symptom:** `AccessError` despite user having the correct group with CRUD ACL.

In v19, groups using `privilege_id` (replacing `category_id`) may not resolve
ACLs correctly with `new_test_user`. Always add `base.group_user` explicitly.

**Wrong:**
```python
user = new_test_user(cls.env, login='admin_user',
    groups='my_module.group_admin',
)
```

**Correct:**
```python
user = new_test_user(cls.env, login='admin_user',
    groups='base.group_user,my_module.group_admin',
)
```

---

## 21.14 `pool.cursor()` Creates Separate Transaction

**Symptom:** SQL changes via `pool.cursor()` invisible in tests; records
written via `pool.cursor()` don't appear in `search()`.

`self.pool.cursor()` opens a NEW database connection with its own transaction.
In tests (TransactionCase), the main cursor's uncommitted data is invisible
to this second cursor. Additionally, `pool.cursor()` auto-commits on `__exit__`,
which is forbidden in tests.

**Wrong:**
```python
def _set_done(self, result):
    with self.pool.cursor() as cr:  # Separate transaction!
        cr.execute("UPDATE ... SET state = 'done' ...")
```

**Correct — use self.env.cr:**
```python
def _set_done(self, result):
    self.env.cr.execute("UPDATE ... SET state = 'done' ...")
    self.invalidate_recordset(['state'])
```

**In tests that call code with `cr.commit()`:** Mock it:
```python
with patch.object(self.env.cr, 'commit', lambda: None):
    record.action_that_commits()
```

---

## 21.15 `all([])` Returns `True` — Empty Set Logic Bug

**Symptom:** Computed state field shows 'cancel' or 'done' for records
with no child records.

Python's `all([])` returns `True` for empty iterables. This breaks
state aggregation logic:

```python
states = rec.child_ids.mapped('state')  # Empty list
all(s == 'cancel' for s in states)  # True!
```

**Fix — always check for empty first:**
```python
if not states:
    rec.state = 'draft'
elif all(s == 'cancel' for s in states):
    rec.state = 'cancel'
```

## 21.16 `account.move.line.display_type == 'product'` (truthy), NOT `False`

On **invoice lines** the real product lines carry `display_type='product'` in v19
(a truthy string), while sections/notes are `'line_section'`/`'line_note'`/
`'line_subsection'`. The habitual guard to skip section/note lines silently skips
**every** line — so assertions that filter "the real lines" match nothing and pass
vacuously (false green).

```python
# WRONG — 'product' is truthy → selects/loops nothing on account.move.line
product_lines = move.invoice_line_ids.filtered(lambda l: not l.display_type)

# RIGHT
product_lines = move.invoice_line_ids.filtered(lambda l: l.display_type == 'product')
```

**Trap:** `sale.order.line.display_type` **is still `False`** for real product lines.
The same `not l.display_type` idiom is correct on `sale.order.line` but wrong on
`account.move.line` — don't copy the filter across models. When asserting invoice
line amounts, verify the filter actually returns the lines first (a vacuous filter
turns a total-inflation bug into a passing test).
