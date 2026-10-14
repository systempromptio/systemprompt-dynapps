<!-- verified: v18.0 @ 2026-06 -->
# Exceptions — Complete Hierarchy (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/exceptions.py`

## Hierarchy

Verified against source: `UserError(Exception)`, and `ValidationError`, `AccessError`,
`AccessDenied`, `MissingError` are all **direct** subclasses of `UserError`.
`RedirectWarning` extends `Exception` directly (NOT `UserError`). `CacheMiss` extends `KeyError`.

```
Exception
├── UserError                    — business logic error shown to user
│   ├── ValidationError          — python constraint violation
│   ├── AccessError              — access rights denied
│   ├── AccessDenied             — authentication failed (login)
│   └── MissingError             — record not found / write on deleted record
├── RedirectWarning(Exception)   — error with redirect action
└── CacheMiss(KeyError)          — field not in cache (internal)
```

## When to use each

### UserError

```python
from odoo.exceptions import UserError

raise UserError(_("Cannot confirm order without lines."))
```

General-purpose business error. Shown to the user as a dialog. Use for:
- Business rule violations
- Invalid operations (wrong state, missing data)
- Any error the user can understand and fix

### ValidationError

```python
from odoo.exceptions import ValidationError

@api.constrains('email')
def _check_email(self):
    for record in self:
        if record.email and '@' not in record.email:
            raise ValidationError(_("Invalid email: %s", record.email))
```

Subclass of UserError. Docstring: "Violation of python constraints." Use inside `@api.constrains` methods. Triggers a rollback of the current write/create.

### AccessError

```python
from odoo.exceptions import AccessError

raise AccessError(_("You don't have access to this document."))
```

Raised automatically by the ORM when ACL or record rules deny access. Raise manually when implementing custom access logic.

### AccessDenied

```python
from odoo.exceptions import AccessDenied

raise AccessDenied(_("Invalid credentials."))
```

Subclass of UserError. Authentication failure. Raised by `res.users._check_credentials(credential, env)`. Default message is `"Access Denied"`. Its `__init__` strips the traceback (`self.with_traceback(None)`, `traceback = ('', '', '')`) so no stack is leaked.

### MissingError

```python
from odoo.exceptions import MissingError

raise MissingError(_("The record was deleted."))
```

Raised automatically when accessing fields on a record that no longer exists in the DB (after `browse()` on a deleted ID). Raise manually when a required record is gone.

### RedirectWarning

```python
from odoo.exceptions import RedirectWarning

action = self.env.ref('account.action_account_config')
raise RedirectWarning(
    _("Please configure your accounting."),
    action.id,
    _("Go to Settings"),
)
```

Extends `Exception` directly (NOT UserError). Shows an error message with a button that redirects to an action (usually a settings page). Signature: `RedirectWarning(message, action, button_text, additional_context=None)` — the second positional arg is named `action` in source (pass the action id).

### CacheMiss

```python
from odoo.exceptions import CacheMiss
```

Internal. Subclass of `KeyError` (NOT UserError). Constructed as `CacheMiss(record, field)`. Raised when a field value is not in cache. Rarely caught by developers.

## Catching exceptions

```python
# Catch and continue (e.g., in a loop processing multiple records)
from odoo.exceptions import UserError, ValidationError

for record in records:
    try:
        record.action_confirm()
    except (UserError, ValidationError) as e:
        # Log and continue with next record
        _logger.warning("Failed for %s: %s", record.id, e)

# Catch in a savepoint (transaction continues)
from odoo import SUPERUSER_ID
try:
    with self.env.cr.savepoint():
        record.unlink()
except UserError:
    # Unlink failed but transaction is not rolled back
    pass
```

## In tests

```python
from odoo.exceptions import UserError, ValidationError, AccessError

with self.assertRaises(UserError):
    order.action_confirm()

with self.assertRaises(ValidationError):
    self.env['res.partner'].create({'email': 'invalid'})

with self.assertRaises(AccessError):
    order.with_user(basic_user).write({'amount': 100})
```
