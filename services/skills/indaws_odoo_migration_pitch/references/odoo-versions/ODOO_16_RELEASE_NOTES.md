#  **Odoo 16**


##  General




#### Rendimiento



De media, cada página del backend es 3,7 veces más rápida de cargar y renderizar. El sitio web y el comercio electrónico son 2,7 veces más rápidos para cargar los controladores.



#### Nuevos tableros



Informes basados en hojas de cálculos reemplazan las varias acciones/vistas de tablero.



#### Campos personalizados dependiendo del contexto



Se pueden crear campos de "propiedad" en sus tickets, leads, tareas y artículos.



#### Derechos de acceso y herencia de grupos



Se te advertirá si la configuración de derechos de acceso no es válida.



#### Archivar servidores de correo



Archivar los servidores de correo para evitar su uso.



#### Paleta de comandos



Abrir menús en pestañas nuevas desde la paleta de comandos.



#### Editor



Los pads colaborativos (etherpads) se sustituyeron por el editor HTML nativo de Odoo.



#### Editor: barra de comandos



Se mejoró la barra de comando de la interfaz del usuario y el diseño en general. Se rediseñaron las secciones de la barra de comandos.



#### Editor: edición de tabla



Podrás añadir, mover y cambiar el tamaño de columnas o filas dentro de una tabla. También podrás configurar fondos específicos para cada celda.



#### Editor: pestañas



Se mejoró el comportamiento de las pestañas



#### Editor: iconos de los usuarios



Mostrar usuarios que están colaborando en campos de texto.



#### Resumen de correo electrónico



Todos los usuarios pueden cancelar la suscripción de resúmenes por correo.



#### Notificaciones por correo electrónico



Se añadieron vínculos a los registros de la base de datos a varias notificaciones de correos automáticos.



#### Plantillas de correo electrónico: Personalización



Personaliza la distribución y contenido de plantillas de correo electrónico desde los ajustes generales.



#### Plantillas de correo electrónico: Restablecer



Restablece las plantillas de correo electrónico rotas con un clic para que vuelvan a funcionar.



#### Vista de Gantt: dependencias



Utiliza la herramienta de creación de dependencias de las tareas del proyecto en cualquier vista de Gantt.



#### Módulo del RGPD



Añadimos una acción de búsqueda privada para encontrar todas las ocurrencias de un contacto y archivarlas o borrarlas si es necesario. También añadimos reglas de reciclado en registro de Limpieza de datos.



#### Google Drive/Spreadsheet: módulos obsoletos



Se quitó la integración con Google Drive y Google Spreadsheet de Odoo 16.



#### Unificación de las etiquetas de los derechos de acceso a las aplicaciones de RR. HH.



Los nombres de los distintos niveles de derechos de acceso para las aplicaciones de RR. HH. se han uniformizado para una mejor comprensión.



#### Pila de enrutamiento HTTP



La pila de enrutamiento HTTP fue refactorizada para hacerla más fácilmente extensible, manejar los errores más limpiamente y ser más fácil de navegar y trabajar con ella.



#### Previsualización móvil



Podrás previsualizar cómo se verán los correos desde el móvil.



#### Varios idiomas



Instale varios idiomas a la vez.



#### OAuth de Outlook



Es posible conectarse a un servidor de correo de Outlook con la nueva autenticación OAuth.



#### Previsualización de PDF: enlaces



Ahora todos los enlaces se abren en una pestaña nueva del navegador al previsualizar un archivo PDF adjunto.



#### Separación de informes en PDF



Se aumentó el espacio entre secciones de PDF en informes para mejorar la legibilidad.



#### Informes de PDF: fuente arábiga



Utiliza la escritura árabe en los informes PDF con la fuente Tajawal.



#### Desactivación de la cuenta del portal



Los usuarios del portal ahora pueden eliminar sus cuentas ellos mismos.



#### Créditos SMS



Se añadió un enlace para comprar créditos SMS en los ajustes generales.



#### Parámetros de UTM



Los usuarios con los derechos de acceso necesarios pueden eliminar los registros UTM, con algunas restricciones.



##  Contabilidad




#### Análisis



Añadimos un nuevo widget analítico a facturas para facilitar la distribución analítica, planes analíticos (que reemplazan a los grupos) y modelo de distribución analítico. Podrás hacer un desglose de los informes financieros. Analice informes financieros según cuentas o planes financieros. Se permite la edición masiva de análisis en las vistas de la lista.



#### Widget de conciliación bancaria



Se mejoraron los widgets de navegación, interfaz y legibilidad de la conciliación bancaria.



#### Usabilidad y navegación



Se ha actualizado la navegación en toda la aplicación de Contabilidad para evitar vistas técnicas y permitir una transición más fácil entre las diferentes pantallas.



#### Auditorías de diarios



Varias mejoras en las auditorías de los diarios: se ha aumentado la legibilidad general, se ha eliminado la información repetida, ahora se ignoran los diarios vacíos y se incluyen los detalles de los impuestos aplicados.



#### Informes contables



Arreglamos los informes contables desde la raíz para mejorar su rendimiento, interfaz del usuario, definición y auditoría.



#### Modo de despachos de contabilidad



Activa el modo de despacho de contabilidad para ingresar a un campo de total rápido y a la edición manual de secuencias para codificar las facturas rápidamente.



#### Menú de contabilidad



Se ha reducido el número de elementos del menú para ayudar a los usuarios a encontrar el menú adecuado en función del tipo de operación.



#### Cancelación de activos



No es posible cancelar un activo.



#### Modelos de activos



Los modelos de activos creados desde cualquier cuenta utilizan automáticamente la cuenta relacionada como cuenta de activos fijos de manera automática.



#### Asistentes de activos



La interfaz de usuario se ha limpiado agrupando varios asistentes de modificación de activos en un solo botón.



#### Activos: negativos y depreciación



Se ha añadido la gestión de activos negativos y la posibilidad de calcular las amortizaciones en función del número efectivo de días de cada periodo. Cualquier modificación de un activo da lugar ahora a un asiento de depreciación que cubre el periodo desde que se contabilizó el último asiento de depreciación. Se ha mejorado el tablero de cálculo y la importación de activos.



#### Activos: no deducible de impuestos



El valor original de un activo ahora se incrementa con la parte no deducible del impuesto (si lo hay).



#### Pagos por lotes: Cahatter y actividades



El chat y la programación de actividades ya están disponibles para los pagos por lotes.



#### Pagos por lotes: gestión de rechazo



Se mejoró el flujo de conciliación de los pagos por lotes que contienen pagos rechazados: ahora se pide a los usuarios que elijan entre cancelar el pago original o dejarlo abierto.



#### Descuentos de efectivo



Añadimos una nueva definición separada en términos de pago que admite diferentes legislaciones fiscales (incluidas o excluidas). Las líneas por cobrar ya no están separadas, lo que mejora la legibilidad de los informes de contabilidad y de seguimiento. El ayudante de registro de pago sugiere usar la cantidad reducida si está disponible. La conciliación sugiere que, si es posible, se utilice un descuento por pago anticipado calculado.



#### Límite de crédito por contacto



Configura un límite de crédito por empresa y/o partner. Cuando el importe total de las facturas abiertas para ese partner alcanza el límite especificado, se muestra un aviso en los pedidos de venta o en las facturas de los clientes.



#### Divisas



Rediseñamos cómo se muestran las divisas en las vistas de formularios de los asientos contables y las vistas de listas de los diarios



#### Tasa de cambio: fecha de la factura



La fecha de facturación ahora se usa para configurar la tasa de cambio de la divisa en lugar de la fecha contable.



#### Etiqueta de asientos de corte



Se mejoró la etiqueta de entradas de corte para mejorar la legibilidad de los libros de contabilidad.



#### Importe de datos



Añadimos una nueva guía, plantillas y funciones para importar los siguientes datos contables: contactos, plan contable y apuntes contables.



#### Categorías de gastos rechazados



Asigne una categoría de gastos rechazados y tasa a una cuenta desde la vista de lista de las categorías de gastos rechazados.



#### Cargar archivos con arrastrar y soltar



Importa archivos arrastrándolos y soltándolos en las tarjetas del tablero de Contabilidad (facturas, recibos, varios, banco) o en las vistas de la Lista de diarios.



#### Debug de los informes financieros



Usa la herramienta de debug de los informes financieros para analizar a qué cuentas potencialmente faltan en la definición del informe.



#### Informes de seguimiento



Se puede asignar diferentes contactos de seguimiento a un contacto. Se podrá excluir un contacto de los seguimientos automáticos. Configurar manualmente niveles de seguimiento en un contacto. Asistente mejorado para crear seguimientos. Interfaz mejorada: lista de seguimiento, informe individual, definición de los niveles de seguimiento. Filtrar informes de seguimiento en el nivel de seguimiento.



#### Renovación de los códigos de productos Intrastat



Los códigos de productos de Intrastat se renovaron, y el código de transacción automático se actualizó al de dos dígitos aplicable desde el 1 de enero de 2022.



#### Widget de pago de facturas



Se ha mejorado el widget de información de pagos en facturas y recibos para las transacciones con varias monedas: se resaltan las entradas de diferencia de cambio (si las hay) y, en el folleto informativo, los importes se expresan ahora tanto en la moneda de la empresa como en la de origen.



#### Búsqueda rápida de asientos y apuntes de diario



Se mejoraron las búsquedas rápidas en apuntes y asientos de diario.



#### Poner nombre a los grupos de diarios



Los nombres de los grupos de diario ahora son únicos para mejorar la legibilidad de informes financieros.



#### Informe de auditoría de diario



Informes de la auditoría de diarios renovados.



#### Rastreo de fechas bloqueadas



Ahora los cambios a fechas bloqueadas se registran en el chatter en la vista de formulario de la empresa.



#### Asientos de cuentas no comerciales



Los asientos de las cuentas no comerciales se excluyen ahora de los informes de antigüedad; hay un filtro disponible para volver a añadirlos. Los asientos de las cuentas no comerciales aparecen ahora en la pestaña Varios del widget de conciliación.



#### Alerta de secuencia no continua



Las alertas inteligentes se muestran en el tablero cuando la secuencia de un diario de ventas o de compras no es continua.



#### Validación de antecedentes OCR



Las facturas y los gastos cargados son validados en segundo plano por el OCR para que la interfaz sea más ágil.



#### Mapeo de campo OCR



Se ha mejorado la interfaz de cotejo manual de campos en las facturas.



#### OCR: ajustes de facturas



Active la digitalización de facturas de proveedores y clientes por separado.



#### OCR: identificación de contacto



Se ha mejorado la detección de contactos en las facturas mediante los números de cuentas bancarias y la información de autocompletar de los contactos.



#### Estados bancarios opcionales



Las transacciones bancarias ahora pueden existir solas: los estados bancarios ahora son opcionales.



#### Gestión de la cuenta bancaria de los contactos



La gestión de cuenta bancaria en contactos ha mejorado: todos los cambios se registran en el chatter. Introducimos un proceso de validación manual para cuentas añadidas automáticamente antes de que se pueda usar para pagos salientes.



#### Número de la cuenta bancaria de los contactos



Asigne el mismo número de cuenta bancaria a diferentes contactos.



#### Términos de pago



Nueva pantalla de definición de términos de pago. Añadimos métodos de cálculo para la fecha de cierre y ejemplos dinámicos.



#### Pagos: cantidades totales



Las cantidades se muestran al agrupar artículos en la vista de lista de pago.



#### Conciliación: Divisa extranjera



Se crea un asiento de diferencia de cambio directamente en cada pago parcial para asegurar el tipo de cambio entre la moneda del importe residual y el importe residual. Cuando se concilian dos líneas, una con una divisa extranjera y otra mostrada en la divisa de la empresa, la conciliación se realiza utilizando la divisa extranjera.



#### Gestión de movimientos de cuenta recurrentes



Se han simplificado los movimientos de cuentas recurrentes para facturas y recibos: se puede contabilizar un asiento automáticamente con diferentes frecuencias.



#### Legibilidad de los informes



Se mejoró el diseño y la legibilidad de los informes contables.



#### Informes: registros desplegables



Todos los informes de fiscales y financieros usan una forma mejorada de declarar datos gracias a los registros desplegables de padres-hijos en los editores.



#### Alertas de los diarios de compras y ventas



Recibirás una alerta en el tablero cuando la secuencia de diario de una compra o una venta no es continua.



#### Gestión de rechazos SEPA



Se mejoró la gestión de rechazos SEPA: al cancelar un pago perteneciente a un lote, el pago se elimina si no se infringe la fecha de bloqueo.



#### SEPA: mapeo de caracteres no latinos



La SEPA se ha ampliado para admitir los caracteres no latinos de todas las lenguas europeas locales.



#### Contabilidad Storno



Utiliza los débitos y créditos negativos para anular los asientos contables originales activando la configuración de la Contabilidad Storno.



#### Activos de vehículos



Todos los asientos de amortización pueden vincularse al vehículo. Se añadió un filtro de "separación de vehículo" al informe de Gastos rechazados.



##  Contabilidad - Localización




#### Austria



Los informes financieros ahora están más completos, ya que se calculan utilizando prefijos de códigos de cuenta. Añadimos dominios de control donde fue posible mediante tipos de cuenta y etiquetas de cuenta con fines de depuración e investigación. Los formatos din5008 ahora son independientes de la localización alemana.



#### Bélgica



Supresión definitiva del anticipo del IVA: se ha eliminado el campo Grid 91 del asistente de exportación XML en el informe fiscal belga. Se han mejorado los nombres de los impuestos y su búsqueda para agilizar la codificación de las facturas. Se han aclarado las etiquetas de los impuestos para mejorar su visualización en los documentos. Los archivos SODA pueden importarse ahora en los diarios diversos.



#### Brasil



Se ha añadido el paquete básico de localización: plan contable, impuestos e informes financieros.



#### Bulgaria



Se ha añadido el paquete básico de localización: plan contable, impuestos, informes financieros, informe de ganancias y pérdidas e informe de impuestos.



#### Chile



Los informes financieros ahora están más completos, ya que se calculan utilizando prefijos de códigos de cuenta. Añadimos dominios de control donde fue posible mediante tipos de cuenta y etiquetas de cuenta para depurar y realizar investigación. Añadimos los campos necesarios para Aduana en las facturas como parte de la factura electrónica para la exportación de bienes. Añadimos un entorno de demostración para probar de manera local los flujos de trabajo de las facturas electrónicas chilenas sin necesidad de contar con un certificado real y CAFs.



#### Dinamarca



Los informes financieros ahora están más completos, ya que se calculan utilizando prefijos de códigos de cuenta. Añadimos dominios de control donde fue posible mediante tipos de cuenta y etiquetas de cuenta con fines de depuración e investigación.



#### Ecuador



Mejoramos el paquete básico de la localización: plan contable, impuestos. Añadimos y adaptamos los informes fiscales: Balance de situación, Informe de resultados, Informes 103 y 104. Añadimos el módulo para enviar documentos electrónicos: facturas de ventas, acuerdos de compra, notas de crédito y débito, retenciones de compra y compatibilidad con retenciones de ventas.



#### Egipto



Integración completa con el portal de la autoridad tributaria del país. Las facturas de empresas y particulares se pueden enviar al portal con un solo clic.



#### Francia



Como todas las líneas de liquidación de impuestos se redondean al euro más cercano en Francia, el asiento de cierre se redondea ahora en consecuencia (Cuenta de Impuestos a Pagar o a Cobrar), teniendo en cuenta si es un beneficio o una pérdida para el redondeo. Los importes flotantes se siguen manteniendo para una correcta conciliación en las cuentas utilizadas para la distribución de los impuestos. Se han añadido nuevos dominios de control para los informes financieros en modo de depuración.



#### Alemania



Exportación de DATEV con o sin los archivos PDF de las facturas



#### Hungría



Se ha actualizado la localización: plan contable, impuestos, posiciones fiscales e informes financieros.



#### Indonesia



Se actualizó la localización del plan contable y de los impuestos. Se han corregido pequeños errores de e-Faktur.



#### Italia



Añadimos una hoja de balance e informes de ganancias y pérdidas.



#### Lituania



Los informes financieros ahora están más completos, ya que se calculan utilizando prefijos de códigos de cuenta. Añadimos dominios de control donde fue posible mediante tipos de cuenta y etiquetas de cuenta con fines de depuración e investigación.



#### Luxemburgo



Se añadieron nuevos dominios de control para los informes financieros en modo de depuración.



#### México



Los informes financieros ahora están más completos, ya que se calculan utilizando prefijos de códigos de cuenta. Añadimos dominios de control donde fue posible mediante tipos de cuenta y etiquetas de cuenta con fines de depuración e investigación.



#### Pakistán



Añadimos el balance, plan contable, estado de ganancias y pérdidas e informes de impuestos.



#### Perú



Añadimos un nuevo campo en los impuestos y adaptamos los XML para poder utilizar el impuesto ISC sobre bienes peruanos en facturas electrónicas.



#### Arabia Saudita



Se añadieron todos los impuestos al valor añadido, que se mapearon con las cuentas correspondientes en el plan contable y se vincularon al informe de impuestos que cumple con el formato ZATCA.



#### Serbia



Se ha añadido el paquete básico de localización: plan contable, impuestos, informes financieros, informe de ganancias y pérdidas e informe de impuestos.



#### Eslovenia



Se ha actualizado la localización del plan contable y de los impuestos. Se han añadido los informes de balance y de pérdidas y ganancias.



#### España



Se añadieron nuevos dominios de control para los informes financieros en modo de depuración.



#### Suecia



Añadimos informes financieros para la contabilidad K1, K2 y K3. Añadimos dominios de control para informes financieros en el modo de debug.



#### Suiza



Los formatos din5008 ahora son independientes de la localización alemana. Imprima facturas QR en lotes.



##  Evaluación




#### Plantilla de evaluación 360



Se mejoró la plantilla automática de encuestas para evaluación 360.



#### Fecha de evaluación



La fecha de evaluación automática se basa ahora en la fecha de inicio del contrato del empleado. Si no existe, se utiliza la fecha de creación del empleado. Puede editar la fecha de evaluación de varios registros a la vez.



#### Habilidades de los empleados



Cree nuevas habilidades de un empleado directamente desde una evaluación.



#### Informes: evolución de habilidades



Añadimos un informe sobre la evolución de las habilidades de los empleados.



##  Aprobaciones




#### Aprobadores



Configura aprobadores en orden secuencial.



#### Derechos heredados



Los usuarios pueden ver las solicitudes que tienen que aprobar sin derechos de acceso específicos.



##  Asistencias




#### Horas extra



Al salir, ve las horas extra del día y el total de horas extra solo si el número es positivo.



#### Autenticación del modo quiosco



Los usuarios podrán elegir su método de autenticación en modo kiosk. Se podrá definir la duración del mensaje de bienvenida y qué camara se usará para escanear el código de barras



##  Código de barras




#### Operaciones



La vista del formulario de operación se ha optimizado para los dispositivos móviles y se ha añadido el chat.



#### Paquetes: movimientos de inventario



Establece los embalajes durante los movimientos de existencias con el botón "+embalaje" para que las operaciones sean aún más rápidas.



#### Eliminación de la recolección



Borra un producto que se escaneó pero que no es parte de una orden.



#### Recibos



Ver la referencia del proveedor en las recolecciones.



#### Escaneo de paquetes



Se puede filtrar transferencias al escanear paquetes. Es posible abrir una transferencia al escanear un paquete o un código de transferencia en la lista de transferencias. Puede escanear un paquete para conocer sus contenidos desde el menú principal.



#### Escaneo por tipo de operación



Define qué se debe escanear o no por tipo de operación: ubicación inicial, producto, seriales, paquete, ubicación de destinación. Mejores instrucciones en la barra superior.



#### Números de serie y lotes



Los números de serie y los lotes están ahora plegados por defecto. Se ha mejorado el desplazamiento de los mismos cuando están desplegados. Visualización de la fecha de caducidad.



#### Transferencias: chatter y kanban



Añadimos el chatter para transferir información y una vista kanban de los movimientos.



#### Asignación de usuario en la recolección



Ahora se podrá evitar que dos personas realicen la misma operación, para esto asigne un usuario en la recolección o en los lotes tan pronto como este usuario cambie cantidades.



##  Calendario




#### Legibilidad



Se mejoraron los colores del calendario para mejorar la legibilidad.



#### Videollamadas



Crear una videollamada desde el formulario del evento. Es posible unirse a la videollamada desde el formulario del evento o la notificación de la junta.



##  Consolidación




#### Ganancias y pérdidas



Las pérdidas y ganancias consolidadas ahora pueden mostrar beneficios positivos y pérdidas negativas (en lugar de débito menos crédito) cuando se utiliza el botón "Invertir el signo del balance".



##  CRM




#### Archivar los niveles de acciones del partner



Archivar los niveles de activación de los partners.



#### Instalación de idiomas



Instala nuevos idiomas desde la vista de lista.



#### Análisis de leads



Ve todos los clientes potenciales que tienes asignados y su evolución desde un único lugar de información.



#### Idioma de leads



Si un lead no tiene un idioma configurado, ahora obtendrá el que esté configurado en el cliente vinculado.



#### Vincular oportunidad



Al crear una invitación de calendario a partir de un lead, el evento resultante está ahora vinculado al lead.



#### Razón de pérdida



Añade un comentario de cierre cuando marques una oportunidad como perdida.



#### Fusionar leads



Al fusionar leads, los informes se pliegan ahora en el chatter y reagrupan toda la información de los leads fusionados.



#### Fusionar leads



Al fusionar leads, todos los campos vacíos del lead de destino se completan con el lead de origen.



#### Ingresos recurrentes



Los totales de ingresos recurrentes se muestran en los encabezados de las columnas de la vista Kanban.



#### Seguimiento de revendedores



Haga un seguimiento de las oportunidades que los revendedores (contactos) están manejando desde su botón de estadísticas y en la vista Kanban.



#### Detectar lead similar



Detectas leads similares según su número de teléfono.



##  Conversaciones




#### Legibilidad del menú desplegable de Conversaciones



Se mejoró la legibilidad del menú desplegable de conversaciones (bandeja de entrada).



#### Copian enlace de invitación



Añadimos un botón para "copiar" en los enlaces de invitación a los canales.



#### Botón para descargar imágenes



Se ha añadido un botón de descarga que aparece al pasar el cursor por encima de una imagen adjunta en la aplicación Conversaciones o en el chatter



#### Emoticones



Añadimos varios emoticones acomodados en categorías y una función de búsqueda.



#### Unirse/abandonar notificaciones



Se rediseñó el botón de unirse o abandonar notificaciones para que estorbe menos.



#### Vincular previsualizaciones



La previsualización de vínculos está disponible cuando se publica un mensaje.



#### Lista de mensajes de acciones



La lista de mensajes de acciones ha sido rediseñada con un modo de compactación/expansión.



#### Notificaciones



Las notificaciones se muestran para todos los pagos a los que se les da seguimiento (por ejemplo, cambio de etapa, tarea lista) en la bandeja del sistema.



#### Abrir los mensajes directos desde la mención/nombre de miembro



Se podrá abrir los mensajes directos con un usuario al hacer clic en la @mención, ya sea en un mensaje directo/chatter, o el nombre del miembro en Conversaciones.



#### Buscar usuario actual



El usuario actual se incluye ahora en los resultados de búsqueda del chatbox.



#### Mostrar invitados



Los invitados ahora se muestran en las listas de miembros.



#### Desenfoque del fondo del video



Desenfoque el fondo durante las videollamadas.



#### Interfaz del usuario en videollamadas



Se ha mejorado la interfaz de usuario de las videollamadas, con un panel lateral dedicado a los ajustes.



##  Documentos




#### Botones deshabilitados



Ahora los botones están deshabilitados en espacios de trabajo de solo lectura.



#### Visita de lista: agrupar por



Usa la opción de agrupar por en la vista de lista



#### Tamaño máximo del archivo



Establece un tamaño máximo para los archivos. Ahora se muestra un mensaje de error claro si el archivo es demasiado grande.



#### Panel de navegación



Cambia el tamaño del panel de navegación para mejorar la legibilidad. El icono de ajustes aparece al pasar el ratón por encima.



#### Búsqueda OCR



Busca el contenido de un documento que pasó por el OCR.



#### Compartir



El proceso para compartir documentos ahora es más rápido.



#### Herramienta Dividir: cancelar y archivar



Añadimos botones de cancelar y archivar a la herramienta de Dividir además de varias mejoras a la interfaz del usuario.



#### Miniaturas y vistas previas



Las miniaturas aparecen para más tipos de archivos. Se mejoró la vista previa de los documentos.



##  Comercio electrónico




#### Recordatorios para cestas abandonadas



Envía recordatorios de cestas abandonadas automáticamente a los clientes.



#### Bloque de creación para productos alternativos



Personaliza el bloque de "productos alternativos" al final de la página del producto.



#### Variantes de productos archivados



Las variantes de productos archivadas ya no se pueden elegir en el comercio electrónico



#### Atributo tipo de visualización



Cambiar los tipos de visualización de los atributos del producto (botón, radio, color, etc.) desde el front-end.



#### Notificación de nuevas existencias



Los clientes ahora podrán recibir una notificación cuando regresen las existencias de un producto. Se pueden registrar directamente en la página del producto, incluso si no iniciaron sesión.



#### Número de artículos en la cesta



El número de artículos en la cesta se actualiza constantemente.



#### Cupones, tarjetas de lealtad y cartera electrónica



Ofrece promociones, cupones, lealtad y tarjetas de regalo a los clientes. Rastrea su uso a través del las actividades de Punto de venta, Ventas y Comercio electrónico.



#### Autocompletado y validación de la dirección del cliente



Si utiliza la API de Google Places: proporciona autocompletar la dirección y la validación durante el pago del comercio electrónico.



#### Inicio de sesión obligatorio



Se mejoró el proceso de pago cuando el inicio de sesión es obligatorio.



#### Entrega de Mondial Relay



Ahora los clientes podrán elegir un punto de recolección Mondial Relay durante el pago



#### Opciones de páginas



Se puede cambiar la página de opciones desde el modo de edición en el panel de la derecha.



#### Recolección y pago en el sitio



Añadimos la opción de recolectar y pagar dentro de la tienda durante el pago.



#### Prevenir la venta de productos



Evita que los visitantes añadan algunos productos a sus cestas. Por ejemplo, las empresas B2B podrían no querer mostrar los precios a los usuarios no conectados o no mostrarlos en absoluto si la tienda debe ser solo un catálogo.



#### Imágenes del producto



Podrás subir varias fotos de un producto en bloque. También, podrás mostrar las imágenes de los productos en una retícula o carrusel.



#### Fragmento del producto



Se volvió a diseñar el fragmento del producto.



#### Etiquetas del producto



Filtrar productos en el backend y en el frontend ahora es más fácil con las etiquetas de productos.



#### Clasificación automática de productos



Establece la forma predeterminada de ordenar los productos en la página de la tienda.



#### Volver a ordenar desde el portal



Ahora los clientes podrán volver a ordenar los mismos productos desde su portal.



#### Alquiler



Nueva integración entre Alquiler y Comercio electrónico: ahora los clientes pueden pedir productos de alquiler directamente desde el sitio web. Los clientes pueden buscar productos de alquiler que estén disponibles durante un periodo específico. La búsqueda puede realizarse mediante un bloque específico añadido en algún lugar de la página web o directamente en el catálogo de productos con un filtro de fechas.



#### Página de tienda



Nuevo diseño automático para la página de tienda



#### Filtros de la barra lateral



La barra lateral de comercio electrónico se rediseñó. Aparece en el panel derecho en los móviles. Filtra las categorías utilizando la nueva barra de películas.



#### Fragmento: añadir producto a la cesta



Crea botones que añadirán un producto a la cesta del cliente.



##  e-learning




#### Banner de encabezado



Personaliza el banner del encabezado para cursos.



#### Cursos comprados



Se mejoró el rastreo de los cursos que los clientes adquirieron. Añadimos un nuevo tipo de producto llamado "curso".



#### Duplicación de cursos



Al duplicar un curso ahora también se copia todo su contenido.



#### Contenido hecho o no hecho



Los asistentes pueden marcar el contenido como hecho o no hecho.



#### Gestión de incorporación de empleados



Ahora hay un acceso rápido que permite que los asistentes inmediatamente ingresen a los cursos que acaban de comprar.



#### Rastreo de publicación



Rastree todo en el chatter cuando el contenido se publique o deje de publicarse.



#### Invitación de registro



Se invita a los visitantes a registrarse para el curso cuando comparten un curso al que no tienen acceso.



#### Historial de pruebas



Los asistentes pueden tener una vista general de todos sus intentos de certificación.



#### Vimeo



Utiliza los enlaces de Vimeo para las lecciones en vídeo.



##  Marketing por correo electrónico




#### Edición en masa de contactos



Editar en maso contactos de correo desde la lista de vista.



#### Formas de imágenes



Usa formas para transformar imágenes de correo.



#### Vista de lista



Los correos enviados y agendados aparecerán en orden cronológico en la vista de lista.



#### Importación de contactos de correo



Importar contactos en una lista de correo pegando su dirección de correo electrónico. Descargue una plantilla de importación de contactos de correo para ver rápidamente cómo debe formatearse.



#### Informes de correo



Desactiva los informes de envío de estadísticas de 24 horas en la configuración.



#### Fragmentos del boletín informativo



Personaliza los fragmentos del boletín informativo del sitio web y cree formularios personalizados.



#### Guardar filtros



Guarda los filtros de marketing utilizados para la segmentación de la audiencia y reutilízalos para futuros envíos.



#### Gestión de la plantilla



Guarda correos como plantillas para volver a usar el diseño más adelante.



##  Empleados




#### Departamentos



Nuevo botón inteligente en la vista de formulario del departamento que muestra el número de empleados.



#### Botón inteligente de e-learning



Añadimos un botón inteligente con un enlace al perfil del usuario dentro de e-learning.



#### Etiquetas de empleados



Los usuarios con derechos de responsable de empleados pueden ahora editar las etiquetas de los empleados.



#### Filtros y búsqueda



Añadimos filtros de "Mi equipo" y "Mi departamento" a la vista de empleados, así como una opción de búsqueda para "gerentes".



#### Flota



El informe "reclamar coche" se sustituye por el botón inteligente "coches", que ahora muestra el historial de vehículos de un empleado.



#### México



Localización para la aplicación de empleados de México con campos adicionales en los registros de los empleados.



#### Varios planes



Inicia planes para varios empleados al mismo tiempo.



#### Nuevo empleado: creación de empleados



Añadimos una opción para crear un empleado directamente al usar el "ayudante de creación de nuevo usuario". Añadimos una nueva acción de "crear usuario". Añadimos una nueva acción "crear usuario" al ver el perfil de un empleado.



#### Planes por departamento



Los planes de incorporación/despedida pueden ahora vincularse a un departamento.



#### Planes: solicitar firma



La actividad "Solicitar firma" ahora envía automáticamente una solicitud de firma por correo electrónico.



#### Asistente de solicitud de firma



Añadimos el ayudante de solicitud de firma a la aplicación.



#### Informe de habilidades



Ingresa a un informe donde se mostrarán todas las habilidades de un empleado.



#### Asistente de habilidades



Se ha modificado el diseño del asistente de creación de habilidades. Se ha añadido un nivel de habilidad automática.



##  Eventos




#### Orden de ubicaciones



Elegir el orden de las ubicaciones del evento.



#### Preguntas obligatorias



Define preguntas obligatorias en los eventos.



#### Estadísticas de registro



Rastree los registros a un evento con el botón de estadísiticas.



#### Informe de ingresos



Revisa las ganancias de un evento en un menú de informe especializado.



#### Orden de etiquetas



Ordena las etiquetas del evento como usted quiera en el sitio web.



#### Tema del sitio web



Las páginas de evento ahora se adaptan dependiendo del tema del sitio web.



#### Sitio web: zonas horarias



Las zonas horarias ahora se muestran junto a todas las horas en todas las páginas de un evento para evitar confusiones.



##  Gastos




#### Diarios automáticos



Configurar los diarios que se usan automáticamente al crear gastos



#### Vincular a pedidos de venta



Ingresa a pedidos de venta que se vean afectadas directamente por gastos de los informes de gastos.



#### Volver a facturar gastos



Los gastos que se refacturan en un pedido de cliente tienen ahora un impacto en el coste total del pedido y no modifican el margen.



#### Compatibilidad de impuestos



Todas las configuraciones de impuestos (incluidos y excluidos del precio) ahora están compatibles con Gastos.



#### Mejoras a la interfaz del usuario



Se añadieron categorías automáticas de gastos para todas las nuevas instalaciones de la aplicación. Los menús y filtros se simplificaron para los encargados de gastos. Se pueden crear gastos en masa si arrastra y suelta documentos en la aplicación.



##  Servicio de campo




#### Abrir informes



Todos los usuarios de servicio de campo tienen acceso al menú de informes y pueden ver la información disponible para su nivel de derechos de acceso.



##  Flota




#### Seguimiento del chatter



Ahora las emisiones de CO2, costes recurrentes y activación de costes ahora se registran en el chatter.



#### Matrícula



Nuevo campo de "fecha de cancelación" para matrículas.



#### Mejoras a la interfaz del usuario



Varios cambios para mejorar la interfaz del usuario.



##  Servicio de asistencia




#### Integración del servicio de asistencia e información



Se ha rediseñado el Centro de Ayuda para animar a los clientes a consultar los cursos de e-learning, los mensajes del foro y los artículos de conocimiento antes de enviar un ticket.



#### Abrir informes



Todos los usuarios del servicio de asistencia tienen acceso al menú de informes y pueden ver la información disponible para su nivel de derechos de acceso.



#### Plantillas SMS en etapas



Envía SMS automáticamente cuando un ticket llega a una etapa en específico.



#### Asignación de tickets: tiempo personal



Cuando los tickets se asignan automáticamente, se salta a los empleados en ausencia.



#### Análisis de tickets



Obtén estadísticas sobre el tiempo medio que se tarda en responder a los tickets y la primera respuesta a un ticket.



#### Secuencia de tickets



Personaliza la secuencia (número y formato de ID) de sus tickets.



#### Conversión de tickets a tarea



Convierte tickets a tareas y viceversa



#### Parámetros de UTM



Cuando un lead se convierte en un ticket, los valores de los parámetros UTM del lead se propagan al ticket.



##  Inventario




#### Transferencias por lote automáticas



A nivel de Tipos de Operación, decida cómo automatizar la creación de lotes, ya sea por contacto, transportista o país de destino.



#### Forzar órdenes parciales



Ahora se puede decidir si las órdenes parciales se crean automáticamente o no por tipo de operación.



#### Ajustes de inventario



Añadimos categorías de desplazamiento y fecha de último conteo. Ahora se muestra un icono de advertencia junto a seriales duplicados. Añadimos un botón de "aplicar a todos" en la parte de arriba de la pantalla y un filtro en los productos favoritos.



#### Lotes y paquetes: códigos de barras GS1-128



Ahora es posible crear etiquetas GS1 para lotes y números de serie (que contienen el producto, el lote o número de serie, la fecha de caducidad y la fecha de venta). La impresión del contenido del paquete también imprime los códigos datamatrix para todo el contenido (incluyendo el lote/SN, las fechas de caducidad, etc.).



#### Informe de recepción



Se mejoró el informe de recepción con la adición de enlaces a los pedidos de venta, los nombres de los productos en las etiquetas, la visualización automática en el código de barras y el traslado de la configuración de la auto-aparición al tipo de operación.



#### Ubicaciones de reabastecimiento



Automatizar el reabastecimiento en ubicaciones específicas, para esto se deben configurar como Ubicaciones de reabastecimiento. Use el panel de la izquierda en el reabastecimiento para acomodar por ubicación o categoría del producto.



#### Reabastecimiento: Niveles de almacén



Cuando se reabastezca de otros almacenes, vea sus existencias disponibles en la información del proveedor del informe de reabastecimiento.



#### Informes y menús



Se han renovado todos los informes. Se puede mover fácilmente de un informe a otro. Menús renovados.



#### Transferencias: cantidad hecha



Edición fácil de las cantidades hechas en operaciones de transferencia.



#### Días de visibilidad



Establece los días de visibilidad en una regla de reabastecimiento para considerar la cantidad necesaria después de la fecha de previsión.



##  Conocimientos




#### Gestionar de Conocimientos



Gestiona los conocimientos de tus empleados mediante una aplicación específica para obtener más información y conocimientos.



#### Copiar y duplicar



Ahora se puede crear una copia de un articulo o duplicar toda su herencia.



#### Selección de portada



Elije imágenes de Unsplash como portadas de artículos.



#### Vistas incrustadas y owlificación



Usa el comando de /item para crear bases de datos personalizadas dentro de los artículos.



#### Artículos anidados



Almacenar artículos con widgets que comparten los mismos campos de propiedades dentro de los artículos.



#### Resumen e índice



Inserta índices y esquemas en los artículos para mostrar los artículos anidados.



#### Índice



Añade una tabla de contenido interactiva dentro de los artículos.



#### Gestión de basura



Los artículos eliminados tienen un periodo de gracia antes de que se les elimine permanentemente.



##  Chat en directo




#### Chatbot



Utiliza los chatbots para dar la bienvenida y guiar a los visitantes del sitio web.



#### Botón de chat en directo



Mostrar una notificación al lado del botón del chat en directo.



##  Comida




#### Notificación de envío



Nuevo botón de "confirmar recepción" para enviar una notificación a los empleados cuando ya hayan recibido su comida.



#### Etapa de envío de la orden



Añadimos una etapa nueva, "orden enviada".



##  Fabricación




#### Informes de asignaciones



Los informes de asignaciones ahora están disponibles para órdenes de fabricación. Si dos órdenes de fabricación están vinculadas, las relaciones de padre/hijo se crean automáticamente. Si una orden de fabricación está vinculada a un pedido de venta, ahora se podrá ver la referencia y estado en el portal del cliente.



#### Producción continua



Para los tipos de operaciones de fabricación, configure el consumo automático de productos con seguimiento de lote/número de serie durante la producción. Propagación automática de los cambios de cantidad en la orden de fabricación, a las transferencias previas y posteriores a la producción.



#### Notificación de lotes vencidos



Recibe notificaciones si usa lotes caducos tanto para ordenes de fabricación como para operaciones de recolección.



#### Valoración de kit



Establece una cuota de costes en los componentes de la lista de materiales de tipo kit para dividir automáticamente los precios de los pedidos en función de la valoración de los componentes.



#### Consumo manual de órdenes de fabricación



Selecciona los componentes de la lista de materiales que deben registrarse manualmente durante la fabricación. A continuación, el consumo de estos componentes debe registrarse manualmente antes de cerrar la orden de fabricación.



#### Sistemas de ejecución de fabricación



Rediseñamos completamente la vista en tableta de la orden de trabajo. Ahora hay cuatro tipos diferentes de comentarios de mejora de la lista de materiales disponibles en el menú Acción. Los empleados pueden registrarse en los centros de trabajo y en las órdenes de trabajo, de forma individual o simultánea. El coste de su parte de horas también se tiene en cuenta en el cálculo del coste final del producto fabricado.



#### Resumen de orden de fabricación y lista de materiales



Rediseñamos completamente el informe de estructura y costes, que ahora se llama Resumen. Ahora están disponibles os plazos de entrega y la fecha de próxima disponibilidad, basados en las existencias actuales y previstas.



#### Programa maestro de producción



Importa y exporta productos y sus pronósticos. Borra y reabastece en lotes.



#### Dependencias de operaciones



Crear relaciones de operación en la lista de materiales que influyan dinámicamente tanto en la planificación como en el estado de las órdenes de trabajo.



#### Importación/exportación de productos y pronósticos



Importación/exportación fácil de productos y pronósticos



#### Devolución y reparación



Crear una orden de reparación a partir de una devolución. Una vez que se haya procesado la devolución, se notifica de la orden de reparación respectiva.



#### Dividir y fusionar ordenes de fabricación



Divide las órdenes de fabricación para la producción por lotes o dividir y asigna la producción a diferentes empleados o centros de trabajo. Combina órdenes de fabricación para la producción por lotes o la planificación.



#### Portal de subcontratación



Los subcontratistas de fabricación pueden ahora registrar su producción desde su portal de clientes. Se pueden conceder derechos de acceso específicos para que solo se compartan en el portal los documentos de producción pertinentes.



#### Usabilidad



Identifica órdenes de fabricación sin construir con un filtro automático en la vista de lista.



#### Capacidad de los centros de trabajo



Ampliamos las capacidades específicas de los centros de trabajo teniendo en cuenta los tiempos de preparación y limpieza.



#### Órdenes de trabajo: buscar producto



Busca órdenes de trabajo por el último producto que produjeron.



##  Citas en línea




#### Autopublicación y redirección



Los tipos de citas que se crean desde el sitio web se publican automáticamente. Cuando solo hay un tipo de cita disponible, se redirige a los visitantes al calendario.



#### Calendario de reservas



El calendario de reservas ahora abre por defecto el mes con la primera disponibilidad, utiliza la zona horaria del visitante si no se establece ninguna ubicación en el tipo de cita, y utiliza el idioma del navegador del visitante para formatear la semana y la hora.



#### Recordatorios y horarios automáticos



Los tipos de citas creados en el sitio web ahora funcionan de forma inmediata gracias a los recordatorios y horarios predeterminados.



#### Ajustes del diseño y la flexibilidad



Los enlaces compartidos ahora mantienen los filtros establecidos a través de todo el flujo de reservas, además de varias mejoras de diseño a pequeña escala.



#### Creación del frontend



Selecciona los usuarios al crear un nuevo tipo de cita desde el frontend. Se han añadido consejos para ayudar a la configuración de nuevos tipos de citas desde el frontend.



#### Vincular tipos



Al compartir un enlace a un tipo de cita, elije qué usuario(s) puede(n) seleccionarse: tú mismo, todos los usuarios asignados o usuarios específicos.



#### Ubicación



Las localizaciones de eventos físicos ahora usan las direcciones completas de los formularios de los contactos.



#### Enlaces de juntas



Cuando una cita es en línea (sin ubicación), se añade un enlace a una reunión en línea a la confirmación de la cita.



#### Media noche



Los espacios para citas ahora pueden estar disponible hasta las 00:00 (media noche).



#### Gestión de incorporación de empleados



Agenda citas con tres pasos rápidos en el panel de capacitación.



#### Selección de operador



Los clientes podrán elegir a sus operadores.



#### Acceso privado



Se pueden crear enlaces a citas personalizados y configurar el URL que se muestra. Comparta los tipos de citas que no se publican en su sitio web con asistentes específicos.



#### Informe de preguntas



Revisa las respuestas que dieron los asistentes al realizar una cita.



#### Usuarios y espacios de tiempo



Asigna usuarios a espacios de tiempo específicos para que sean los únicos usuarios disponibles durante ese horario.



#### Enlace entre el visitante y el lead



Al reservar una cita, el visitante queda vinculado al lead generado.



#### Consejos para los visitantes



Añadimos consejos para los visitantes cuando no aparecen espacios de citas.



#### Restricciones de las horas de trabajo



Evite las restricciones de las horas de trabajo y programe juntas en cualquier momento que esté disponible según el tipo de cita.



##  Pagos en línea




#### Adyen: captura manual



Añadimos la opción de activar pagos de dos pasos para Adyen. Para esto, primero reservamos la cantidad en la tarjeta y después capturar la cantidad manualmente (por ejemplo, para reservas de hoteles).



#### Alipay/Ogone/PayUmoney/PayU Latam obsoletos



Alipay, Ogone, PayUmoney, y PayU Latam son obsoletos. Los quitaremos en el futuro.



#### Servicios de pago de Amazon



Los servicios de pago de Amazon están disponibles como un proveedor de pago en línea para el Medio Oriente.



#### AsiaPay



AsiaPay está disponibles como un proveedor de pagos en línea para países de Asia-Pacífico.



#### Authorize.net: Reembolsos



Los pagos en línea realizados con Autorize.net ahora se pueden reembolsar desde Odoo o desde el tablero de Authorize.net.



#### Portal del cliente



Los métodos de pago (tokens) de los proveedores inhabilitados están ocultos en el portal del cliente.



#### Proveedor de prueba



El proveedor de pagos "de Prueba" soporta ahora todas las funciones de pago adicionales: guardar tarjetas para más tarde, captura manual, tarifas de clientes y reembolsos. Los usuarios también pueden elegir el resultado de su pago para probar diferentes flujos de pago.



#### Pago exprés



Habilita el pago rápido para los usuarios de Google Pay y Apple Pay con Stripe: la dirección de facturación y de envío se rellena automáticamente.



#### Filtrar proveedores



Se puede filtrar qué proveedores se muestran en los formularios de pago según la cantidad máxima de pago.



#### Flutterwave



Flutterwave está disponible como proveedor de pago en África.



#### Facturas/Contabilidad



Los proveedores de pago en línea ahora se pueden usar sin tener que instalar las aplicaciones de facturación/contabilidad.



#### Mercado Pago



Mercado Pago está disponible como proveedor de pago en Latinoamérica.



#### Visualización de tokens de pago



La visualización de las fichas de pago se ha adaptado para que quepa en cualquier pantalla.



#### Razorpay



Razorpay está disponible como proveedor de pago en la India.



#### SEPA: confirmación inmediata



Los pagos de Adeudos Directos SEPA se confirman inmediatamente cuando se crea el mandato para agilizar el flujo de pagos y permitir una mejor gestión de las cancelaciones y los pagos fallidos.



#### Stripe: captura manual



Añadimos la opción de activar pagos de dos pasos para Stripe. Para esto, primero reservamos la cantidad en la tarjeta y después capturar la cantidad manualmente (por ejemplo, para reservas de hoteles).



#### Stripe: reembolsos



Los pagos en línea realizados con Stripe ahora se pueden reembolsar desde Odoo o desde el tablero de Stripe.



#### Stripe: Restricción de países



Evita que los usuarios configuren Stripe cuando no están en un país donde se admite Stripe.



#### Elimina proveedores



Elimina proveedores de pago en línea para ocultarlos de los clientes.



##  Nómina




#### Bélgica



En un puesto de trabajo de Contratación, se añadió una pestaña de Nómina para definir si el puesto de trabajo es elegible para la "Propiedad Intelectual" y la "Exención de Retención de Impuestos". Se añadieron advertencias relacionadas con el tablero de control de la nómina.



#### Escritorio



Crear un tablero para que los responsables de la nómina tengan un resumen.



#### Carpeta de documentos para exempleados



Envía un enlace a exempleados para que puedan ingresar a sus documentos.



#### Entradas desde Planificación y asistencias



Los contratos pueden tener entradas a trabajo dependiendo de Planificación, Asistencias, o un horario fijo.



#### Kenia



Añadimos un módulo de localización para Kenya.



#### Luxemburgo



Añadimos un módulo nuevo, "localización de Luxemburgo", que contiene la estructura salarial y reglas dedicadas.



#### Derechos de acceso del gerente



Añadimos un nivel de gerente a los derechos de acceso, lo que les permite leer los contratos de los miembros de su equipo.



#### Nombre del PDF de Payslip



El mes se incluye ahora en el nombre del archivo PDF generado para la nómina.



#### Configurador de salarios



Las listas desplegables ahora responden al teclado: vaya a la selección que corresponda con las teclas que está apretando.



#### Firma electrónica



Al solicitar una forma, el nombre del documento en Firma electrónica dependerá del nombre del archivo firmado.



#### Recorrido



Añadimos un recorrido para facilitar la capacitación.



##  Planificación




#### Gestión de horas flexible



Gestiona la planificación de empleados con horas flexibles.



#### Función de los recursos materiales



Define una función para recursos materiales.



#### Abrir informes



Todos los usuarios de planificación tienen acceso al menú de informes y pueden ver la información disponible para su nivel de derechos de acceso.



#### Imprimir



Imprimir la planificación de los empleados.



##  Gestión del ciclo de vida del producto




#### Creación de lista de materiales y órdenes de cambio de ingeniería



Añadimos una manera nueva y más rápida de crear órdenes de cambio de ingeniería tipo listas de materiales.



##  Punto de venta




#### Cupones y tarjetas de lealtad



Ofrece promociones, cupones, lealtad y tarjetas de regalo a los clientes. Rastrea su uso a través del las actividades de Punto de venta, Ventas y Comercio electrónico.



#### Cartera electrónica



Introducimos la nueva cartera multicanal al Punto de venta para permitir pagar por bienes o colectar reembolsos.



#### Francia: restricción de precios



Relajación de la restricción contra la modificación de precios para Francia.



#### Ajustes generales



Gestiona las configuraciones del TPV desde los ajustes generales. Cambia entre las configuraciones del TPV como entre los sitios web. Busca fácilmente en las configuraciones limpias y reorganizadas.



#### Nomenclatura global del código de barras



La nomenclatura del código de barras ahora es global y se aplica a todos los TPV.



#### Facturas: sesión cerrada y solicitud del portal del cliente



Generación de facturas para pedidos de venta de sesiones previamente cerradas. Ahora los clientes pueden exigir las facturas ellos mismos a través del portal, con la introducción de una opción para añadir códigos QR en los recibos.



#### Modelo de lealtad



El TPV está ahora alineado con el modelo multicanal para gestionar los programas de lealtad a través de las aplicaciones de Ventas, Sitio Web y TPV.



#### Visibilidad de los márgenes y costes



Ocultar márgenes e información de costes en la interfaz del punto de venta para todos los usuarios excepto el gerente del punto de venta.



#### Confirmación automática de orden



Confirmación automática de órdenes de TPV después de un pago electrónico cuando no queda nada por pagar



#### Total de presupuestos/órdenes



Ahora se muestran los totales de presupuestos/órdenes, así podrás saber qué falta por pagar antes de cargar el presupuesto o la orden.



#### Interfaz simplificada



Varias mejoras a la usabilidad de todas las pantallas del punto de venta parta facilitar su uso.



##  Proyecto




#### Gestión de documentos



Gestiona documentos directamente desde tareas con el botón inteligente de "documentos".



#### Espacio de trabajo de documentos



Obtén el espacio de trabajo adecuado para documentos de proyecto. Esto se logra definiendo una plantilla en los productos de servicio.



#### Vista de Gantt



Ve la asignación actual de recursos con la barra de progreso en la vista Gantt.



#### Objetivos en tareas



Vincula los hitos a las tareas y marque los objetivos como alcanzados cuando las tareas estén hechas.



#### Abrir informes



Todos los usuarios de proyecto tienen acceso al menú de informes y pueden ver la información disponible para su nivel de derechos de acceso.



#### Planificación de tareas recurrentes



Ahorra tiempo en la planificación de tareas recurrentes con el cálculo automático de su fecha prevista.



#### Informes: actualización de proyecto



El panel lateral de "Project Updates" se ha renovado para hacer un mejor seguimiento de la rentabilidad de los proyectos, ofreciendo una visión general de los costes, los ingresos, el margen y el presupuesto.



#### Políticas de facturación de servicios: Objetivos y Manual



Entrega los servicios automáticamente vinculándolos a los hitos del proyecto con la política de facturación "basada en los hitos". Entregarlos manualmente con la política de facturación "basada en la cantidad entregada (manual)".



#### Programación inteligente de tareas



Programa las tareas en lote desde la vista de Gantt sin que se produzcan conflictos: las horas restantes de las tareas, el tiempo libre del usuario y su carga de trabajo se tienen en cuenta de forma automática para encontrar la mejor programación posible.



#### Plantillas SMS en etapas



Envía SMS automáticamente cuando una tarea llega a una etapa en específico.



#### Gestión de etiquetas



Etiquetas relevantes para tareas de proyecto: solo se muestran las etiquetas que ya están presentes en un proyecto en el autocompletado de etiquetas.



##  Compra




#### Comparación de los precios en las facturas



El coste se corrige cuando el precio de la factura es diferente al precio del pedido de compra.



#### Licitaciones



Licitaciones renovadas. Crea solicitudes de ofertas alternativas y compárelas fácilmente, de forma global o línea por línea. Ya no es necesario crear un contrato de compra dedicado.



#### Historial de compras



Ve el historial de compras de un producto en las solicitudes de presupuesto, ayudando a seguir su precio.



#### Estado de recepción



Se añadió el estado de la recepción en el pedido de compra. Este estado permite que se pueda ver de inmediato si los productos se recibieron, se entregaron parcialmente, o si no se entregaron. También permite ver si se entregaron a tiempo.



##  Calidad




#### Operaciones: revisión de calidad



Crear revisiones de calidad en las operaciones.



##  Reclutamiento




#### Habilidades en candidato



Establece las habilidades de un candidato; se transferirán al empleado si se crea.



#### Candidatos duplicados



Al añadir o editar la dirección de correo electrónico de un formulario de solicitud, la búsqueda de una dirección existente ahora no distingue entre mayúsculas y minúsculas.



#### Digitalización de CV



Se puede digitalizar los CV para que automáticamente se extraiga el nombre, correo y teléfono. Acceso a las previsualizaciones de un CV directamente desde la aplicacón.



#### Vistas mejoradas



Se ha renovado el tablero de mandos para hacerlo más intuitivo, además de varias mejoras en diversas vistas.



#### Acceso de entrevistador



Añadir usuarios como "entrevistadores" a una descripción de empleo o al registro de un candidato para que puedan ingresar a los archivos del candidato sin tener todos los derechos de acceso a la aplicación de Reclutamiento.



#### Solicitud de empleo



Se mejoraron tanto la UX como las posibilidades de solicitar un empleo. Se han añadido nuevos campos y nuevos elementos visuales a la página de solicitud de empleo.



#### Nuevos informes



Añadimos dos informes nuevos: 'Análisis de la fuente" y "Análisis del tiempo en etapa".



#### Solicitudes de firma



Envía documentos para que los candidatos los firmen.



#### Informe del rendimiento del equipo



Añadimos un nuevo informe sobre el rendimiento del equipo.



##  Referencias




#### Imagen de fonto



Cambiar la imagen de fondo de la aplicación de referencias desde los ajustes de la aplicación.



##  Alquiler




#### Programar: cantidades



Observa la cantidad que tiene de cada producto almacenable en la vista de calendario.



##  Ventas




#### Capacitación sobre la cuenta de Amazon



Vincular una cuenta de Amazon con Seller Central solo toma unos segundos ahora.



#### Amazon Marketplaces



El conector de Amazon ahora es compatible con todos los Marketplaces, incluidos los nuevos.



#### Confirmación de pedido de venta cancelada



Al cancelar un pedido de venta ya enviada a un cliente, el usuario tiene que confirmar la cancelación. También existe la opción de enviar un correo electrónico al cliente.



#### Pedido de venta cancelado/Pago de facturas



Evita que los usuarios paguen un pedido de venta o una factura cancelada a través de un enlace de pago.



#### Cupones, tarjetas de lealtad y cartera electrónica



Ofrece promociones, cupones, lealtad y tarjetas de regalo a los clientes. Rastrea su uso a través del las actividades de Punto de venta, Ventas y Comercio electrónico.



#### Notificación del envío



Se añadió el estado del envío en el pedido de venta. Este estado permite que se pueda ver de inmediato si los productos se entregaron, se entregaron parcialmente, o si no se entregaron. También permite ver si se entregaron a tiempo.



#### Legibilidad de los anticipos



Se mejoró la experiencia de usuario cuando se vincula una factura de anticipo a un pedido de venta.



#### Lista de precio de los contactos



Al cambiar la lista de precios de un contacto se muestra una advertencia si hay pedidos de venta abiertos, incluyendo la cesta de la compra del sitio web.



##  Conectores de envío




#### DHL y UPS: facturas comerciales



Añadimos soporte para facturas comerciales para conectores DHL y UPS. Añadimos un país de origen en el formulario de producto (necesario para facturas comerciales).



#### Colectar puntos FedEx



Seleccionar un punto de recolección de FedEx desde el comercio electrónico.



#### Asegurar: UPS, FedEx, DHL y EasyPost



Asegura paquetes que se envíen por UPS, DHL, FedEx y EasyPost. Define el porcentaje del valor total del envío que se debe asegurar.



#### Sendcloud



Nuevo conector de envíos con Sendcloud. Sendcloud es un agregador de los transportistas más populares de Europa occidental. Soporta, entre otros, DPD, Mondial Relay, GLS, PostNL, etc.



#### Métodos de envío



En los métodos de envío, el rango de códigos postales fue reemplazado por una selección de múltiples prefijos de códigos postales. Se ha añadido la opción de "Recogida en tienda". Ahora está disponible el campo de descripción adicional para el método de envío.



#### Límite de peso



Añadimos límite de peso para limitar los métodos de envío disponibles para elegir dependiendo del peso total.



##  Firma electónica




#### Auto por defecto



La firma automática se sugiere por defecto cuando un usuario jamás ha puesto una firma (en lugar de Dibujar).



#### Créditos de autenticación extra



Permita que los firmantes envíen su firma incluso si los créditos de compras dentro de la aplicación (por ejemplo, para la autenticación de SMS) se han agotado. Además advierta a los usuarios sobre créditos bajos o faltantes a través de un correo electrónico o en la interfaz de la aplicación de Firma electrónica.



#### Integración con itsme®



Los firmantes podrán indicar su identidad con el servicio de identificación itsme® (solo para Bélgica y Países Bajos).



#### Diseño móvil



Se mejoró la experiencia al firmar en dispositivos móviles.



#### Multiempresa



Se mejoró la compatibilidad con entornos multiempresa: ahora todas las solicitudes de firma salientes proceden de la empresa correcta, incluso si el usuario remitente cambia de empresa.



#### Volver a asignar a un signatario



Se puede cambiar el signatario de un documento después de mandarlo.



#### Rechazos



Ahora los destinatarios se pueden negar a firmar un documento.



#### Respuestas a solicitudes de firma



Las respuestas por correo electrónico a las solicitudes de firma se envían al correo electrónico del solicitante.



#### Informes



Nuevo informe que calcula el ahorro ecológico realizado (papel, agua, madera, carbono, residuos, energía) utilizando la firma electrónica en lugar de la firma en tinta.



#### Compartir documento



Se mejoró el comportamiento al compartir documentos



#### Orden de firma



Envía solicitudes de firmas en una orden específica, así aseguras que los firmantes solo reciban una notificación si es su turno en el documento.



#### Marco de firma



Se añadió un marco visual a las firmas. Así las copias en papel tienen una indicación visual de que el documento tiene una firma electrónica.



#### Mejoras a la interfaz del usuario



Podrás ingresar con mayor facilidad a documentos que todavía debes formar y documentos en enviaste. Todo gracias a los nuevos elementos del menú.



##  Marketing social




#### Propietarios de la cuenta



Observa quién vinculó cuentas y creó flujos.



#### Prospectos



Cree leads desde cualquier publicación social o comentario.



#### Twitter



Haz retweet y cita tweets desde el tablero. Ve cuántos caracteres quedan al escribir un Tweet.



##  Hoja de cálculo




#### Código abierto



La librería Spreadsheet ha sido de código abierto en LGPL.



#### Conversión de los archivos XLSX



Convierte un archivo XLSX a hoja de cálculo de Odoio para editarlo.



#### Congelar Paneles



Congela filas y columnas para una mejor legibilidad.



#### Registros en list()



Observa los registros en la función list().



#### Atajo de teclado para referencias de celda absolutas



Actualiza las referencias absolutas de las celdas utilizando el atajo de teclado F4.



#### Funciones de contabilidad



Añadimos un nuevo conjunto de funciones para obtener medidas de contabilidad (por ejemplo, balance, crédito y débito).



#### Eliminación de grupos de hojas autoexpandibles



Se eliminaron los grupos de hojas autoexpandibles.



#### Formato de divisa automático



El formato de divisa se aplica automáticamente a los valores monetarios en los pivotes y las listas.



#### Gráficas



El color automático de la fuente ahora se elige dependiendo el color del fondo de la tabla para maximizar la legibilidad. Añadimos un filtro global para tablas. Ahora se omiten las filas vacías al trazar las tablas.



#### Menú de contexto de gráficas/figuras



Al hacer clic con el botón derecho del ratón en un gráfico/figura se abre el menú contextual.



#### Borrar formato



Añadimos una opción de "eliminar formato" en el menú de Formato.



#### Selector de color



Se mejoraron los colores automáticos del selector de color y se añadió la posibilidad de crear colores personalizados.



#### Prioridad de las reglas del formato condicional



Gestiona la prioridad de las reglas de formato condicionales.



#### Copiar, cortar, pegar tablas



Corte, copie y pegue las tablas para moverlas a través de las hojas.



#### Atajo ctrl+A



Selecciona el rango actual con ctrl + A



#### Función de conversión de divisas



Se implementó una función de conversión de divisa ('currency.rate()') según el modelo res.currency.rate.



#### Formatos de divisa personalizados



Creación de formatos de divisa personalizados.



#### Modo de tablero



Se introdujo un modo de solo lectura para usar hojas de cálculo como tableros



#### Filtros de Datos



Establece filtros en un rango de celdas.



#### Introducción de la fuente de datos



Cuando se utiliza el botón Insertar en la hoja de cálculo, se muestran vistas previas de las hojas de cálculo existentes para proporcionar más contexto.



#### Desplazamiento del filtro de fecha



Coincidir los datos con los filtros de fecha con un desplazamiento.



#### Formato de fecha



Ahora todas las fechas se almacenan como números enteros.



#### Filtros globales de fecha



Selecciona cualquier año como filtro global.



#### Filtro de fecha automático



Se mejoró el filtro de fecha automática para seleccionar un periodo relativo al actual.



#### Borrar listas/tablas dinámicas



Quitar una tabla dinámica o una lista de una hoja de cálculo.



#### Formato de fórmula dinámica



Se ha introducido la gestión dinámica del formato: crear fórmulas cuyo formato del resultado final depende de las entradas de la fórmula.



#### Atajos de teclado para contenido final



Para ir al final del contenido en cualquier dirección solo se debe presionar los atajos de teclado de "ctrl + Tecla de flecha.



#### Funciones financieras



Se añadieron más de 30 funciones financieras.



#### Legibilidad de los paréntesis/cadenas de fórmulas



Se aumentó el contraste del color de las fórmulas destacadas para mejorar la legibilidad.



#### Rango completo de columna/fila



Añadimos asistencia al rango completo de filas y columnas.



#### Diagrama Gauge



Se ha añadido un tipo de gráfico de indicadores para mostrar los KPI en el tablero.



#### Filtros globales para encabezado dinámico



Configura filtros globales desde celdas de encabezado dinámico.



#### Ocultar hoja



Añadimos la posibilidad de ocultar (y mostrar) hojas.



#### Insertar gráficas



Se puede insertar gráficas en la hoja de cálculo con el botón de "insertar en hoja de cálculo".



#### Formato de números largo



Añadimos un nuevo formato para mostrar números grandes en modo corto, por ejemplo, 100,000 ahora se acorta a 100k.



#### Leyenda: opción "ninguno"



Quitar leyendas de la tabla con la opción "ninguna".



#### Vincular tablas a menús de Odoo



Vincula cualquier tabla a un menú de Odoo para poder ingresar a él desde la tabla.



#### Enlace emergente al pasar el ratón por encima



La ventana emergente del enlace ahora se abre al pasar el ratón por encima.



#### Funciones NA(), ISERR() y ISNA()



Se implementaron las funciones NA(), ISERR() y ISNA() para asistencia de XLXS.



#### Prefijo de las funciones de Odoo



Las funciones de Odoo ahora tienen el prefijo "ODOO" para distinguirlas y recuperarlas con facilidad.



#### Manejo del símbolo de porcentaje



Se mejoró el rendimiento del símbolo de porcentaje (%) en las fórmulas.



#### Edición del dominio dinámico



Edita el dominio de una tabla dinámica desde la ventada de propiedades de la tabla.



#### Clasificación de inserción dinámica



Al insertar una tabla dinámica, la ordenación utilizada en la vista de la tabla dinámica se conserva ahora en la hoja de cálculo.



#### Nombres dinámicos



Nombre y renombre tablas dinámicas



#### Función de posición dinámica



Se ha ampliado el soporte de la función de tabla dinámica a todas las hojas de cálculo y no sólo a las plantillas.



#### Información contextual dinámica/en lista.



Al insertarse, se brinda más información contextual en el nombre de la tabla dinámica o lista.



#### Filtro de fecha relativa



Añadimos un nuevo intervalo de tiempo de "periodo relativo" (los últimos x días) a la configuración del filtro de fechas.



#### Formato de fecha de devolución



Cuando la función de encabezado dinámico regresa una fecha como día, su valor numerico se regresa y el formato de la fecha del día se configura en él.



#### Escala tablas en series de tiempo/números



Cuando las etiquetas de una tabla tienen formato de fecha o números, el tabla muestra los datos como Series Temporales y escala en consecuencia.



#### Tabla de puntuación



Se ha añadido un tipo de gráfico de tabla de puntuación para mostrar los principales KPI en los tableros.



##  Studio




#### Flujos de valoración



Integra los flujos de aprobación personalizados con las Actividades siguientes para que los usuarios puedan encontrar fácilmente los registros en los que se requiere su aprobación. Los flujos de aprobación ahora tienen un usuario responsable específico al que se le asigna una Actividad cuando un usuario solicita una aprobación.



##  Suscripciones




#### Objeto de pedido de venta



Los pedidos de suscripción se basan ahora en el objeto Pedido de Venta para beneficiarse de todas sus funcionalidades: renovar, hacer venta adicional, comprobar el MRR, etc., directamente desde el pedido de venta.



##  Encuesta




#### Asistente de acción



El ayudante de acción, que aparece cuando no se encuentran encuestas, ofrece ahora cargar una de las tres plantillas de encuestas para empezar a probar la aplicación de inmediato.



#### Fondos



Personaliza encuestas y sesiones en vivo con imágenes de fondo.



#### Propagación de reglas de visualización condicional



Al copiar una encuesta, sus reglas de visualización condicional también se propagan a la copia.



#### Zoom en imagen



Haz zoom en imágenes al realizar una encuesta.



#### Vista de kanban



Se mejoró el diseño de la vista kanban para brindar un resumen mejor y poder acceder automáticamente a cualquier resultado.



#### Sesiones en vivo



Los mensajes finales ahora se muestran en las sesiones en vivo.



#### Diseño de la pregunta



La disposición de las preguntas se decide mediante reglas para que el formulario sea lo más ordenado posible.



#### Colaboración de los usuarios



Revisa los resultados de los participantes y colabore en esos documentos.



##  Ausencia




#### Planes de acumulación: transferencias



Seleccionar cuántos días se deben asignar al final del año en la planificación de asignación.



#### Aprobaciones



El menú de aprobaciones se muestra automáticamente cuando los usuarios tienen solicitudes de tiempo personal o asignaciones por aprobar, sin la necesidad de nuevos derechos de acceso.



#### Avatares



Cuando se crea una ausencia para más de un empleado a la vez en "todo las ausencias", se muestran sus avatares.



#### Cancelación



Los usuarios pueden eliminar su propio tiempo libre aprobado y rechazado, siempre que no hayan comenzado todavía.



#### Escritorio



Borra de la vista de tablero las solicitudes de tiempo personal que no se han aprobado.



#### Tiempo personal automático



Al crear una nueva solicitud de ausencia, la selección del tipo de ausencia por defecto se basa ahora en el orden de los tipos de ausencia en la configuración.



#### Filtro de departamentos



El filtro de departamentos en el resumen de tiempo personal ahora muestra a los miembros de los subdepartamentos.



#### Día libre público



Las solicitudes de ausencia se cancelarán y se concederán de nuevo si se crea un día libre público en ese día.



#### Iniciar recorrido



Añadimos Empezar recorrido



#### Días de estrés



Se podrán crear "días de estrés" donde los usuarios no podrán pedir permisos.



#### Días de estrés por departamento



Los días de estrés se pueden vincular a un departamento.



#### Tipos de ausencias



Añadimos filtros automáticos para solo mostrar registros importantes al usar los botones inteligentes de "tiempo personal" y "asignación" en la pantalla de configuración de "tipos de tiempo personal".



#### Validación



Se mejoró el mensaje de error que aparece cuando dos periodos de tiempo personal se sobreponen.



#### Fecha de validez



Desde el tablero, se podrá ver la fecha de validez si tiene asignaciones de tiempo libre disponibles.



##  Partes de horas




#### Bloque de ausencia



Ahora se puede bloquear a los empleados para que no creen o modifiquen partes de horas que estén vinculadas al tiempo personal.



#### Vista de rejilla



Ve las horas restantes de los proyectos, las tareas y los artículos de pedidos de venta prepagados en la vista de retícula.



#### Fechas bloqueadas



Se puede bloquear la codificación de los partes de horas de un usuario según la fecha de su parte de horas que se validó más recientemente.



#### Abrir informes



Todos los usuarios de partes de horas tienen acceso al menú de informes y pueden ver la información disponible para su nivel de derechos de acceso.



#### Horas extra: contratos



Los contratos ahora se tienen en cuenta al calcular tiempo extra.



#### Informes de pedidos de venta/facturas/tickets



Obtén informes de entradas de los partes de horas desde los pedidos de venta, facturas y tickets del servicio de asistencia.



#### Tickets



Registra tiempo en tickets del servicio de asistencia desde la aplicación de partes de horas.



##  Sitio web




#### Herramienta de análisis: Plausible.io



Nuevo tablero analítico que funciona de inmediato para usuarios SaaS.



#### Vista del Backend



Podrás ver productos, eventos y citas desde el backend solo tras hacer clic en un nuevo botón.



#### Controlador como página de inicio



Define la página de inicio del sitio web en los ajustes: elige una página dinámica, como eventos o tienda.



#### Consentimiento de cookies



Pídale permiso a los visitantes antes de guardar cookies no esenciales en sus sitios web.



#### Eventos dinámicos



Muestra los eventos en cualquier lugar de un sitio web con el bloque de creación de eventos dinámicos.



#### Formas GIF



Ahora se pueden utilizar formas con imágenes GIF.



#### Diseño de rejilla



Se podrá activar el modo de rejilla en un bloque de creación para tener control total de sus elementos.



#### Servicio de asistencia



Añadir una página de servicio de asistencia al crear un sitio web.



#### Hipervínculos en las imágenes



Se arregló la adición de hipervínculos a imágenes.



#### Formas de imágenes: dispositivos



Utiliza la nueva categoría de formas de imagen llamada "Dispositivos" para hacer que las imágenes parezcan estar en un dispositivo (teléfono, tableta, ordenador).



#### Icono de chat en directo



La pestaña de chat en directo se reemplazó con un icono flotante.



#### Transparencia del mega menú



El mega menú ahora puede tener un color de fondo transparente.



#### Edición de la vista móvil



Edita sitios web desde la vista móvil



#### Código de teléfono del país prestablecido



El código de teléfono del país ahora se preestablece utilizando la información GeoIP en el formulario de contacto del sitio web.



#### Propagar idioma



Se puede optener el idioma de un cliente en las aplicaciones de CRM, servicio de asistencia o en el calendario (para citas en línea) dependiendo de la versión lingüística que usaron del sitio web al enviar el formulario.



#### Animaciones de desplazamiento



Añadimos animación de desplazamiento.



#### Ajustes



Limpiamos los ajustes del sitio web y añadimos una nueva comparación de precios de los productos.



#### Restructuración del menú del sitio



Observa todas las páginas y objetos del sitio web de inmediato con el menú del sitio



#### Bloque de construcción de redes sociales



Añadimos un bloque de creación de red social para gestionar todos los enlaces de las redes sociales con facilidad.



#### Fragmento de etapas



Es posible cambiar el estilo de los conectores en los bloques de creación Step.



#### Visibilidad: móvil o ordenador



Se puede seleccionar si un bloque de creación debe ser visible solo en el móvil, el ordenador o en ambos.



#### Menú del sitio web



Los menús del backend y del frontend se fusionaron para navegar entre ambos sin problemas.
