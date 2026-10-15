# XML Data and noupdate

## The noupdate Gotcha

Records defined with `noupdate="1"` in XML will **NOT be updated** on module upgrade.
This is critical during development when iterating on collector/template code.

**Problem:** You update `python_code` in the XML but `noupdate="1"` prevents the change
from reaching the database on `-u module`.

**Solutions:**

1. **During development**, set noupdate to false in the database:
```sql
UPDATE ir_model_data SET noupdate = false
WHERE module = 'your_module' AND noupdate = true;
```

2. **For production**, use noupdate="1" to protect user customizations.

3. **Alternative:** Update records directly via API/SQL during testing.

**Best practice:** Use `noupdate="1"` for collectors and templates (users may customize them),
but be aware that XML changes won't apply on upgrade without the DB fix above.
