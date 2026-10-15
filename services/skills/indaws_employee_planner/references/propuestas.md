# Cómo se escribe la propuesta

Una propuesta que obliga a abrir Odoo para entenderla no sirve. Las reglas de forma vienen de la doctrina y se aplican tal cual:

- **Un id nunca va solo**: *"la tarea «01 · Configuraciones generales» (22553)"*, nunca *"la 22553"*.
- **Se vuelve a nombrar en cada mención**: quien lee la propuesta 5 no ha leído la 4.
- **Cada turno se lee como una frase de calendario**: *"miércoles 8 de octubre, de 9:00 a 13:00, Urbanitae, tarea «Migración de contactos» (22701)"*.
- **El rol deducido del departamento se dice.**
- **Ni un nombre técnico de modelo ni de campo.**

## Estructura de la salida en el chat

1. **Cabecera**: persona, modo, horizonte, fuentes que ha tenido (doctrina con su fecha, Odoo, Outlook), parámetros leídos de la tabla de la doctrina y los que hayan salido como `ignorados`, **el reparto semanal validado** (quién y cuándo) y **las previsiones reservadas** en soft.
2. **Tabla resumen** (arranque): semanas × proyectos con sus horas, más soft, reuniones e interno. Es lo primero que se mira para ver desequilibrios: un proyecto que se come semanas enteras o uno que desaparece.
3. **Propuestas numeradas**:
   - **Arranque**: una por semana. *"3 · Semana del 13 de octubre — 32 h de proyecto (Rotger 20 h, Almacenes del Real 6 h, TFT Food 6 h), 2 h soft (Prioritic), 4 h de reuniones"*, con sus turnos debajo, uno por línea.
   - **Diario**: una por cambio. Si pisa algo publicado o ajeno, la misma línea lleva el antes y el después: *"Jueves 1 de octubre: bajar de 8 h a 6 h el turno publicado de Distribuidora Rotger, tarea «Sislog · catorce interfaces» (21861), porque entran 2 h de reunión de TFT Food"*.
4. **Solo se señala** (aparte, sin número): no cabe, recorte, vencidas que salen por hechas, `vencidas_superan_reparto`, `proyecto_a_cero` (con su motivo), `prevision_sin_fecha_fin`, `trabajo_cortado_en_go_live`, `proyecto_sin_reparto`, capacidad libre y perdida en trozos por semana, días por encima del estándar, tareas sin fechas, tareas de desarrollo sin desarrollador, reuniones sin tarea, reuniones internas sin proyecto con forma de preparación, imputación sin tarea, soft pasado de fecha, contradicciones de doctrina.
5. **Cierre**: horas de proyecto frente a horas que no son de proyecto del plan, y la pregunta de si se publica lo que se acepte.

En **arranque**, los demás asignados del proyecto salen al final, cada uno con su encaje en una línea (cabe / no cabe y cuánto, recorte que le supondría). No se numeran: lo aceptará cada uno en su propia pasada.

## `proposals.json`

Cada propuesta guarda exactamente lo que se escribirá si se acepta:

```json
{"generated":"2026-10-01","employee_id":12,"mode":"arranque","proposals":[
 {"n":3,"title":"Semana del 13 de octubre","ops":[
  {"op":"create","start":"2026-10-13T09:00","end":"2026-10-13T13:00","project_id":7,"task_id":22701,"name":null},
  {"op":"write","slot_id":5512,"before":{"start":"…","end":"…"},"after":{"start":"…","end":"…"},"published":true,"foreign":true},
  {"op":"unlink","slot_id":5520,"published":false,"foreign":false}]}]}
```

Horas en local de la persona; la conversión a UTC se hace al escribir.
