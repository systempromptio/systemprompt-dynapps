## Principio Fundamental

Cuando se ejecutan tests en produccion, la creacion de registros (facturas, pedidos de venta,
albaranes, etc.) consume numeros de secuencia. Al eliminar los registros de test, los numeros
ya consumidos **no se recuperan automaticamente**, dejando huecos en la numeracion.

Este documento define el protocolo completo para capturar, verificar y restaurar secuencias
tras la ejecucion de tests.

---

