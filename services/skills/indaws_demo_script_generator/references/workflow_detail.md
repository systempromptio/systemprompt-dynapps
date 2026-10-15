# Demo Script Generator - Workflow Detail

## Paso 0: Recopilacion de Datos (CRITICO)

Antes de hacer cualquier cosa, **SIEMPRE** recopila la siguiente informacion:

1. **Credenciales Odoo corporativo** (si no estan configuradas):
   - URL, base de datos, API Key, usuario

2. **Identificador de la oportunidad**: ID numerico o nombre de la oportunidad en el CRM.

3. **Duracion de la demo**: 30, 60 o 90 minutos. Default: 60 minutos.

4. **Fecha y hora de la demo** (opcional): Para incluir en la agenda del cliente.

5. **Instancia demo** (opcional): URL, DB, API Key de la instancia donde estan los datos demo.
   - Si no se proporciona, el guion se genera con datos de ejemplo y se recomienda preparar datos.

6. **Contexto adicional** (opcional): Informacion que el consultor quiera anadir (ej. "el cliente ya usa SAP", "interesados sobre todo en inventario").

**DETENTE** y espera la respuesta del usuario antes de continuar.

---

## Paso 1: Extraccion de Datos

Ejecuta el script de extraccion para obtener los datos de la oportunidad:

```bash
# Configurar credenciales corporativas
export ODOO_URL="<url>"
export ODOO_DB="<db>"
export ODOO_KEY="<api_key>"
export ODOO_USER="<usuario>"

# Extraer datos de la oportunidad (por ID)
./scripts/fetch_opportunity_data.sh <lead_id> --pilot-dir /path/to/odoo-pilot/scripts > /tmp/dsg_opportunity.json

# O por nombre (busqueda ilike):
./scripts/fetch_opportunity_data.sh "Nombre del cliente" --pilot-dir /path/to/odoo-pilot/scripts > /tmp/dsg_opportunity.json

# Con instancia demo (opcional):
./scripts/fetch_opportunity_data.sh <lead_id> --pilot-dir /path/to/odoo-pilot/scripts \
  --demo-url "https://demo.example.com" --demo-db "demo_db" --demo-key "api_key" --demo-user "admin" \
  > /tmp/dsg_opportunity.json
```

**El script obtiene**:
- `opportunity`: Datos del crm.lead (name, description, expected_revenue, stage_id, tags, **xx_data_collection**)
- `partner`: Datos del contacto (name, industry, website, city, country)
- `tags`: Nombres de los tags del lead
- `chatter_notes`: Ultimas 10 notas/emails del lead
- `previous_scripts`: Guiones anteriores detectados en el chatter (fecha y asunto de cada nota titulada "Guion de demo"). Solo mira las ultimas 10 notas, asi que el conteo es orientativo
- `toma_de_datos`: Contenido del campo xx_data_collection (informacion de la toma de datos con el cliente)
- `attachments`: Lista de archivos adjuntos en la oportunidad (id, name, mimetype, file_size)
- `demo_instance`: Disponibilidad y conteo de datos demo

**Verificaciones post-extraccion**:

1. **Si `summary.description_empty` es true**: AVISAR al consultor. La descripcion del lead esta vacia, se trabajara con tags, sector, toma de datos y contexto proporcionado.

2. **Si `summary.has_previous_scripts` es true**: INFORMAR al consultor (no detenerse): "Esta
   oportunidad ya tiene N guion(es) de demo previos: <fechas>. El guion nuevo se anadira como una
   nota y un adjunto adicionales; los anteriores se conservan." No hay sobrescritura posible: el
   historico se acumula.

3. **Si `summary.has_toma_de_datos` es true**: Informar al consultor que se ha encontrado informacion de la toma de datos y se usara como fuente principal para personalizar el guion.

4. **Si `summary.has_relevant_attachments` es true**: **OBLIGATORIO** -- Revisar los adjuntos de la oportunidad. Los adjuntos frecuentemente contienen informacion critica del cliente (presentaciones con necesidades, propuestas anteriores, documentos de requerimientos). Procedimiento:
   - Listar los adjuntos relevantes (PDFs, documentos, presentaciones) al consultor
   - Descargar y leer los PDFs/documentos relevantes usando odoo-pilot (`/web/content/<attachment_id>`)
   - Extraer informacion clave: necesidades del cliente, estructura de empresa, requerimientos funcionales, numeros de usuarios, entidades, modulos solicitados
   - Incorporar esta informacion como fuente de datos para el analisis (misma prioridad que `xx_data_collection`)

   **Tipos de adjuntos a revisar** (por mimetype):
   - `application/pdf` -- Presentaciones, propuestas, documentos de requerimientos
   - `application/vnd.openxmlformats*` -- Word, Excel, PowerPoint
   - `application/vnd.ms-*` -- Word, Excel, PowerPoint (formato legacy)

   **NUNCA ignorar adjuntos** -- pueden contener la informacion mas detallada y precisa sobre las necesidades del cliente, superior incluso a la descripcion del lead o la toma de datos.

---

## Paso 2: Analisis y Seleccion de Flujos (ROL DEL LLM)

**Este paso es completamente del LLM.** Lee y analiza:

1. Lee `/tmp/dsg_opportunity.json`
2. Lee `./references/pain_to_demo_mapping.md`
3. Lee `./references/flujos_demo_por_app.md`
4. Lee `./references/demo_best_practices.md`

**Tareas del LLM**:

a) **Identificar sector**: A partir de `partner.industry_id`, tags, description, toma de datos y adjuntos relevantes. Usar tabla de deteccion de sector por keywords.

b) **Extraer pain points**: De `opportunity.description`, `chatter_notes`, `attachments` (PDFs descargados) y especialmente de `toma_de_datos` (PRIORIDAD). Si la toma de datos existe, es la fuente MAS RICA de informacion sobre necesidades del cliente. Los **adjuntos** (PDFs, documentos) pueden contener informacion incluso mas detallada que la descripcion -- revisar SIEMPRE los adjuntos relevantes antes de seleccionar flujos.

c) **Mapear a flujos**: Usar tabla pain -> modulo -> flujo para seleccionar 3-5 flujos segun:
   - Duracion disponible (30 min -> max 2 flujos, 60 min -> 3-4, 90 min -> 4-5)
   - Prioridad de pain points
   - Flujos combinados si aplica (Lead-to-Invoice, Procure-to-Pay, etc.)

d) **Seleccionar momentos WOW**: Uno principal por flujo, priorizando los de mayor impacto visual.

e) **Evaluar datos demo**: Si `demo_instance.available`:
   - Verificar que hay datos suficientes para cada flujo seleccionado
   - Si faltan datos: Proponer crearlos (via `demo-builder` o manualmente via API)
   - Si no hay demo: El guion incluira datos de ejemplo y recomendacion de preparar datos

f) **WARN Feature Overload**: Si se detectan 6+ apps relevantes, AVISAR al consultor y sugerir dividir en 2 sesiones.

---

## Paso 2.5: Validacion del Plan de Demo (OBLIGATORIO)

**DETENTE** y presenta al usuario:

1. **Contexto del cliente**: Nombre, sector detectado, pain points extraidos
2. **Fuentes de informacion utilizadas**: Descripcion, toma de datos, adjuntos revisados, chatter, tags (indicar cuales tenian contenido)
3. **Flujos seleccionados** (tabla):

| # | Flujo | App | Pain que resuelve | Momento WOW | Duracion |
|---|-------|-----|-------------------|-------------|----------|
| 1 | ... | ... | ... | ... | X min |

4. **Estructura temporal**: Apertura (X min) -> Bloque 1 -> ... -> Q&A (X min) -> Cierre (X min)
5. **Estado datos demo**: OK Disponibles / WARN Faltan datos / ERROR Sin demo

**Espera la respuesta del usuario**:
- **Aprobar**: Continua con Paso 3
- **Modificar**: Ajustar flujos, orden, duracion o anadir/quitar flujos
- **Cancelar**: Terminar sin generar guion

---

## Paso 3: Generacion del Guion Interno (ROL DEL LLM)

1. Lee el template `./templates/guion_demo.md`
2. Lee `./references/demo_best_practices.md` para framework F.A.V.O.R.I.T.E. y framing

**Genera el guion Markdown** rellenando el template con:

- Contexto del cliente personalizado (datos reales, no genericos)
- Pain points extraidos (priorizando los de la toma de datos)
- Estructura temporal adaptada a la duracion
- Preguntas de apertura relevantes al sector y pain points
- N bloques de demo con:
  - Frame before/after personalizado con datos del cliente
  - Pasos detallados con talking points en segunda persona ("Ustedes van a poder...")
  - Momento WOW destacado
  - Datos necesarios con estado (OK existe en demo / WARN hay que crearlo)
  - Precauciones especificas
  - Transicion narrativa al siguiente bloque
- Preguntas esperadas del cliente (basadas en sector y pain points)
- Resumen de valor y proximos pasos
- Notas tecnicas para el consultor

**Reglas de generacion**:
- Talking points SIEMPRE en segunda persona ("ustedes", "su equipo")
- NUNCA jerga tecnica en talking points (no "many2one", no "ORM", no "API")
- Transiciones narrativas entre bloques
- Si la toma de datos menciona algo especifico, incluirlo como talking point

---

## Paso 4: Generacion de Agenda Cliente (ROL DEL LLM)

1. Lee el template `./templates/agenda_cliente_html.md`

**Genera el HTML de la agenda** con:

a) **Parrafo introductorio**: Personalizado con nombre del cliente, mencionando que se mostrara como la solucion puede ayudar con {pain_principal}. Lenguaje de negocio, NO tecnico.

b) **Items de agenda** (3-5): Cada item describe un RESULTADO de negocio, no una funcionalidad.
   - MAL: "Modulo CRM y pipeline de ventas"
   - BIEN: "Gestion comercial: como tener visibilidad completa de su cartera de oportunidades"

c) **Parrafo de cierre**: Invitacion a preparar preguntas, tono cercano y profesional.

**Reglas HTML**:
- Estilos inline obligatorios (Odoo no procesa CSS externo)
- Color principal: `#875A7B` (violeta Odoo)
- NO mencionar "Odoo", "ERP", "modulos" -> usar "la solucion", "la plataforma"
- Tablas con bordes y padding
- Ultimo item siempre: "Preguntas y proximos pasos"

---

## Paso 4.5: Validacion de Entregables (OBLIGATORIO)

**DETENTE** y presenta al usuario:

1. **Guion interno** (Markdown): Contenido completo del guion
2. **Agenda cliente** (HTML): Codigo HTML de la agenda
3. **Plan de almacenamiento** (todo sobre el lead, ningun campo del lead se escribe):
   - `mail.message` (nota interna) <- Guion convertido a HTML
   - `ir.attachment` en el lead <- El mismo guion HTML, como archivo descargable
   - `mail.message` (nota interna) <- HTML de la agenda del cliente
   - Si ya habia guiones previos, indicar que este se **anade** sin tocar los anteriores

**Espera la respuesta del usuario**:
- **Aprobar**: Continua con Paso 5
- **Modificar**: Ajustar contenido (guion, agenda o ambos) y volver a presentar
- **Cancelar**: No almacenar en Odoo, solo guardar como archivos locales

---

## Paso 5: Almacenamiento en Odoo

### 5a. Convertir Guion Markdown a HTML

El guion viaja como **HTML** tanto en la nota como en el adjunto. Convierte el guion Markdown a HTML con estilos inline basicos para legibilidad en Odoo:

- Headings con color `#875A7B`
- Tablas con bordes y padding
- Listas con margen
- Blockquotes con borde izquierdo violeta
- Codigo con fondo gris claro

**IMPORTANTE**: El HTML debe ser auto-contenido con estilos inline (Odoo no procesa CSS externo).

El HTML **empieza con el titulo** `Guion de demo — {client_name} — {demo_date}` (como `<h2>`), para
que la nota se distinga de la de la agenda y se pueda encontrar buscando en el chatter.

### 5b. Guardar el Guion: nota interna + adjunto

El guion ya no se escribe en ningun campo del lead. El campo de Studio que se usaba se retiro del
Odoo de produccion y **no tiene sustituto**. **NUNCA** crees un campo nuevo en `crm.lead`: es un
modelo compartido por los cinco paises del grupo.

**b.1 — Nota interna con el guion:**

```bash
$PILOT_DIR/create_record.sh "mail.message" '{
  "model": "crm.lead",
  "res_id": <lead_id>,
  "body": "<html_guion_content>",
  "message_type": "comment",
  "subtype_id": 2,
  "subject": "Guion de demo — {client_name} — {demo_date}"
}'
```

**b.2 — Adjunto con el mismo HTML** (para que aparezca en los archivos del lead, no solo en el
chatter). `datas` va en base64:

```bash
$PILOT_DIR/create_record.sh "ir.attachment" '{
  "name": "guion-demo-{client_name}-{demo_date}.html",
  "res_model": "crm.lead",
  "res_id": <lead_id>,
  "mimetype": "text/html",
  "datas": "<html_guion_content_base64>"
}'
```

**Cada ejecucion anade una nota y un adjunto nuevos.** No sobrescribas ni borres los de ejecuciones
anteriores: el historico de guiones del lead se acumula y queda fechado.

### 5c. Crear Nota Interna con Agenda

```bash
$PILOT_DIR/create_record.sh "mail.message" '{
  "model": "crm.lead",
  "res_id": <lead_id>,
  "body": "<html_agenda_content>",
  "message_type": "comment",
  "subtype_id": 2,
  "subject": "Agenda Demo - {client_name} - {demo_date}"
}'
```

Nota: `subtype_id: 2` corresponde a `mail.mt_note` (nota interna, no enviada al cliente).

**Presenta el resultado**: Confirmacion de registros creados/actualizados.

---

## Rules and Guidelines

### Generales

- **SIEMPRE** pide los datos obligatorios antes de empezar (Paso 0)
- **OBLIGATORIO**: Presenta el plan de demo al usuario (Paso 2.5) y espera aprobacion
- **OBLIGATORIO**: Presenta los entregables al usuario (Paso 4.5) y espera aprobacion
- **NUNCA** generes registros en Odoo sin validacion del usuario
- **NUNCA** inventes datos del cliente; si algo falta, indicalo y pregunta
- **NUNCA** generes un guion sin haber leido los datos de la oportunidad
- **SIEMPRE** prioriza la informacion de `xx_data_collection` y de los adjuntos relevantes sobre inferencias de sector
- **SIEMPRE** revisa los adjuntos de la oportunidad (PDFs, documentos, presentaciones) antes de seleccionar flujos

### Contenido del Guion (Interno)

- El guion es para el CONSULTOR, puede incluir jerga interna
- Talking points en segunda persona ("ustedes", "su equipo")
- Framework F.A.V.O.R.I.T.E. completo para momentos WOW, simplificado para el resto
- Framing before/after para cada bloque principal
- Precauciones y plan B incluidos
- Datos necesarios con estado verificado

### Contenido de la Agenda (Cliente)

- **Tono**: Profesional, espanol formal, dirigido al cliente
- **NO mencionar**: "Odoo", "ERP", "modulo", nombres tecnicos, jerga interna
- **SI usar**: "la solucion", "la plataforma", "el sistema"
- Cada item describe un RESULTADO de negocio, no una funcionalidad
- Duraciones aproximadas (redondear a 5 o 10 min)
- 3-5 items maximo (incluyendo Q&A final)

### HTML

- Estilos inline obligatorios en AMBOS HTMLs (guion y agenda)
- Color principal: `#875A7B` (violeta Odoo)
- Tablas con bordes y padding
- El HTML del guion debe ser legible en el formulario de Odoo (crm.lead)
- El HTML de la agenda debe verse bien como nota interna

### Estructura Temporal

- **30 min**: Apertura 5 min + Demo 20 min (max 2 flujos) + Cierre 5 min
- **60 min**: Apertura 10 min + Demo 40 min (3-4 flujos) + Q&A 5 min + Cierre 5 min
- **90 min**: Apertura 10 min + Demo Parte 1 35 min + Pausa 5 min + Demo Parte 2 25 min + Q&A 10 min + Cierre 5 min
- **NUNCA** mas de 5 flujos en una demo (feature overload)

### Conexion a Odoo

- Usa odoo-pilot para leer y escribir registros
- Si `create_record.sh` falla, muestra el HTML para que el usuario lo copie manualmente
- El guion no se escribe en ningun campo del lead: va como nota interna + adjunto (ver Paso 5b).
  **NUNCA** crees campos nuevos en `crm.lead` -- modelo compartido por los cinco paises del grupo
- `mail.message` con `subtype_id: 2` para nota interna (NO se envia al cliente)

---

## Validations

**Validaciones interactivas (obligatorias)**:

1. **Validacion Plan de Demo (Paso 2.5)**: Usuario aprueba flujos seleccionados, estructura temporal y estado de datos
2. **Validacion Entregables (Paso 4.5)**: Usuario aprueba guion interno y agenda cliente antes de almacenar

**Validacion condicional**:

3. **Busqueda por Nombre (Paso 1)**: Si la busqueda por nombre devuelve multiples resultados, usuario selecciona
4. **Adjuntos Relevantes (Paso 1)**: Si hay PDFs/documentos adjuntos, descargarlos y revisarlos ANTES de seleccionar flujos

**Validaciones automaticas**:

5. JSON de oportunidad extraido correctamente con campos requeridos
6. Adjuntos relevantes revisados (si existen)
7. Al menos 1 pain point identificado (de description, toma_de_datos, adjuntos, tags o contexto)
8. Flujos seleccionados no exceden el tiempo disponible
9. HTML del guion y agenda tienen estilos inline
10. Agenda no contiene terminos tecnicos ("Odoo", "ERP", "modulo", "sale.order")

---

## Success Criteria

- Se han extraido los datos de la oportunidad correctamente
- Se ha identificado el sector y pain points del cliente
- Si existe toma de datos (`xx_data_collection`), se ha utilizado como fuente principal
- Si existen adjuntos relevantes (PDFs, documentos), se han descargado, leido y utilizado como fuente de datos
- **El usuario ha validado el plan de demo** (Paso 2.5)
- El guion interno incluye: contexto, estructura temporal, bloques con F.A.V.O.R.I.T.E., momentos WOW, datos necesarios, precauciones
- La agenda cliente usa lenguaje de negocio (no tecnico) y tiene 3-5 items
- **El usuario ha validado los entregables** (Paso 4.5)
- El guion HTML se ha guardado sobre el lead **dos veces**: como nota interna (`mail.message`) y como adjunto (`ir.attachment`, `text/html`)
- No se ha escrito en ningun campo del lead ni se ha creado ningun campo nuevo en `crm.lead`
- Los guiones anteriores del lead siguen intactos: el nuevo se ha anadido, no ha reemplazado nada
- La agenda HTML se ha creado como nota interna en `mail.message`
- Los HTMLs tienen estilos inline y se visualizan correctamente en Odoo
- No se ha mostrado jerga tecnica en la agenda del cliente
