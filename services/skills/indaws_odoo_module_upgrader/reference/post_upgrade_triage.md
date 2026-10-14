# Post-Upgrade Module Triage

Referencia para clasificar y gestionar módulos después de que el Upgrade Service
haya migrado la base de datos. Usado en FASE 3.5 del `odoo-database-upgrader`.

## Detección automática de contexto

Si existe `logs/upgrade_credentials.json` en el workdir, leerlo para obtener:

```python
import json, os

def detect_upgrade_context():
    """Detecta si estamos en contexto post-database-upgrade."""
    config_path = "logs/upgrade_credentials.json"
    if not os.path.exists(config_path):
        return None
    with open(config_path) as f:
        cfg = json.load(f)
    return {
        'version_from': cfg.get('version_from'),
        'version_to': cfg.get('version_to'),
        'db_name': cfg.get('db_name'),
        'contract': cfg.get('contract'),
    }
```

## Query de módulos pendientes

```sql
-- Módulos que necesitan acción post-upgrade
SELECT
    m.name,
    m.author,
    m.state,
    m.latest_version,
    CASE
        WHEN m.author ILIKE '%Odoo%' OR m.author ILIKE '%OpenERP%' THEN 'CORE'
        WHEN m.author ILIKE '%OCA%' OR m.author ILIKE '%Tecnativa%'
          OR m.author ILIKE '%Akretion%' THEN 'OCA'
        WHEN m.name LIKE 'indaws_%' THEN 'INDAWS'
        ELSE 'CUSTOM'
    END AS type
FROM ir_module_module m
WHERE m.state IN ('to upgrade', 'to install')
ORDER BY
    CASE
        WHEN m.author ILIKE '%Odoo%' THEN 1
        WHEN m.author ILIKE '%OCA%' OR m.author ILIKE '%Tecnativa%' THEN 2
        WHEN m.name LIKE 'indaws_%' THEN 3
        ELSE 4
    END,
    m.name;
```

## Clasificación y acciones

### CORE (author = Odoo S.A.)
**Acción: Ninguna** — ya migrado por el Upgrade Service.
Si aparece como `to upgrade`, algo fue mal. Investigar.

### OCA
**Acción: Instalar código v18 desde repo OCA. NO migrar código.**

Pasos:
1. Verificar que el módulo existe en el repo OCA para la versión destino
2. Si existe: montar el repo en Docker y ejecutar `-u module`
3. Si NO existe: el módulo fue absorbido en core o eliminado

**Módulos OCA comunes absorbidos en core:**

| Módulo OCA | Absorbido en | Versión |
|---|---|---|
| `l10n_es` | Odoo CE (`addons/l10n_es`) | v16+ |
| `l10n_es_reports` | Odoo EE (`l10n_es_reports`) | v16+ |
| `partner_firstname` | `res.partner` core fields | v17 |
| `base_comment_template` | Odoo core | v17 |
| `sale_order_line_date` | `sale.order.line` core | v17 |

**Módulos OCA con port v18 confirmado** (verificar en `$OCA_SRC/v18/`):

Repos principales:
- `bank-payment`: account_banking_mandate, account_banking_pain_base, account_banking_sepa_*
- `account-payment`: account_payment_mode, account_payment_order, account_payment_partner, account_payment_sale, account_payment_purchase
- `brand`: brand, account_brand, sale_brand, product_brand
- `product-pack`: product_pack
- `server-tools`: auditlog, server_action_mass_edit

### INDAWS
**Acción: Delegar a `indaws-core-developer` skill.**

Los módulos `indaws_*` tienen patrones de migración propios (5 capas de arquitectura,
cadenas de dependencia, fixes Odoo 18 específicos).

### CUSTOM
**Acción: Migrar código con este skill (`odoo-module-upgrader`).**

Este es el trabajo principal del skill. Seguir el workflow STEP 1-7 del SKILL.md.

### OBSOLETE
**Acción: Desinstalar.**

```sql
-- Marcar como desinstalado (no requiere el código)
UPDATE ir_module_module SET state = 'uninstalled' WHERE name = '<module>';
```

Señales de que un módulo es obsoleto:
- No tiene port en OCA ni en ningún repo para la versión destino
- Sus tablas están vacías o solo tienen datos de demo
- El usuario confirma que ya no se usa

## Verificación post-triage

Después de instalar OCA, migrar custom y desinstalar obsoletos:

```sql
-- Debe retornar 0 filas
SELECT name, state FROM ir_module_module
WHERE state IN ('to upgrade', 'to install')
ORDER BY name;
```

## Mapping módulo → repo OCA

Script para encontrar en qué repo OCA está cada módulo:

```bash
# Construir índice de módulos OCA
find $OCA_SRC/v18 -maxdepth 3 -name "__manifest__.py" -printf "%h\n" | \
    awk -F'/' '{print $NF, $(NF-1)}' | sort > /tmp/oca_index.txt

# Buscar un módulo
grep "^account_payment_mode " /tmp/oca_index.txt
# → account_payment_mode account-payment
```

## Gotchas

1. **author "Spanish Localization Team" NO es OCA** — `l10n_es` y `l10n_es_reports` están en core CE/EE desde v16, aunque el author no diga "Odoo S.A."
2. **`studio_customization` es especial** — contiene las personalizaciones de Odoo Studio. NO desinstalar. Se migra automáticamente con el Upgrade Service.
3. **Módulos con `x_studio_` en el nombre** — son modelos Studio, no módulos Python. Se gestionan via la BD, no via código.
4. **Un módulo OCA en `to upgrade` necesita el código v18 en el addons_path** — si solo tienes v16, el `-u` fallará. Clona primero el repo OCA v18.
5. **Dependencias cruzadas OCA↔CUSTOM** — si un módulo custom depende de uno OCA, instalar primero el OCA y luego migrar+instalar el custom.
