# Cuestionario: TPV (Terminal Punto de Venta)

## 1. Configuracion General del Punto de Venta

### Estructura de Tiendas/Puntos de Venta

1. **Cuantas tiendas fisicas o puntos de venta distintos utilizaran el TPV de Odoo?**

2. **Cuantos terminales TPV (cajas registradoras/dispositivos) habra por cada tienda/punto de venta?**

3. **Desde que Almacen/Ubicacion especifica debe descontarse el stock vendido a traves de cada TPV/tienda? (Necesario para la configuracion del PoS)**

## 2. Interfaz de Usuario y Flujo de Trabajo en el TPV

### Diseno y Usabilidad

4. **Como desean organizar los productos en la pantalla principal del TPV? (Por categorias, busqueda rapida, botones de acceso directo para productos populares). Describan la disposicion ideal:**

5. **Describan el flujo de trabajo tipico de una transaccion en el TPV, desde que llega el cliente hasta que se completa el pago y la entrega del ticket:**

6. **Existen flujos alternativos importantes (ej. devoluciones, pedidos para llevar vs. comer en local)?**

### Usuarios y Roles en TPV

7. **Cuantos cajeros/empleados utilizaran el TPV en cada tienda (simultaneamente o por turnos)?**

8. **Necesitan definir roles distintos dentro del TPV? (ej. Cajero estandar, Encargado de tienda)**

9. **Que acciones especificas requieren permisos elevados o la aprobacion de un encargado? (ej. aplicar descuentos superiores a X%, procesar devoluciones, abrir el cajon portamonedas manualmente, cancelar pedidos pagados)**

## 3. Productos y Precios en el TPV

### Disponibilidad y Visualizacion

10. **Que productos o categorias de productos deben estar disponibles para la venta en el TPV? (Todos los productos del sistema o solo un subconjunto?)**

11. **Necesitan visualizar informacion especifica del producto directamente en la pantalla del TPV? (ej. stock disponible en esta tienda, descripcion ampliada, imagen, ingredientes/alergenos)**

12. **Si usan variantes de producto (talla, color, sabor), como debe ser el proceso de seleccion en el TPV?**

### Precios y Descuentos

13. **Utilizaran listas de precios especificas para el TPV que difieran de las listas de precios generales (ej. precios de tienda vs. precios online)?**

14. **Como se gestionaran los descuentos en el TPV? (Descuentos manuales por linea de pedido, descuentos manuales globales sobre el total, promociones predefinidas - ej. 2x1, descuento por cantidad, menus/combos)**

15. **Existen limites para los descuentos manuales? Requieren aprobacion?**

16. **Necesitan gestionar "combos" o menus (un conjunto de productos a un precio fijo) directamente desde el TPV?**

## 4. Proceso de Pedido y Pago

### Metodos de Pago

17. **Que metodos de pago deben aceptar en el TPV? (Efectivo, Tarjeta de Credito/Debito - que tipos?, Pagos moviles - Bizum, Apple Pay, Google Pay, Vales/Cupones, Tarjetas Regalo, Credito en tienda, Pago a cuenta/Factura posterior para clientes registrados)**

18. **Necesitan poder dividir un pago entre varios metodos? (ej. parte en efectivo, parte en tarjeta)**

### Gestion de Pedidos

19. **Como se gestionan las devoluciones o cambios en el TPV? (Requieren el ticket original? Se devuelve efectivo, se genera credito en tienda, se cambia por otro producto? Requiere aprobacion?)**

20. **Necesitan poder generar una factura completa (con datos fiscales del cliente) directamente desde el TPV, ademas del ticket simplificado?**

21. **Gestionan ventas "a cuenta" para clientes especificos, donde el pago se realiza posteriormente?**

## 5. Gestion de Clientes (Desde el TPV)

### Identificacion y Creacion

22. **Necesitan poder buscar e identificar clientes existentes en el TPV? Mediante que datos (nombre, telefono, email, tarjeta de fidelizacion)?**

23. **Necesitan poder crear nuevos clientes directamente desde la interfaz del TPV? Que campos serian obligatorios/deseables?**

24. **Necesitan visualizar informacion relevante del cliente en el TPV? (ej. historial de compras recientes, notas especificas, saldo de puntos de fidelidad, credito disponible)**

### Fidelizacion y Credito

25. **Tienen o desean implementar un programa de fidelizacion? Describan su funcionamiento (acumulacion de puntos por importe/producto, niveles de cliente, canje de puntos por descuentos/productos/vales):**

26. **Necesitan gestionar "credito en tienda" para clientes (ej. generado por devoluciones)? Como se utiliza para pagar?**

## 6. Integracion de Hardware

### Dispositivos Requeridos

Por cada tipo de terminal TPV, especifiquen que hardware se conectara:

27. **Impresora de Tickets:** Modelo, tipo de conexion (USB, Red/Ethernet, Wifi, Bluetooth). Protocolo ESC/POS estandar?

28. **Lector de Codigos de Barras:** Modelo, tipo de conexion (USB, Bluetooth).

29. **Cajon Portamonedas:** Se conecta a traves de la impresora o directamente (USB)?

30. **Terminal de Pago (Datafono):** Marca y modelo (ej. Ingenico, Verifone). Proveedor del servicio (ej. Redsys, Adyen, Stripe). Tipo de integracion requerida (Necesita conexion a traves de la IoT Box de Odoo? Integracion directa certificada en Espana?).

31. **Visor/Display para Cliente:** Necesario? Que informacion debe mostrar (productos escaneados, total, publicidad)?

32. **Balanza:** Necesaria para productos vendidos por peso? Marca y modelo. Requiere integracion para enviar el peso automaticamente al TPV?

33. **Conectividad:** Se utilizara la IoT Box de Odoo para gestionar las conexiones de hardware o se requieren otras configuraciones/drivers?

## 7. Gestion de Sesiones y Control de Efectivo

### Apertura y Cierre

34. **Describan el procedimiento para iniciar una sesion de TPV/turno de cajero. Se registra un fondo de caja inicial (cambio)?**

35. **Describan el procedimiento para cerrar una sesion. Como se realiza el arqueo de caja (conteo de efectivo y otros metodos de pago)? Como se declaran los importes?**

### Movimientos de Caja

36. **Necesitan registrar entradas y salidas de efectivo durante la sesion que no sean ventas? (ej. Retirada de exceso de efectivo para llevar al banco, Ingreso de cambio). Se registran los motivos? Requieren aprobacion?**

37. **Como se gestionan y justifican las diferencias encontradas en el arqueo de caja?**

## 8. Capacidad Offline

38. **La conexion a internet en las tiendas es estable y fiable?**

## 10. Informes (Especificos del TPV)

39. **Que informes necesitan generar relacionados directamente con las sesiones del TPV? (ej. Resumen de ventas por sesion, Ventas por cajero, Desglose de pagos por metodo, Informe de control de efectivo, Productos mas vendidos en la sesion). (Evitar informes consolidados de toda la empresa)**

## 11. Migracion de Datos (Especifico TPV)

40. **Necesitan migrar saldos activos de puntos de fidelidad de clientes existentes? En que formato disponen de ellos?**

41. **Necesitan migrar saldos activos de tarjetas regalo existentes? En que formato?**

## 12. Personalizaciones del TPV

42. **Existe algun flujo de trabajo, boton, funcionalidad o regla de negocio muy especifica para su operativa en TPV que crean que no esta cubierta por el estandar de Odoo y requeriria personalizacion? Describanla:**

## 13. Roles de Usuario y Formacion (Usuarios TPV)

43. **Definan los perfiles de usuario que operaran el TPV (ej. Cajero, Encargado de Tienda, Camarero). Que permisos especificos debe tener cada uno dentro de la interfaz del TPV?**

44. **Cuantos empleados necesitaran formacion para usar el TPV de Odoo?**

45. **Cual es su nivel general de familiaridad con sistemas TPV tactiles o basados en tablet/PC?**
