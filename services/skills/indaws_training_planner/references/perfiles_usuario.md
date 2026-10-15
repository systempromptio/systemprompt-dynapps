# Perfiles de Usuario en Implantaciones Odoo

Este documento define los perfiles tipicos de usuario que se encuentran en proyectos de implantacion Odoo. El skill lo usa para adaptar el temario, nivel de detalle y formato de la formacion.

---

## Taxonomia de Perfiles

### Operario de Almacen

**Nivel tecnico**: Bajo
**Apps tipicas**: Inventario, Codigo de barras, Fabricacion (parte de planta)
**Formato sesion recomendado**: Presencial en el puesto de trabajo, practica guiada
**Duracion tipica**: 1.5-2h por sesion
**Caracteristicas**:
- Usa tablet o terminal, no PC de escritorio
- Necesita flujos muy concretos: recepcion, picking, empaquetado, transferencias
- No necesita entender configuracion ni informes
- Formacion en grupos pequeños (3-5 personas)
- Repeticion y practica son clave
**Que incluir**: Solo las pantallas que van a usar a diario. Flujo lineal paso a paso. Que hacer si algo falla (contactar a su responsable).
**Que NO incluir**: Configuracion, informes, campos tecnicos, explicaciones de modelos de datos.

### Comercial / Equipo de Ventas

**Nivel tecnico**: Medio
**Apps tipicas**: CRM, Ventas, Contactos
**Formato sesion recomendado**: Mixto (demo + ejercicios con su pipeline real)
**Duracion tipica**: 2-3h
**Caracteristicas**:
- Motivacion si ven que el sistema les ahorra tiempo
- Necesitan ver su pipeline, sus oportunidades, sus clientes
- Interesan: presupuestos rapidos, seguimiento de oportunidades, email desde Odoo
- Preguntas frecuentes: "¿Puedo ver solo mis ventas?", "¿Como duplico un presupuesto?"
**Que incluir**: Flujo de venta completo, atajos para crear presupuestos rapidos, filtros y favoritos, plantillas de email.
**Que NO incluir**: Configuracion de listas de precios complejas, contabilidad, configuracion de impuestos.

### Responsable de Compras

**Nivel tecnico**: Medio
**Apps tipicas**: Compras, Inventario (recepciones), Contactos (proveedores)
**Formato sesion recomendado**: Presencial, flujo completo de compra
**Duracion tipica**: 2-3h
**Caracteristicas**:
- Necesita entender flujo completo: solicitud → pedido → recepcion → factura
- Aprobaciones y presupuestos son criticos
- Comparacion de proveedores, acuerdos de compra
- Suele gestionar pocos usuarios pero mucho volumen
**Que incluir**: Flujo de compra end-to-end, aprobaciones, acuerdos de compra, recepciones, control de facturas.
**Que NO incluir**: Configuracion de rutas de almacen complejas, fabricacion.

### Contable / Equipo Financiero

**Nivel tecnico**: Alto
**Apps tipicas**: Contabilidad, Facturacion, Gastos, Activos
**Formato sesion recomendado**: Presencial intensivo con ejercicios de conciliacion
**Duracion tipica**: 3-4h (puede requerir varias sesiones)
**Caracteristicas**:
- Perfil mas exigente: necesita precision y confianza en el sistema
- Preguntas detalladas sobre plan contable, impuestos, retenciones
- Necesitan flujo de cierre mensual/anual completo
- Conciliacion bancaria es siempre una sesion dedicada
- Muy sensibles a errores — necesitan saber como corregir
**Que incluir**: Plan contable, diarios, impuestos, conciliacion bancaria, facturas manuales, asientos de ajuste, informes financieros, cierre mensual/anual.
**Que NO incluir**: CRM, ventas operativas (solo la parte contable de las facturas).

### Responsable de Produccion / Fabricacion

**Nivel tecnico**: Medio
**Apps tipicas**: Fabricacion, PLM, Inventario, Calidad, Mantenimiento
**Formato sesion recomendado**: Presencial en planta + sala, con datos reales de BOMs
**Duracion tipica**: 2-3h
**Caracteristicas**:
- Necesita ver SUS productos y listas de materiales
- Planificacion de produccion, ordenes de fabricacion, centros de trabajo
- Suele ser un perfil muy practico: "muestrame como se hace"
- Interesa: ordenes de fabricacion, consumo de materiales, productividad
**Que incluir**: BOMs, ordenes de fabricacion, centros de trabajo, planificacion, subcontratacion (si aplica), control de calidad.
**Que NO incluir**: Contabilidad de costes (a menos que sea responsable tambien), configuracion de rutas complejas.

### Responsable de RRHH / Personas

**Nivel tecnico**: Medio
**Apps tipicas**: Empleados, Ausencias, Nomina, Gastos, Evaluaciones
**Formato sesion recomendado**: Presencial con ejercicios de gestion de empleados
**Duracion tipica**: 2-3h
**Caracteristicas**:
- Datos sensibles: enfasis en permisos y privacidad
- Necesita flujos de alta/baja de empleados, gestion de ausencias
- Portal del empleado: como se autogestiona el empleado
- Nomina: sesion separada y dedicada si aplica
**Que incluir**: Ficha de empleado, departamentos, ausencias, gastos, portal del empleado, informes de RRHH.
**Que NO incluir**: Contabilidad de nominas (solo la parte de RRHH), configuracion tecnica.

### Direccion / Gerencia

**Nivel tecnico**: Bajo (en Odoo)
**Apps tipicas**: Tableros, Aprobaciones, cualquier app (vista ejecutiva)
**Formato sesion recomendado**: Remoto o presencial corta, demo ejecutiva
**Duracion tipica**: 1-1.5h
**Caracteristicas**:
- No van a operar el dia a dia — necesitan vision ejecutiva
- Dashboards, KPIs, aprobaciones, informes de alto nivel
- Tiempo limitado: sesion corta y al grano
- Preguntas tipicas: "¿Como veo las ventas del mes?", "¿Donde apruebo?"
**Que incluir**: Dashboards, informes clave (ventas, compras, finanzas), flujo de aprobaciones, donde encontrar la informacion que les importa.
**Que NO incluir**: Operativa diaria, configuracion, flujos detallados.

### Administrador del Sistema / IT

**Nivel tecnico**: Alto
**Apps tipicas**: Ajustes, Studio, Tecnico, todas las apps (configuracion)
**Formato sesion recomendado**: Taller tecnico presencial
**Duracion tipica**: 3-4h (puede requerir varias sesiones)
**Caracteristicas**:
- Sera quien mantenga el sistema post-implantacion
- Necesita entender: usuarios, grupos de acceso, permisos, Studio, automatizaciones
- Importaciones/exportaciones, acciones programadas, parametros del sistema
- Este perfil suele tener formacion adicional de "train the trainer"
**Que incluir**: Gestion de usuarios y permisos, Studio (campos, vistas, automatizaciones), parametros del sistema, copias de seguridad, actualizaciones, troubleshooting basico.
**Que NO incluir**: Desarrollo Python, administracion de servidor (eso es IT/devops).

### Atencion al Cliente / Helpdesk

**Nivel tecnico**: Medio
**Apps tipicas**: Helpdesk, CRM, Contactos, Conocimiento
**Formato sesion recomendado**: Presencial con ejercicios de resolucion de tickets
**Duracion tipica**: 2h
**Caracteristicas**:
- Necesita agilidad: crear tickets, asignar, resolver, escalar
- SLAs y prioridades son importantes
- Base de conocimiento para respuestas estandar
- Portal del cliente: como el cliente crea y sigue tickets
**Que incluir**: Gestion de tickets, SLAs, asignacion, equipos, base de conocimiento, portal del cliente.
**Que NO incluir**: Configuracion de reglas de asignacion complejas, integraciones de email avanzadas.

---

## Combinaciones Frecuentes

En implantaciones reales, los usuarios suelen tener roles combinados:

| Combinacion | Enfoque Formacion |
|-------------|-------------------|
| Comercial + Facturacion | Ventas: flujo completo hasta factura. Contabilidad: solo facturacion |
| Compras + Almacen | Compras: flujo completo. Inventario: recepciones y stock |
| RRHH + Nomina | Sesion 1: empleados y ausencias. Sesion 2: nomina (dedicada) |
| Produccion + Calidad | Fabricacion con controles de calidad integrados |
| Admin + Train the Trainer | Sesion tecnica + metodologia para que forme a sus equipos |

---

## Reglas para Adaptar el Temario

1. **Menos es mas**: Un operario no necesita ver 30 menus. Solo los 3-5 que usara a diario.
2. **Datos reales**: SIEMPRE usar datos del entorno del cliente. Si ven "Mueble Modelo Valencia" en vez de "Producto 1", la adopcion es inmediata.
3. **Contexto de negocio**: Cada ejercicio debe empezar con un escenario real: "El cliente Ferreteria Lopez pide 200 tornillos M8..."
4. **Nivel tecnico**: Si el perfil es bajo, no uses palabras como "many2one", "computed field" o "RPC". Usa "campo relacionado", "se calcula automaticamente", "boton".
5. **Duracion**: Nunca mas de 4h seguidas. Si el temario es amplio, divide en sesiones.
