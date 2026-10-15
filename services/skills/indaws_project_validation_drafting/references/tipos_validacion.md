# Tipos de Validacion

## Proposito

Esta referencia define los 4 tipos de validacion del cliente, como identificarlos automaticamente a partir del analisis y las tareas, y que criterios de aceptacion son tipicos para cada tipo y app.

---

## Tipo 1: APP (Validacion de app configurada)

### Identificacion

- Se genera **1 validacion por cada articulo hijo** del articulo "Analisis" en Knowledge
- Cada articulo hijo corresponde a una app/modulo del proyecto (Ventas, Inventario, etc.)
- Buscar la tarea de configuracion correspondiente en `project.task` por nombre similar

### Vinculacion con tarea

Buscar tarea cuyo nombre contenga el nombre de la app:
- `[CONFIG] Ventas` o `Configuracion Ventas` → app "Ventas"
- Si no hay match exacto, buscar por palabras clave
- Si no hay tarea, crear la validacion sin `x_task_id` y avisar al consultor

### Calculo de deadline

- Fecha fin de la tarea de configuracion de esa app (`date_deadline` o fin de `planned_date_begin` + horas)
- Si no hay fecha → usar la fecha del hito de la fase de implementacion
- Añadir +3 dias laborables tras la fecha fin de la tarea (el cliente necesita tiempo para revisar)

### Criterios de aceptacion tipicos por app

#### Ventas
- Los presupuestos se generan con la informacion correcta (tarifas, descuentos, impuestos)
- Los pedidos de venta siguen el flujo acordado (confirmacion, facturacion)
- Las plantillas de documentos (presupuesto, pedido) tienen el formato aprobado
- Los equipos comerciales y territorios estan correctamente asignados

#### Inventario
- Los almacenes y ubicaciones reflejan la estructura real
- Las recepciones y expediciones siguen el flujo operativo acordado
- Los productos aparecen con stock, lotes/numeros de serie correctos (si aplica)
- Las reglas de reaprovisionamiento funcionan segun lo definido

#### Contabilidad
- El plan contable refleja la estructura fiscal de la empresa
- Las facturas se generan con los impuestos correctos (IVA, retenciones)
- La numeracion de facturas sigue las secuencias acordadas
- Los asientos contables se generan automaticamente segun lo esperado
- La conexion con obligaciones fiscales (SII, VeriFactu) funciona (si aplica)

#### Compras
- Los pedidos de compra se crean con los proveedores y condiciones correctas
- El flujo de aprobacion funciona segun lo definido
- Las recepciones de mercancia actualizan stock correctamente

#### CRM
- El pipeline comercial tiene las etapas y automatismos acordados
- Los leads/oportunidades se asignan correctamente al equipo
- Los KPIs y tableros reflejan la informacion acordada

#### RRHH
- Los empleados aparecen con sus datos y departamentos correctos
- Las ausencias y vacaciones siguen la politica definida
- Los partes de horas / fichaje funcionan segun lo acordado

#### Fabricacion
- Las listas de materiales reflejan los productos y componentes reales
- Las ordenes de produccion siguen el flujo operativo definido
- Los centros de trabajo y rutas estan configurados correctamente

#### Proyecto / Servicios
- Los tipos de proyecto estan configurados segun lo definido
- La facturacion de servicios (por horas/fijo) funciona correctamente
- Los empleados pueden registrar horas en las tareas

#### eCommerce
- El catalogo online muestra los productos con imagenes y precios correctos
- El proceso de compra funciona de principio a fin (carrito → pago → confirmacion)
- Las pasarelas de pago estan operativas

---

## Tipo 2: MIGRACION (Validacion de datos migrados)

### Identificacion

- Buscar tareas con `[MIGR]` en el nombre o tags de migracion
- Tambien buscar en el analisis menciones a "migrar", "importar datos", "datos historicos"
- Se genera **1 unica validacion** de migracion (agrupa todas las entidades migradas)

### Vinculacion con tarea

- Vincular a la tarea principal de migracion (la de mayor peso/horas)
- Si hay varias tareas de migracion, vincular a la primera cronologicamente

### Calculo de deadline

- Fecha fin de la ultima tarea de migracion + 5 dias laborables (revision de datos)
- La migracion suele ser una de las ultimas tareas antes de pre-produccion

### Criterios de aceptacion tipicos

- Todos los registros migrados aparecen en Odoo (verificar cantidades: X productos, Y clientes, Z facturas)
- Los datos de cada registro son correctos (nombres, referencias, precios, direcciones)
- Las relaciones entre registros se mantienen (producto → categoria, factura → cliente)
- Los saldos contables coinciden con el sistema anterior (si aplica)
- Los historicos de movimientos son consultables
- No hay registros duplicados ni datos corruptos

---

## Tipo 3: DESARROLLO (Validacion de desarrollo complejo o integracion)

### Identificacion

- Buscar en los articulos de analisis la **Seccion 7 (Tareas Detalladas)** con items de peso >= 4 y etiqueta [DEV]
- Buscar tareas con `[DEV]` o `[INT]` en el nombre
- Buscar menciones a integraciones con sistemas externos (pasarelas, ERPs, APIs)
- Solo generar validacion si es un desarrollo **complejo** (no para campos Studio simples)
- Se genera **1 validacion por cada desarrollo/integracion significativo**

### Criterios para considerar "significativo"

- Peso >= 4 en la Seccion 7 del analisis
- Integracion con sistema externo (API, webhooks, sincronizacion)
- Modulo custom completo (no solo un campo o vista)
- Workflow o automatismo complejo que cambia procesos del cliente

### Vinculacion con tarea

- Vincular a la tarea de desarrollo/integracion especifica

### Calculo de deadline

- Fecha fin de la tarea de desarrollo + 3 dias laborables

### Criterios de aceptacion tipicos

#### Para desarrollos custom
- La funcionalidad hace exactamente lo descrito en el analisis (referencia a la Seccion 7)
- Los usuarios pueden usar la funcionalidad sin intervencion tecnica
- Los datos se calculan/muestran correctamente en los escenarios tipicos
- La funcionalidad se integra bien con el resto de la implantacion

#### Para integraciones
- Los datos se sincronizan correctamente entre Odoo y el sistema externo
- La frecuencia de sincronizacion es la acordada (tiempo real, cada X horas, diaria)
- Los errores de sincronizacion se notifican de forma clara
- El volumen de transacciones soportado es el esperado

---

## Tipo 4: PREPRODUCCION (UAT Final / Go-Live)

### Identificacion

- Se genera **siempre 1 unica validacion** de pre-produccion
- Es transversal a todo el proyecto, no se vincula a una tarea concreta

### Vinculacion con tarea

- `x_task_id`: false (no vinculada a tarea)
- Referencia general al proyecto

### Calculo de deadline

- Go-Live (`date` del proyecto o `xx_golive_date`) menos 5 dias laborables
- Si no hay fecha de Go-Live, pedir al consultor

### Criterios de aceptacion tipicos

- Todas las validaciones anteriores (apps, migracion, desarrollos) estan aprobadas
- Los usuarios clave han sido formados y pueden operar de forma autonoma
- Los flujos completos de negocio (end-to-end) funcionan correctamente:
  - Ejemplo: Presupuesto → Pedido → Albaran → Factura → Cobro
  - Ejemplo: Pedido de compra → Recepcion → Factura proveedor → Pago
- Los reports e informes generan datos correctos
- Los accesos y permisos de cada perfil de usuario son correctos
- El rendimiento es aceptable para el volumen real de datos
- Existe un plan de contingencia para los primeros dias

---

## Notas Generales

### Numero de validaciones por proyecto tipico

- Proyecto pequeno (2-3 apps, sin migracion): 3-5 validaciones
- Proyecto mediano (4-6 apps, con migracion): 6-10 validaciones
- Proyecto grande (7+ apps, migracion, desarrollos): 10-15 validaciones

### Orden de validacion recomendado

1. Apps configuradas (por orden de implementacion)
2. Desarrollos/integraciones
3. Migracion de datos
4. Pre-produccion (siempre la ultima)

### Regla de no-duplicacion

Antes de crear una validacion, buscar en `x_client_request` del proyecto:
```
domain: [('x_project_id', '=', project_id), ('x_type', '=', 'validation'), ('x_name', 'ilike', nombre_app)]
```
Si ya existe, preguntar al consultor si actualizar o crear nueva.
