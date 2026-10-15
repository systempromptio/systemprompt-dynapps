# Mensajes de error literales — qué significa cada uno

Para qué sirve esto: **distinguir «el sistema te está diciendo que no» de «esto
está roto»**. La mitad de los mensajes de esta tabla son el vertical funcionando
bien, y reportarlos como defecto quema la credibilidad del informe entero.

Los literales van en inglés (código fuente) y, cuando se observaron traducidos,
también en castellano. **Casar por el literal, no por el tipo de excepción.**

---

## Importación FIEBDC-3

| Literal | Cuándo sale | Veredicto |
|---|---|---|
| `The FIEBDC3 file must be a BC3 file` · «El archivo FIEBDC3 debe ser un archivo con extensión .bc3» | Se adjunta un fichero cuya extensión no es `.bc3`/`.BC3`. Salta en un `@api.constrains`, antes de crear pedido ni trabajo | **Correcto.** Camino A-11. Ojo: valida la **extensión**, no el contenido; una copia byte a byte con otro nombre también se rechaza |
| *(ningún mensaje)* — la etiqueta del campo se pone en rojo y el asistente se queda abierto | Se pulsa Importar sin cliente y sin pedido de destino | **Comportamiento observado, y es una pega de usabilidad.** Camino A-12. No es un error del servidor: es la validación de campo obligatorio del cliente web. Se registra tal cual, porque es lo que el usuario ve, y lo que el usuario concluye es «el botón no funciona» |
| `The sale order {} already has a FIEBDC3 job.` · «El pedido de venta … ya tiene un trabajo FIEBDC3.» | Se importa sobre un pedido cuyo trabajo **no** está en `done` ni en `failed` | **Correcto.** Camino A-13. Sobre un trabajo en `done` la reimportación **sí** se permite. Si sale sin que nadie haya lanzado una importación, sospechar de un trabajo atascado en `running` (ver el error de transacción abortada) |
| `current transaction is aborted, commands ignored until end of transaction block` | Al procesar un trabajo con un parser de porcentaje en `import_mode = 'cost'` | **DEFECTO grave conocido nº 7.** El trabajo se queda en `running` con el campo `error` **vacío** —la transacción abortada impide hasta registrar el fallo— y el pedido queda a 0,00. Además bloquea futuras importaciones sobre ese pedido |
| `ValueError: Type must be M or A. Type: N` | El `.bc3` trae un registro `~N` (mediciones añadidas) **con líneas de detalle** | **DEFECTO conocido nº 20.** No es un fichero mal formado: `~N` con detalle es lo normal en una certificación o una modificación de obra. Un `.bc3` real así no se puede importar |
| `TypeError: replace() argument 2 must be str, not float` | El `.bc3` trae un `~D` con rendimiento no numérico | **DEFECTO conocido nº 21.** El estándar dice que ese caso se resuelve poniendo el rendimiento a 1,0 y siguiendo; aquí muere **al formatear el propio aviso** |
| `KeyError: None` en `Budget.getRecord` | El `.bc3` no tiene ningún concepto (`~C`) | **Correcto en lo funcional**: sin conceptos no hay raíz. Deja el trabajo en `failed` con el texto en `error`, que es lo que el camino negativo espera |

## Parsers de porcentaje

| Literal | Cuándo sale | Veredicto |
|---|---|---|
| `price_effect='add' is incompatible with import_mode 'line' because it would duplicate the amount. Use import_mode 'none' (only price effect) or 'cost' (price effect + margin tracking).` | Al **guardar** el parser con esa pareja | **Correcto.** Camino B-07. El registro no se guarda |
| El mismo literal con `'global'` en lugar de `'line'` | Ídem con `import_mode = 'global'` | **Correcto.** Camino B-08. Lo que se comprueba aquí es que **interpola el modo**; un texto genérico sería el defecto |
| `Cost Category is required when Import Mode is 'Cost Line'.` · «La Categoría de Coste es obligatoria cuando el Modo de Importación es 'Línea de Coste'.» | Al guardar un parser en modo coste sin categoría | **Correcto en el fondo, defectuoso en la forma.** Defecto conocido nº 12: el mensaje **sale duplicado**, concatenado consigo mismo sin separador |

## Certificación

| Literal | Cuándo sale | Veredicto |
|---|---|---|
| `Certified value cannot be less than the previously certified value.` · «El valor certificado no puede ser menor que el valor certificado anteriormente.» | Se baja lo certificado por debajo de lo de periodos anteriores | **Correcto.** Camino E-05. Este tope **inferior** lo conservan también los módulos de exceso |
| `Certified value cannot be greater than the original quantity` | Se certifica por encima de lo presupuestado | **Depende del alcance.** Con los módulos de **exceso instalados** este mensaje **no debe salir**: si sale, es defecto (camino E-04). Sin ellos, es el tope estándar y es correcto |
| `You cannot manually edit certified values on a line that has measurements. Certified quantities are derived from the measurements.` | Se escribe a mano cantidad o porcentaje en una línea que tiene mediciones | **Correcto.** Camino E-06. **Intercepta antes que el control de rango**: por eso teclear 150 % en una línea con mediciones da este mensaje y no el de porcentaje fuera de rango |
| `Only one draft certification is allowed per project.` · «Solo se permite una certificación en borrador por proyecto.» | Se crea una segunda certificación en borrador | **Correcto.** Camino E-07. Es la razón de que un recorrido de varios periodos tenga que facturar o cancelar cada certificación antes de crear la siguiente. También aparece al **borrar una factura**, que devuelve su certificación a borrador |
| `The percentage must be between 0 and 100.` · «El porcentaje debe estar entre 0 y 100.» | Porcentaje certificado fuera de rango | **Correcto.** Camino E-08. **El campo se guarda como fracción** (1,0 = 100 %): anotar el valor tecleado o el caso no es reproducible |
| `You cannot delete certification lines linked to certifications that are not in draft state.` | Se borra una línea de una certificación ya facturada o confirmada | **Correcto.** Camino E-09 |
| «Las siguientes líneas están certificadas por encima del presupuesto original. El presupuesto se mantiene sin cambios; esto se registra como exceso de medición.» | Al certificar por encima del 100 % con los módulos de exceso | **No es un error: es el aviso esperado.** Confirma que E-04 va bien |

## Facturación de certificaciones

| Literal | Cuándo sale | Veredicto |
|---|---|---|
| `Previous Certifications Product must be set when Billing Type is Origin.` | Al **guardar el proyecto** con facturación a origen y sin producto de anteriores | **Correcto.** Camino F-12. Salta al guardar el proyecto, **no** al facturar: quien lo espere en el momento de facturar concluirá que el control no existe |
| `No consolidated certification product configured on the project or company. Please configure one to proceed.` | Al **crear la factura** en modo consolidado sin producto | **Correcto.** Camino F-13. Solo se alcanza vaciándolo en el proyecto **y** en la compañía: con el de la compañía puesto, el código cae a él y la factura se crea sin protestar |
| `You cannot create an invoice from a certification without lines.` | Certificación en borrador sin líneas | **Correcto.** Camino F-14 |
| `You cannot create an invoice from a certification with zero amount to certify.` · «No puede crear una factura a partir de una certificación con importe cero para certificar.» | Certificación con líneas pero nada certificado | **Correcto… salvo en un caso.** Si las líneas **sí** tienen importe certificado escrito a mano y son líneas **sin mediciones**, este mensaje es el síntoma del **defecto conocido nº 3** (E-02): la cabecera no recoge lo certificado en la línea. Mirar siempre la línea antes de dar el mensaje por bueno |
| `This certification already has an invoice.` | Debería salir al facturar dos veces | **DEFECTO conocido nº 11: es inalcanzable.** Al facturar, el estado computado pasa a `invoiced` y salta antes el guardián de borrador. Si algún día se ve este mensaje, es que el orden de los controles cambió |
| `You can only create an invoice from a draft certification.` · «Solo puede crear una factura a partir de una certificación en borrador.» | Certificación confirmada, facturada o cancelada | **Correcto.** Camino F-17 — y también lo que sale de verdad en el caso de F-16 |
| «No puede cambiar la configuración de certificación porque este proyecto ya tiene certificaciones creadas» | Se toca la configuración de facturación de un proyecto que ya tiene certificaciones | **Correcto: la configuración se congela a propósito.** Cada modo necesita su propio proyecto. **Pero si sale sobre un proyecto que se acaba de crear**, es el síntoma del defecto conocido nº 5: la copia del presupuesto arrastró `project_id` y el «proyecto nuevo» es en realidad el de la obra original |

## Material de obra

| Literal | Cuándo sale | Veredicto |
|---|---|---|
| «No existe un precio de proveedor coincidente para generar el pedido de compra» | Se reaprovisiona un material cuyo producto no tiene precio de proveedor | **Correcto y estándar de Odoo**, no del vertical. Es preparación de dato: conviene saberlo antes de probar el bloque |
| Error de dominio al **pintar** la vista de una tarea con material | Al abrir la tarea | **Defecto si aparece.** Es el fallo que el backport del dominio del selector tenía que corregir (camino D-03) |

## Errores del arnés, no del producto

| Literal | Causa | Qué hacer |
|---|---|---|
| `KeyError` al crear el asistente de crear pedido de compra | Se instanció sin el contexto de la acción (`active_model`, `active_id`) | Reproducir con el contexto. **No es defecto**: por interfaz nunca ocurre |
| Un método que devuelve `None` y una respuesta JSON-RPC sin clave `result` | Normal en métodos que no devuelven nada | **No es fallo.** Si no hay clave `error`, la llamada fue bien |
| Un `search` que no encuentra el cron de importación | El cron está inactivo y Odoo lo filtra | Leerlo con `context={"active_test": False}` |
| Un método sin `@api.model` llamado por RPC que «no hace nada» y devuelve `None` | El primer argumento posicional se consume como `ids` | Pasar una lista de ids vacía delante de los argumentos reales |

## Precondiciones de almacén (material de obra)

Salen al **generar tareas** cuando está `inbuilt_sale_project_stock_material`.
Ninguno es defecto: son datos de almacén que faltan.

| Mensaje | Qué falta |
|---|---|
| `Destination location is not set for material %(material)s on task %(task)s.` | la ruta de la línea no tiene ubicación de destino en su primera regla |
| `Stock route is not set for sale line {}.` | la línea de venta no tiene ruta asignada |
