# Temario Base por Aplicacion de Odoo

Este documento contiene el temario estandar de formacion para cada aplicacion principal de Odoo. El skill lo usa como punto de partida y lo adapta segun el proyecto, perfil de usuario y personalizaciones realizadas.

Cada app tiene: conceptos clave, temario por nivel y ejercicios tipo.

---

## Ventas (sale)

### Conceptos clave
- Pipeline de presupuestos: Borrador → Enviado → Pedido de venta → Factura
- Variantes de producto y listas de precios
- Plazos de pago y posiciones fiscales
- Equipos de ventas y asignacion de comerciales

### Temario por nivel
- **Basico** (1.5h): Crear cliente, crear presupuesto con productos, enviar por email, confirmar, crear factura
- **Intermedio** (+1.5h): Listas de precios, descuentos, plantillas de presupuesto, productos variantes, plazos de pago
- **Avanzado** (+1h): Informes de ventas, configuracion de equipos, reglas de aprobacion, intercompany, margenes

### Ejercicios tipo
1. "Crea un presupuesto para {CLIENTE} con 3 productos de {CATALOGO}, aplica un descuento del 10% al segundo producto"
2. "Envia el presupuesto por email al cliente, confirma como pedido de venta y genera la factura"
3. "Consulta el informe de ventas del ultimo mes filtrado por tu equipo comercial"
4. "Duplica el presupuesto anterior y cambiale los productos para otro cliente"

---

## CRM (crm)

### Conceptos clave
- Pipeline de oportunidades: Nuevo → Cualificado → Propuesta → Ganado/Perdido
- Etapas personalizables del pipeline
- Actividades programadas y seguimiento
- Lead vs Oportunidad

### Temario por nivel
- **Basico** (1.5h): Crear oportunidad, mover entre etapas, programar actividad, registrar nota, marcar ganado/perdido
- **Intermedio** (+1h): Leads y conversion, equipo de ventas, reglas de asignacion, informes de pipeline
- **Avanzado** (+1h): Lead scoring, automatizaciones, integracion con Ventas y Marketing

### Ejercicios tipo
1. "Crea una oportunidad para {EMPRESA} por {IMPORTE}€, asignala a tu equipo y programa una llamada para manana"
2. "Mueve la oportunidad a 'Propuesta', adjunta un presupuesto y registra una nota con el resumen de la reunion"
3. "Consulta tu pipeline filtrado por oportunidades > 5.000€ creadas este mes"

---

## Compras (purchase)

### Conceptos clave
- Flujo de compra: Solicitud → Pedido → Recepcion → Factura
- Acuerdos de compra (contratos marco)
- Aprobaciones de pedidos
- Comparacion de proveedores

### Temario por nivel
- **Basico** (1.5h): Crear proveedor, crear pedido de compra, confirmar, recibir productos, registrar factura
- **Intermedio** (+1.5h): Acuerdos de compra, aprobaciones, solicitudes de presupuesto a multiples proveedores, reabastecimiento
- **Avanzado** (+1h): Reglas de reabastecimiento automatico, dropshipping, informes de compras, evaluacion de proveedores

### Ejercicios tipo
1. "Crea un pedido de compra al proveedor {PROVEEDOR} con 3 productos, confirmalo y registra la recepcion"
2. "Solicita presupuesto a 3 proveedores para el producto {PRODUCTO} y compara precios"
3. "Registra la factura del proveedor asociada al pedido de compra y validala"

---

## Inventario (stock)

### Conceptos clave
- Almacenes, ubicaciones y rutas
- Operaciones: recepciones, entregas, transferencias internas
- Reglas de reabastecimiento
- Trazabilidad: lotes y numeros de serie

### Temario por nivel
- **Basico** (1.5h): Consultar stock, hacer transferencia interna, recibir productos, preparar y enviar pedido
- **Intermedio** (+1.5h): Inventario fisico, ajustes de stock, multiples almacenes, reglas de reabastecimiento, lotes/series
- **Avanzado** (+1h): Rutas avanzadas, cross-docking, dropshipping, informes de valoracion, ubicaciones por packing

### Ejercicios tipo
1. "Consulta el stock disponible de {PRODUCTO} en todos los almacenes"
2. "Realiza una transferencia interna de 50 unidades de {PRODUCTO} del almacen A al almacen B"
3. "Haz un inventario fisico de la ubicacion {UBICACION} y ajusta las diferencias"
4. "Prepara y valida el envio del pedido {PEDIDO} al cliente"

---

## Contabilidad (account)

### Conceptos clave
- Plan contable, diarios y cuentas
- Facturacion: facturas de cliente y de proveedor
- Conciliacion bancaria
- Impuestos: IVA, retenciones, IRPF
- Cierre mensual y anual

### Temario por nivel
- **Basico** (2h): Crear factura manual, registrar pago, conciliacion basica, consultar estado de cuentas
- **Intermedio** (+2h): Conciliacion bancaria avanzada, importacion de extractos, asientos de ajuste, informes financieros (P&L, balance)
- **Avanzado** (+2h): Cierre mensual/anual, activos fijos, presupuestos contables, analitica, informes fiscales (SII, modelo 303)

### Ejercicios tipo
1. "Crea una factura manual para el cliente {CLIENTE} con 2 lineas y registra el pago"
2. "Importa el extracto bancario de {BANCO} y concilia los movimientos con las facturas pendientes"
3. "Genera el informe de Perdidas y Ganancias del mes pasado y exportalo a Excel"
4. "Crea un asiento de ajuste para reclasificar {IMPORTE}€ de la cuenta {CUENTA_A} a {CUENTA_B}"

---

## Fabricacion (mrp)

### Conceptos clave
- Lista de materiales (BOM) y variantes
- Ordenes de fabricacion (MO) y ordenes de trabajo
- Centros de trabajo y capacidad
- Planificacion de produccion
- Subcontratacion

### Temario por nivel
- **Basico** (1.5h): Consultar BOM, crear orden de fabricacion, producir y consumir materiales, marcar como hecho
- **Intermedio** (+1.5h): Centros de trabajo, ordenes de trabajo, planificacion MPS, subcontratacion, variantes en BOM
- **Avanzado** (+1h): PLM (gestion de cambios de ingenieria), costes de produccion, informes de rendimiento, OEE

### Ejercicios tipo
1. "Crea una orden de fabricacion de 10 unidades de {PRODUCTO} usando la BOM existente"
2. "Procesa las ordenes de trabajo paso a paso en el centro de trabajo {CENTRO}"
3. "Consulta el coste real de la ultima orden de fabricacion y comparalo con el coste estandar"

---

## Proyecto (project)

### Conceptos clave
- Proyectos, tareas y subtareas
- Etapas Kanban personalizables
- Partes de horas (timesheets)
- Checklists y progreso

### Temario por nivel
- **Basico** (1h): Crear tarea, mover entre etapas, registrar tiempo, marcar como hecha
- **Intermedio** (+1h): Subtareas, etiquetas, filtros avanzados, informes de tiempo, hitos
- **Avanzado** (+1h): Planificacion de recursos, burndown, integracion con Ventas (proyectos facturables)

### Ejercicios tipo
1. "Crea una tarea en el proyecto {PROYECTO}, asignala a ti mismo y registra 2h de trabajo"
2. "Consulta las tareas vencidas del proyecto y reprogramalas"

---

## RRHH / Empleados (hr)

### Conceptos clave
- Ficha de empleado y departamentos
- Contratos y puestos de trabajo
- Ausencias y tipos de ausencia
- Portal del empleado

### Temario por nivel
- **Basico** (1.5h): Ficha de empleado, organigrama, solicitar ausencia, consultar saldo de vacaciones
- **Intermedio** (+1.5h): Contratos, evaluaciones, gastos, portal del empleado, informes de RRHH
- **Avanzado** (+1.5h): Nomina (sesion dedicada), configuracion de tipos de ausencia, acumuladores, integracion contable

### Ejercicios tipo
1. "Da de alta un nuevo empleado en el departamento {DEPARTAMENTO} con su contrato"
2. "Solicita 3 dias de vacaciones y aprueba la solicitud de tu compañero"
3. "Registra un gasto de {IMPORTE}€ con el justificante y envialo a aprobacion"

---

## Punto de Venta (pos)

### Conceptos clave
- Sesion de TPV: apertura, operacion, cierre
- Productos y categorias TPV
- Metodos de pago
- Devoluciones y notas de credito

### Temario por nivel
- **Basico** (1.5h): Abrir sesion, cobrar venta, multiples metodos de pago, cerrar sesion y cuadrar
- **Intermedio** (+1h): Devoluciones, descuentos, clientes en TPV, informes de ventas, productos con variantes
- **Avanzado** (+1h): Multi-tienda, restaurante (mesas/comandas), integracion contable, programas de fidelizacion

### Ejercicios tipo
1. "Abre la sesion del TPV, realiza 3 ventas con diferentes metodos de pago y cierra la sesion"
2. "Procesa una devolucion del ticket {TICKET} y emite la nota de credito"

---

## Helpdesk (helpdesk)

### Conceptos clave
- Tickets, equipos y asignacion
- SLAs y prioridades
- Base de conocimiento
- Portal del cliente

### Temario por nivel
- **Basico** (1.5h): Crear ticket, asignar, cambiar etapa, resolver, comunicar al cliente
- **Intermedio** (+1h): SLAs, escalado, base de conocimiento, informes de rendimiento, portal del cliente
- **Avanzado** (+1h): Automatizaciones, integracion con email/web, encuestas de satisfaccion

### Ejercicios tipo
1. "Crea un ticket para el cliente {CLIENTE} con prioridad alta, asignalo a tu equipo y responde con una solucion"
2. "Consulta los tickets que superan su SLA y escalados pendientes"

---

## Website / eCommerce (website, website_sale)

### Conceptos clave
- Editor web: bloques, paginas, menus
- Catalogo de productos online
- Carrito de compra y checkout
- Integracion con Inventario y Contabilidad

### Temario por nivel
- **Basico** (1.5h): Editar pagina, añadir bloque, publicar producto, hacer un pedido de prueba
- **Intermedio** (+1.5h): Categorias, atributos y filtros, promociones, metodos de pago, metodos de envio
- **Avanzado** (+1h): SEO basico, formularios, blog, integracion con CRM (leads desde web)

### Ejercicios tipo
1. "Publica 3 productos en la tienda online con fotos, precio y categoria"
2. "Realiza un pedido completo como cliente desde la web y verifica que llega a Ventas"

---

## Notas para el Skill

- Este temario es el **punto de partida**. Siempre adaptar segun:
  - Campos Studio/custom configurados en el proyecto
  - Modulos Indaws Core instalados (añaden funcionalidad)
  - GAPs detectados en el analisis funcional
  - Integraciones con sistemas externos
- Si la formacion es de un **modulo custom**, no usar este temario. Leer la documentacion del modulo y generar temario especifico.
- Si la formacion es de una **funcionalidad concreta** (ej: "conciliacion bancaria"), extraer solo ese bloque del temario de la app correspondiente.
- Apps no listadas aqui (ej: Suscripciones, Alquiler, Field Service): generar temario basandose en el conocimiento general de Odoo y la informacion del analisis funcional.
