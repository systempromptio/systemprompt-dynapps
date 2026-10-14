# Horas por Aplicacion

Tabla unificada de duracion por app segun complejidad.

Implantacion funcional: **no lleva factor IA** (configuracion, reuniones y formacion van a ×1.00).

---

## Configuraciones Iniciales

**Siempre 1 semana (10h)**
- Companias, usuarios, idiomas, monedas
- Impuestos, secuencias, datos maestros base
- Setup tecnico del entorno

---

## Tabla de Apps

### CRM

| Semanas | Criterio |
|---------|----------|
| 1 | CRM basico: pipeline estandar, sin automatizaciones ni desarrollos |
| 2 | CRM con reglas de asignacion, equipos de ventas, algunas automatizaciones |
| 3 | CRM complejo: multiples pipelines, integraciones, desarrollos asociados |

**Senales de complejidad**: multiples equipos comerciales, reglas de asignacion automatica, integracion con web/formularios, scoring de leads, desarrollos custom.

### Ventas

| Semanas | Criterio |
|---------|----------|
| 2 | Ventas estandar: presupuestos, pedidos, tarifas basicas |
| 3 | Tarifas complejas, multiples listas de precios, descuentos por volumen, comisiones |
| 4 | Ventas multi-compania, intercompany, procesos de aprobacion complejos, EDI |

**Senales de complejidad**: multiples tarifas, descuentos escalonados, proceso de aprobacion, venta intercompany, portal del cliente, tipos de pedido especiales.

### Compras

| Semanas | Criterio |
|---------|----------|
| 1 | Compras basicas: pedidos simples, pocos proveedores |
| 2 | Acuerdos de compra, multiples proveedores por producto, reglas de reabastecimiento |
| 3 | Compras con procesos de aprobacion, compras intercompany, EDI con proveedores |

**Senales de complejidad**: acuerdos marco, reglas de reabastecimiento automatico, aprobaciones multi-nivel, dropshipping, subcontratacion.

### Inventario

| Semanas | Criterio |
|---------|----------|
| 1 | Inventario basico: 1 almacen, entradas/salidas simples, sin trazabilidad |
| 2 | Multi-almacen O trazabilidad (lotes/series) O operaciones multi-paso |
| 3 | Picking avanzado, codigos de barras, rutas complejas, cross-docking |
| 4 | Todo lo anterior + multiples ubicaciones internas, reglas de reposicion complejas |

**Senales de complejidad**: mas de 1 almacen, trazabilidad por lotes/series, operaciones de picking, uso de scanner/codigos de barras, rutas de almacen personalizadas.

### Fabricacion

| Semanas | Criterio |
|---------|----------|
| 2 | Fabricacion basica: listas de materiales simples, ordenes manuales |
| 3 | Planificacion MRP, centros de trabajo, control de calidad basico |
| 4 | Subcontratacion, rutas avanzadas, planificacion de capacidad |
| 5-6 | MRP II completo, multiples centros, mantenimiento integrado, PLM |

**Senales de complejidad**: listas de materiales multinivel, operaciones de subcontratacion, centros de trabajo con capacidad, control de calidad, planificacion de capacidad, mantenimiento.

### Contabilidad

| Semanas | Criterio |
|---------|----------|
| 2 | Contabilidad estandar: facturacion, cobros/pagos, plan contable basico |
| 3 | Multi-compania, activos fijos, contabilidad analitica avanzada |
| 4 | Consolidacion, localizacion compleja, integracion bancaria avanzada |

**Senales de complejidad**: multi-compania, activos fijos, presupuestos contables, conciliacion bancaria avanzada, multiples diarios, contabilidad analitica multi-eje.

### Proyecto

| Semanas | Criterio |
|---------|----------|
| 1 | Gestion de proyectos basica: tareas, plazos, equipo |
| 2 | Hojas de horas, facturacion por proyecto, planificacion avanzada |

### RRHH

| Semanas | Criterio |
|---------|----------|
| 1 | RRHH basico: empleados, departamentos, ausencias |
| 2 | Nominas, gastos, evaluaciones, reclutamiento |
| 3 | RRHH completo: todos los modulos anteriores + flota + aprobaciones |

### Helpdesk

| Semanas | Criterio |
|---------|----------|
| 1 | Helpdesk basico: tickets, equipos, SLA simple |
| 2 | Multiples equipos, SLA complejos, portal del cliente, base de conocimiento |

### Pagina Web / eCommerce

| Semanas | Criterio |
|---------|----------|
| 1 | Web corporativa basica con Odoo Website |
| 2 | eCommerce simple: catalogo, carrito, pasarela de pago |
| 3 | eCommerce avanzado: multiples tarifas web, variantes, marketplace, envios |

### TPV (Terminal Punto de Venta)

| Semanas | Criterio |
|---------|----------|
| 1 | TPV basico: 1-2 terminales, productos simples |
| 2 | Multiples tiendas, programas de fidelizacion, restaurante |

### Suscripciones

| Semanas | Criterio |
|---------|----------|
| 1 | Suscripciones basicas: planes simples, facturacion recurrente |
| 2 | Multiples planes, upgrades/downgrades, metricas MRR, churn |

### Integraciones

**Siempre 1 semana por integracion como minimo.**

| Tipo | Semanas |
|------|---------|
| API REST estandar | 1 |
| EDI, sincronizacion bidireccional | 2 |
| Sistemas legacy o formatos propietarios | 2-3 |

### Apps Sencillas (1 semana cada una)

- Contactos (si se implanta independiente)
- Calendario
- Notas
- Conversaciones/Discuss
- Aprobaciones
- Encuestas
- Firma electronica
- Documentos
