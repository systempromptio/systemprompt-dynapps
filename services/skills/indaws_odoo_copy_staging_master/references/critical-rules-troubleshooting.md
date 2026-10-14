# Critical Rules & Troubleshooting

## CRITICAL RULES (from 8 iterations of real migrations)

1. **NEVER match by ID** between staging and production - always use name, code, xmlid
2. **ALWAYS use `allow_none=True`** on XML-RPC ServerProxy
3. **ALWAYS filter fields via `fields_get`** before create/write (safe_create pattern)
4. **Clean `id_mapping.json`** before each fresh run
5. **Import Studio BEFORE migration** (models -> inverse m2o -> fields -> ACLs -> views)
6. **Create automations with `active=False`** always
7. **Sendcloud carriers**: force `delivery_type='fixed'`
8. **ftp.instance**: match by `host`, not `name` (field doesn't exist)
9. **prestashop.instance**: match by `url`, not `name`
10. **ir.actions.server**: read in batches of 10 (singleton computed field bug in staging)
11. **database.is_neutralized**: NEVER copy from staging. **Y el motivo es mecánico, no de higiene**
    (verificado 2026-08-07):
    - **Quién la escribe**: la propia neutralización, en SQL —
      `odoo/addons/base/data/neutralize.sql:22`: `VALUES ('database.is_neutralized', true)`.
    - **Quién la lee**: `ir.cron.toggle` (`odoo/addons/base/models/ir_cron.py:638`, `ir_cron.toggle`, v18 / `odoo/addons/base/models/ir_cron.py:725`, `ir_cron.toggle`, v19),
      con el comentario de Odoo al lado: *"Prevent deactivated cron jobs from being re-enabled through
      side effects on neutralized databases."* Si la marca está, `toggle` devuelve `True` y **no
      reactiva** el cron.

    O sea: la marca es lo único que impide que un cron desactivado **se vuelva a activar por efecto
    lateral**. Copiarla desde staging a un master la deja puesta donde no toca (crons que ya no se
    pueden reactivar por la vía normal); y arrastrar el `ir.config_parameter` de un master **sobre** un
    staging neutralizado la borra y **re-arma** esa reactivación. Las dos direcciones son un fallo
    silencioso: nada avisa, y lo que se rompe es un cron que dispara — o que no dispara — cuando no
    debía.
12. **product.product Studio fields**: delegated to product.template — `_inherits = {'product.template':
    'product_tmpl_id'}` (`product/models/product_product.py:19`, `ProductProduct._inherits`, idéntico en v18 y v19).
13. **res.users Studio fields**: delegated to res.partner — `_inherits = {'res.partner': 'partner_id'}`
    (`odoo/addons/base/models/res_users.py:333`, `Users._inherits`, en v18; **`odoo/addons/base/models/res_users.py:165` en v19** — el fichero se acorta mucho
    porque `res.groups` se muda a `res_groups.py` en v19).

    Es **delegación** (`_inherits`), no extensión: los valores viven en la tabla del padre, así que un
    campo x_studio "de producto" o "de usuario" **no está en `product_product` ni en `res_users`**. Si
    comparas o migras solo la tabla hija, esos campos no aparecen y parece que no existen. Verificado
    2026-08-07.
14. **Carriers need `product_id`**: create a "Delivery" service product as fallback
15. **Install ALL custom modules BEFORE migration** (otherwise fields don't exist)
16. **DB owner must match db_user** in .cfg (typically 'odoo')
17. **Reset admin password** after every dump restore
18. **one2many Studio fields need inverse m2o** created first in the target model
19. **Website views**: match by `key` field (COW system), never by ID
20. **Clear asset bundles** after installing modules or migrating views
21. **ALWAYS migrate translations**: In v17+ translations are stored as JSONB. Read with `context={'lang': False}` to get raw JSONB dict with ALL translations. In v14-v16, also migrate `ir.translation` records.
22. **ALWAYS request repo access**: Before STEP 0, ask for custom module repos path. Without source code, cannot detect noupdate issues, hardcoded IDs, or module dependencies.
23. **Studio = REPLAY, not copy**: Never raw-copy Studio records. Replay creation sequence resolving ALL references by natural key.
24. **Hardcoded IDs in Python code MUST be remapped**: Scan all `code` fields, detect numeric IDs, resolve staging references, replace with production IDs.

## TROUBLESHOOTING

| Problem | Solution |
|---------|----------|
| Auth returns uid=False | Reset password: `UPDATE res_users SET password='admin' WHERE id=2;` |
| Model not found (ERR) | Install missing module or run import_studio.py |
| Access denied on x_* model | Create ir.model.access with group_id=NULL |
| DB not in selector | `ALTER DATABASE mydb OWNER TO odoo;` |
| Singleton error on read | Use `safe_search_read` with `batch_size=10` |
| Invalid field error | Script auto-retries without bad field (safe_create) |
| PLACEHOLDER still there | Re-run copy_credentials.py, check staging has real value |
| Carrier create fails (product_id) | Ensure "Delivery" service product exists |
| Sendcloud carrier fails | Force delivery_type='fixed', set credentials after |
| Automation blocks attr value creation | Disable automations on product.attribute.value first |
| OdooRPC create returns None | Check dry_run parameter: staging=False, production=dry_run |
| ir.config_parameter duplicate key | Delete conflicting param before module install |
| Session cut / context limit | Read `scripts/logs/migration_progress.json`, resume from last incomplete step. Do NOT ask for credentials again. |
| Indaws connection failed | Log warning, continue migration. Re-enable tracking when connection restored. |
| OWL/WYSIWYG asset loading error | `DELETE FROM ir_attachment WHERE url LIKE '/web/assets/%';` then restart Odoo |
| Backup not restored | `createdb $PROD_DB && psql $PROD_DB < dump.sql` then `odoo-bin -c $CFG -d $PROD_DB -u all --stop-after-init` |
