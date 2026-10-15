# Principios Arquitectura Módulos Odoo

## Principios

1. **Single Responsibility**: cada módulo = 1 problema de negocio. Si necesita "and" → dividir
2. **Nunca modificar core**: SIEMPRE extender con herencia en módulo custom
3. **Dependencias mínimas**: solo lo estrictamente necesario en manifest
4. **Bridge modules**: si A y B son independientes pero interactúan → módulo C con `auto_install=True`
5. **Separation of Concerns**: modelos=lógica, vistas=presentación, security=permisos, JS=frontend

## Naming Conventions

- Modelos: `module.name` (dot notation)
- Campos: `snake_case`
- XML IDs: `module_model_view_type`
- ACLs: `access_model_name_group`
- Menús: `menu_module_name`

## Estructura Módulo

```
module_name/
├── __init__.py, __manifest__.py
├── models/          # Python models
├── views/           # XML views
├── security/        # ir.model.access.csv + security.xml
├── data/            # datos iniciales
├── wizards/         # transient models
├── reports/         # QWeb reports
├── static/src/      # js/, xml/ (OWL), scss/
├── controllers/     # HTTP
├── tests/           # unit tests
└── i18n/            # traducciones
```

## Patrones Diseño

- **Herencia**: `_inherit = 'model'` (extensión sin nuevo modelo)
- **Delegación**: `_inherits = {'product.product': 'product_id'}`
- **Mixin**: `AbstractModel` reutilizable
- **Wizard**: `TransientModel` para flujos temporales

## Rendimiento

| Regla | Malo → Bueno |
|-------|-------------|
| No N+1 | `for rec: rec.partner_id.name` → `records.mapped()` |
| Batch ops | `for: Model.create(d)` → `Model.create(list)` |
| Agregados | `sum(r.amount for r in recs)` → `read_group()` |
| Indexar | `Char()` → `Char(index=True)` si se filtra |
| Limitar | `search([])` → `search([], limit=X)` |
