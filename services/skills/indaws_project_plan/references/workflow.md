# project-plan: Workflow Details

## Paso 0: Recopilacion de Datos (STOP)

**SIEMPRE** recopila antes de empezar:
1. **Documento de Analisis**: El archivo de analisis funcional
2. **Bolsa de Horas Total**: Total de horas vendidas/presupuestadas
3. **Tiempo de Implantacion**: Duracion estimada (ej. 3 meses, 12 semanas)
4. **Fecha de inicio del proyecto**
5. **Requisitos especiales de horas** (opcional): Horas concretas para tareas especificas

**DETENTE** y espera respuesta del usuario.

---

## Paso 1: Procesamiento del Analisis

Consulta template `./templates/tabla_planificacion.md` para estructura de salida.

### a) Tareas de Modulo (Agrupadas)

**REGLA CRITICA**: Para cada seccion del analisis, crear UNA UNICA TAREA AGRUPADA llamada "[Nombre Modulo]".

NO crear tareas individuales para cada linea del checklist.

**Calculo del Peso**: Suma de puntos de "Peso" de TODAS las lineas del "Checklist General de Tareas" (excepto las de seccion 7).

### b) Tareas Detalladas (Solo Seccion 7)

SOLO las secciones "7. Tareas Detalladas" del analisis. Estas SI son tareas individuales (normalmente [DEV] o [MIGR] complejos).

### c) Tareas Genericas (siempre incluir)

Consulta `./references/tareas_genericas.md`:
1. Configuraciones generales (12h default)
2. Soporte salida a produccion (18-48h segun bolsa)
3. Reuniones/formaciones (3h semanales x semanas)

---

## Paso 2: Calculo de Horas

Consulta `./references/reglas_calculo_horas.md` para algoritmo peso -> horas.

Referencia compartida: skill **odoo-effort-estimator** para velocidades de equipo (12h/sem funcional, 25h/sem dev, split 70/30).

**Principio**: Pesos son guia orientativa. Aplicar criterio experto segun complejidad, tipo de tarea, volumen de datos, contexto.

**Validacion**: Suma total coherente con bolsa (+-15%). Si discrepancia, informar y sugerir ajustes.

---

## Paso 2.5: Validacion de Tareas y Horas (STOP)

Presentar resumen con:
- Tareas agrupadas por modulo con horas
- Tareas detalladas con horas
- Tareas genericas con horas
- TOTAL vs Bolsa (diferencia %)
- Criterios aplicados

Espera respuesta: Si -> Paso 3, Ajustes -> recalcular y volver a presentar.

**NO continuar sin aprobacion.**

---

## Paso 3: Cronograma

Consulta:
- `./references/fases_implantacion.md` -> Fases Waterfall/Hibrido, dependencias
- `./references/reglas_planificacion.md` -> Distribucion temporal, secuencias

**Principios clave**:
- Configuraciones generales al inicio
- Maestros antes que flujos
- Configuracion antes que migracion del mismo modulo
- Migraciones antes que testing
- Go-live 1 mes antes de finalizacion
- [DEV] pueden ir en paralelo con configuraciones

---

## Paso 3.5: Validacion de Cronograma (STOP)

Presentar:
- Duracion total y fechas
- Go-Live propuesto
- Resumen de fases con tareas clave
- Principios aplicados
- Puntos de atencion (alta carga, dependencias, conflictos)
- Hitos clave

Espera respuesta: Si -> Paso 4, Ajustes -> recalcular, Cambio duracion -> redistribuir.

**NO generar documentos sin aprobacion.**

---

## Paso 4: Generacion de Entregables Finales

**Solo tras las 2 validaciones.**

### Entregable A: Tabla de Planificacion
Template: `./templates/tabla_planificacion.md`
7 columnas: #, Nombre, Modulo, Peso, Horas, Fecha Inicio, Fecha Fin
+ Resumen por Categoria, por Modulo, hitos, fila TOTAL.

### Entregable B: Diagrama de Gantt
Template: `./templates/gantt.md`
Tabla semanal con bloques visuales.

### Entregable C: Resumen de Tareas con Checklist
Apartado "Detalle de Tareas" con checklist interno de cada tarea agrupada (origen: Seccion 6 del analisis). Para tareas detalladas: copiar checklist de seccion 7.
