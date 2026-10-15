# Reglas del Juego: Compromisos Mutuos y SLAs

Referencia detallada para la slide 16 (Compromisos mutuos) y la slide 17 (Impacto de retrasos).

---

## Filosofia

Los compromisos no son un contrato legal adicional. Son un **acuerdo de trabajo** que establece las expectativas de ambas partes para que el proyecto fluya. El tono debe ser:

- **Transparente**: Sin letra pequena ni ambiguedades
- **Equilibrado**: Indaws tambien se compromete, no solo el cliente
- **Constructivo**: El objetivo es que el proyecto salga bien, no protegerse
- **Firme pero respetuoso**: No amenazar, pero no dejar margen a la ambiguedad

---

## Compromisos de Indaws

### Plazos y entregas
- Cumplir las fechas acordadas en el plan de proyecto
- Si un entregable se va a retrasar, informar con al menos 48h de antelacion
- Entregar documentacion de calidad, revisada internamente antes de compartir

### Comunicacion
- Responder consultas del cliente en un maximo de **48h laborables**
- Enviar actualizaciones semanales del estado del proyecto (informe escrito)
- Convocar reuniones de seguimiento con agenda previa (minimo 24h antes)
- Documentar por escrito todas las decisiones tomadas en reuniones

### Equipo
- Asignar un equipo dedicado y estable durante todo el proyecto
- Si hay un cambio de persona, hacer traspaso formal con solapamiento
- El PM es el punto de contacto unico para temas de gestion

### Proactividad
- Informar proactivamente de cualquier riesgo o desviacion detectada
- Proponer soluciones, no solo comunicar problemas
- Anticipar necesidades de informacion del cliente (no esperar al ultimo momento)

---

## Compromisos del Cliente

### Personas y disponibilidad
- Asignar un **SPOC** con:
  - Disponibilidad para reuniones semanales de seguimiento
  - Capacidad de tomar decisiones operativas (o escalar rapidamente)
  - Conocimiento transversal de la empresa
- Asignar **Key Users** por cada area funcional con:
  - Conocimiento profundo de los procesos de su area
  - Autoridad para validar decisiones de su ambito
  - Disponibilidad para asistir a las sesiones planificadas

### Tiempos de respuesta (SLAs)
- **Solicitudes de informacion**: Responder en un maximo de **5 dias laborables**
  - Ejemplos: listados de productos, plan contable, formatos de documentos, reglas de negocio
- **Validacion de entregables**: Aprobar o rechazar en un maximo de **5 dias laborables**
  - Ejemplos: documento de analisis, configuraciones validadas, flujos aprobados
- **Asistencia a reuniones**: Confirmar asistencia en **24h**. Si un Key User no puede asistir, designar sustituto o reprogramar con 48h de antelacion
- **Toma de decisiones**: Las decisiones que bloquean tareas deben resolverse en **3 dias laborables**

### Datos y acceso
- Proporcionar datos de migracion en el formato y plazo acordados
- Dar acceso a sistemas existentes cuando se solicite (para integraciones o migracion)
- Enviar formatos de documentos actuales (facturas, albaranes, pedidos, informes)

### Testing y validacion
- Participar activamente en las pruebas UAT con los Key Users
- Reportar incidencias de forma clara y estructurada
- No arrancar en produccion sin completar la validacion formal

---

## Impacto de Retrasos: Cadena de Consecuencias

### El principio
> Los plazos del proyecto son una cadena. Si un eslabon se retrasa, todos los eslabones posteriores se desplazan proporcionalmente.

### Escenarios tipicos

#### Escenario 1: Informacion no entregada a tiempo
```
Cliente no envia listado de productos (retraso +2 semanas)
  → Migracion de datos no puede configurarse
    → Demo al cliente se retrasa +2 semanas
      → UAT se retrasa +2 semanas
        → Go-Live se desplaza +2 semanas
```

#### Escenario 2: Key User no disponible para sesion de analisis
```
Sesion de Compras cancelada (Key User no disponible)
  → Se reprograma para la siguiente semana
    → Analisis de Compras se retrasa +1 semana
      → Documento de analisis se retrasa +1 semana
        → Inicio de implantacion se desplaza +1 semana
```

#### Escenario 3: Validacion de analisis retrasada
```
Cliente no valida documento de analisis (retraso +3 semanas)
  → Implantacion no puede iniciar
    → Equipo Indaws reasignado a otros proyectos
      → Replanificacion necesaria (nuevo timeline)
        → Go-Live se desplaza significativamente
```

### Consecuencias operativas de retrasos prolongados

- **Reasignacion de recursos**: Si el proyecto se detiene mas de 2 semanas por falta de respuesta del cliente, Indaws puede necesitar reasignar recursos a otros proyectos. La reincorporacion requerira replanificacion.
- **Perdida de contexto**: Los retrasos prolongados causan perdida de contexto en ambas partes, requiriendo sesiones de retoma que consumen horas del proyecto.
- **Impacto en costes**: Las horas estan estimadas para un flujo de trabajo continuo. Retrasos pueden generar ineficiencias que consuman horas adicionales.
- **Condiciones contractuales**: Segun las condiciones del presupuesto, las horas sobrepasadas por causas ajenas a Indaws requieren aprobacion del cliente mediante nuevo presupuesto.

### Como presentarlo

**TONO CORRECTO**:
> "Queremos ser transparentes sobre como funcionan los plazos. No se trata de culpar a nadie, sino de entender que el exito depende de que ambos cumplamos nuestros compromisos. Si hay un bloqueo, hablemos rapido para resolverlo juntos."

**TONO INCORRECTO**:
> "Si no nos contestais a tiempo, el proyecto se retrasara y sera culpa vuestra."

---

## Client Requests: Solicitudes Post-Kick-Off

### Plantilla de solicitudes inmediatas

Estas son las solicitudes tipicas que se hacen al cliente en el kick-off. Personalizar segun el proyecto.

| Prioridad | Solicitud | Responsable | Plazo sugerido | Notas |
|-----------|-----------|-------------|---------------|-------|
| CRITICA | Confirmar SPOC del proyecto | Direccion cliente | +3 dias | Sin SPOC no se agenda nada |
| CRITICA | Confirmar Key Users por area | SPOC | +5 dias | Nombre, cargo, email, telefono |
| CRITICA | Confirmar disponibilidad sesiones analisis | SPOC | +3 dias | Para cerrar calendario |
| ALTA | Enviar formatos actuales (facturas, albaranes, pedidos) | Key Users | +7 dias | PDFs o Excel de ejemplo |
| ALTA | Acceso a sistemas actuales (si aplica integracion) | IT cliente | +7 dias | Credenciales, documentacion API |
| MEDIA | Listado de productos/servicios (para migracion) | Key User Inventario | +10 dias | Excel con campos: codigo, nombre, precio, categoria |
| MEDIA | Listado de clientes/proveedores (para migracion) | Key User Ventas/Compras | +10 dias | Excel con campos: nombre, NIF, direccion, contacto |
| MEDIA | Plan contable actual (para migracion) | Key User Contabilidad | +10 dias | PDF o Excel del plan de cuentas |
| BAJA | Organigrama / estructura de la empresa | SPOC | +15 dias | Para configurar permisos y roles |

### Reglas para las solicitudes
- Toda solicitud tiene un **responsable** y una **fecha limite**
- Las fechas se calculan desde la fecha del kick-off
- Las solicitudes CRITICAS bloquean el inicio del analisis
- Las solicitudes de prioridad MEDIA pueden entregarse durante la fase de analisis
- Si una solicitud se retrasa, el PM debe hacer seguimiento activo (no esperar pasivamente)
