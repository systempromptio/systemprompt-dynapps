# Guia de Contenido para Actualizacion Semanal

## Principio General

La actualizacion semanal es un documento **dirigido al cliente**. El tono es profesional, claro y orientado a resultados. El cliente NO necesita conocer detalles internos del proyecto (scores, penalizaciones, metricas tecnicas).

---

## Seccion 1: Trabajo Realizado

### Objetivo
Describir de forma narrativa el trabajo ejecutado durante la semana. Los timesheets se usan como fuente de informacion para entender que se ha hecho, pero las horas son datos internos y NUNCA se muestran al cliente.

### Formato
Parrafo narrativo (NO lista de timesheets, NO tabla de horas). Agrupar por area funcional, describir avances concretos.

### Ejemplo BUENO

> Durante esta semana se ha avanzado significativamente en la configuracion del modulo de CRM. Se han definido los equipos comerciales, configurado el pipeline de ventas con las etapas acordadas y establecido las reglas de asignacion automatica de oportunidades. En paralelo, se ha continuado con la parametrizacion de Facturacion, completando la configuracion de las secuencias de facturas y los impuestos aplicables.
>
> Adicionalmente, se han completado 5 hitos del proyecto esta semana, destacando la finalizacion de la configuracion del pipeline de CRM y las reglas de asignacion.

### Ejemplo MALO

> - Rafael: Config equipos 3h
> - Rafael: Pipeline ventas 4.5h
> - Maria: Facturas secuencias 3h
> - Maria: Impuestos 5h

### Reglas
- Usar voz pasiva: "Se ha configurado...", "Se han completado..."
- **NUNCA incluir horas, desglose de tiempo ni tabla de horas** - las horas son datos internos
- Los timesheets se leen SOLO para entender que trabajos se han realizado y redactar la narrativa
- Si hay checklists completados, integrarlos en la narrativa como "hitos"
- No mencionar nombres de empleados individuales (a menos que el usuario lo pida)
- No usar terminologia tecnica de Odoo (models, fields, etc.)

---

## Seccion 2: Progreso del Proyecto

### Objetivo
Mostrar el avance global del proyecto de forma visual (porcentaje + color).

### Datos
- `C_pct` como porcentaje de progreso global
- Color segun delta con T% (verde/naranja/rojo)
- Interpretacion cualitativa

### Interpretaciones segun estado

| Delta (C% - T%) | Color | Interpretacion |
|-----------------|-------|----------------|
| > -15pp | Verde (#27ae60) | "El proyecto avanza segun lo previsto" |
| -15pp a -20pp | Naranja (#f39c12) | "El proyecto presenta un ligero desfase respecto al calendario previsto" |
| < -20pp | Rojo (#e74c3c) | "El proyecto presenta un desfase significativo que estamos gestionando activamente" |

### Formato
Bloque visual con el porcentaje grande y un breve parrafo interpretativo. NO mostrar Health Score numerico.

---

## Seccion 3: Estado de Tareas

### Objetivo
Tabla con el estado actual de cada tarea/modulo del proyecto.

### Formato
Tabla HTML directa desde los datos del JSON. Sin narrativa adicional.

### Columnas
- Tarea/Modulo
- Etapa (En progreso, Hecho, etc.)
- Progreso (% completado)

### Reglas
- Ordenar por relevancia: primero tareas en progreso, luego completadas
- No incluir tareas canceladas
- El progreso es el C% de la tarea (basado en checklists ponderados, pero NO lo llames asi)

---

## Seccion 4: Proximos Pasos

### Objetivo
3-5 items concretos y accionables que describen las proximas actividades.

### Formato
Lista HTML (`<ul>`) con items claros.

### Ejemplo BUENO

> - Completar la configuracion de reglas de precios y tarifas en el modulo de Ventas
> - Iniciar la parametrizacion del modulo de Inventario: ubicaciones y reglas de reabastecimiento
> - Revisar con el equipo comercial los informes personalizados de CRM
> - Preparar el entorno de pruebas para la primera validacion de usuario

### Ejemplo MALO

> - Resolver bottleneck en Ventas (risk score 4.2)
> - Reducir pen_checklist de 35 a 20
> - Cerrar items overdue de xx.checklist.item

### Reglas
- Lenguaje orientado al cliente, no tecnico
- Cada item debe ser accionable y comprensible
- Basarse en los bottlenecks y tareas vencidas del JSON, pero traducirlos
- Si hay alertas criticas, mencionarlas como "puntos de atencion"
- No usar jerga interna: "bottleneck" → "area prioritaria", "overdue" → "pendiente de completar"

---

## Casos Especiales

### Semana sin timesheets registrados
- Seccion 1 breve: "Esta semana no se han registrado actividades en el proyecto."
- Mantener el resto de secciones normalmente

### Proyecto en estado critico (score < 40)
- No alarmar al cliente con lenguaje catastrofista
- Usar tono constructivo: "Se han identificado areas que requieren atencion prioritaria"
- Los proximos pasos deben reflejar acciones concretas de reconduccion

### Primera semana del proyecto
- Seccion 1: Describir tareas de configuracion inicial y kickoff
- Progreso esperado bajo; no interpretar como desviacion

### Semana con muchos hitos completados
- Destacar el avance positivo en el parrafo narrativo
- Usar tono de logro: "Se han alcanzado hitos significativos esta semana"
