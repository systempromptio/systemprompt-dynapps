# Horas por Tipo de Tarea

Conversion de pesos del analisis funcional a horas estimadas.

**Horas base, sin asistencia IA.** Aplicar los factores de `ai-factors.md` antes de la contingencia.

---

## Tabla de Conversion Base

| Peso | Descripcion | Rango Horas | Tiempo |
|------|-------------|-------------|--------|
| **1** | Muy Simple — Configuracion trivial | 0.5 - 1h | minutos - 1h |
| **2** | Simple — Config estandar, Studio/Auto basico | 2 - 4h | 1-4h |
| **3** | Medio — Config avanzada, Studio con logica, Migracion simple | 4 - 8h | 4h - 1 dia |
| **4** | Complejo — DEV pequeno/mediano, Migracion con transformaciones | 8 - 24h | 1-3 dias |
| **5** | Muy Complejo — DEV extenso, Migracion compleja | 24 - 80h | >3 dias |

Los rangos son amplios intencionalmente. El numero exacto depende del contexto.

---

## Factores por Tipo

### [CONFIG] Configuraciones
- Peso 1: 0.5-1h (checkbox, parametro simple)
- Peso 2: 2-3h (configuracion multi-paso)
- Peso 3: 4-6h (configuracion compleja con analisis)

### [STUDIO/AUTO] Studio y Automatizaciones
- Peso 2: 2-4h (1-3 campos custom simples, 1 automated action basica)
- Peso 3: 4-8h (4-10 campos custom, automated action con logica)
- Peso 4: 8-16h (>10 campos complejos, automated action con Python)

### [DEV] Desarrollos
- Peso 3: 8-16h (< 100 lineas, 1 modelo simple)
- Peso 4: 16-40h (< 500 lineas, 1-2 modelos, logica moderada)
- Peso 5: 40-120h (> 500 lineas, multiples modelos, integraciones)

Los desarrollos suelen necesitar MAS horas que la conversion base. Si el analisis indica complejidad tecnica alta, ajustar al extremo superior.

### [MIGR] Migraciones de Datos
- Peso 2: 2-4h (< 100 registros sin transformaciones)
- Peso 3: 4-12h (100-1000 registros con transformaciones simples)
- Peso 4: 12-32h (1000-10000 registros con transformaciones complejas)
- Peso 5: 32-80h (>10000 registros o reglas de negocio muy complejas)

Factores criticos: volumen datos, calidad datos origen, transformaciones, dependencias entre maestros.

### [TEST] Pruebas
- Peso 1: 1h (test simple de 1 funcionalidad)
- Peso 2: 2-3h (test de flujo completo sin muchas variantes)
- Peso 3: 4-6h (test exhaustivo de multiples escenarios o flujo E2E)

### [TRAIN] Formaciones
- Peso 2: 2-3h (formacion basica 1-2h a <5 usuarios)
- Peso 3: 4-8h (formacion completa 1 dia, incluye preparacion)
- Peso 4: 12-16h (formacion extensiva multi-sesion a multiples grupos)

---

## Reglas de Validacion

### Coherencia con Bolsa de Horas

| Diferencia | Estado | Accion |
|------------|--------|--------|
| < 10% | Excelente | Continuar |
| 10-15% | Aceptable | Revisar |
| > 15% | Requiere ajuste | Informar al usuario, priorizar o ampliar bolsa |

### Contingencia Recomendada

Reservar 10-15% de la bolsa total para imprevistos. Ver `contingencia-riesgo.md` para tabla detallada por tipo de proyecto.

---

## Casos Especiales

- **Requisito concreto de horas**: Si el usuario indica horas fijas para una tarea, respetar y ajustar las demas proporcionalmente
- **>3 desarrollos de peso 5**: Anadir 5-10% del total de desarrollo para coordinacion tecnica
- **Migraciones >10.000 registros**: Anadir 10-20% del tiempo de migracion para analisis de calidad de datos

---

## Regla de Oro

**Criterio experto > formula.** Los pesos son orientativos. La experiencia del implantador prevalece.
