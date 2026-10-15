# Odoo Changelogs - Funcionalidades Clave por Version

Referencia de funcionalidades principales anadidas en cada version de Odoo.
Enfocado en detectar si una funcionalidad existe en estandar antes de proponer desarrollo.

Ultima actualizacion: 2026-02-24

---

## Odoo 19.0 (2025)

### Contabilidad
- Nuevo motor de conciliacion bancaria rediseñado con IA mejorada
- Soporte nativo para consolidacion multi-empresa mejorado
- Mejora del modulo de activos fijos

### CRM
- Nuevo sistema de scoring de leads basado en reglas configurables
- Vista de pipeline rediseñada con drag & drop mejorado
- Integracion nativa con WhatsApp Business API

### Ventas
- Catalogo de productos con vista de cuadricula
- Mejora del configurador de productos
- Gestion de descuentos por volumen nativa

### Inventario
- Nuevo modulo de planificacion de demanda (demand planning)
- Reglas de reabastecimiento mejoradas con IA
- Soporte para dropshipping mejorado

### Fabricacion
- Planificacion de produccion con IA
- Mejoras en la gestion de subcontratacion
- Control de calidad integrado en ordenes de trabajo

### Website / eCommerce
- Nuevo builder de paginas con componentes drag & drop
- Checkout rediseñado con menos pasos
- Integracion nativa con Google Merchant Center

### RRHH
- Nuevo modulo de evaluacion del desempeño (Appraisals mejorado)
- Mejoras en planificacion de turnos

### General
- Nuevo sistema de notificaciones push
- Vistas `<list>` reemplazan `<tree>` en XML
- API: `_compute_display_name` reemplaza `name_get()`
- Spreadsheets mejorado con mas funciones y conectores

---

## Odoo 18.0 (2024)

### Contabilidad
- Modulo de consolidacion contable (account_consolidation) incluido en Enterprise
- Sincronizacion automatica de tipos de cambio mejorada
- Nuevo asistente de configuracion contable inicial
- Soporte mejorado para multi-divisa con revaluacion automatica
- Mejoras en el motor de conciliacion bancaria con propuestas automaticas

### CRM
- Duplicados de leads detectados automaticamente
- Nuevo sistema de lead mining integrado
- Pipeline con vistas Kanban mejoradas
- Actividades planificadas con recordatorios automaticos

### Ventas
- Firmas electronicas en pedidos de venta (integrado con Sign)
- Mejoras en suscripciones y ventas recurrentes
- Catalogo de productos por cliente
- Descuentos en cascada mejorados
- Nuevo flujo de aprobacion de ventas nativo

### Compras
- Mejoras en acuerdos de compra (blanket orders)
- Portal de proveedores mejorado
- Integracion con presupuestos de venta (dropship)

### Inventario
- Nuevo modulo de codigos de barras rediseñado
- Reglas de almacenamiento (putaway) mejoradas
- Trazabilidad de lotes y numeros de serie mejorada
- Inventario por ciclos (cycle counting) mejorado
- Picking por olas (wave picking)

### Fabricacion
- Subcontratacion mejorada con portal de subcontratista
- Ordenes de trabajo en tablet rediseñadas
- MRP con planificacion por capacidad

### Proyectos
- Tareas con checklist ponderados
- Vista de mapa para tareas con geolocalizacion
- Mejoras en la planificacion de recursos

### Website / eCommerce
- Comparador de productos nativo
- Lista de deseos (wishlist) mejorada
- Filtros de busqueda avanzados en tienda
- Mejoras de rendimiento y SEO

### Helpdesk
- SLA con escalado automatico
- Portal del cliente mejorado
- Integracion con base de conocimiento

### RRHH
- Nuevo modulo de planificacion de turnos (Planning)
- Mejoras en control de asistencia con kiosko
- Evaluaciones de desempeño mejoradas

### Firma Electronica (Sign)
- Flujos de firma multi-firmante
- Campos personalizados en documentos
- Integracion con ventas y RRHH

### General
- Odoo Spreadsheets (hojas de calculo integradas para BI/reporting)
- Nuevo framework de reportes con filtros avanzados
- Studio mejorado: acciones automatizadas con Python basico
- Mensajeria interna (Discuss) rediseñada
- API JSON-RPC/JSON2 estable
- Mejoras de rendimiento general

---

## Odoo 17.0 (2023)

### Contabilidad
- Conciliacion bancaria rediseñada con matching automatico
- Nuevo modulo de gestion de activos
- Mejoras en reportes contables (Balance, P&L, Libro Mayor)
- Soporte mejorado para SEPA y pagos bancarios
- Modulo de presupuestos contables

### CRM
- Nuevo sistema de lead nurturing
- Predictive lead scoring con IA
- Integracion con VoIP mejorada

### Ventas
- Nuevo catalogo de productos interactivo
- Margenes en tiempo real en lineas de pedido
- Flujo de upselling mejorado

### Compras
- Aprobacion multi-nivel de ordenes de compra nativa
- Evaluacion de proveedores basica

### Inventario
- Reabastecimiento automatico con lead times dinamicos
- Mejoras en cross-docking
- Nuevo modulo de codigos de barras movil

### Fabricacion
- MRP con scheduling mejorado
- Subcontratacion como modulo separado
- Workorders con instrucciones paso a paso

### Proyectos
- Milestones (hitos) de proyecto
- Burndown charts
- Integracion mejorada con timesheets

### Website / eCommerce
- Nuevo page builder con snippets mejorados
- Multiwebsite mejorado
- Portal del cliente rediseñado

### RRHH
- Nuevo modulo de reclutamiento rediseñado
- Referral program para empleados
- Fleet management mejorado

### General
- OWL 2 como framework frontend (reemplaza legacy widgets)
- Mejoras de rendimiento en listas y formularios
- Studio: computed fields con formulas
- `aggregator` en campos Float/Integer (sustituye `group_operator`)
- `_read_group()` con nueva signatura

---

## Odoo 16.0 (2022)

### Contabilidad
- Conciliacion bancaria completamente rediseñada
- Reportes contables reconstruidos
- Mejoras de multi-divisa
- Modulo de impuestos diferidos

### CRM
- Pipeline mejorado
- Vistas de informes de ventas

### Ventas
- Productos por suscripcion (sale_subscription Enterprise)
- Mejoras en tarifas y listas de precios
- eSign integrado

### Inventario
- Nuevo modulo de calidad (Quality)
- Mejoras en trazabilidad
- Reglas de reabastecimiento mejoradas

### Fabricacion
- PLM (Product Lifecycle Management) mejorado
- Subcontratacion basica

### Website / eCommerce
- Rendimiento mejorado
- Nuevo tema por defecto
- Mejoras en SEO

### General
- OWL 1 como framework frontend oficial
- Nuevo sistema de assets (bundles)
- Mejoras de rendimiento significativas
- `_read_group()` nueva API (16.3+)
- Python 3.10+ requerido

---

## Odoo 15.0 (2021)

### Contabilidad
- Motor de conciliacion nuevo
- Reportes contables mejorados
- Integracion bancaria mejorada (Plaid, Yodlee, Ponto)

### Ventas
- Catalogos de productos en PDF
- Suscripciones rediseñadas

### Inventario
- Push/Pull rules simplificadas
- Valoracion de inventario mejorada

### Website / eCommerce
- Theme editor mejorado
- Portal de empleado

### General
- `name_get()` todavia activo
- `group_operator` para campos Float
- Legacy JS (no OWL)
- Python 3.8+

---

## Odoo 14.0 (2020)

### Contabilidad
- Nuevo modulo de contabilidad (reescritura completa)
- Reportes contables basicos
- Pagos en linea

### Ventas
- Portal del cliente mejorado

### Inventario
- Multi-paso de almacen simplificado

### Website / eCommerce
- Editor de website rediseñado

### General
- Python 3.6+
- Legacy JS framework
- `name_get()` como metodo principal
- `group_operator` en campos
- Ultimo version con soporte para decorators legacy

---

## Notas de Compatibilidad API

| Feature | 14.0-15.0 | 16.0 | 17.0 | 18.0 | 19.0 |
|---------|-----------|------|------|------|------|
| `name_get()` | Obligatorio | Funciona | Funciona | Deprecated | Eliminado |
| `_compute_display_name` | No existe | No existe | Disponible | Recomendado | Obligatorio |
| `group_operator` | Obligatorio | Funciona | Deprecated | Deprecated | Eliminado |
| `aggregator` | No existe | No existe | Disponible (17.2+) | Recomendado | Obligatorio |
| Frontend | Legacy JS | OWL 1 | OWL 2 | OWL 2+ | OWL 2+ |
| Python | 3.6+ | 3.10+ | 3.10+ | 3.10+ | 3.10+ |
| Views `<tree>` | Obligatorio | Funciona | Funciona | Funciona | Usar `<list>` |
| `search_fetch()` | No existe | No existe | No existe | Disponible | Recomendado |

---

## Studio - Capacidades por Version

| Capacidad | 16.0 | 17.0 | 18.0 | 19.0 |
|-----------|------|------|------|------|
| Campos custom | Si | Si | Si | Si |
| Vistas custom | Si | Si | Si | Si |
| Acciones automatizadas | Basicas | Mejoradas | Con Python basico | Con Python basico |
| Campos computados | No | Formula basica | Formula basica | Formula mejorada |
| Approval flows | No | No | Basico | Mejorado |
| Reportes custom | Limitado | Limitado | Mejorado | Mejorado |
| Dashboards | No | No | Basico | Si |

---

## Fuentes

- Odoo Release Notes: https://www.odoo.com/odoo-{version}-release-notes
- Odoo Documentation: https://www.odoo.com/documentation/{version}
- Odoo GitHub: https://github.com/odoo/odoo/tree/{version}
