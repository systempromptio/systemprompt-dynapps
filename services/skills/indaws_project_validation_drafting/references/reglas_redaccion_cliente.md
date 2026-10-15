# Reglas de Redaccion para Documentos de Validacion

## Principio Fundamental

El cliente no es tecnico. No sabe Odoo, no sabe programacion, no sabe bases de datos. Habla de su negocio: pedidos, facturas, almacenes, empleados, clientes. Cada frase del documento debe estar en **su idioma**.

---

## Diccionario Tecnico → Negocio

### Terminos de Odoo

| Tecnico (PROHIBIDO) | Negocio (USAR) |
|---------------------|----------------|
| Many2one / Many2many | Lista desplegable / Relacion |
| Campo computado | Calculo automatico |
| Workflow / flujo automatizado | Proceso automatico |
| Modulo / app instalado | Funcionalidad activada |
| Vista tree / list | Listado |
| Vista form | Ficha / Formulario |
| Vista kanban | Tablero visual |
| Vista pivot / graph | Informe / Grafico |
| Registro / record | Ficha / Elemento |
| Modelo / model | — (no mencionar, describir la entidad: "producto", "cliente") |
| Campo / field | Dato / Informacion |
| Boton de accion / action | Opcion / Boton |
| Chatter / mail.thread | Historial de comunicaciones |
| Wizard / transient model | Asistente / Proceso guiado |
| Cron / scheduled action | Proceso programado / Tarea automatica |
| Domain / filtro tecnico | Filtro / Condicion |
| XML-ID / External ID | — (no mencionar) |
| Onchange / compute | Se actualiza automaticamente |
| Access rights / ir.rule | Permisos / Quien puede ver/editar |
| Sequence | Numeracion automatica |
| Journal / Diario contable | Registro contable |
| Account move / Asiento | Apunte contable |
| Stock move / Movimiento | Movimiento de mercancia |
| Picking / Albaran | Albaran de entrada/salida |
| Sale order line | Linea del presupuesto/pedido |
| Product template / variant | Producto / Variante de producto |
| Pricelist | Tarifa / Lista de precios |

### Terminos de desarrollo

| Tecnico (PROHIBIDO) | Negocio (USAR) |
|---------------------|----------------|
| Modulo custom / addon | Funcionalidad a medida |
| API / endpoint | Conexion con el sistema externo |
| Webhook | Notificacion automatica entre sistemas |
| Script / codigo | Proceso automatico |
| Bug / error de codigo | Incidencia |
| Deploy / despliegue | Puesta en marcha |
| Base de datos | Sistema |
| Servidor | — (no mencionar) |
| Migracion de datos | Importacion de datos historicos |
| CSV / XML / JSON | Fichero de datos |
| Query / consulta SQL | — (no mencionar) |

---

## Reglas de Estilo

### Tono

- **Profesional y cercano**: Como le explicarias a un director de empresa que ha contratado tus servicios
- **Segunda persona**: Usa "tu" / "vosotros" de forma natural
- **Orientado a resultados**: No "hemos configurado el modulo de ventas", sino "ahora podeis crear y enviar presupuestos desde Odoo"
- **Concreto**: No "se ha implementado la gestion de inventario", sino "vuestros 3 almacenes (Central, Norte, Sur) estan configurados con sus ubicaciones y responsables"

### Ejemplos concretos

**SIEMPRE** usa datos reales del entorno del cliente. Extrae del analisis funcional:
- Nombres de productos, clientes, proveedores
- Almacenes, departamentos, equipos
- Procesos de negocio especificos
- Volumenes (numero de productos, facturas/mes, empleados)

**MALO** (generico):
> "Se ha configurado el modulo de Ventas con las tarifas del cliente."

**BUENO** (concreto):
> "Ahora podeis crear presupuestos con vuestras 3 tarifas (mayorista, minorista, VIP). Los descuentos por volumen se aplican automaticamente a partir de 100 unidades, como teneis definido."

### Criterios de aceptacion

Los criterios deben ser **verificables** por el cliente sin ayuda tecnica:

**MALO** (ambiguo):
> "El sistema funciona correctamente"

**BUENO** (verificable):
> "Al crear un presupuesto para el cliente 'Hotel Plaza', el precio unitario del producto 'Sofa Modelo Valencia' muestra 1.250 EUR (tarifa mayorista)"

### Estructura de frases

- Frases cortas y directas (maximo 2 lineas)
- Un concepto por frase
- Usar listas para mas de 2 elementos
- Evitar subordinadas largas

---

## Lista de Terminos Prohibidos

Si cualquiera de estos terminos aparece en el documento final, es un error:

```
Many2one, Many2many, One2many, compute, onchange, domain,
ir.model, ir.rule, ir.sequence, mail.thread, mail.activity,
XML-ID, external_id, res.partner, product.template, sale.order,
account.move, stock.picking, stock.move, mrp.production,
project.task, hr.employee, knowledge.article,
x_studio_, x_custom_, __export__, __import__,
API, endpoint, webhook, JSON, XML, CSV, SQL, query,
modulo, addon, widget, view, tree, form, kanban, pivot, graph,
cron, scheduled action, transient, wizard (tecnico),
deploy, servidor, base de datos, codigo, script, bug
```

**Excepcion**: Si el cliente usa algun termino tecnico el mismo (ej: "CSV" porque exporta ficheros asi), se puede usar con contexto: "el fichero Excel o CSV que nos habeis enviado".
