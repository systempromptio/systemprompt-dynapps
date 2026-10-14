# Tipos de Peticion al Cliente

## Clasificacion

| Tipo | Codigo | Descripcion | Cuando se usa | Deadline |
|------|--------|-------------|---------------|----------|
| Informacion | `information` | Datos concretos que necesitamos del cliente | Preguntas sobre su negocio, procesos, volumenes | Dia anterior al inicio de la tarea |
| Fichero de datos | `data_file` | Fichero, export o documento que debe proporcionar | Exports de sistema actual, listados, catalogos | Dia anterior al inicio de la tarea |
| Validacion | `validation` | El cliente debe aprobar o validar algo | Configuraciones propuestas, categorias, flujos | Dia anterior al inicio de la tarea |
| Decision | `decision` | El cliente debe tomar una decision de negocio | Opciones de implementacion, politicas, reglas | Dia anterior al inicio de la tarea |
| Acceso | `access` | Credenciales, accesos o permisos | VPN, integraciones externas (NUNCA al software propio del cliente) | Dia anterior al inicio de la tarea |

## Calculo de Deadlines

**Regla principal**: El deadline de una peticion es el **dia anterior al `planned_date_begin`** de la tarea asociada.

- Si la tarea empieza un **lunes**, el deadline es el **viernes anterior** (ultimo dia laborable)
- Si la tarea empieza un **martes a viernes**, el deadline es el **dia anterior**
- Si la tarea **no tiene fecha planificada**, usar +14 dias como fallback y avisar al consultor

**Logica**: Necesitamos la informacion ANTES de empezar la tarea. No tiene sentido pedir datos en marzo si la tarea no empieza hasta mayo.

**Ejemplo**:
- Tarea "[MIGR] Migracion Suscripciones" empieza el 27/04 (lunes) → deadline: 24/04 (viernes)
- Tarea "Ventas" empieza el 06/04 (lunes) → deadline: 03/04 (viernes)

## Criterios de Prioridad

| Prioridad | Codigo | Criterio |
|-----------|--------|----------|
| **Bloqueante** | `1` (urgent) | Sin esta informacion NO se puede avanzar en la tarea. Bloquea el progreso del proyecto. |
| **Normal** | `0` | Se necesita para completar la tarea pero hay otras cosas en las que avanzar mientras tanto. |

### Cuando es bloqueante

- Ficheros de migracion de datos (sin datos no se puede migrar)
- Decisiones que afectan a la arquitectura (ej: multi-empresa si/no)
- Validaciones de configuraciones criticas antes de seguir construyendo encima

### Cuando es normal

- Informacion complementaria que no bloquea el avance inmediato
- Validaciones de aspectos cosmeticos o de detalle
- Datos de referencia que se pueden simular temporalmente

## Restricciones Importantes

- **NUNCA** solicitar acceso al software del cliente (Redmine, SAP, Sage, etc.). Los datos para migrar deben venir en Excel o CSV
- **NUNCA** solicitar acceso a la banca online del cliente. La sincronizacion bancaria es self-service: se proporcionan instrucciones claras al cliente

## Formato de la Descripcion (x_description)

**Tono**: Segunda persona (tu/vosotros), cercano pero profesional. Como hablarías con un cliente al que conoces y con el que tienes buena relación.

Toda peticion debe incluir en su descripcion HTML:

```html
<p><strong>Que necesitamos:</strong></p>
<p>[Descripcion clara y directa, dirigida al cliente en segunda persona]</p>

<p><strong>Por que lo necesitamos:</strong></p>
<p>[Contexto breve, explicando el impacto en su implantacion]</p>

<p><strong>Formato preferido:</strong></p>
<p>[Excel, CSV, listado, respuesta por email, etc.]</p>
```

### Ejemplos por tipo

**information**:
> Que necesitamos: Necesitamos que nos indiqueis los almacenes fisicos que gestionais, con nombre, direccion y persona responsable de cada uno.
> Por que: Con esta info configuraremos las ubicaciones de inventario en Odoo y asignaremos los permisos correctos a cada responsable.
> Formato: Podeis mandarlo por email o en un Excel sencillo.

**data_file**:
> Que necesitamos: Os pedimos un export del catalogo de productos actual desde vuestro sistema, incluyendo referencia, nombre, precio de venta y precio de coste.
> Por que: Necesitamos estos datos para migrar los productos a Odoo y poder configurar las tarifas correctamente.
> Formato: Excel o CSV con cabeceras, por favor.

**validation**:
> Que necesitamos: Os enviamos las categorias de producto que hemos propuesto en Odoo (ver adjunto). Necesitamos que las reviseis y nos deis el OK o nos indiqueis las correcciones.
> Por que: Necesitamos vuestra confirmacion antes de migrar los productos con esta clasificacion.
> Formato: Respondednos a este email con OK o con las correcciones que veais.

**decision**:
> Que necesitamos: Necesitamos que decidais si preferis facturar por pedido (cada pedido genera factura) o por envio (se factura al entregar).
> Por que: Esta decision afecta a la configuracion de ventas y contabilidad, y tenemos que tenerla clara antes de configurar los flujos.
> Formato: Respondednos a este email indicando la opcion que preferis.

**access**:
> Que necesitamos: Necesitamos las credenciales de acceso a vuestra pasarela de pago para configurar la integracion con Odoo.
> Por que: Sin estas credenciales no podemos configurar los pagos online en la tienda.
> Formato: Enviadnos las credenciales por email de forma segura.
