# Guia de Analisis de Peticiones por Tipo de Tarea

## Proposito

Esta guia ayuda al LLM a identificar que peticiones al cliente son tipicamente necesarias segun el tipo de tarea de implantacion. Se usa como **punto de partida** que luego se coteja con la descripcion real de la tarea, los checklists y el documento de analisis funcional.

**IMPORTANTE**: Esta guia proporciona patrones tipicos, NO una lista cerrada. Siempre prioriza el contenido real de la tarea y el analisis sobre estos patrones.

---

## Reglas de Cotejo con el Analisis Funcional

### Cuando DESCARTAR una peticion

La peticion se descarta si el analisis funcional ya contiene la informacion con suficiente detalle para configurar/implementar:

- El analisis especifica cantidades, nombres, listas concretas
  - Ejemplo: "3 almacenes: Central, Norte, Sur" → NO pedir listado de almacenes
- El analisis describe procesos con suficiente detalle para implementar
  - Ejemplo: "Facturan por pedido, 1 factura por pedido confirmado" → NO pedir decision de facturacion
- El analisis adjunta o referencia documentos ya proporcionados
  - Ejemplo: "Ver Excel adjunto con catalogo de productos" → NO pedir catalogo

### Cuando MANTENER una peticion

La peticion se mantiene (posiblemente ajustada) cuando:

- El analisis menciona el tema pero falta el detalle operativo
  - Ejemplo: "Tienen 3 almacenes" pero no dice nombres/direcciones → PEDIR nombres y direcciones
- El analisis confirma la necesidad pero no hay fichero
  - Ejemplo: "Migrar 2.000 productos desde Sage" → PEDIR fichero de export
- El analisis no menciona el tema en absoluto
  - Ejemplo: No habla de reglas de descuento → PREGUNTAR si las tienen
- El analisis indica una decision pendiente
  - Ejemplo: "Pendiente definir si multi-empresa" → PEDIR decision

### Cuando AJUSTAR una peticion

Si el analisis tiene informacion parcial, ajusta la peticion para pedir solo lo que falta:

- Analisis dice "3 tarifas" → No pedir "listado de tarifas", pedir "detalle de precios por tarifa"
- Analisis dice "usan lotes" → No preguntar "usan lotes?", pedir "formato de codificacion de lotes"

---

## Patrones por Tipo de Tarea

### Ventas

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Catalogo de productos con precios | data_file | blocker | Si el analisis lista productos, pedir solo el fichero |
| Listas de precios / tarifas | data_file | blocker | Si el analisis describe las tarifas, descartar |
| Proceso de aprobacion de pedidos | decision | normal | Si el analisis lo describe, descartar |
| Plantilla de presupuesto actual | data_file | normal | Para replicar formato en Odoo |
| Reglas de descuento | information | normal | Frecuentemente no esta en el analisis |
| Equipos comerciales y territorios | information | normal | Si el analisis describe la estructura, descartar |
| Condiciones de pago habituales | information | normal | A veces en el analisis de contabilidad |

### Inventario

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Listado de almacenes y ubicaciones | information | blocker | Si el analisis los lista con detalle, descartar |
| Flujo de recepcion de mercancia | information | normal | Si el analisis lo describe, descartar |
| Uso de lotes / numeros de serie | decision | blocker | Si el analisis lo confirma, descartar |
| Reglas de reaprovisionamiento | information | normal | Raramente en el analisis |
| Uso de escaner / pistolas barcode | decision | normal | A veces en el analisis |
| Categorias de producto para inventario | data_file | normal | Si el analisis las lista, descartar |

### Contabilidad

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Plan contable actual | data_file | blocker | Casi nunca en el analisis |
| Datos bancarios para remesas SEPA | information | normal | Nunca en el analisis |
| Gestoria o contabilidad interna | decision | blocker | Frecuentemente en el analisis |
| Obligacion SII / TicketBAI / VeriFactu | information | blocker | A veces en el analisis |
| Plantillas de factura actuales | data_file | normal | Para replicar formato |
| Regimen de IVA especial | information | normal | A veces en el analisis |

### Facturacion

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Metodo de facturacion (por pedido/envio) | decision | blocker | Frecuentemente en el analisis |
| Volumen mensual de facturas | information | normal | A veces en el analisis |
| Plantilla de factura actual (PDF/imagen) | data_file | normal | Para disenar report en Odoo |
| Secuencias de numeracion | information | normal | Raramente en el analisis |

### Compras

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Listado de proveedores principales | data_file | blocker | Raramente con detalle en el analisis |
| Flujo de aprobacion de compras | decision | normal | A veces en el analisis |
| Uso de dropshipping | decision | normal | Si el analisis lo menciona, descartar |
| Acuerdos marco con proveedores | information | normal | Raramente en el analisis |

### CRM

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Etapas del pipeline comercial | information | blocker | A veces en el analisis |
| Equipo comercial y asignacion | information | normal | A veces en el analisis |
| Datos de clientes/oportunidades a migrar | data_file | normal | Si el analisis confirma migracion |
| KPIs comerciales que quieren medir | information | normal | Raramente en el analisis |

### RRHH

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Listado de empleados | data_file | blocker | Casi nunca con fichero en el analisis |
| Organigrama / departamentos | information | normal | A veces en el analisis |
| Politica de vacaciones y ausencias | information | normal | Raramente en el analisis |
| Sistema de fichaje actual | information | normal | A veces en el analisis |

### Proyecto / Servicios

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Tipos de proyecto que gestionan | information | normal | Frecuentemente en el analisis |
| Modelo de facturacion (horas/fijo) | decision | blocker | Frecuentemente en el analisis |
| Plantilla de parte de horas actual | data_file | normal | Para replicar formato |

### eCommerce

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Catalogo de productos online con imagenes | data_file | blocker | Diferente del catalogo general |
| Pasarelas de pago deseadas | decision | blocker | A veces en el analisis |
| Dominio web actual | information | normal | A veces en el analisis |
| Diseno/mockup de la web actual | data_file | normal | Para replicar estilo |

### Fabricacion

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Listas de materiales (BOM) | data_file | blocker | Casi nunca con fichero |
| Rutas de fabricacion / centros de trabajo | information | blocker | A veces en el analisis |
| Gestion de subcontratacion | decision | normal | A veces en el analisis |

### Configuracion General / Transversal

| Peticion tipica | Tipo | Prioridad habitual | Notas de cotejo |
|----------------|------|--------------------|-----------------|
| Logo y datos generales de la empresa (logo, NIF, direccion fiscal, telefono, email, web) | information | blocker | Necesario para documentos oficiales (facturas, presupuestos, pedidos). Pedir confirmacion del NIF si ya lo tenemos. Explicar que estos datos apareceran en todos los documentos generados por Odoo |
| Listado de usuarios y roles/permisos | information | blocker | Raramente completo en el analisis |
| Acceso al sistema actual | access | blocker | Nunca en el analisis |
| Datos de contacto del interlocutor tecnico | information | normal | A veces en el CRM |

---

## Notas Adicionales

### Peticiones de migracion de datos

Cuando la tarea es de migracion, las peticiones tipicas son:
1. **Acceso al sistema origen** (access, blocker)
2. **Export de datos** por cada entidad: productos, clientes, proveedores, facturas... (data_file, blocker)
3. **Mapeo de campos** si el cliente tiene campos custom (information, normal)
4. **Validacion post-migracion** (validation, normal)

### Peticiones de integraciones

Cuando la tarea involucra integraciones externas:
1. **Documentacion de la API externa** (data_file, blocker)
2. **Credenciales de test** del sistema externo (access, blocker)
3. **Persona de contacto tecnico** del proveedor (information, normal)
4. **Volumenes esperados** de transacciones (information, normal)

### Peticiones de reports / documentos

Cuando la tarea es de disenar reports (facturas, albaranes, presupuestos):
1. **Ejemplo/plantilla del documento actual** (data_file, blocker)
2. **Logo en alta resolucion** (data_file, normal)
3. **Requisitos legales especificos** (information, normal)
