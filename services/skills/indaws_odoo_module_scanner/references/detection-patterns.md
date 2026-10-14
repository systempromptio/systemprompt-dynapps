# Detection Patterns — Patrones de busqueda para escaneo de modulos

## Fase 1: Estructura de ficheros

```
module_name/
├── __manifest__.py          → LEER: name, version, depends, data, assets, external_dependencies
├── __init__.py              → LEER: imports (models, controllers, wizard, report)
├── models/
│   ├── __init__.py          → LEER: que modelos se importan
│   └── *.py                 → ESCANEAR: modelos, campos, metodos
├── views/
│   └── *.xml               → ESCANEAR: vistas, menus, acciones
├── security/
│   ├── ir.model.access.csv  → ESCANEAR: reglas de acceso
│   └── *.xml               → ESCANEAR: grupos, record rules
├── controllers/
│   └── *.py                → ESCANEAR: rutas HTTP
├── wizard/
│   └── *.py                → ESCANEAR: modelos transient
├── report/
│   └── *.xml               → ESCANEAR: plantillas QWeb de informes
├── data/
│   └── *.xml               → ESCANEAR: datos iniciales, cron jobs, menus
├── static/
│   └── src/**/*.js         → ESCANEAR: componentes JS/OWL
└── i18n/
    └── *.po                → CONTAR: cadenas traducidas por idioma
```

---

## Fase 2: Grep patterns para modelos

```bash
# Modelos: nuevos y heredados
grep -rn "_name\s*=" --include="*.py" <module>/models/
grep -rn "_inherit\s*=" --include="*.py" <module>/models/
grep -rn "_inherits\s*=" --include="*.py" <module>/models/
grep -rn "_transient\s*=\s*True" --include="*.py" <module>/

# Campos
grep -rn "fields\.\(Char\|Text\|Integer\|Float\|Boolean\|Many2one\|One2many\|Many2many\|Selection\|Date\|Datetime\|Binary\|Html\|Monetary\)" --include="*.py" <module>/models/

# Campos computados
grep -rn "compute=" --include="*.py" <module>/models/

# Constraints
grep -rn "@api.constrains" --include="*.py" <module>/models/
grep -rn "_sql_constraints" --include="*.py" <module>/models/

# Onchanges
grep -rn "@api.onchange" --include="*.py" <module>/models/

# Acciones de workflow
grep -rn "def action_" --include="*.py" <module>/models/

# CRUD overrides
grep -rn "def create\b\|def write\b\|def unlink\b\|def copy\b" --include="*.py" <module>/models/

# Excepciones
grep -rn "raise UserError\|raise ValidationError\|raise AccessError" --include="*.py" <module>/

# Cron jobs
grep -rn "def _cron_" --include="*.py" <module>/
grep -rn "ir.cron" --include="*.xml" <module>/

# APIs externas
grep -rn "requests\.\(get\|post\|put\|delete\)\|urllib\|http\.client" --include="*.py" <module>/

# State fields (workflows)
grep -rn "fields.Selection.*state\|selection=.*draft.*confirm" --include="*.py" <module>/models/
```

---

## Fase 3: Clasificacion de modelos

> **Vocabulario corregido 2026-08-07.** Esta tabla llamaba "herencia delegada" a `_inherit` sin
> `_name` y "herencia prototipo" a `_inherits`. Las dos etiquetas estaban mal, y borraban justo la
> distincion que decide el modelo de datos. Los docstrings de Odoo lo dicen literalmente
> (`odoo/models.py:578-596`, `BaseModel._inherit`, v18; `odoo/orm/models.py:396-424`, `BaseModel._inherit`, v19):
>
> - `_inherit` con `_name` **sin definir** → *"name of a single model to **extend in-place**"* → una
>   sola tabla, ningun modelo nuevo. Es **extension**, no delegacion.
> - `_inherits` → *"implements **composition-based inheritance**: the new model exposes all the fields
>   of the inherited models but **stores none of them**: the values themselves remain stored on the
>   linked record"* → **dos tablas** y una FK obligatoria. Eso es la delegacion.
> - "Prototipo" es el caso de arriba (`_inherit` + `_name` distinto): copia la estructura en una tabla
>   propia con datos independientes.
>
> Por que importa aqui y no es una discusion de nombres: esta skill **clasifica** para que otras
> decidan alcance de test e impacto. "Delegada" implica dos tablas y escritura al padre; si el modelo
> es en realidad una extension in-place, el alcance calculado es el equivocado en las dos direcciones.
> `odoo-orm-modeler/references/inheritance-and-relations.md:13` y
> `odoo-impact-analyzer/SKILL.md:84` ya usaban el vocabulario correcto: esta tabla era la discrepante.

| Patron encontrado | Tipo de modelo | Prioridad de test |
|-------------------|---------------|-------------------|
| `_name = 'x'` sin `_inherit` | **Nuevo** | CRUD + logica completo |
| `_name = 'x'` con `_inherit = 'x'` | **Extension** (mismo modelo) | Solo comportamiento anadido |
| `_inherit = 'y'` sin `_name` | **Extension in-place** (MISMA tabla) | Metodos sobreescritos |
| `_name = 'x'` con `_inherit = 'y'` | **Prototipo** (copia estructural, tabla propia) | Completo + verificar herencia |
| `_inherits = {'y': 'y_id'}` | **Delegacion / composicion** (DOS tablas + FK required) | Completo + campo delegacion |
| `_transient = True` | **Wizard** | Form tests + accion ejecutar |

---

## Fase 4: Clasificacion de campos

| Patron del campo | Atributo clave | Relevancia |
|-----------------|----------------|------------|
| `compute='_compute_X'` | Calculado | HIGH — necesita test unitario |
| `compute='_compute_X', store=True` | Calculado y almacenado | HIGH — test + triggers |
| `compute='_compute_X', inverse='_inverse_X'` | Con inverso | HIGH — test compute + inverse |
| `related='path.to.field'` | Relacionado | MEDIUM — test propagacion |
| `default=lambda self:` | Default dinamico | MEDIUM — test via Form |
| `required=True` | Obligatorio | LOW — test constraint implicita |
| `tracking=True` | Con seguimiento | LOW — test chatter (si mail en depends) |
| `groups='group.xml_id'` | Con restriccion de grupo | MEDIUM — test seguridad |
| `currency_field='currency_id'` | Monetario | MEDIUM — test multi-moneda |

---

## Fase 5: Deteccion de controladores

```bash
# Controladores
grep -rn "@http.route" --include="*.py" <module>/controllers/
grep -rn "class.*http.Controller" --include="*.py" <module>/controllers/

# Tipo de auth
grep -rn "auth=['\"]public['\"]" --include="*.py" <module>/controllers/
grep -rn "auth=['\"]user['\"]" --include="*.py" <module>/controllers/
grep -rn "auth=['\"]none['\"]" --include="*.py" <module>/controllers/

# Tipo de request
grep -rn "type=['\"]json['\"]" --include="*.py" <module>/controllers/
grep -rn "type=['\"]http['\"]" --include="*.py" <module>/controllers/

# Website
grep -rn "website=True" --include="*.py" <module>/controllers/

# CSRF deshabilitado (flag de seguridad)
grep -rn "csrf=False" --include="*.py" <module>/controllers/
```

---

## Fase 6: Deteccion de seguridad

```bash
# Grupos custom
grep -rn "category_id\|group_" --include="*.xml" <module>/security/

# Record rules
grep -rn "model=\"ir.rule\"" --include="*.xml" <module>/security/

# Multi-company
grep -rn "company_id" --include="*.xml" <module>/security/

# Accesos en CSV
# Leer ir.model.access.csv linea por linea
```

**CRITICO**: Si no existe `security/ir.model.access.csv` y el modulo define modelos nuevos, es un **fallo de seguridad**.

---

## Fase 7: Deteccion de JavaScript

```bash
# Componentes OWL
grep -rn "extends Component\|class.*Component" --include="*.js" <module>/static/

# Widgets custom
grep -rn "extends AbstractField\|extends Widget" --include="*.js" <module>/static/

# Registros en registry
grep -rn "registry.category" --include="*.js" <module>/static/

# Servicios
grep -rn "useService\|useEnv" --include="*.js" <module>/static/

# Llamadas RPC
grep -rn "this\.rpc\|this\.orm\.call\|useService.*rpc" --include="*.js" <module>/static/
```

---

## Fase 8: Metricas de complejidad

```bash
# Lineas de codigo por tipo
wc -l <module>/models/*.py 2>/dev/null
wc -l <module>/controllers/*.py 2>/dev/null
find <module>/static/src -name "*.js" -exec wc -l {} + 2>/dev/null
find <module>/views -name "*.xml" -exec wc -l {} + 2>/dev/null
```

### Clasificacion por complejidad

| Metrica | SIMPLE | STANDARD | COMPLEX | ADVANCED |
|---------|--------|----------|---------|----------|
| Lineas Python | < 200 | 200-1000 | 1000-5000 | > 5000 |
| Modelos nuevos | 0-1 | 2-5 | 6-15 | > 15 |
| Herencias | 0-2 | 3-8 | 9-20 | > 20 |
| Controladores | 0 | 0-2 | 3-10 | > 10 |
| Componentes JS | 0 | 0-1 | 2-5 | > 5 |
| APIs externas | 0 | 0 | 1-2 | > 2 |

---


## Lo que el motor NO veía (medido el 2026-08-16, banco `tools/bench-scanner`)

Tres puntos ciegos, los tres **sin bandera de duda**: el inventario salía como fiable.

| Defecto | Consecuencia |
|---|---|
| solo se miraban `models/` y `wizard(s)/` | un modelo en `model/` (singular), en `report/` o en la raíz **no existía** para el inventario |
| `_inherit = ["a", "b"]` | **invisible** — el patrón exigía comilla justo tras el `=`. En 174 módulos reales, la forma de lista aparece **43 veces** |
| `_name` sin anclar | `_rec_name = "display_name"` casa con `_name\s*=`, y el modelo entraba **llamándose `display_name`** (caso real: `OCA/account_banking_sepa_direct_debit`) |

Importa más de lo que parece porque **este motor es la entrada de otras dos skills**:
`odoo-module-creator` y `odoo-test-generator` lo invocan antes de diseñar o de generar tests. Un
mixin que no aparece en el inventario es un mixin para el que nadie escribe pruebas.

Corregido y con banco de regresión (`check.py`, 8/8), más el control que de verdad manda: **176
módulos reales comparados contra el AST de Python, 0 diferencias**.

## Fase 9: Deteccion de patrones deprecated

Para informar al usuario de posibles problemas (no para corregir — eso es `odoo-code-auditor`):

```bash
# APIs deprecated
grep -rn "@api.one\|@api.returns\|fields.function" --include="*.py" <module>/
grep -rn "name_get\b" --include="*.py" <module>/models/
grep -rn "group_operator" --include="*.py" <module>/models/

# Patrones de seguridad preocupantes
grep -rn "\.execute.*%" --include="*.py" <module>/
grep -rn "t-raw" --include="*.xml" <module>/
grep -rn "\.sudo()" --include="*.py" <module>/
```
