# Golden Rules — Reglas Obligatorias de Clasificacion

Estas reglas se evaluan ANTES que cualquier otra clasificacion. Si una Golden Rule aplica, el resultado es OBLIGATORIO independientemente de otros factores.

## Golden Rule 1: DATA & INTEGRATIONS

**Si la funcionalidad es Import/Export** (Excel, CSV, conectores, integraciones API):

| Contexto | Resultado |
|----------|-----------|
| Evaluacion nueva / Refactoring / No-Studio | → 🟣 **Core OBLIGATORIO** (`indaws_data_import` o `indaws_data_export`) |
| Migracion 1:1 | → 🔴 Custom (migrar tal cual) |

**Razon:** Indaws tiene modulos Core especificos para importacion/exportacion de datos que cubren el 95% de los casos. Recomendar desarrollo custom para esto es malgastar recursos.

**Ejemplos que activan esta regla:**
- Importar contactos desde Excel
- Exportar facturas a CSV
- Sincronizar productos con API externa
- Conector bidireccional con ERP/WMS externo
- Importacion automatica desde SFTP

## Golden Rule 2: DASHBOARDS & KPIs

**Si la funcionalidad es Dashboards/KPIs/Business Intelligence:**

| Contexto | Resultado |
|----------|-----------|
| Evaluacion nueva / Refactoring / No-Studio | → 🟢 **Standard OBLIGATORIO** (Spreadsheets nativos de Odoo) |
| Migracion 1:1 | → 🔴 Custom (migrar tal cual) |

**Razon:** Odoo incluye Spreadsheets nativo (Enterprise) que cubre dashboards y KPIs sin desarrollo. Recomendar custom para esto ignora funcionalidad existente.

**Ejemplos que activan esta regla:**
- Dashboard de ventas con graficos
- Cuadro de mando de produccion
- KPIs de rendimiento por departamento
- Informe interactivo de margen comercial
- Metricas de helpdesk/SLA

## Golden Rule 3: MODIFICAR UN ESTANDAR = HERENCIA, NUNCA COPIA

**Si la funcionalidad es "cambiar/ampliar algo que ya existe en Odoo estandar"** (un informe, una vista,
un flujo del core), la via es **heredar/extender**, no clonar:

| Enfoque | Resultado |
|---------|-----------|
| Herencia (vista `mode="extension"` con xpath, `_inherit`, gestionar el binding) | → ✅ correcto |
| Copiar la plantilla/vista estandar entera y editar la copia | → ❌ anti-patron |

**Razon:** copiar un estandar **degrada la fidelidad** y se pudre con el tiempo — te pierdes las
variables computadas y widgets calculados del core (p.ej. en un informe: `lines_to_report`,
el widget `tax_totals`/`o_total_table`), y dejas de recibir las mejoras/fixes del core en cada
actualizacion. La herencia por xpath es **reversible** y sobrevive a los upgrades.

**Caso "sustituir el informe estandar por el mio" sin tocar la plantilla:** se gestiona el **binding**
del menu Imprimir (`ir.actions.report.binding_model_id` / `binding_type`), desvinculando el estandar y
bindeando el custom — reversible, con el original intacto. (Detalle tecnico en
`odoo-qweb-report-builder/references/report-structure.md`.)

**Ejemplos que activan esta regla:** "quiero el presupuesto de ventas pero con una columna mas",
"cambiar el pie de la factura", "anadir un campo al formulario de contacto del core". → herencia, no copia.

## Core Supremacy Rule

**Si un modulo Indaws Core resuelve el problema, es OBLIGATORIO usarlo** (excepto en migracion 1:1).

Esta regla aplica DESPUES de las Golden Rules y ANTES de evaluar Studio o Custom:
- Si Core cubre el 100% → 🟣 Core
- Si Core cubre el 80%+ y el gap es simple (<100 lineas) → 🟣 Core + modulo bridge
- Si Core cubre <80% o el gap es complejo → Evaluar Studio o Custom

**Donde buscar modulos Core:** para los indices de modulos Indaws Core (123 modulos core + 8 enterprise + 15 construccion vertical), delega en la skill `content-collection`.

**Estrategia de busqueda:** nombre tecnico → descripcion → dependencias → categoria → version
