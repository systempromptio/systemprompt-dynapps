# Plantilla: Tabla de Planificación

Esta plantilla define la estructura de la tabla de planificación que debe generarse como output del skill.

---

## Estructura de la Tabla

```markdown
# PLANIFICACIÓN DEL PROYECTO - {NOMBRE_CLIENTE}

**Consultor**: {Nombre del Consultor Líder}
**Bolsa de Horas Total**: {XXX}h
**Duración del Proyecto**: {X} meses / {XX} semanas
**Fecha de Inicio**: {DD/MM/YYYY}
**Fecha Estimada de Go-Live**: {DD/MM/YYYY}
**Fecha de Fin**: {DD/MM/YYYY}

---

## Tabla de Planificación de Tareas

| # | Nombre de Tarea | Módulo/Categoría | Peso | Horas | Fecha Inicio | Fecha Fin |
|---|----------------|------------------|------|-------|--------------|-----------|
| 1 | {Nombre de la tarea} | {Módulo o categoría} | {1-5} | {XX}h | {DD/MM/YYYY} | {DD/MM/YYYY} |
| 2 | ... | ... | ... | ... | ... | ... |
| ... | ... | ... | ... | ... | ... | ... |
| **TOTAL** | | | | **{XXX}h** | | |

---

## Resumen por Categoría

| Categoría | Horas Totales | % del Proyecto |
|-----------|---------------|----------------|
| [CONFIG] Configuraciones | {XX}h | {XX}% |
| [STUDIO/AUTO] Personalizaciones | {XX}h | {XX}% |
| [DEV] Desarrollos | {XX}h | {XX}% |
| [MIGR] Migraciones | {XX}h | {XX}% |
| [TEST] Pruebas | {XX}h | {XX}% |
| [TRAIN] Formaciones | {XX}h | {XX}% |
| [SOPORTE] Post Go-Live | {XX}h | {XX}% |
| [REUNIONES] Semanales | {XX}h | {XX}% |
| **TOTAL** | **{XXX}h** | **100%** |

---

## Resumen por Módulo

| Módulo | Horas Totales | % del Proyecto |
|--------|---------------|----------------|
| Configuraciones Generales | {XX}h | {XX}% |
| {Módulo 1} | {XX}h | {XX}% |
| {Módulo 2} | {XX}h | {XX}% |
| ... | ... | ... |
| Soporte y Reuniones | {XX}h | {XX}% |
| **TOTAL** | **{XXX}h** | **100%** |

---

## Hitos del Proyecto

| Hito | Descripción | Fecha Estimada | Semana |
|------|-------------|----------------|--------|
| **M1** | Entorno Odoo operativo | {DD/MM/YYYY} | S{X} |
| **M2** | Maestros configurados (Clientes, Productos) | {DD/MM/YYYY} | S{X} |
| **M3** | Módulos operativos funcionando | {DD/MM/YYYY} | S{X} |
| **M4** | Desarrollos completados (si aplica) | {DD/MM/YYYY} | S{X} |
| **M5** | Migraciones completadas | {DD/MM/YYYY} | S{X} |
| **M6** | UAT aprobado | {DD/MM/YYYY} | S{X} |
| **M7** | **Go-Live** | {DD/MM/YYYY} | S{X} |
| **M8** | Cierre del proyecto | {DD/MM/YYYY} | S{X} |

---

## Detalle de Tareas

{Para cada tarea agrupada y tarea detallada, incluir su checklist de implementación}

### Tarea 1: Configuraciones Generales

**Horas totales**: 12h | **Fecha**: {DD/MM/YYYY} - {DD/MM/YYYY}

**Checklist de implementación**:
- [ ] Subtarea 1 (breve descripción)
- [ ] Subtarea 2 (breve descripción)
- [ ] Subtarea N (breve descripción)

---

### Tarea 2: {Nombre del Módulo, ej. Ventas}

**Horas totales**: {XX}h | **Fecha**: {DD/MM/YYYY} - {DD/MM/YYYY}

**Checklist de implementación**:
- [ ] [CONFIG] Descripción de la tarea de configuración
- [ ] [STUDIO/AUTO] Descripción de la automatización
- [ ] [MIGR] Descripción de la migración
- [ ] [TEST] Descripción de las pruebas
- [ ] [TRAIN] Descripción de la formación

{Repetir para cada tarea agrupada}

---

### Tarea X: [DEV] {Nombre del Desarrollo Detallado}

**Horas totales**: {XX}h | **Fecha**: {DD/MM/YYYY} - {DD/MM/YYYY}

**Checklist de implementación** (extraído de la sección 7 del análisis):
- [ ] [DEV] Componente 1 del desarrollo
- [ ] [DEV] Componente 2 del desarrollo
- [ ] [TEST] Validación funcional
- [ ] [DOC] Documentación técnica

{Repetir para cada tarea detallada de la sección 7}

---

## Notas Importantes

{Incluir aquí cualquier nota relevante sobre:}
- {Riesgos identificados en la planificación}
- {Dependencias críticas del cliente}
- {Requisitos de disponibilidad del equipo del cliente}
- {Puntos de decisión importantes}
```

---

## Reglas de Formato

### Columnas de la Tabla Principal

1. **#**: Número secuencial de la tarea
2. **Nombre de Tarea**: Descripción breve y clara de la tarea (máx. 60 caracteres)
3. **Módulo/Categoría**: Indica el módulo Odoo (ej. "Ventas", "CRM") o categoría genérica (ej. "Configuraciones Generales", "Soporte")
4. **Peso**: El peso original del análisis funcional (1-5)
5. **Horas**: Horas estimadas para esa tarea
6. **Fecha Inicio**: Fecha de inicio en formato DD/MM/YYYY
7. **Fecha Fin**: Fecha de finalización en formato DD/MM/YYYY

### Tipos de Tareas

Hay **tres tipos** de tareas en la planificación:

1. **Tareas Agrupadas por Módulo** (ej. "Ventas", "Compras", "Inventario")
   - Representan TODO el trabajo de configuración/migración/testing de un módulo
   - NO tienen prefijo [CONFIG], [DEV], etc. - solo el nombre del módulo
   - Su checklist interno (en apartado "Detalle de Tareas") contiene todas las subtareas

2. **Tareas Detalladas** (ej. "[DEV] Módulo de Tarificación Dinámica")
   - Son tareas complejas que requieren descripción específica (sección 7 del análisis)
   - SÍ tienen prefijo [DEV], [MIGR], etc.
   - Tienen su propio checklist interno en el apartado "Detalle de Tareas"

3. **Tareas Genéricas** (Configuraciones Generales, Go-Live, Soporte, Reuniones)
   - Tareas transversales que siempre se incluyen
   - Pueden tener o no checklist según aplique

### Orden de las Tareas

Las tareas deben aparecer en **orden cronológico** (por fecha de inicio), no por módulo ni por categoría.

**Ejemplo de orden correcto:**
```
1. Configuraciones Generales (Semana 1)
2. Ventas (Semana 2-5)  ← Tarea agrupada
3. [DEV] Tarificación Dinámica (Semana 2-8)  ← Desarrollo en paralelo (tarea detallada)
4. Compras (Semana 5-7)  ← Tarea agrupada
5. [MIGR] Clientes (Semana 6)  ← Migración compleja (tarea detallada)
...
```

### Formato de Nombres de Tarea

**Formato general**: `[PREFIJO] Descripción clara`

**Ejemplos:**
- `[CONFIG] Configuración de maestros de productos`
- `[DEV] Módulo de tarificación dinámica`
- `[MIGR] Migración de clientes desde XGest`
- `[TEST] Validación de flujo completo de ventas`
- `[TRAIN] Formación a comerciales en CRM`

**Excepciones (sin prefijo):**
- `Configuraciones Generales`
- `Soporte en Salida a Producción`
- `Reuniones / Formaciones Semanales`
- `Go-Live`

### Fechas

- **Formato**: DD/MM/YYYY (ej. 15/03/2026)
- **Lunes a viernes**: Planifica en semanas laborables
- **Evita festivos**: Si conoces festivos locales del cliente, evítalos

### Fila TOTAL

La última fila de la tabla DEBE ser:
```
| **TOTAL** | | | | **{XXX}h** | | |
```

Con la suma exacta de todas las horas del proyecto.

---

## Validaciones

Antes de entregar la tabla, valida:

1. ✅ La suma de horas coincide con la bolsa (±10%)
2. ✅ Las fechas están en orden cronológico
3. ✅ No hay tareas que empiecen antes de que terminen sus dependencias
4. ✅ El go-live está 1 mes antes del fin del proyecto
5. ✅ Todas las tareas genéricas obligatorias están incluidas
6. ✅ Los resúmenes por categoría y módulo suman 100%
7. ✅ **Las tareas están agrupadas por módulo** (no granulares)
8. ✅ Se ha incluido el apartado "Detalle de Tareas" con checklists

---

## Ejemplo Completo de Salida

```markdown
# PLANIFICACIÓN DEL PROYECTO - UNIKAS

**Consultor**: Juan García (Indaws)
**Bolsa de Horas Total**: 300h
**Duración del Proyecto**: 5 meses / 20 semanas
**Fecha de Inicio**: 03/03/2026
**Fecha Estimada de Go-Live**: 28/07/2026
**Fecha de Fin**: 28/08/2026

---

## Tabla de Planificación de Tareas

| # | Nombre de Tarea | Módulo/Categoría | Peso | Horas | Fecha Inicio | Fecha Fin |
|---|----------------|------------------|------|-------|--------------|-----------|
| 1 | Configuraciones Generales | Generales | - | 12h | 03/03/2026 | 07/03/2026 |
| 2 | [CONFIG] Ventas - Maestros y flujos | Ventas | 20 | 60h | 10/03/2026 | 28/03/2026 |
| 3 | [DEV] Módulo de tarificación dinámica | Ventas | 5 | 80h | 10/03/2026 | 09/05/2026 |
| 4 | [MIGR] Migración de clientes | Ventas | 5 | 50h | 31/03/2026 | 11/04/2026 |
| 5 | [CONFIG] Compras - Configuración completa | Compras | 15 | 40h | 14/04/2026 | 02/05/2026 |
| ... | ... | ... | ... | ... | ... | ... |
| 18 | Go-Live | Producción | - | 8h | 28/07/2026 | 28/07/2026 |
| 19 | Soporte en Salida a Producción | Soporte | - | 24h | 29/07/2026 | 28/08/2026 |
| 20 | Reuniones / Formaciones Semanales | Transversal | - | 60h | 03/03/2026 | 28/08/2026 |
| **TOTAL** | | | | **300h** | | |

---

## Resumen por Categoría

| Categoría | Horas Totales | % del Proyecto |
|-----------|---------------|----------------|
| [CONFIG] Configuraciones | 110h | 37% |
| [DEV] Desarrollos | 80h | 27% |
| [MIGR] Migraciones | 50h | 17% |
| [TEST] Pruebas | 20h | 7% |
| [SOPORTE] Post Go-Live | 24h | 8% |
| [REUNIONES] Semanales | 60h | 20% |
| **TOTAL** | **344h** | **115%** ← ⚠️ Sobrepasa bolsa, requiere ajuste

---

## Detalle de Tareas

### Tarea 1: Configuraciones Generales

**Horas totales**: 12h | **Fecha**: 03/03/2026 - 07/03/2026

**Checklist de implementación**:
- [ ] Alta de compañía, usuarios y permisos básicos
- [ ] Configuración de correo saliente y plantillas
- [ ] Activación de módulos base
- [ ] Configuración de idiomas y zonas horarias

---

### Tarea 2: Ventas - Maestros y flujos

**Horas totales**: 60h | **Fecha**: 10/03/2026 - 28/03/2026

**Checklist de implementación**:
- [ ] [CONFIG] Activar módulo de Ventas y CRM
- [ ] [CONFIG] Configurar equipos de ventas y pipeline
- [ ] [CONFIG] Configurar productos y tarifas
- [ ] [STUDIO/AUTO] Campos personalizados en clientes
- [ ] [STUDIO/AUTO] Automatizaciones de notificaciones
- [ ] [TEST] Validación de flujo completo de ventas

---

### Tarea 3: [DEV] Módulo de tarificación dinámica

**Horas totales**: 80h | **Fecha**: 10/03/2026 - 09/05/2026

**Checklist de implementación**:
- [ ] [DEV] Crear modelo de datos de tarifas
- [ ] [DEV] Implementar lógica de cálculo de precios
- [ ] [DEV] Integrar con líneas de pedido
- [ ] [DEV] Crear vistas de administración
- [ ] [TEST] Validación con múltiples escenarios
- [ ] [DOC] Documentación funcional del módulo

{... continuar para cada tarea ...}
```

**Nota**: Este ejemplo muestra un caso donde la planificación excede la bolsa. El skill debe detectar esto y sugerir ajustes al usuario.
