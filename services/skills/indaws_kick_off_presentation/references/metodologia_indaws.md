# Metodologia de Implantacion Indaws

Referencia para las slides 10 (Metodologia) y 11 (Fase de Analisis).

---

## Las 6 Fases de la Implantacion

### Fase 1: Kick-Off

**Duracion**: 1 sesion (60-90 min)
**Participantes**: Equipo Indaws completo + SPOC + Direccion cliente
**Objetivo**: Alinear expectativas, presentar equipo, establecer reglas y compromisos
**Entregable**: Acta de kick-off con acuerdos y proximos pasos

**Actividades**:
- Presentacion del equipo de proyecto (ambas partes)
- Revision del alcance contratado
- Explicacion de la metodologia de trabajo
- Establecimiento de compromisos mutuos y canales de comunicacion
- Definicion de roles (SPOC, Key Users)
- Agenda de sesiones de analisis

---

### Fase 2: Analisis Funcional

**Duracion tipica**: 2-4 semanas (segun tamano del proyecto)
**Participantes**: Consultor(es) + Key Users
**Objetivo**: Entender los procesos del cliente, detectar GAPs con Odoo estandar, disenar la solucion
**Entregable**: Documento de Analisis Funcional

**Subfases**:

1. **Sesiones de discovery** (50% del tiempo de analisis)
   - Reuniones con Key Users por area funcional
   - Revision de procesos actuales, casuisticas especiales, excepciones
   - Recopilacion de formatos, datos y reglas de negocio
   - Identificacion de pain points y expectativas

2. **Estudio interno** (30% del tiempo de analisis)
   - El equipo Indaws analiza la informacion recopilada
   - Diseno de la solucion tecnica en Odoo
   - Identificacion de GAPs y propuesta de resolucion (escalera Estandar > Core > Studio > Custom; criterio en `odoo-classification-guide`):
     - Configuracion estandar de Odoo
     - Paquetes Indaws Core
     - Automatizaciones Studio (para lo trivial)
     - Desarrollos a medida (modulo gobernado) cuando la logica no es trivial
   - Planificacion detallada de tareas y horas

3. **Generacion de documentacion** (15% del tiempo de analisis)
   - Redaccion del documento de analisis funcional
   - Revision interna de calidad
   - Planificacion del proyecto (Gantt, horas, fechas)

4. **Presentacion y validacion** (5% del tiempo de analisis)
   - Sesion de presentacion del analisis al cliente
   - Resolucion de dudas y ajustes
   - Aprobacion formal del analisis por parte del cliente

**Que esperamos del cliente**:
- Asistencia puntual de los Key Users a las sesiones programadas
- Respuestas a consultas entre sesiones en maximo 5 dias laborables
- Entrega de formatos, datos y accesos en los plazos acordados
- Validacion del documento de analisis en maximo 5 dias laborables

---

### Fase 3: Implantacion

**Duracion tipica**: 40-60% de la duracion total del proyecto
**Participantes**: Consultor(es) + Desarrollador(es) + Key Users (para validaciones parciales)
**Objetivo**: Configurar Odoo, desarrollar personalizaciones, migrar datos
**Entregable**: Entorno Odoo configurado y funcional

**Actividades**:
- Configuracion de aplicaciones segun analisis funcional
- Instalacion y configuracion de modulos Indaws Core
- Desarrollo de personalizaciones ([DEV] del analisis)
- Configuracion de automatizaciones (Studio/Server Actions)
- Migracion de datos maestros (productos, clientes, proveedores, plan contable)
- Configuracion de formatos (facturas, albaranes, pedidos)
- Demos parciales al cliente por modulo completado

**Que esperamos del cliente**:
- Entrega de datos de migracion en formato acordado y en plazo
- Participacion en demos parciales para validar configuracion
- Feedback rapido sobre demos (maximo 3 dias)
- Toma de decisiones sobre casuisticas pendientes

---

### Fase 4: UAT (User Acceptance Testing)

**Duracion tipica**: 1-3 semanas
**Participantes**: Key Users + Consultor(es)
**Objetivo**: Validar que todo funciona correctamente con datos y escenarios reales
**Entregable**: Informe de UAT aprobado

**Actividades**:
- Sesiones de testing guiado con Key Users
- Pruebas de flujos completos end-to-end
- Validacion de datos migrados
- Pruebas de integraciones (si aplica)
- Correccion de incidencias detectadas
- Validacion de formatos impresos
- Aprobacion formal para Go-Live

**Que esperamos del cliente**:
- Dedicacion de los Key Users para las pruebas (tiempo protegido)
- Reporte de incidencias de forma estructurada
- Validacion formal por escrito antes de Go-Live

---

### Fase 5: Go-Live (Salida a Produccion)

**Duracion**: 1-2 dias
**Participantes**: Equipo completo Indaws + Key Users + Usuarios finales
**Objetivo**: Arrancar en produccion con Odoo como sistema principal
**Entregable**: Sistema en produccion

**Actividades**:
- Migracion de datos finales (stocks, saldos contables, pendientes)
- Desactivacion del sistema anterior (si aplica)
- Formacion de refresco rapida a usuarios finales
- Soporte presencial/remoto intensivo durante el primer dia
- Monitorizacion de incidencias criticas

**Que esperamos del cliente**:
- Key Users disponibles a tiempo completo durante el Go-Live
- Decision firme de "go/no-go" basada en los resultados de UAT
- Comunicacion interna a toda la organizacion sobre el cambio

---

### Fase 6: Soporte Post-Arranque

**Duracion tipica**: 2-4 semanas despues del Go-Live
**Participantes**: Consultor(es) + Key Users
**Objetivo**: Acompanar al cliente en las primeras semanas de uso real
**Entregable**: Proyecto cerrado y transicion a soporte de mantenimiento

**Actividades**:
- Resolucion de dudas e incidencias del dia a dia
- Ajustes menores de configuracion basados en uso real
- Formacion adicional si se detectan carencias
- Optimizacion de flujos basada en feedback de usuarios
- Cierre formal del proyecto
- Transicion al contrato de mantenimiento/soporte

**Que esperamos del cliente**:
- Reportar incidencias de forma ordenada
- Distinguir entre incidencias (errores) y mejoras (nuevas funcionalidades)
- Participar en la reunion de cierre del proyecto

---

## Principios de la Metodologia

### Waterfall adaptado
Seguimos un modelo waterfall con elementos hibridos:
- Las fases se ejecutan en secuencia, pero con solapamientos controlados
- El desarrollo puede empezar en paralelo con la implantacion de otros modulos
- Las migraciones se preparan durante la implantacion, se ejecutan antes de UAT

### Validaciones como gates
Cada fase tiene un entregable que debe ser validado antes de pasar a la siguiente:
- Analisis → Documento validado → Implantacion
- Implantacion → Demos validadas → UAT
- UAT → Informe aprobado → Go-Live

### Comunicacion continua
- Reuniones de seguimiento semanales con el SPOC
- Actualizaciones escritas semanales
- Escalacion rapida si hay bloqueos

### Sin sorpresas
- Los riesgos se comunican en cuanto se detectan
- Las desviaciones se reportan con propuesta de solucion
- El presupuesto se monitoriza y se informa al cliente del consumo de horas
