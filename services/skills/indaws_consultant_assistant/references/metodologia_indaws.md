# Metodología Indaws

## Qué es

La metodología Indaws define los principios fundamentales que guían el trabajo de consultoría en proyectos de implantación de Odoo. Se estructura en 6 pilares y una escalera de decisión que determina cómo abordar cada necesidad del cliente.

## Cuándo aplica

Siempre. Estos principios aplican a todas las fases del proyecto, desde la pre-venta hasta el soporte post-implantación.

## Qué debe saber el consultor

### Escalera de Decisión (Decision Ladder)

Antes de proponer cualquier solución, sigue la escalera **Estándar → Core → Studio → Custom**. El criterio para elegir cada peldaño (y la frontera Studio-vs-desarrollo) lo fija `odoo-classification-guide` (fuente de verdad única):

1. **Odoo Estándar** → Agotar las posibilidades de configuración nativa
2. **Indaws Core** → Revisar el catálogo de 40+ paquetes funcionales propios (ya probados y mantenidos)
3. **Odoo Studio** → Solo para lo trivial y de bajo riesgo (un campo, una vista, una automatización de un paso)
4. **Desarrollo a medida (módulo gobernado)** → Para la lógica no trivial (reglas de negocio, flujos, validaciones) o cuando pisa algo ya instalado

El criterio no es "evita el código" sino **código gobernado**: para lógica no trivial, un módulo revisado/testeable pesa más que un Studio ingobernable.

### Los 6 Pilares

#### Pilar 1: Filosofía de Implementación
- Aprovechar el estándar y revisar Indaws Core antes de reinventar (no duplicar lo que ya existe)
- Studio para lo trivial; para lógica no trivial, módulo gobernado (criterio en `odoo-classification-guide`)
- No sobrecomplicar la solución: simplicidad ante todo
- Entender la necesidad de negocio, no solo el ticket
- **Cita clave**: "Estándar y Core primero; y cuando hay que escribir código, que sea gobernado"

#### Pilar 2: Planificación y Ejecución
- Planificar tareas técnicas con antelación (anticipación técnica)
- Documentar antes de entrar en planificación del sprint
- Mantener tareas y checklists siempre al día
- Ayudar a los TLs a reservar horas de desarrollo
- Preguntar por carga del equipo de desarrollo (flexibilidad)
- Usar checklists activamente (un checklist desactualizado es peor que no tener checklist)

#### Pilar 3: Relación con el Cliente
- **OBLIGATORIO**: Enviar informe semanal al cliente (weekly update)
- Decir "no" a tiempo antes que prometer de más
- Traducir tecnicismos al lenguaje de negocio (account.move.line → "registro de factura")
- Si una llamada de 5 min resuelve antes que un email largo, llama
- El cliente es compañero de viaje, no destinatario de entregables

#### Pilar 4: Ética Financiera
- Vigilar el presupuesto constantemente
- Solicitar traspaso de horas si cambia el rol (consultoría ↔ dev)
- Registrar horas en el timesheet **cada día** (no dejar para el viernes)
- Comunicar desbordes de alcance al PM cuanto antes

#### Pilar 5: Organización Interna
- Mantener a los PMs informados de todo (son tu escudo)
- Apoyarte en PMs de otros equipos si lo necesitas
- No justificar tu trabajo ante otros departamentos sin indicación
- Tener el calendario y planificación siempre actualizados

#### Pilar 6: Calidad y Crecimiento
- Probar en staging con datos reales antes de entregar (UAT)
- Verificar flujos completos, no solo funcionalidades aisladas
- Comprobar permisos de usuario para cada rol
- Documentar errores complejos en la base de conocimiento interna
- **Medida del éxito**: "La calidad de una implementación se mide por lo poco que el cliente necesita llamarnos después de la puesta en marcha"

### 2 Prácticas Obligatorias

1. **Weekly Updates**: Informe semanal al cliente, sin excepción. Contenido: tareas completadas, en curso, próximos pasos, bloqueos, decisiones pendientes del cliente.
2. **Timesheets diarios**: Registrar horas cada día con descripciones precisas. La memoria falla si esperas al viernes.

### Principios Fundamentales

- **Consultor ≠ programador de tickets**: Entiende el "por qué" antes de decidir el "cómo"
- **PM como escudo**: El PM defiende tus decisiones ante el resto de la empresa
- **Traducción técnico → negocio**: Habla de Ventas, Contabilidad y Almacén, no de módulos y modelos
- **Calidad = independencia post go-live**: El cliente debe poder operar sin nosotros

## Qué debe hacer el consultor

1. Antes de cada propuesta de solución, recorrer la escalera de decisión
2. Verificar si existe un paquete Indaws Core antes de cotizar desarrollo
3. Registrar horas al final de cada jornada
4. Enviar weekly update cada semana (viernes preferentemente)
5. Documentar cualquier decisión técnica relevante en Knowledge

## Skills relacionados

- `project-weekly-update`: Genera el informe semanal obligatorio
- `project-health-monitor`: Evalúa salud del proyecto (score 0-100)
- `odoo-configurator`: Ejecuta configuraciones siguiendo la filosofía estándar-primero
- `project-analysis`: Aplica la escalera de decisión en el análisis funcional

## Errores comunes

1. **Proponer desarrollo sin verificar Core**: Hay 40+ paquetes que cubren muchas necesidades frecuentes (verificar Core es un paso de descubrimiento, no un mandato de evitar código: si la lógica no trivial no está cubierta, el módulo gobernado es lo correcto)
2. **No enviar weekly update**: El cliente pierde confianza y genera ansiedad
3. **Dejar timesheets para el viernes**: La precisión cae drásticamente después de 24h
4. **Hablar en tecnicismos con el cliente**: "Many2one" no significa nada para un director financiero
5. **No comunicar desbordes de alcance al PM**: Una factura inesperada destruye la relación
