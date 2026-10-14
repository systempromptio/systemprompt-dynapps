# Classification Rules

Reglas para etiquetar cada modulo como `standard | oca | indaws_core | custom | studio`.

## Decision tree

Aplicar en orden. El primer match gana.

```
1. nombre del modulo == "studio_customization" o estado de Studio activo?
   -> NO se etiqueta el modulo como `studio`.
   -> Las customizaciones Studio NO viven como modulo en ir.module.module.
   -> Se inventarian aparte en studio.yaml.
   -> En modules.yaml puede aparecer `studio_customization` como custom (es un export Studio).

2. addons_path apunta a /opt/odoo/odooNN/addons o a /opt/odoo_enterprise/NN/?
   o el author es "Odoo S.A." o "Odoo Community Association (OCA)" via core?
   -> standard

3. addons_path apunta a /opt/OCA/vNN/?
   o el author/manifest contiene "Odoo Community Association (OCA)"?
   o el repositorio del modulo es github.com/OCA/?
   -> oca

4. nombre del modulo empieza con prefijo `indaws_`?
   o author == "Indaws" / "Dynapps" en manifest?
   o addons_path apunta a /opt/PROYECTS/indawsodoo/NN/?
   -> indaws_core

5. cualquier otro modulo (incluyendo `studio_customization` si aparece como modulo)
   -> custom
```

## Senales por fuente

### Filesystem path

Lookup del `__manifest__.py` por `name` del modulo:

| Path pattern | Type |
|--------------|------|
| `/opt/odoo/odoo*/addons/` | standard |
| `/opt/odoo_enterprise/*/` | standard (enterprise) |
| `/opt/OCA/v*/` | oca |
| `/opt/PROYECTS/indawsodoo/*/` | indaws_core |
| `/opt/PROYECTS/<cliente>/...` (cualquier otro) | custom |

### Manifest fields

```python
# manifest['author']
"Odoo S.A."                                    -> standard
"Odoo Community Association (OCA)"             -> oca
"OCA, ..."                                     -> oca
"Indaws"                                       -> indaws_core
"Dynapps"                                      -> indaws_core
"Indaws, OCA"                                  -> indaws_core (mas restrictivo gana)
otro                                           -> custom
```

### Prefijos y patrones de nombre

| Pattern | Type sugerido |
|---------|--------------|
| `indaws_*` | indaws_core |
| `dynapps_*` | indaws_core |
| `account_*`, `sale_*`, `stock_*` (sin prefijo cliente) en `/opt/odoo*/addons/` | standard |
| `oca_*` o `account_invoice_*`, `partner_*` en `/opt/OCA/` | oca |
| Cualquier otro nombre fuera de los paths anteriores | custom |

## Casos limite

### Forks de OCA con cambios locales
Si el modulo vive en `/opt/PROYECTS/<cliente>/oca_forks/` (o similar fuera de `/opt/OCA/`), tratar como `custom` aunque el manifest diga OCA. Es un fork que el cliente mantiene.

### Modulos OCA renombrados con prefijo cliente
Ej: `cliente_x_account_invoice_report` (fork de `account_invoice_report` de OCA con cambios). -> `custom` (el cliente lo mantiene).

### Modulos Indaws redistribuidos por terceros
Si un partner ha empaquetado un `indaws_*` en su propio repo y vive fuera de `/opt/PROYECTS/indawsodoo/`, marcar como `indaws_core` SOLO si el manifest sigue siendo Indaws sin modificar. Si hay cambios locales, `custom`.

### `studio_customization` como modulo
Cuando se exporta Studio a un modulo instalable, aparece en `ir.module.module` como modulo regular. Marcar como `custom` (no como `studio`) — el `studio` etiqueta solo se usa en el bloque studio.yaml para las customizaciones vivas en BD via base.module='studio'.

### Modulos sin manifest accesible
Si scan_source=remote_only (sin filesystem), clasificar por:
1. `name.startswith('indaws_')` o `name.startswith('dynapps_')` -> indaws_core
2. `name` corresponde a un modulo estandar/enterprise conocido de Odoo (`base`, `sale`, `account`, `stock`, `mrp`, `hr`, `website`, `project`...) -> standard
3. `name.startswith('account_') and name in oca_well_known` -> oca (fallback)
4. resto -> custom (con flag `classification_confidence: low`)

### Bridge modules
Modulos cuya unica funcion es conectar 2 modulos (ej: `indaws_sale_stock`). Se clasifican igual que cualquier otro segun las reglas anteriores (no son una categoria aparte) pero su `purpose` debe indicar "Bridge: conecta X con Y" para facilitar el briefing.

## Confidence flag

Cada entrada en `modules.yaml` puede llevar `classification_confidence: high|medium|low`:

- `high`: clasificado por path + manifest + nombre (todos coherentes)
- `medium`: clasificado por 1-2 senales coherentes, falta el path
- `low`: clasificado solo por nombre (scan_source=remote_only)

El briefing al usuario destaca los `low` para revision manual.
