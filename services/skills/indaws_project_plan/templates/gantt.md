# Plantilla: Diagrama de Gantt

Esta plantilla define la estructura del diagrama de Gantt visual que debe generarse como output del skill.

---

## Estructura del Gantt

```markdown
## Diagrama de Gantt (Vista Semanal)

**Leyenda:**
- `██` = Tarea en ejecución esa semana
- `▓▓` = Tarea en ejecución parcial (< 50% de la semana)
- `──` = Dependencia / Espera
- `🚀` = Go-Live
- `✓` = Hito completado

---

### Gantt Completo

| # | Tarea | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | ... | S{N} |
|---|-------|----|----|----|----|----|----|----|----|-----|------|
| 1 | Configuraciones Generales | ██ |  |  |  |  |  |  |  |  |  |
| 2 | [CONFIG] Ventas |  | ██ | ██ | ██ |  |  |  |  |  |  |
| 3 | [DEV] Tarificación |  | ██ | ██ | ██ | ██ | ██ | ██ | ██ |  |  |
| 4 | [MIGR] Clientes |  |  |  | ██ |  |  |  |  |  |  |
| ... | ... | ... | ... | ... | ... | ... | ... | ... | ... | ... | ... |
| {N} | Go-Live |  |  |  |  |  |  |  |  |  | 🚀 |
| {N+1} | Soporte Post Go-Live |  |  |  |  |  |  |  |  | ██ | ██ |
| {N+2} | Reuniones Semanales | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ |

---

### Gantt por Fases (Opcional - Vista Resumen)

| Fase | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | ... | S{N} |
|------|----|----|----|----|----|----|----|----|-----|------|
| 🔧 Configuraciones | ██ | ██ | ██ | ██ | ▓▓ |  |  |  |  |  |
| 💻 Desarrollos |  | ██ | ██ | ██ | ██ | ██ | ██ |  |  |  |
| 📦 Migraciones |  |  |  | ██ | ██ | ██ |  |  |  |  |
| ✅ Pruebas |  |  |  |  |  | ██ | ██ | ██ |  |  |
| 📚 Formaciones |  |  |  |  |  | ▓▓ | ▓▓ | ██ | ██ |  |
| 🚀 Go-Live & Soporte |  |  |  |  |  |  |  |  | 🚀 | ██ |
| 📅 Reuniones | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ |

**Nota**: Esta vista resume las fases principales para dar una visión de alto nivel del proyecto.

---

### Hitos en el Gantt

Marca los hitos importantes en las semanas correspondientes:

| Semana | Hito | Descripción |
|--------|------|-------------|
| S1 | ✓ Entorno Listo | Configuraciones generales completadas |
| S{X} | ✓ Maestros OK | Clientes, productos, tarifas configurados |
| S{X} | ✓ Módulos Operativos | Ventas, compras, inventario funcionando |
| S{X} | ✓ Desarrollos OK | Todos los módulos custom en staging |
| S{X} | ✓ Datos Migrados | Migraciones completadas |
| S{X} | ✓ UAT Aprobado | Cliente valida sistema |
| S{X} | 🚀 **Go-Live** | Sistema en producción |
| S{X} | ✓ Proyecto Cerrado | Soporte finalizado |

```

---

## Reglas de Generación

### Columnas (Semanas)

- **Primera columna**: Número de tarea + Nombre de tarea
- **Siguientes columnas**: Una por cada semana del proyecto (S1, S2, S3...)
- **Formato de semana**: "SX" donde X es el número de semana (S1, S2, S3...)

**Ejemplo para proyecto de 12 semanas:**
```
| # | Tarea | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 | S11 | S12 |
```

### Filas (Tareas)

- **Una fila por tarea** de la tabla de planificación
- **Orden**: Mismo orden que en la tabla de planificación (cronológico)
- **Incluir tareas transversales**: Reuniones, desarrollos en paralelo, soporte

### Indicadores Visuales

| Indicador | Significado | Cuándo Usar |
|-----------|-------------|-------------|
| `██` | Tarea en ejecución completa | La tarea ocupa >50% de la semana |
| `▓▓` | Tarea en ejecución parcial | La tarea ocupa <50% de la semana |
| `──` | Espera / Dependencia | La tarea no puede empezar aún |
| `🚀` | Go-Live | Semana del go-live |
| `✓` | Hito completado | Marca hitos importantes |
| (vacío) | Sin actividad | La tarea no está activa esa semana |

### Cálculo de Duración en Semanas

**Fórmula básica**:
```
Semanas de tarea = ceil(Horas de tarea / Horas disponibles por semana)
```

**Ejemplo 1:**
- Tarea: 20 horas
- Horas disponibles/semana: 15h
- Semanas: ceil(20 / 15) = ceil(1.33) = **2 semanas**

**Ejemplo 2:**
- Tarea: 60 horas (Configuración de Ventas)
- Horas disponibles/semana: 20h
- Semanas: ceil(60 / 20) = **3 semanas**

**Horas disponibles por semana típicas:**
- Consultor a tiempo completo: 25-30h efectivas
- Desarrollador a tiempo completo: 30-35h efectivas
- Proyecto con múltiples tareas en paralelo: 15-20h por tarea

### Tareas Transversales

**Reuniones semanales**: Se marcan en TODAS las semanas del proyecto
```
| X | Reuniones Semanales | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ |
```

**Desarrollos en paralelo**: Se marcan durante su ventana completa de ejecución
```
| 3 | [DEV] Tarificación | | ██ | ██ | ██ | ██ | ██ | ██ | | | |
```
(Semanas 2-7 en este ejemplo)

**Soporte post go-live**: Se marca el mes posterior al go-live
```
| Y | Soporte Post Go-Live | | | | | | | | | ██ | ██ | ██ | ██ |
```
(Últimas 4 semanas en este ejemplo)

---

## Validaciones

Antes de entregar el Gantt, valida:

1. ✅ El número de columnas (semanas) coincide con la duración del proyecto
2. ✅ Cada tarea aparece en las semanas correctas según la tabla de planificación
3. ✅ No hay solapamientos prohibidos (ej. config y test del mismo módulo simultáneos)
4. ✅ Los solapamientos permitidos están correctamente representados (ej. config + desarrollo en paralelo)
5. ✅ El go-live (🚀) está en la semana correcta
6. ✅ Las tareas transversales (reuniones) aparecen en todas las semanas
7. ✅ Los hitos están marcados en las semanas correspondientes

---

## Ejemplo Completo

### Proyecto de 12 Semanas (100h)

```markdown
## Diagrama de Gantt (Vista Semanal)

**Leyenda:**
- `██` = Tarea en ejecución
- `🚀` = Go-Live

---

| # | Tarea | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 | S11 | S12 |
|---|-------|----|----|----|----|----|----|----|----|-----|-----|-----|-----|
| 1 | Configuraciones Generales | ██ |  |  |  |  |  |  |  |  |  |  |  |
| 2 | [CONFIG] CRM |  | ██ | ██ |  |  |  |  |  |  |  |  |  |
| 3 | [CONFIG] Ventas |  |  | ██ | ██ | ██ |  |  |  |  |  |  |  |
| 4 | [MIGR] Clientes |  |  |  |  | ██ |  |  |  |  |  |  |  |
| 5 | [MIGR] Productos |  |  |  |  | ██ |  |  |  |  |  |  |  |
| 6 | [CONFIG] Inventario |  |  |  |  |  | ██ | ██ |  |  |  |  |  |
| 7 | [MIGR] Stock inicial |  |  |  |  |  |  | ██ |  |  |  |  |  |
| 8 | [CONFIG] Contabilidad |  |  |  |  |  |  |  | ██ | ██ |  |  |  |
| 9 | [MIGR] Saldos contables |  |  |  |  |  |  |  |  | ██ |  |  |  |
| 10 | [TEST] UAT completo |  |  |  |  |  |  |  |  |  | ██ |  |  |
| 11 | [TRAIN] Formaciones |  |  |  |  |  |  |  |  |  | ██ |  |  |
| 12 | Go-Live |  |  |  |  |  |  |  |  |  |  | 🚀 |  |
| 13 | Soporte Post Go-Live |  |  |  |  |  |  |  |  |  |  | ██ | ██ |
| 14 | Reuniones Semanales | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ | ██ |

---

### Hitos del Proyecto

| Semana | Hito | Estado |
|--------|------|--------|
| S1 | ✓ Entorno Listo | Completado |
| S5 | ✓ Maestros OK | Completado |
| S7 | ✓ Módulos Operativos | Completado |
| S9 | ✓ Datos Migrados | Completado |
| S10 | ✓ UAT Aprobado | Completado |
| S11 | 🚀 Go-Live | En producción |
| S12 | ✓ Proyecto Cerrado | Finalizado |
```

---

## Consejos de Formato

### Para Proyectos Largos (>20 semanas)

Si el proyecto tiene muchas semanas, considera:

**Opción 1**: Dividir el Gantt en dos partes
```markdown
### Gantt - Primera Mitad (S1-S10)
| ... tabla con S1 a S10 ... |

### Gantt - Segunda Mitad (S11-S20)
| ... tabla con S11 a S20 ... |
```

**Opción 2**: Usar vista de fases resumida (como se muestra arriba)

### Para Proyectos con Muchos Desarrollos

Agrupa desarrollos similares en una sola fila si hay >5:
```
| X | [DEV] Desarrollos (3 módulos) | | ██ | ██ | ██ | ██ | ██ | ██ | | |
```

Y añade un desglose en notas:
```
**Nota**: Los desarrollos incluyen:
- Módulo de tarificación (S2-S7)
- Integración con transportistas (S3-S6)
- Informes personalizados (S5-S7)
```

### Alineación de Columnas

Asegura que las columnas estén alineadas para facilitar lectura:
- Usa espacios para alinear el contenido
- Markdown renderizado ajusta automáticamente, pero el código fuente debe ser legible

---

## Notas Finales

- El Gantt es una **representación visual** de la tabla de planificación - deben ser coherentes
- Si una tarea tiene dependencias, estas deben ser evidentes en el Gantt (no solapamiento)
- El Gantt ayuda al cliente a **visualizar el timeline** del proyecto de un vistazo
- Es normal que algunas tareas se solapen (ej. config + desarrollo) - el Gantt debe mostrarlo claramente
