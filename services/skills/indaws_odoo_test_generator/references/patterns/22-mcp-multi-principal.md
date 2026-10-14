# 22. MCP / Exposure — Multi-Principal Testing

For modules that **expose Odoo to an agent** over a generic surface (MCP server
`mcp.server.config`, JSON-RPC/OpenAPI bridge, or any tool where the caller picks
the model / fields / domain at runtime). These surfaces have privilege-leak bugs
that a single all-powerful identity **cannot reveal**.

> **Rule:** never test an exposure with only `admin`. Admin reads everything, so
> "a restricted user sees X it shouldn't" is structurally invisible. Always add a
> **restricted principal** and assert the difference.

> **Building the exposure, not just testing it?** The `odoo-mcp-builder` skill
> designs/creates the MCP (domain playbooks + secure defaults) and points back
> here for the validation gate; `odoo-code-auditor` → `mcp-exposure-security.md`
> is the audit counterpart.

The exposure invariant under test: a caller never gets more than
**`admin positive-list ∩ caller ir.model.access ∩ ir.rule`** — and **metadata
(model names, schemas, field lists) is data**, so it must be ACL-filtered too.

### 22.1 Two-principal fixture

```python
from odoo.exceptions import UserError
from odoo.tests import new_test_user

@classmethod
def setUpClass(cls):
    super().setUpClass()
    cls.admin = cls.env.ref("base.user_admin")
    # Restricted: internal user, no special groups → ACL denies system models.
    cls.limited = new_test_user(cls.env, login="mcp_ltd", groups="base.group_user")
    # A server/exposure that exposes BOTH a normal model and a model only the
    # admin can read (e.g. ir.config_parameter is system-only).
    cls.server = cls.env["mcp.server.config"].create(
        {"name": "ACL Test", "user_ids": [(6, 0, [cls.admin.id, cls.limited.id])]})
    for model in ("res.partner", "ir.config_parameter"):
        cls._expose(cls.server, model)   # helper that creates the mcp.server.model
```

### 22.2 Metadata is ACL-filtered (the leak class)

```python
def test_metadata_hides_acl_denied_model(self):
    # The shared filter (e.g. config._readable_exposures) backs list_models AND
    # capabilities — test it directly to cover both surfaces in one assertion.
    admin_models = self.server._readable_exposures(self.admin).mapped("model_name")
    ltd_models = self.server._readable_exposures(self.limited).mapped("model_name")
    self.assertIn("ir.config_parameter", admin_models)      # admin (system) sees it
    self.assertNotIn("ir.config_parameter", ltd_models)     # restricted: name must NOT leak
    self.assertLessEqual(set(ltd_models), set(admin_models))  # restricted ⊆ admin
```

### 22.3 Read tools enforce the caller's ACL (not just exposure)

```python
def test_describe_blocked_by_user_acl(self):
    # Exposed BUT ACL-denied → must be refused even though the admin exposed it.
    # (describe_model/get_selection leak schema/values if they skip this.)
    self.assertTrue(_ct_describe_model(self.server, self.admin, {"model": "ir.config_parameter"}))
    with self.assertRaises(UserError):
        _ct_describe_model(self.server, self.limited, {"model": "ir.config_parameter"})
```

### 22.4 Runtime query parts validated against the positive-list

```python
def test_aggregate_rejects_unexposed_and_dotted(self):
    # group_by / measures / domain (incl. dotted relation traversal) must all be
    # checked against the exposed-field set — a raw domain is an inference leak.
    for args in (
        {"model": "res.partner", "group_by": ["email"]},                      # unexposed group_by
        {"model": "res.partner", "measures": ["credit:sum"]},                 # unexposed measure
        {"model": "res.partner", "domain": [["email", "!=", False]]},         # unexposed domain field
        {"model": "res.partner", "domain": [["parent_id.email", "=", "x"]]},  # exposed rel → hidden field
    ):
        with self.assertRaises(UserError):
            _ct_aggregate(self.server, self.admin, args)
```

### 22.5 HTTP-level isolation (HttpCase / pentest)

When testing over the wire, mint a token per principal and assert the restricted
one can't see/read what its ACL denies. Multi-company gotcha: the restricted
user must share the **company** of the exposed server, or `initialize` returns
"MCP not configured for this company" (a false negative, not isolation).

```python
# parametrize the pentest with two tokens: MCP_ACCESS_TOKEN (admin) + RESTRICTED_TOKEN
# admin_models = list_models(admin) ; rest_models = list_models(restricted)
# assert sensitive_model in admin_models and sensitive_model not in rest_models
```

## Checklist for any exposure module
- [ ] At least one **restricted principal** in the suite (not only admin).
- [ ] Metadata surfaces (list_models / describe / capabilities) ACL-filtered.
- [ ] Read tools denied on exposed-but-ACL-denied models.
- [ ] `domain` / `group_by` / `measures` / `fields` validated vs the positive-list, dotted paths included.
- [ ] Cross-server and cross-company isolation (token for A can't read B).

See `odoo-code-auditor` → `references/mcp-exposure-security.md` for the audit-side detection guide.
