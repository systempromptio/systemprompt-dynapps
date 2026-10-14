# Reglas de Cálculo de Horas

Este documento define el algoritmo para calcular horas estimadas a partir de los pesos de las tareas del análisis funcional.

---

## Algoritmo Base

Los pesos asignados en los checklists sirven como **guía orientativa**, pero debes aplicar tu **criterio experto** como implantador senior de Odoo para ajustar.

### Tabla de Conversión Orientativa

| Peso | Descripción | Rango de Horas | Tiempo Estimado |
|------|-------------|----------------|-----------------|
| **1** | Muy Simple - Configuración trivial | 0.5 - 1h | minutos - 1h |
| **2** | Simple - Config. estándar clara, Studio/Auto básico | 2 - 4h | 1-4h |
| **3** | Medio - Config. avanzada, Studio/Auto con lógica, Migración simple | 4 - 8h | 4h - 1 día |
| **4** | Complejo - DEV pequeño/mediano, Migración con transformaciones | 8 - 24h | 1-3 días |
| **5** | Muy Complejo - DEV extenso, Migración compleja | 24 - 80h | >3 días |

**Nota crítica**: Los rangos son amplios intencionalmente. El número exacto depende de múltiples factores.

### Factor IA

Los rangos de esta tabla son horas **base, sin asistencia IA**. Aplicar el factor después de los
rangos y **antes** de la contingencia. Criterio y factores: delega en la skill `odoo-effort-estimator`.

Las partidas [CONFIG], [STUDIO/AUTO], [TRAIN] y la gestión de proyecto **no llevan factor**.

---

## Factores de Ajuste

### Por Tipo de Tarea

#### [CONFIG] - Configuraciones
- **Peso 1**: 0.5-1h (checkbox, parámetro simple)
- **Peso 2**: 2-3h (configuración multi-paso, varios parámetros relacionados)
- **Peso 3**: 4-6h (configuración compleja que requiere análisis)

**Ejemplos:**
- Activar módulo: 0.5h (Peso 1)
- Configurar tarifas estándar: 2h (Peso 2)
- Configurar SII: 6h (Peso 3)

#### [STUDIO/AUTO] - Studio y Automatizaciones
- **Peso 2**: 2-4h (1-3 campos custom simples, 1 automated action básica)
- **Peso 3**: 4-8h (4-10 campos custom, automated action con lógica)
- **Peso 4**: 8-16h (>10 campos, automated action de un paso sin Python). La **lógica no trivial / Python va a [DEV]**, no aquí (criterio en `odoo-classification-guide`).

**Ejemplos:**
- 3 campos custom en cliente: 3h (Peso 2)
- 8 campos custom + lógica: 6h (Peso 3)
- Automated action con Python: 12h (Peso 4)

#### [DEV] - Desarrollos
- **Peso 3**: 8-16h (< 100 líneas, 1 modelo simple, sin lógica compleja)
- **Peso 4**: 16-40h (< 500 líneas, 1-2 modelos, lógica moderada)
- **Peso 5**: 40-120h (> 500 líneas, múltiples modelos, lógica compleja, integraciones)

**Ejemplos:**
- Herencia simple de modelo: 12h (Peso 3)
- Módulo de tarificación básica: 30h (Peso 4)
- Sistema de tarificación complejo con integraciones: 80h (Peso 5)

**Consideración especial**: Los desarrollos suelen necesitar **más horas** que la conversión base. Si el análisis indica complejidad técnica alta, ajusta al extremo superior del rango.

#### [MIGR] - Migraciones
- **Peso 2**: 2-4h (< 100 registros sin transformaciones)
- **Peso 3**: 4-12h (100-1000 registros con transformaciones simples)
- **Peso 4**: 12-32h (1000-10000 registros con transformaciones complejas)
- **Peso 5**: 32-80h (>10000 registros o con reglas de negocio muy complejas)

**Factores críticos:**
- **Volumen de datos**: >10.000 registros aumenta considerablemente el esfuerzo
- **Calidad de datos origen**: Datos sucios requieren limpieza manual intensiva
- **Transformaciones necesarias**: Mapeos complejos, normalización, deduplicación
- **Dependencias**: Migración de maestros debe hacerse antes que transacciones

**Ejemplos:**
- 50 proveedores desde Excel limpio: 3h (Peso 2)
- 2.000 clientes con normalización empresa/contacto: 20h (Peso 4)
- 15.000 leads con deduplicación y clasificación: 60h (Peso 5)

#### [TEST] - Pruebas
- **Peso 1**: 1h (Test simple de 1 funcionalidad)
- **Peso 2**: 2-3h (Test de flujo completo sin muchas variantes)
- **Peso 3**: 4-6h (Test exhaustivo de múltiples escenarios o flujo end-to-end)

#### [TRAIN] - Formaciones
- **Peso 2**: 2-3h (Formación básica de 1-2 horas a <5 usuarios)
- **Peso 3**: 4-8h (Formación completa de 1 día a grupo de usuarios) - incluye preparación
- **Peso 4**: 12-16h (Formación extensiva multi-sesión a múltiples grupos)

---

## Reglas de Validación

### 1. Coherencia con Bolsa de Horas

La suma total de horas estimadas debe ser coherente con la bolsa de horas del proyecto:

- **Diferencia < 10%**: ✅ Excelente ajuste
- **Diferencia 10-15%**: ⚠️ Aceptable, pero revisar
- **Diferencia > 15%**: ❌ Requiere ajuste o comunicación al usuario

**Acciones según diferencia:**

| Situación | Acción Recomendada |
|-----------|-------------------|
| **Horas calculadas > Bolsa (+15%)** | Informar al usuario. Sugerir priorización de tareas o aumentar bolsa |
| **Horas calculadas < Bolsa (-15%)** | Sugerir dónde usar horas restantes: más testing, más formación, margen de contingencia |

### 2. Contingencia Recomendada

Reserva un **margen de contingencia del 10-15%** de la bolsa total para:
- Imprevistos técnicos
- Reuniones no planificadas
- Refinamiento de requisitos
- Soporte extra en go-live

---

## Casos Especiales

### Tareas con Requisitos Específicos de Horas

Si el usuario indica que una tarea específica tiene un requisito concreto de horas (ej. "el desarrollo X debe hacerse en 40 horas"), **respeta ese valor** y ajusta las demás tareas proporcionalmente.

### Proyectos con Múltiples Desarrollos Complejos

Si el proyecto tiene >3 desarrollos de peso 5, considera:
- Planificar en paralelo si hay múltiples desarrolladores
- Añadir horas de coordinación técnica (5-10% del total de desarrollo)
- Reservar tiempo para code review y refactoring

### Proyectos con Migraciones Masivas

Si hay migraciones >10.000 registros:
- Añade tiempo para análisis de calidad de datos (10-20% del tiempo de migración)
- Considera migración por lotes con validaciones intermedias
- Reserva tiempo para limpieza post-migración

---

## Ejemplos de Aplicación

### Ejemplo 1: Proyecto Pequeño (100h, 3 meses)

**Análisis funcional:**
- Ventas: 8 tareas CONFIG (peso 2 cada una)
- CRM: 5 tareas CONFIG (peso 1-2)
- 1 desarrollo simple (peso 3)
- Migración de 500 clientes (peso 3)

**Cálculo:**
- Ventas: 8 × 2.5h = 20h
- CRM: 5 × 2h = 10h
- Desarrollo: 12h (peso 3)
- Migración: 8h (peso 3)
- Config generales: 12h
- Reuniones/formaciones: 3h/semana × 12 semanas = 36h
- Soporte go-live: 18h (< 100h total)

**Total**: 116h → Sobrepasa bolsa en 16%
**Ajuste**: Reducir horas de reuniones a 2.5h/semana = 100h ✅

### Ejemplo 2: Proyecto Grande (500h, 6 meses)

**Análisis funcional:**
- 6 módulos principales con CONFIG/STUDIO (peso total: 80)
- 3 desarrollos complejos (peso 5, 4, 4)
- 2 migraciones grandes (peso 5, 4)

**Cálculo:**
- Módulos: 80 puntos × ~4h/punto = 320h
- Desarrollos: 60h + 30h + 30h = 120h
- Migraciones: 50h + 25h = 75h
- Config generales: 12h
- Reuniones: 3h × 24 semanas = 72h
- Soporte go-live: 32h (400-700h rango)

**Total**: 631h → Sobrepasa bolsa en 26%
**Ajuste necesario**: Priorizar funcionalidades o ampliar bolsa

---

## Notas Finales

- **Criterio experto > fórmula**: Los pesos son orientativos. Tu experiencia como implantador debe prevalecer.
- **Lee el análisis completo**: Los detalles en las tareas detalladas (sección 7) pueden indicar mayor complejidad.
- **Considera el contexto**: Un cliente con recursos técnicos propios puede requerir menos horas de formación.
- **Valida con el equipo**: Si dudas en la estimación, consulta con otros implantadores o el equipo técnico.
