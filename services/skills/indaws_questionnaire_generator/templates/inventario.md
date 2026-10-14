# Cuestionario: Inventario

## 1. Estructura de Almacenes y Ubicaciones

### Almacenes Fisicos

1. **Cuantos almacenes o centros de distribucion fisicos gestionan? Nombrelos y describan brevemente su proposito principal (ej: Almacen Central, Tienda X, Almacen de Devoluciones):**

### Estructura de Ubicaciones Internas

2. **Describan la estructura fisica de almacenamiento dentro de cada almacen (ej: Zonas -> Pasillos -> Estanterias -> Niveles/Baldes). Necesitan replicar esta jerarquia en Odoo?**

3. **Utilizan ubicaciones especificas para procesos concretos? Detallen cuales necesitan (ej: Recepcion, Control de Calidad, Embalaje/Picking, Expedicion, Produccion, Scrap/Desecho, Ajustes, Cross-Docking, Devoluciones):**

4. **Como identifican actualmente las ubicaciones (codigos, nombres)? Se mantendra el sistema actual?**

## 2. Configuracion de Productos (Enfoque Inventario)

### Gestion de Stock

5. **Necesitan definir Unidades de Medida (UdM) especificas para el almacenamiento que difieran de la UdM de compra o venta? (ej: comprar en Pallets, almacenar en Cajas, vender en Unidades). Como se gestionan las conversiones?**

6. **Utilizan definiciones de Empaquetado/Packaging? (ej: Caja de 6 Uds, Paquete de 12 Uds, Pallet de 100 Uds). Como se usan estos empaquetados en los movimientos internos, picking o almacenamiento?**

7. **Necesitan aplicar estrategias de ubicacion especificas al recibir productos (Putaway Rules)? (ej: Productos tipo A a Zona Rapida, Productos pesados a Nivel 0). Describan las reglas:**

8. **Necesitan aplicar estrategias de extraccion/picking especificas (Removal Strategies)? (FIFO, LIFO, FEFO - First Expiry First Out, Ubicacion mas cercana). Varian por producto o zona?**

## 3. Operaciones y Movimientos de Inventario

### Tipos de Operaciones

9. **Que tipos de movimientos de inventario internos necesitan gestionar de forma diferenciada? (ej: Transferencias entre almacenes, Transferencias internas dentro del mismo almacen, Ajustes de inventario positivos/negativos, Movimientos a Scrap, Entradas/Salidas de produccion, Movimientos para Kitting/Ensamblaje):**

10. **Necesitan definir Tipos de Operacion personalizados en Odoo con secuencias o reglas especificas?**

### Transferencias Internas

11. **Describan el proceso para mover stock entre diferentes ubicaciones o almacenes:**

12. **Requieren pasos intermedios (ej: un estado "en transito")?**

### Ajustes de Inventario

13. **Como gestionan las diferencias encontradas entre el stock fisico y el del sistema (fuera de los conteos planificados)?**

14. **Cual es el proceso para registrar estos ajustes? Se requiere aprobacion?**

### Gestion de Scrap (Desechos/Mermas)

15. **Cual es el proceso para dar de baja productos del inventario (scrap)?**

## 4. Trazabilidad (Lotes y Numeros de Serie)

### Seguimiento

16. **Que productos requieren seguimiento por Lote o Numero de Serie unico?**

17. **En que punto del proceso se captura o genera el Lote/Numero de Serie? (Recepcion, final de produccion, movimiento interno especifico)**

18. **Necesitan registrar informacion adicional asociada a los lotes? (Fecha de caducidad, Fecha de consumo preferente, Lote del proveedor, Certificado de calidad, Pais de origen)**

19. **Como utilizan la informacion de trazabilidad actualmente? (Consultas, informes, retiradas de producto)**

20. **Necesitan realizar seguimiento de trazabilidad ascendente y descendente?**

### Gestion de Caducidad

21. **Gestionan fechas de caducidad o consumo preferente?**

22. **Necesitan alertas o informes sobre stock proximo a caducar?**

23. **Aplican la estrategia de picking FEFO (First Expiry First Out)?**

## 5. Valoracion de Inventario

24. **Que metodo de valoracion de inventario utilizan o requieren? (Precio Estandar, Precio Medio Ponderado - AVCO, FIFO)**

25. **Utilizan o necesitaran la funcionalidad de "Landed Costs" (costes indirectos anadidos al valor del stock)? Como se aplicarian desde la perspectiva puramente de inventario/coste?**

## 6. Reaprovisionamiento y Reglas de Stock

26. **Como se determina la necesidad de reponer stock? (Reglas de Min./Max., Puntos de Pedido)**

27. **Describan las reglas que utilizan (ej: Minimo X unidades, Maximo Y unidades por ubicacion Z):**

28. **Estas reglas deben generar automaticamente borradores de ordenes (de compra, fabricacion) o solo sugerencias/alertas?**

## 7. Inventario Fisico y Conteos Ciclicos

### Proceso de Conteo

29. **Realizan inventarios fisicos completos? Con que frecuencia?**

30. **Realizan conteos ciclicos (por zona, por producto, por valor)? Describan la metodologia y frecuencia:**

31. **Como es el proceso actual de conteo? (Listados en papel, dispositivos moviles, scanners)**

## 8. Control de Calidad (En Procesos de Inventario)

### Puntos de Control

32. **Necesitan realizar controles de calidad durante operaciones de inventario? (ej: al recibir mercancia - en la ubicacion de QC, antes de una transferencia interna, sobre el stock almacenado)**

33. **Describan los tipos de controles (Medicion, Pass/Fail, Toma de muestra):**

34. **Que acciones se desencadenan segun el resultado del control? (Mover a ubicacion de cuarentena, bloquear lote, generar alerta, mover a scrap)**

## 9. Informes y Analisis de Inventario

35. **Que informes de inventario son esenciales para su operativa diaria y toma de decisiones? (ej: Niveles de stock actuales por ubicacion/producto, Stock disponible vs. previsto, Valoracion de inventario, Rotacion de stock, Stock obsoleto o de baja rotacion, Informe de caducidades, Historial de movimientos de un producto/lote, Precision del inventario - resultados de conteos)**

## 10. Uso de Codigos de Barras y Hardware

36. **Utilizan o planean utilizar lectores de codigos de barras para las operaciones de inventario?**

37. **Para que operaciones son imprescindibles? (Recepciones, Ubicacion (Putaway), Picking, Embalaje, Transferencias, Conteos, Salidas a produccion)**

38. **Que tipo de terminales/scanners utilizan o planean utilizar? (Terminales de mano con Android/Windows CE, scanners USB conectados a PC). Hay requisitos de compatibilidad?**

## 11. Migracion de Datos (Especifico Inventario)

39. **Necesitan migrar los saldos iniciales de stock ("Stock on Hand")?**

40. **Disponen de esta informacion en formato digital (CSV/Excel)?**

41. **Que detalle necesitan migrar? (Producto, Ubicacion, Cantidad disponible, Lote/Numero de Serie asociado, Fecha de caducidad si aplica, Coste unitario si no se recalculara)**

42. **Necesitan migrar la estructura de Almacenes y Ubicaciones si es compleja?**

## 12. Personalizaciones e Integraciones (Enfoque Inventario)

43. **Identifican algun proceso de gestion de inventario muy especifico de su negocio que crean que requerira un desarrollo a medida en Odoo? Describanlo:**

44. **Necesitan integrar Odoo Inventario con sistemas externos?**

## 13. Roles de Usuario y Formacion (Enfoque Inventario)

45. **Que roles especificos interactuaran principalmente con la aplicacion de Inventario? (ej: Mozo de almacen, Carretillero, Preparador de pedidos (picker/packer), Responsable de Calidad, Responsable de Inventario, Jefe de Almacen)**

46. **Cuantos usuarios trabajaran principalmente con las funciones de Inventario?**
