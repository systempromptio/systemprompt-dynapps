# Flujos de Demo por Aplicacion Odoo

## Indice de Flujos por App

| App | Flujos | Duracion estimada |
|-----|--------|-------------------|
| CRM | Lead → Oportunidad → Presupuesto | 8-10 min |
| Ventas | Quote-to-Cash completo | 10-12 min |
| Compras | Solicitud → Recepcion → Pago | 8-10 min |
| Inventario | Recepcion + Envio + Dashboard | 8-10 min |
| Fabricacion | Orden produccion + Planning | 10-12 min |
| Contabilidad | Facturacion + Conciliacion | 8-10 min |
| RRHH | Alta + Ausencias + Gastos | 10-12 min |
| Proyecto | Kanban + Timesheets | 8-10 min |
| Helpdesk | Ticket → Resolucion + SLA | 8-10 min |
| eCommerce | Tienda → Pedido | 10-12 min |

---

## CRM: Lead → Oportunidad → Presupuesto

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Mostrar pipeline Kanban | "Toda su cartera comercial de un vistazo" |
| 2 | Abrir una oportunidad | "Cada oportunidad tiene todo el contexto: contacto, historico, valor" |
| 3 | Mostrar chatter con emails | "Toda la comunicacion centralizada, sin buscar en el email" |
| 4 | Drag-and-drop de etapa | "Actualizar el estado es tan facil como arrastrar" |
| 5 | Crear presupuesto desde oportunidad | "Desde la oportunidad, un clic para crear el presupuesto" |
| 6 | Mostrar presupuesto con lineas | "El presupuesto ya tiene los datos del cliente, solo añadir productos" |

### Momento WOW
**Pipeline Kanban + Drag-and-drop**: La vista Kanban del pipeline es visualmente impactante. El drag-and-drop de oportunidades entre etapas genera reaccion inmediata.

### Datos Necesarios
- 8-15 oportunidades en diferentes etapas
- 2-3 con actividades programadas
- 1 con emails en el chatter
- Productos para el presupuesto

### Precauciones
- No mostrar configuracion de etapas (es setup, no uso diario)
- No abrir multiples oportunidades (una es suficiente)

---

## Ventas: Quote-to-Cash

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Crear presupuesto rapido | "Su comercial crea un presupuesto en menos de un minuto" |
| 2 | Añadir productos con buscador | "Buscar productos es instantaneo, con precios actualizados" |
| 3 | Enviar por email al cliente | "Un clic para enviar al cliente con PDF profesional" |
| 4 | Mostrar portal del cliente | "El cliente ve su presupuesto online y puede confirmar" |
| 5 | Confirmar pedido | "Cuando el cliente acepta, se convierte en pedido de venta" |
| 6 | Crear factura desde pedido | "La factura se genera automaticamente con todos los datos" |
| 7 | Registrar pago | "Y el ciclo se cierra: presupuesto → pedido → factura → cobro" |

### Momento WOW
**3 clics de presupuesto a cobro**: La trazabilidad completa desde la oportunidad hasta el pago es el momento mas impactante para equipos comerciales.

### Datos Necesarios
- 5-10 productos con precios y imagenes
- 3-5 clientes con direcciones
- 1 lista de precios (opcional)
- Cuenta bancaria configurada

### Precauciones
- No mostrar configuracion de impuestos (asumirlos configurados)
- No profundizar en contabilidad (es otro flujo)
- Si el cliente no usa portal, saltar paso 4

---

## Compras: Solicitud → Recepcion → Pago

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Crear pedido de compra | "Su departamento de compras puede gestionar proveedores y pedidos" |
| 2 | Añadir productos | "Productos con proveedores preferidos y precios negociados" |
| 3 | Confirmar pedido | "Una vez aprobado, el proveedor recibe el pedido" |
| 4 | Recibir mercancia | "Cuando llega la mercancia, se registra contra el pedido" |
| 5 | Crear factura de proveedor | "La factura del proveedor se coteja automaticamente" |
| 6 | Mostrar reglas de reabastecimiento | "Y lo mejor: Odoo puede pedir automaticamente cuando baja el stock" |

### Momento WOW
**Reabastecimiento automatico**: Mostrar como Odoo detecta stock bajo y genera automaticamente un pedido de compra al proveedor.

### Datos Necesarios
- 3-5 proveedores con productos asociados
- Productos con stock minimo configurado
- Ubicaciones de almacen

### Precauciones
- No mostrar multiples almacenes (complejidad innecesaria en primera demo)
- Reglas de reabastecimiento solo si el cliente tiene inventario

---

## Inventario: Recepcion + Envio + Dashboard

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Mostrar dashboard de inventario | "Vision global del estado de su almacen en tiempo real" |
| 2 | Abrir listado de productos | "Cada producto con su stock actual, valoracion y movimientos" |
| 3 | Ejecutar recepcion (entrada) | "Registrar la entrada de mercancia con lectura de codigos o manual" |
| 4 | Ejecutar envio (salida) | "Los envios se generan automaticamente desde los pedidos de venta" |
| 5 | Mostrar trazabilidad | "Pueden rastrear cada producto: de donde vino, donde esta, a donde fue" |
| 6 | Informe de valoracion de stock | "Valor del inventario actualizado al instante" |

### Momento WOW
**Stock en tiempo real + Trazabilidad**: Click en un producto y ver todos sus movimientos, de donde vino y donde esta ahora. El dashboard con graficos actualizados impresiona.

### Datos Necesarios
- 10-20 productos con stock
- Movimientos de entrada y salida recientes
- Al menos 2 ubicaciones (si aplica)
- Lotes o numeros de serie (si el sector lo requiere)

### Precauciones
- No mostrar rutas complejas (multi-step) en primera demo
- Evitar configuracion de almacenes

---

## Fabricacion: Orden Produccion + Planning

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Mostrar lista de materiales (BOM) | "Definen que componentes necesita cada producto" |
| 2 | Crear orden de produccion | "Una orden de produccion con todo: componentes, operaciones, tiempo" |
| 3 | Iniciar produccion | "Sus operarios ven que hay que fabricar y registran avance" |
| 4 | Mostrar planning (calendario) | "Vision de toda la planificacion de fabricacion" |
| 5 | Completar produccion | "Al terminar, el producto terminado entra automaticamente en stock" |
| 6 | Mostrar trazabilidad de produccion | "Saben exactamente que componentes entraron en cada producto" |

### Momento WOW
**Planning drag-and-drop**: La vista de planificacion de fabricacion con drag-and-drop para reprogramar ordenes. Visual e intuitivo.

### Datos Necesarios
- 3-5 productos fabricados con BOM
- Componentes con stock
- Centros de trabajo configurados
- 2-3 ordenes de produccion en diferentes estados

### Precauciones
- No mostrar BOM multinivel en primera demo (demasiada complejidad)
- Si el cliente no fabrica, OMITIR este flujo

---

## Contabilidad: Facturacion + Conciliacion

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Mostrar dashboard contable | "Estado financiero de la empresa de un vistazo" |
| 2 | Crear factura de cliente | "Facturacion integrada: se genera desde ventas o manualmente" |
| 3 | Enviar factura por email | "Envio directo al cliente con PDF adjunto" |
| 4 | Importar extracto bancario | "Importar los movimientos del banco es automatico" |
| 5 | Conciliacion automatica | "Odoo propone automaticamente los matches entre facturas y pagos" |
| 6 | Informes financieros | "Balance, P&L, flujo de caja: todo en tiempo real" |

### Momento WOW
**Conciliacion bancaria automatica**: Odoo propone automaticamente la conciliacion entre movimientos bancarios y facturas. Es rapido y visualmente claro.

### Datos Necesarios
- 5-10 facturas de cliente (pagadas y pendientes)
- 3-5 facturas de proveedor
- Extracto bancario para conciliar (o datos de ejemplo)
- Plan contable configurado

### Precauciones
- NUNCA mostrar configuracion del plan contable
- Evitar hablar de asientos contables (el cliente quiere facturar, no contabilizar)
- Si el cliente tiene asesor externo, enfocar en facturacion, no en contabilidad completa

---

## RRHH: Alta + Ausencias + Gastos

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Ficha de empleado | "Toda la informacion del empleado centralizada" |
| 2 | Crear solicitud de ausencia | "El empleado solicita vacaciones desde su portal" |
| 3 | Aprobar ausencia (manager) | "El responsable aprueba con un clic, se actualiza el calendario" |
| 4 | Calendario de ausencias | "Vision global de ausencias del equipo" |
| 5 | Crear nota de gastos | "El empleado fotografía el ticket y crea el gasto desde el movil" |
| 6 | Aprobar gastos | "Aprobacion rapida con vista de todos los gastos pendientes" |

### Momento WOW
**App movil para gastos**: Mostrar (o describir) como el empleado fotografía un ticket con el movil y en segundos crea una nota de gastos. La aprobacion del manager es un clic.

### Datos Necesarios
- 5-10 empleados con departamentos
- Tipos de ausencia configurados
- Algunas ausencias aprobadas en el calendario
- Categorias de gastos

### Precauciones
- No mostrar nominas (es un modulo separado y complejo)
- No profundizar en contratos (setup, no demo)

---

## Proyecto: Kanban + Timesheets

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Tablero Kanban de proyectos | "Todos sus proyectos organizados visualmente" |
| 2 | Abrir proyecto y ver tareas | "Cada proyecto con sus tareas, responsables y fechas" |
| 3 | Mover tarea entre etapas | "Actualizar el progreso es tan simple como arrastrar" |
| 4 | Registrar horas (timesheet) | "Los consultores registran horas directamente en la tarea" |
| 5 | Vista calendario/Gantt | "Planificacion visual del equipo y del proyecto" |
| 6 | Facturar horas automaticamente | "Las horas registradas se convierten en factura automaticamente" |

### Momento WOW
**Facturacion automatica de horas**: Del timesheet a la factura con un clic. El cliente ve como las horas se convierten en ingresos sin trabajo manual.

### Datos Necesarios
- 2-3 proyectos con tareas en diferentes etapas
- Timesheets registrados
- Al menos 1 proyecto facturable

### Precauciones
- No mostrar configuracion de etapas de proyecto
- Si el cliente no factura horas, omitir paso 6

---

## Helpdesk: Ticket → Resolucion + SLA

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Dashboard de tickets | "Vision global del soporte: abiertos, en progreso, resueltos" |
| 2 | Crear ticket (o recibir por email) | "Los clientes pueden crear tickets por email, portal o formulario" |
| 3 | Asignar y clasificar | "Asignacion automatica o manual, con prioridades y SLA" |
| 4 | Resolver ticket | "El agente responde, adjunta archivos, resuelve" |
| 5 | Mostrar portal del cliente | "El cliente sigue el estado de su ticket en tiempo real" |
| 6 | Informes de SLA | "Cumplimiento de SLA, tiempos de respuesta, satisfaccion" |

### Momento WOW
**Portal de autoservicio**: El cliente crea tickets, ve el estado, adjunta archivos y recibe respuestas sin necesidad de llamar.

### Datos Necesarios
- 5-10 tickets en diferentes estados
- SLAs configurados
- Categorias de ticket (tipos de incidencia)

### Precauciones
- No mostrar configuracion de SLA (asumirlo configurado)
- Si el cliente no tiene helpdesk, OMITIR este flujo

---

## eCommerce: Tienda → Pedido

### Pasos de Demo

| Paso | Accion | Talking Point |
|------|--------|---------------|
| 1 | Mostrar tienda online | "Su catalogo online, integrado con inventario y precios" |
| 2 | Navegar productos | "Los clientes buscan, filtran y comparan productos" |
| 3 | Añadir al carrito y comprar | "Proceso de compra estandar, con pasarela de pago integrada" |
| 4 | Mostrar pedido en backend | "El pedido aparece automaticamente en ventas" |
| 5 | Verificar stock actualizado | "El stock se descuenta automaticamente al confirmar" |
| 6 | Portal del cliente | "El cliente ve sus pedidos y facturas desde su area privada" |

### Momento WOW
**Integracion tienda-inventario**: Cuando un cliente compra online, el stock se actualiza en tiempo real y el pedido aparece en el backend de ventas. Todo conectado.

### Datos Necesarios
- 10-20 productos publicados con imagenes
- Categorias de tienda
- Metodo de pago configurado (al menos transferencia)
- Al menos 1 pedido completado

### Precauciones
- No mostrar configuracion del sitio web
- Tener el tema y colores preparados (no mostrar el default)
- Si la tienda esta vacia visualmente, usar capturas de respaldo

---

## Flujos Combinados

### Lead-to-Invoice (CRM + Ventas + Contabilidad)
**Duracion**: 12-15 min
**Historia**: "Un lead llega por el sitio web. El comercial lo cualifica, crea un presupuesto, el cliente acepta, se genera la factura y se cobra."
**Pasos**: CRM (1-5) → Ventas (2-7) → Contabilidad (3)
**WOW**: Trazabilidad completa: click en factura → pedido → presupuesto → oportunidad

### Procure-to-Pay (Compras + Inventario + Contabilidad)
**Duracion**: 12-15 min
**Historia**: "Detectamos que un producto esta bajo de stock. Se genera un pedido al proveedor, llega la mercancia, se registra y se paga la factura."
**Pasos**: Inventario (dashboard) → Compras (1-6) → Inventario (3) → Contabilidad (5)
**WOW**: Reabastecimiento automatico + conciliacion

### Make-to-Order (Ventas + Fabricacion + Inventario)
**Duracion**: 12-15 min
**Historia**: "Un cliente pide un producto fabricado. Se genera la orden de produccion, se fabrica y se envia al cliente."
**Pasos**: Ventas (1-5) → Fabricacion (2-5) → Inventario (4)
**WOW**: Del pedido del cliente a la orden de produccion automaticamente

### Day-in-the-Life (Multi-app)
**Duracion**: 15-20 min
**Historia**: "Un dia tipico en su empresa: el comercial gestiona oportunidades, el almacenero recibe mercancia, la contable concilia pagos, el director ve los KPIs."
**Pasos**: CRM (1-4) → Inventario (3) → Contabilidad (5) → Dashboard gerencial
**WOW**: Todo conectado, cada persona ve lo que necesita

---

## Criterios de Seleccion de Flujos

Al elegir flujos para una demo, considerar:

1. **Pain points del cliente**: Priorizar flujos que resuelvan sus problemas principales
2. **Sector**: Fabricacion para industriales, Helpdesk para servicios, eCommerce para retail
3. **Tamaño de empresa**: Empresas pequeñas → flujos simples. Medianas → flujos combinados
4. **Audiencia**: Gerencia → flujos combinados + dashboards. Operativos → flujos especificos
5. **Tiempo disponible**: 30 min → 2 flujos. 60 min → 3-4 flujos. 90 min → 4-5 flujos
6. **Datos disponibles**: Solo mostrar flujos para los que se tienen datos demo preparados
