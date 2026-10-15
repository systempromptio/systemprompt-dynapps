# Estructura del Kick-Off: 20 Slides en 5 Bloques

Definicion detallada de cada slide, su proposito y contenido esperado.

---

## BLOQUE 1: CONEXION (Slides 1-4)

Objetivo: Generar confianza y presentar al equipo. El cliente debe sentir que esta en buenas manos.

### Slide 1 — Portada

- **Tipo**: Title Slide (fondo oscuro con gradiente)
- **Contenido**:
  - Logo Indaws (centrado, variante blanca)
  - Titulo: "Indaws - {Nombre Cliente}"
  - Subtitulo: "KICK-OFF — Implantacion Odoo {version}"
  - Fecha de la reunion
- **Notas**: Primera impresion. Debe ser limpia y profesional.

### Slide 2 — Agenda

- **Tipo**: Content Slide (fondo blanco)
- **Contenido**:
  - Titulo: "Agenda de hoy"
  - Lista numerada de los 5 bloques:
    1. Presentacion del equipo
    2. Alineacion del proyecto
    3. Nuestra metodologia
    4. Compromisos y fechas
    5. Proximos pasos y preguntas
  - **REGLA**: NO mostrar tiempos estimados por bloque. Solo la lista de temas.
- **Notas**: Mostrar que la reunion esta bien planificada. Respetar el tiempo del cliente.

### Slide 3 — Indaws en Cifras

- **Tipo**: Data/KPI Slide (fondo claro)
- **Contenido**:
  - Titulo: "Indaws Business Solutions"
  - **Seccion 1 — Indaws** (KPIs en formato grande):
    - +50 empleados en Espana
    - +140 referencias
    - Odoo Gold Partner
    - 10+ anos de experiencia
  - **Seccion 2 — Grupo Dynapps** (KPIs en formato grande):
    - Gold Partner en 5 paises (ES, BE, NL, FR, CH)
    - +270 profesionales
    - +850 referencias
    - Mayor partner de Odoo en el mundo
  - Breve parrafo: Quienes somos y que nos diferencia
- **Notas**: No es un pitch comercial (eso ya se hizo en la fase comercial). Es recordar brevemente por que somos la eleccion correcta. Transmitir solidez.
- **Datos**: Se pueden hardcodear (no cambian por proyecto). Actualizar periodicamente.
- **Regla**: NO incluir logos de clientes similares ni referencias de otros clientes. Solo datos de KPIs de Indaws y Dynapps.

### Slide 4 — Vuestro Equipo Indaws

- **Tipo**: Content Slide (fondo blanco)
- **Contenido**:
  - Titulo: "Vuestro equipo de proyecto"
  - Por cada miembro del equipo:
    - Foto (desde hr.employee.image_128, base64)
    - Nombre completo
    - Rol en ESTE proyecto (proporcionado por el usuario, ej: "Project Manager", "Consultora Principal", "Desarrollador")
    - Cargo en Indaws (desde hr.employee.job_title)
    - Email de contacto (desde hr.employee.work_email)
  - Indicar claramente quien es el **punto de contacto principal** del proyecto (normalmente el PM)
- **Layout**: Grid de tarjetas con foto circular + datos debajo. Maximo 6 personas por slide.
- **Regla**: NUNCA mostrar niveles de seniority (Junior, Medior, Senior) en el cargo. Usar solamente el rol funcional (ej: 'Consultor Funcional', 'Consultora Funcional').
- **Notas**: No solo presentar nombres — el rol en el proyecto es clave. El cliente debe saber a quien acudir para que.

---

## BLOQUE 2: ALINEACION DEL PROYECTO (Slides 5-9)

Objetivo: Que ambas partes esten en la misma pagina sobre que se va a hacer y que no.

### Slide 5 — Equipo del Cliente

- **Tipo**: Content Slide (fondo blanco)
- **Contenido**:
  - Titulo: "Vuestro equipo de proyecto"
  - Subtitulo: "Necesitamos definir juntos los roles clave"
  - Explicacion de los 2 roles:
    - **SPOC (Single Point of Contact)**: Interlocutor principal del proyecto. Responsable de coordinar internamente, priorizar y tomar decisiones. Toda la comunicacion formal pasa por esta persona.
    - **Key Users**: Responsables de area que participaran en las sesiones de analisis y validaran la configuracion de su modulo. Seran los referentes internos post-arranque.
  - Tabla por completar:

    | Rol | Nombre | Area | Email | Telefono |
    |-----|--------|------|-------|----------|
    | SPOC | (por definir) | | | |
    | Key User - {App 1} | (por definir) | | | |
    | Key User - {App 2} | (por definir) | | | |
    | ... | | | | |

- **Notas**: Las apps de la tabla se extraen del sale.order (lineas de implantacion). El cliente debe comprometerse a asignar personas concretas. Sin SPOC y Key Users, el proyecto no puede avanzar.
- **IMPORTANTE**: En este punto del proyecto los roles AUN NO estan definidos. Esta slide es para que el cliente los defina. La tabla se presenta vacia (con "por definir") para que sea el cliente quien asigne nombres. Es uno de los compromisos que se le pide en el kick-off.

### Slide 6 — Vision del Proyecto

- **Tipo**: Content Slide (fondo blanco, dos columnas o layout antes/despues)
- **Contenido**:
  - Titulo: "Vision del proyecto"
  - Resumen ejecutivo personalizado (2-3 parrafos basados en x_studio_resumen y x_studio_toma_de_datos):
    - Situacion actual del cliente (sistemas actuales, problemas, pain points)
    - Que va a cambiar con Odoo (vision del estado futuro)
    - Beneficios clave esperados
  - Opcionalmente: diagrama visual "Antes → Despues"
- **Fuente de datos**: `sale.order.x_studio_resumen` y `sale.order.x_studio_toma_de_datos`
- **Notas**: Esta slide conecta emocionalmente. El cliente debe verse reflejado y entusiasmarse con el resultado.

### Slide 7 — Aplicaciones Odoo Incluidas

- **Tipo**: Content Slide (fondo blanco, grid de iconos)
- **Contenido**:
  - Titulo: "Aplicaciones Odoo de tu proyecto"
  - Grid visual con las aplicaciones incluidas:
    - Icono representativo (SVG simple o cuadrado de color)
    - Nombre de la app
    - Descripcion de una linea de lo que cubre
  - Las apps se extraen de la toma de datos y del alcance del sale.order
- **Notas**: Visualmente atractivo. Que el cliente vea el ecosistema completo que va a tener. Usar colores diferentes para cada icono de app (no todos del mismo color) y tarjetas con box-shadow para dar visibilidad y relieve.

### Slide 8 — Modulos Indaws Core

- **Tipo**: Content Slide (fondo blanco)
- **Contenido**:
  - Titulo: "Modulos Indaws Core incluidos"
  - Subtitulo: "Desarrollos propios que potencian tu Odoo"
  - Lista de cada modulo CORE contratado (de sale.order.line con prefijo [CORE]):
    - Nombre del modulo
    - Descripcion breve (1-2 lineas) de que hace y por que es util para el cliente
  - Si hay muchos modulos, usar layout de tarjetas compactas
- **Notas**: Cada modulo Core debe explicarse en terminos de valor para el cliente, no tecnicos.
- **Regla**: Si no hay modulos Core en el pedido, OMITIR esta slide y renumerar.

### Slide 9 — Alcance: Incluido / No Incluido

- **Tipo**: Content Slide (fondo blanco, dos columnas)
- **Contenido**:
  - Titulo: "Alcance del proyecto"
  - Columna izquierda — **Incluido** (fondo verde suave o check marks):
    - Analisis funcional completo
    - Configuracion de las apps listadas en slide 7
    - Modulos Indaws Core de slide 8
    - Migracion de datos (segun x_studio_migracion_datos)
    - Formacion a Key Users
    - Soporte en salida a produccion
    - Soporte post-arranque (segun x_studio_mantenimiento)
  - Columna derecha — **No incluido** (fondo rojo suave o X marks):
    - Desarrollos fuera de alcance (nuevas funcionalidades no presupuestadas)
    - Migracion de historicos (salvo indicado)
    - Hardware (TPV, impresoras, lectores)
    - Formacion a usuarios finales (solo Key Users)
    - Integraciones no especificadas en el presupuesto
  - Nota al pie 1: **"El alcance se confirma al finalizar el analisis"** (en formato destacado, visible)
  - Nota al pie 2: "Cualquier funcionalidad fuera de este alcance sera evaluada y presupuestada por separado"
- **Notas**: CRITICA para evitar scope creep. Dejar muy claro desde el dia 1. Adaptar la columna "No incluido" segun el contexto del proyecto. El mensaje de que el alcance se confirma tras el analisis es fundamental porque en el kick-off aun no hay analisis funcional.

---

## BLOQUE 3: METODOLOGIA (Slides 10-13)

Objetivo: Mostrar que tenemos un proceso probado y estructurado. No improvisamos.

### Slide 10 — Nuestra Metodologia

- **Tipo**: Content Slide (fondo blanco, diagrama de flujo horizontal)
- **Contenido**:
  - Titulo: "Metodologia de implantacion Indaws"
  - Diagrama de 6 fases con iconos:
    1. **Kick-Off** (estamos aqui) — marcado visualmente como fase actual
    2. **Analisis** — Sesiones con Key Users, documento de analisis funcional
    3. **Implantacion** — Configuracion, desarrollo, migracion de datos
    4. **UAT (Validacion)** — Testing con el cliente, ajustes
    5. **Go-Live** — Salida a produccion
    6. **Soporte Post-Arranque** — Acompanamiento durante las primeras semanas
  - Cada fase con 1 linea descriptiva
  - Indicador visual "Estamos aqui" en la fase de Kick-Off
- **Notas**: Consulta `./references/metodologia_indaws.md` para el detalle de cada fase.

### Slide 11 — Fase Actual: El Analisis

- **Tipo**: Content Slide (fondo blanco)
- **Contenido**:
  - Titulo: "Proxima fase: El Analisis"
  - Subtitulo: "Entender tu negocio para disenar la mejor solucion"
  - 4-5 subfases del analisis:
    1. **Sesiones de discovery**: Reuniones con Key Users por area. Revisamos procesos, casuisticas y necesidades
    2. **Estudio interno**: El equipo Indaws analiza la informacion y disena la solucion tecnica
    3. **Documento de analisis funcional**: Entregable con el diseno completo de la implantacion
    4. **Presentacion y validacion**: Sesion con el cliente para revisar el analisis y aclarar dudas
    5. **Aprobacion y arranque**: El cliente valida el analisis, se inicia la implantacion
  - Que esperamos del cliente en esta fase:
    - Asistencia de Key Users a las sesiones (obligatorio)
    - Preparar informacion previa (formatos, listados, datos)
    - Disponibilidad para resolver dudas entre sesiones
- **Notas**: Enfatizar que la calidad del analisis depende directamente de la participacion del cliente.

### Slide 12 — Plan de Comunicacion

- **Tipo**: Content Slide (fondo blanco, tabla)
- **Contenido**:
  - Titulo: "Plan de comunicacion"
  - Subtitulo: "Como vamos a trabajar juntos"
  - Tabla de canales:

    | Canal | Frecuencia | Participantes | Objetivo |
    |-------|-----------|---------------|----------|
    | Reunion de seguimiento | Mensual | PM + SPOC | Estado del proyecto, bloqueos, decisiones. Se puede combinar con una sesion de trabajo |
    | Sesiones de analisis/implantacion | Segun calendario | Consultor + Key Users | Trabajo tecnico por area |
    | Email/Chat | Diario (si necesario) | Equipo completo | Consultas puntuales, documentacion |
    | Actualizacion semanal | Semanal (escrito) | PM → SPOC | Informe de progreso formal |
    | Comite de proyecto | Mensual (si necesario) | PM + SPOC + Sponsor | Decisiones estrategicas, escalaciones |

  - Regla de oro: "Toda decision relevante queda documentada por escrito"
- **Notas**: Adaptar frecuencias segun tamano del proyecto. Proyectos grandes pueden necesitar reuniones de seguimiento mas frecuentes.

### Slide 13 — Gestion de Cambios

- **Tipo**: Content Slide (fondo blanco, flujo visual)
- **Contenido**:
  - Titulo: "Gestion de cambios"
  - Subtitulo: "Como gestionamos las peticiones fuera de alcance"
  - Flujo visual en 4 pasos:
    1. **Solicitud**: El cliente identifica una necesidad no contemplada
    2. **Evaluacion**: Indaws analiza el impacto (horas, coste, timeline)
    3. **Propuesta**: Presupuesto adicional o reajuste de prioridades dentro de la bolsa
    4. **Aprobacion**: El cliente decide si proceder. Si aprueba, se integra en el plan
  - Mensaje clave: "Las nuevas solicitudes no bloquean el proyecto. Se evaluan y planifican sin afectar los compromisos existentes"
  - Referencia a las condiciones contractuales (sin mostrar importes)
- **Notas**: Protege a ambas partes. El cliente sabe que puede pedir cosas nuevas, pero de forma ordenada.

---

## BLOQUE 4: COMPROMISOS Y FECHAS (Slides 14-17)

Objetivo: Establecer fechas firmes, compromisos mutuos y las consecuencias de no cumplirlos. Sin ser agresivos, con total transparencia.

### Slide 14 — Timeline del Proyecto

- **Tipo**: Data Slide (fondo blanco, diagrama temporal)
- **Contenido**:
  - Titulo: "Timeline del proyecto"
  - Gantt simplificado horizontal con **3 filas** (pistas paralelas):

    **Fila 1 (principal)**:
    - Kick-Off (punto de inicio)
    - Fase de Analisis (con fecha inicio y fin estimada)
    - Fase de Implantacion (grueso del proyecto)
    - UAT Final (al final, justo antes de Go-Live)
    - Go-Live (fecha proporcionada por el usuario, resaltada en Warm Yellow)
    - Soporte Post-Arranque

    **Fila 2 (paralela, debajo)**:
    - Formacion: empieza un poco despues del inicio de Implantacion, termina en el Go-Live
    - Se muestra como barra paralela a Implantacion

    **Fila 3 (paralela, debajo)**:
    - UAT parciales: empiezan un poco despues de Formacion, terminan en el Go-Live
    - Se muestra como barra paralela a Implantacion y Formacion

    Las 3 lineas (Implantacion, Formacion, UAT parciales) convergen en el Go-Live.

  - Hitos clave marcados con puntos:
    - Entrega del analisis funcional
    - Inicio de implantacion
    - Primera demo al cliente
    - Migracion de datos
    - UAT Final
    - Go-Live
  - Barra inferior con meses del calendario
- **Fuente de datos**: Fechas del Paso 0 + milestones del sale.order si existen
- **Notas**: NO mostrar horas ni costes. Solo fases y fechas. El Go-Live debe ser muy visible. Las pistas paralelas muestran que la formacion y las UAT parciales se hacen DURANTE la implantacion, no despues.

### Slide 15 — Calendario de Sesiones de Analisis

- **Tipo**: Data Slide (fondo blanco, tabla)
- **Contenido**:
  - Titulo: "Propuesta calendario de analisis"
  - Tabla:

    | Semana | Dia | Hora | Sesion | Key Users requeridos |
    |--------|-----|------|--------|---------------------|
    | Semana 1 | {dia} | {hora} | {area/app} | Key User de {area} |
    | Semana 1 | {dia} | {hora} | {area/app} | Key User de {area} |
    | ... | | | | |

  - Nota: "Este calendario es una propuesta. Lo confirmaremos hoy mismo"
  - Las sesiones se distribuyen segun las apps del proyecto, agrupadas logicamente:
    - Sesion 1: Configuraciones generales + Ventas/CRM
    - Sesion 2: Compras + Inventario
    - Sesion 3: Facturacion + Contabilidad
    - Sesion 4: Apps especificas (TPV, Fabricacion, etc.)
    - Sesion 5: Integraciones (si aplica)
    - Sesion final: Revision y dudas
  - Frecuencia sugerida: 2-3 sesiones por semana, duracion 1.5-2h cada una
- **Notas**: Adaptar segun tamano del proyecto. Es la propuesta, no inamovible. Pero dejar claro que la frecuencia es necesaria para cumplir plazos.

### Slide 16 — Reglas del Juego: Compromisos Mutuos

- **Tipo**: Content Slide (fondo blanco, dos columnas)
- **Contenido**:
  - Titulo: "Reglas del juego"
  - Subtitulo: "El exito del proyecto depende de ambos"
  - **Columna izquierda — Indaws se compromete a:**
    - Cumplir las fechas acordadas en el plan de proyecto
    - Entregar documentacion de calidad en cada fase
    - Responder consultas en un maximo de 48h laborables
    - Informar proactivamente de cualquier riesgo o desviacion
    - Asignar un equipo dedicado y estable durante todo el proyecto
    - Proporcionar actualizaciones semanales del estado del proyecto
  - **Columna derecha — {Nombre cliente} se compromete a:**
    - Asignar un SPOC con disponibilidad y poder de decision
    - Garantizar la asistencia de Key Users a las sesiones planificadas
    - Responder solicitudes de informacion en un maximo de **5 dias laborables**
    - Validar entregables en un maximo de **5 dias laborables** desde su entrega
    - Proporcionar acceso a sistemas, datos y formatos cuando se soliciten
    - Escalar internamente si hay bloqueos que impidan el avance
  - Mensaje al pie: "Estos compromisos son la base para cumplir el timeline acordado"
- **Notas**: Consulta `./references/reglas_compromisos.md` para el detalle completo. Los plazos de respuesta (5 dias) son orientativos, ajustar segun proyecto.

### Slide 17 — Impacto de Retrasos

- **Tipo**: Content Slide (fondo blanco, diagrama causa-efecto)
- **Contenido**:
  - Titulo: "Transparencia: el impacto de los retrasos"
  - Diagrama visual mostrando la cadena de consecuencias:
    ```
    Solicitud sin respuesta (+1 semana)
         ↓
    Sesion de analisis no se puede realizar
         ↓
    Documento de analisis se retrasa
         ↓
    Inicio de implantacion se desplaza
         ↓
    Go-Live se desplaza
    ```
  - Mensaje equilibrado: "No se trata de culpar a nadie. Es importante entender que los plazos del proyecto dependen de la velocidad de respuesta de ambas partes"
  - Punto adicional: "Si hay retrasos prolongados, puede ser necesario replanificar el proyecto, lo que podria implicar reasignacion de recursos y ajustes en el calendario"
  - Tono: Transparente y profesional, nunca amenazante
- **Notas**: Esta slide genera respeto por las fechas sin crear hostilidad. Es una de las slides mas importantes.

---

## BLOQUE 5: PROXIMOS PASOS Y CIERRE (Slides 18-20)

Objetivo: Que el cliente sepa exactamente que tiene que hacer y cuando. Cerrar con energia positiva.

### Slide 18 — Client Requests: Lo que Necesitamos YA

- **Tipo**: Content Slide (fondo blanco, checklist visual)
- **Contenido**:
  - Titulo: "Lo que necesitamos de vosotros"
  - Lista de acciones concretas con fecha limite:

    | # | Accion | Responsable | Fecha limite |
    |---|--------|-------------|-------------|
    | 1 | Confirmar SPOC del proyecto con nombre y datos de contacto | Cliente | +3 dias |
    | 2 | Confirmar Key Users por area (nombre, cargo, email) | Cliente/SPOC | +5 dias |
    | 3 | Enviar formatos actuales (facturas, albaranes, pedidos, informes) | Key Users | +7 dias |
    | 4 | Proporcionar acceso a sistemas actuales (si aplica integracion) | IT/SPOC | +7 dias |
    | 5 | Preparar listados de datos a migrar (productos, clientes, proveedores, plan contable) | Key Users | +10 dias |
    | 6 | Confirmar disponibilidad para sesiones de analisis | SPOC | +3 dias |

  - Las fechas limite se calculan desde la fecha del kick-off y se muestran como **fecha directa** (ej: "14 de marzo de 2026"), NUNCA como texto relativo ("manana", "la semana que viene", "en 3 dias")
  - Adaptar la lista segun el proyecto (ej: si hay integracion con sistema externo, pedir acceso API)
  - Personalizar con datos de x_studio_migracion_datos y x_studio_toma_de_datos
- **Notas**: Ser muy concretos. No dejar nada ambiguo. Cada item tiene un responsable y una fecha.
- **REGLA**: NUNCA usar expresiones temporales relativas como "manana", "la semana que viene", "pasado manana", "en X dias". Siempre poner la fecha concreta calculada (DD de Mes de YYYY).

### Slide 19 — Proximos Pasos

- **Tipo**: Content Slide (fondo blanco)
- **Contenido**:
  - Titulo: "Proximos pasos"
  - 4-5 acciones inmediatas con responsable y **fecha concreta** (DD de Mes):
    1. Indaws envia acta del kick-off con resumen de acuerdos — (PM, {fecha_kickoff + 1 dia})
    2. Cliente confirma SPOC y Key Users — (Cliente, {fecha_kickoff + 3 dias})
    3. Indaws envia cuestionarios previos para las sesiones de analisis — (Consultor, {fecha_kickoff + 3 dias})
    4. Primera sesion de analisis — ({fecha concreta del calendario, slide 15})
    5. Siguiente reunion de seguimiento — ({fecha concreta})
  - Frase de cierre: "A partir de hoy, el proyecto esta en marcha"
- **REGLA**: NUNCA usar expresiones temporales relativas ("manana", "la semana que viene"). Siempre fecha concreta calculada.
- **Notas**: Que nadie salga de la reunion sin saber que tiene que hacer. Las fechas concretas generan mayor compromiso que las relativas.

### Slide 20 — Cierre + Q&A

- **Tipo**: Closing Slide (fondo oscuro con gradiente)
- **Contenido**:
  - Titulo: "Preguntas?"
  - Subtitulo: "Estamos aqui para construir juntos"
  - Datos de contacto del PM (nombre y email)
  - Logo Indaws
  - Mensaje: "Gracias por vuestra confianza"
- **Regla**: NO incluir telefono personal. Solo nombre y email.
- **Notas**: Dejar tiempo real para preguntas (5-10 min). No apresurarse. Anotar cualquier compromiso adicional que surja.

---

## Reglas de Adaptacion

### Slides opcionales (omitir si no aplica)

- **Slide 8 (Modulos Core)**: Omitir si no hay modulos Core en el pedido. Renumerar slides siguientes.

### Slides que pueden desdoblarse

- **Slide 4 (Equipo Indaws)**: Si el equipo tiene mas de 6 personas, usar 2 slides.
- **Slide 7 (Apps Odoo)**: Si hay mas de 8 apps, usar 2 slides.
- **Slide 15 (Calendario analisis)**: Si hay mas de 8 sesiones, usar 2 slides.

### Datos que se hardcodean (no vienen del sale.order)

- Slide 3: KPIs de Indaws (actualizar periodicamente)
- Slide 10: Fases de la metodologia (estandar)
- Slide 12: Plan de comunicacion (plantilla estandar, adaptar frecuencias)
- Slide 13: Gestion de cambios (proceso estandar)
- Slide 16: Compromisos (plantilla estandar, adaptar plazos)
- Slide 17: Impacto de retrasos (diagrama estandar)
