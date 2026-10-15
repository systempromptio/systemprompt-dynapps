# Reglas de Planificación Temporal

Este documento define las reglas para distribuir tareas en el tiempo y crear el cronograma del proyecto.

---

## Principios Fundamentales

### 1. Secuencialidad con Solapamiento Controlado

**Modelo híbrido Waterfall**: Las fases tienen dependencias claras, pero se permiten solapamientos cuando hay equipos independientes.

**Regla de oro**: Una tarea NO puede empezar si sus dependencias no están completadas (o en progreso avanzado).

### 2. Recursos Multidisciplinares

**Equipos típicos en un proyecto Odoo:**
- **Consultor funcional** (líder del proyecto)
- **Desarrollador** (si hay [DEV])
- **Cliente** (usuarios clave para validación)

**Aprovechamiento de recursos**: Consultor y desarrollador pueden trabajar en paralelo en tareas independientes.

### 3. Carga de Trabajo Realista

**Horas efectivas por semana:**
- Consultor: **25-30h efectivas** (de 40h laborables)
- Desarrollador: **30-35h efectivas** (más enfocado en tareas técnicas)

**Razón**: Las 10-15h restantes se destinan a reuniones internas, administración, imprevistos.

---

## Reglas de Secuencia por Tipo de Tarea

### [CONFIG] - Configuraciones

**Orden interno de configuraciones:**
1. **Maestros primero**: Clientes, Productos, Proveedores
2. **Después flujos**: Ventas, Compras, Inventario
3. **Último financiero**: Facturación, Contabilidad

**Razón**: Los flujos dependen de que existan maestros. La contabilidad depende de que los flujos estén definidos.

**Duración por módulo típico**: 1-3 semanas

**Solapamiento**: Módulos independientes (ej. CRM y Compras) pueden configurarse en paralelo si hay recursos.

---

### [STUDIO/AUTO] - Studio y Automatizaciones

**Posición**: Se pueden hacer durante o inmediatamente después de la configuración del módulo.

**Regla**: NO hacer Studio/Auto ANTES de configurar el módulo estándar - primero valida lo que da Odoo.

**Duración típica**: Pocas horas por módulo, se integran con la configuración.

---

### [DEV] - Desarrollos

**Posición**: Pueden empezar temprano y ejecutarse en paralelo con configuraciones.

**Excepciones que deben esperar:**
- Desarrollos que heredan modelos: Necesitan que el módulo base esté instalado
- Desarrollos que dependen de maestros: Necesitan ejemplos reales del cliente

**Duración típica**: Variable (1-8 semanas según complejidad)

**Principio de paralelización**: Si hay >1 desarrollo, asignarlos a semanas solapadas pero no idénticas (evita cuellos de botella en code review).

---

### [MIGR] - Migraciones

**Orden obligatorio:**
1. **Maestros primero** (Clientes, Productos, Proveedores)
2. **Dependencias** (Tarifas, Categorías)
3. **Transacciones** (Pedidos, Facturas)
4. **Saldos contables** (ÚLTIMO - requiere todo lo anterior)

**Posición temporal**: DESPUÉS de configurar el módulo correspondiente.

**Ejemplo:**
- Config de Ventas (semanas 2-3)
- Migración de Clientes (semana 4)
- Migración de Productos (semana 4)
- Config de Contabilidad (semanas 8-9)
- Migración de Saldos Contables (semana 10)

**Duración típica**: 1-3 días por migración (según volumen)

---

### [TEST] - Pruebas

**Posición**: DESPUÉS de completar Config + Dev + Migr del módulo.

**Regla crítica**: NO puedes probar un módulo que no está completo (config + datos + desarrollos).

**Duración típica**: 1-2 semanas para todas las pruebas (pueden agruparse al final).

**Tipos de testing:**
- **Funcional**: Por el consultor
- **UAT**: Por el cliente (usuarios clave)
- **Técnico**: Por desarrolladores (si hay integraciones)

---

### [TRAIN] - Formaciones

**Posición**: Flexible, pueden empezar a mitad de proyecto.

**Estrategia recomendada:**
- **Formaciones tempranas** (semana 50-60% del proyecto): Usuarios clave (alpha testers)
- **Formaciones masivas** (semana 80-90%): Todos los usuarios
- **Formaciones de cierre** (última semana antes de go-live): Administradores

**Duración por sesión**: 2-4h (incluye preparación)

**Solapamiento**: Pueden hacerse durante la fase de testing.

---

## Reglas de Distribución Temporal

### Inicio del Proyecto (Semanas 1-2)

**Tareas obligatorias:**
- [CONFIG] Configuraciones Generales (12h)
- [Reuniones] Primera reunión de kick-off (3h)

**Output**: Entorno Odoo operativo.

---

### Primera Mitad del Proyecto (Semanas 20-50%)

**Tareas típicas:**
- [CONFIG] Configuración de módulos principales (Ventas, CRM, Compras)
- [STUDIO/AUTO] Campos personalizados y automatizaciones
- [DEV] Inicio de desarrollos complejos (si los hay)
- [Reuniones] Semanales (3h/semana)

**Output**: Módulos principales configurados.

---

### Segunda Mitad del Proyecto (Semanas 50-80%)

**Tareas típicas:**
- [CONFIG] Configuración de módulos financieros (Facturación, Contabilidad)
- [MIGR] Migraciones de maestros y transacciones
- [DEV] Finalización de desarrollos
- [TEST] Inicio de pruebas funcionales
- [TRAIN] Formaciones a usuarios clave
- [Reuniones] Semanales (3h/semana)

**Output**: Sistema completo listo para UAT.

---

### Fase de Cierre (Semanas 80-90%)

**Tareas típicas:**
- [TEST] UAT completo
- [TRAIN] Formaciones masivas a todos los usuarios
- [CONFIG] Ajustes finales de configuración
- [MIGR] Migración final de saldos contables
- [Reuniones] Semanales (3h/semana)

**Output**: Sistema validado y usuarios formados, listo para go-live.

---

### Go-Live (Semana 90%)

**Evento puntual**: 1 día (8h de soporte intensivo)

**Tareas:**
- Migración final de datos frescos
- Validación final
- Arranque en producción
- Soporte intensivo durante el día

**Posición**: 1 mes antes de la finalización del proyecto.

---

### Post Go-Live (Últimas 4 semanas)

**Tareas obligatorias:**
- [Soporte] Soporte en salida a producción (18-48h según bolsa)
- [Reuniones] Semanales (3h/semana)

**Distribución del soporte:**
- Semana 1 post go-live: 40% de las horas
- Semana 2: 30%
- Semana 3: 20%
- Semana 4: 10%

**Output**: Sistema estabilizado, proyecto cerrado.

---

## Reglas de Solapamiento

### ✅ Permitido

| Tarea A | Tarea B | Condición |
|---------|---------|-----------|
| [CONFIG] Módulo A | [CONFIG] Módulo B | Si no hay dependencias entre módulos |
| [CONFIG] Cualquiera | [DEV] Cualquiera | Siempre (equipos distintos) |
| [CONFIG] Módulo A | [MIGR] Módulo A | Si config está >80% completa |
| [TEST] Módulo A | [CONFIG] Módulo B | Si módulos independientes |
| [TRAIN] | Cualquiera | Formaciones son flexibles |
| [Reuniones] | TODO | Las reuniones son transversales |

### ❌ NO Permitido

| Tarea A | Tarea B | Razón |
|---------|---------|-------|
| [CONFIG] Módulo A | [TEST] Módulo A | No puedes probar lo que no está configurado |
| [MIGR] Clientes | [MIGR] Pedidos | Los pedidos requieren que existan clientes |
| [MIGR] Módulo A | [TEST] Módulo A | No puedes probar sin datos |
| [DEV] X | [TEST] X | No puedes probar código no terminado |
| Cualquiera | Go-Live | El go-live es un evento único |

---

## Algoritmo de Distribución Temporal

### Paso 1: Ordenar Tareas por Prioridad

1. [CONFIG] Configuraciones Generales
2. [CONFIG] Maestros (Clientes, Productos, Proveedores)
3. [CONFIG] Módulos operativos (Ventas, Compras, Inventario)
4. [MIGR] Maestros
5. [CONFIG] Módulos financieros (Facturación, Contabilidad)
6. [MIGR] Transacciones y saldos
7. [TEST] Todos los módulos
8. [TRAIN] Formaciones masivas
9. Go-Live
10. [Soporte] Post go-live

**Desarrollos [DEV]**: Se insertan en paralelo desde la semana 2-3 hasta semana 70-80%.

**Reuniones**: Transversales, todas las semanas.

---

### Paso 2: Calcular Carga Semanal

**Fórmula**:
```
Horas disponibles por semana = (Bolsa total - Tareas genéricas) / Número de semanas de trabajo
```

**Ejemplo**:
- Bolsa total: 300h
- Tareas genéricas: 96h (config generales 12h + soporte 24h + reuniones 60h)
- Disponible para módulos: 204h
- Duración proyecto: 20 semanas
- Carga semanal promedio: 204h / 20 = **10.2h/semana** (sin contar reuniones ni soporte)

**Validación**: Si la carga semanal calculada es >30h, hay sobrecarga - sugiere ampliar duración o bolsa.

---

### Paso 3: Asignar Fechas

1. **Semana 1-2**: Configuraciones generales
2. **Semanas siguientes**: Distribuir [CONFIG] + [STUDIO/AUTO] + [MIGR] secuencialmente
3. **Semanas 2-N-4**: [DEV] en paralelo (si hay)
4. **Semanas N-3 a N-1**: [TEST] + [TRAIN]
5. **Semana N-1**: Go-Live
6. **Semanas N-1 a N**: Soporte post go-live

Donde N = número total de semanas del proyecto.

---

### Paso 4: Ajustar por Recursos

Si hay **un solo consultor**:
- NO solapar [CONFIG] de módulos distintos
- Secuencia pura: Módulo A → Módulo B → Módulo C

Si hay **múltiples consultores** (o consultor + cliente muy disponible):
- Permitir solapamiento de 2-3 módulos simultáneos
- Reducir duración total del proyecto

Si hay **desarrolladores**:
- Solapar [DEV] con [CONFIG] desde semana 2-3
- Asignar desarrollos en franjas amplias (ej. 4-8 semanas por desarrollo complejo)

---

## Ejemplo Completo de Distribución

### Proyecto de 300h en 20 semanas

**Tareas genéricas:**
- Config generales: 12h (semana 1)
- Soporte post go-live: 24h (semanas 20-21)
- Reuniones: 3h × 20 semanas = 60h (transversal)

**Disponible para módulos**: 204h

**Módulos a implantar**:
- Ventas (60h)
- Compras (40h)
- Inventario (50h)
- Contabilidad (54h)

**Cronograma:**

| Semana | Tareas | Horas |
|--------|--------|-------|
| 1 | Config generales + Reunión | 12h + 3h = 15h |
| 2-4 | [CONFIG] Ventas + Reuniones | 60h / 3 = 20h/semana |
| 5-6 | [MIGR] Clientes + Productos + Reuniones | 16h + 3h = 19h/semana |
| 7-9 | [CONFIG] Compras + Reuniones | 40h / 3 = 13h/semana |
| 10-12 | [CONFIG] Inventario + Reuniones | 50h / 3 = 17h/semana |
| 13 | [MIGR] Stock inicial + Reuniones | 8h + 3h = 11h |
| 14-16 | [CONFIG] Contabilidad + Reuniones | 54h / 3 = 18h/semana |
| 17 | [MIGR] Saldos contables + Reuniones | 20h + 3h = 23h |
| 18 | [TEST] UAT completo + Reuniones | 12h + 3h = 15h |
| 19 | [TRAIN] Formaciones + Go-Live + Reuniones | 8h + 8h + 3h = 19h |
| 20 | [Soporte] Post go-live + Reuniones | 24h + 3h = 27h |

**Total**: 300h ✅

---

## Validaciones Finales

### Check 1: Dependencias Respetadas

- ✅ Config generales antes que todo
- ✅ Maestros antes que transacciones
- ✅ Config antes que migración
- ✅ Migración antes que testing
- ✅ Testing antes que go-live
- ✅ Go-live antes que soporte

### Check 2: Carga Semanal Realista

- ⚠️ Ninguna semana >35h efectivas para el consultor
- ⚠️ Ninguna semana >40h efectivas para el desarrollador

### Check 3: Solapamientos Correctos

- ✅ Solo se solapan tareas permitidas
- ❌ No hay solapamientos prohibidos

### Check 4: Bolsa Respetada

- ✅ Suma total de horas = Bolsa ±10%

---

## Notas Finales

- **Flexibilidad**: El cronograma puede ajustarse durante el proyecto (reuniones semanales para revisar)
- **Buffer**: Reserva 10-15% de margen para imprevistos - no planifiques al 100% de capacidad
- **Comunicación**: El cronograma debe validarse con el cliente - puede haber restricciones de disponibilidad
- **Iteraciones**: Si una fase se retrasa, ajusta las siguientes - no comprimas sin razón
