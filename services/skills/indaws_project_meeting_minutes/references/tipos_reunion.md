# Tipos de Reunion

Referencia de los 5 tipos de reunion soportados. Cada tipo define secciones especificas en el acta, color de badge, duracion tipica y acciones post-reunion.

---

## 1. Seguimiento

**Proposito**: Revisar el progreso del proyecto, identificar blockers y acordar proximos pasos.

**Frecuencia tipica**: Semanal o quincenal.

**Duracion tipica**: 30-60 minutos.

**Color badge**: Blue Lilac (#6B68FA)

**Asistentes tipicos**:
- Indaws: PM, Consultor principal
- Cliente: SPOC, Key Users involucrados

**Secciones especificas en el acta**:
- Estado de tareas desde la ultima reunion (que se ha avanzado)
- Blockers y problemas activos (que esta parado y por que)
- Revisiones de acciones previas (se cumplieron las acciones de la reunion anterior?)
- Estado de peticiones al cliente (que falta por entregar)
- Proximos hitos y fechas

**Deteccion de alcance**: ACTIVA. Es el tipo de reunion donde mas frecuentemente aparecen nuevas solicitudes del cliente.

**Acciones post-reunion tipicas**:
- Actualizar estado de tareas
- Crear/actualizar peticiones al cliente
- Senalar blockers al PM

---

## 2. Kick-Off

**Proposito**: Primera reunion del proyecto con el cliente. Presentar equipo, metodologia y establecer reglas del juego.

**Nota**: El kick-off completo se gestiona con el skill `kick-off-presentation`. Este tipo se usa solo cuando la reunion fue informal o se quiere documentar como acta complementaria.

**Frecuencia**: Una vez por proyecto.

**Duracion tipica**: 60-90 minutos.

**Color badge**: Warm Yellow (#E5B92B)

**Asistentes tipicos**:
- Indaws: PM, Consultor principal, Comercial
- Cliente: Sponsor, SPOC, Key Users principales

**Secciones especificas en el acta**:
- Presentacion del equipo (quien es quien)
- Confirmacion de alcance y metodologia
- SPOC y Key Users confirmados
- Calendario de sesiones de analisis acordado
- Reglas del juego aceptadas

**Deteccion de alcance**: LIGERA. En un kick-off no suele haber cambios de alcance, pero si pueden surgir expectativas fuera del scope.

**Acciones post-reunion tipicas**:
- Confirmar SPOC y Key Users
- Agendar sesiones de analisis
- Enviar presentacion y documentacion

---

## 3. Validacion

**Proposito**: Revisar y aprobar entregables del proyecto (documento de analisis, configuraciones, datos migrados, etc.).

**Frecuencia**: Segun hitos del proyecto.

**Duracion tipica**: 45-90 minutos.

**Color badge**: Verde (#28a745)

**Asistentes tipicos**:
- Indaws: PM, Consultor principal, Desarrollador (si aplica)
- Cliente: SPOC, Key Users del area, Decision maker

**Secciones especificas en el acta**:
- Entregable revisado (nombre, version, estado)
- Tabla de validacion: elemento revisado + estado (aprobado / rechazado / con ajustes)
- Lista de ajustes requeridos (si los hay)
- Firma/confirmacion del cliente (verbal o escrita)
- Fecha limite para aplicar ajustes

**Deteccion de alcance**: ACTIVA. Durante validaciones es comun que el cliente pida funcionalidades adicionales al ver el entregable.

**Acciones post-reunion tipicas**:
- Marcar entregable como validado (o registrar ajustes)
- Crear acciones para los ajustes con deadline
- Si hay rechazos, escalar al PM

---

## 4. Formacion

**Proposito**: Sesion de formacion al equipo del cliente sobre una app o funcionalidad de Odoo.

**Frecuencia**: Segun plan de formacion.

**Duracion tipica**: 60-120 minutos.

**Color badge**: Light Sky (#8AC2DB)

**Asistentes tipicos**:
- Indaws: Formador (consultor o PM)
- Cliente: Key Users del area, usuarios finales

**Secciones especificas en el acta**:
- Temas cubiertos (con nivel de detalle)
- Ejercicios realizados y resultados
- Dudas planteadas (con respuesta si se dio)
- Temas que requieren sesion adicional
- Nivel de confianza del equipo (percepcion del formador)
- Materiales compartidos (enlaces, documentos)

**Deteccion de alcance**: LIGERA. En formaciones raramente hay cambios de alcance, pero pueden surgir solicitudes de funcionalidades al ver el sistema.

**Acciones post-reunion tipicas**:
- Compartir materiales y cheat sheets
- Programar sesion de seguimiento si es necesario
- Registrar dudas no resueltas para investigar

---

## 5. Analisis

**Proposito**: Sesion de levantamiento de requisitos con Key Users del cliente. Se analiza una app o area funcional concreta.

**Frecuencia**: Varias sesiones durante la fase de analisis (1 o mas por app).

**Duracion tipica**: 60-120 minutos.

**Color badge**: Blue Space (#1C265D)

**Asistentes tipicos**:
- Indaws: Consultor principal (que dirige la sesion)
- Cliente: Key Users del area analizada

**Secciones especificas en el acta**:
- App/area analizada
- Flujos de negocio discutidos
- Decisiones funcionales tomadas (como van a trabajar con Odoo)
- GAPs identificados (lo que Odoo estandar no cubre)
- Soluciones propuestas para cada GAP (estandar, Studio, Core, desarrollo)
- Datos a migrar identificados
- Integraciones mencionadas

**ACCION ESPECIAL**: Rellenar el cuestionario de la app con las respuestas extraidas de la sesion (ver Paso 2b del SKILL.md).

**Deteccion de alcance**: MUY ACTIVA. Las sesiones de analisis son donde se descubren la mayoria de requisitos nuevos. Ser especialmente riguroso cotejando contra el analisis previo (si existe) y las tareas/checklists del proyecto.

**Acciones post-reunion tipicas**:
- Actualizar cuestionario en Knowledge
- Alimentar el documento de analisis funcional (skill `project-analysis`)
- Crear items de checklist para nuevos requisitos dentro del alcance
- Senalar cambios de alcance para presupuestar

---

## Resumen de Colores de Badge por Tipo

| Tipo | Color fondo | Color texto | Hex fondo |
|------|-------------|-------------|-----------|
| seguimiento | Blue Lilac | Blanco | #6B68FA |
| kick-off | Warm Yellow | Blue Space | #E5B92B |
| validacion | Verde | Blanco | #28a745 |
| formacion | Light Sky | Blue Space | #8AC2DB |
| analisis | Blue Space | Blanco | #1C265D |

**HTML del badge**:
```html
<span style="display: inline-block; padding: 4px 12px; border-radius: 4px; font-size: 12px; font-weight: 600; background: {hex_fondo}; color: {color_texto}; text-transform: uppercase; letter-spacing: 0.5px;">
  {TIPO}
</span>
```
