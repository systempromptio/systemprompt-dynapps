# Métricas, HR Health Score y Plantilla — HR People Report

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-001-REF-002

Referencia de detalle del informe mensual. La definición canónica de cada KPI (fórmula,
fuente HiBob/Odoo, umbrales) vive en `kpi_definitions.md`. Este documento resume el catálogo,
documenta el HR Health Score y define la estructura del informe.

> Las métricas NUNCA se calculan a mano. Todas las fórmulas las aplica
> `scripts/calculate_hr_health.py`, que lee los pesos y umbrales de `lib/config.py`.

---

## HR Health Score (0-100)

Score ponderado por 5 dimensiones (pesos en `lib/config.py` → `HR_HEALTH_WEIGHTS`):

| Dimensión | Peso | Mide |
|-----------|------|------|
| Retención | 30% | Rotación voluntaria/involuntaria, tendencia, departamentos críticos |
| Staffing | 25% | Cobertura de posiciones, time-to-fill, vacantes abiertas |
| Engagement | 20% | Antigüedad media, tasa de promoción, movilidad interna |
| Compliance | 15% | Documentación al día, formaciones obligatorias, evaluaciones completadas |
| Diversidad | 10% | Distribución por género, ubicación, tipo de contrato |

`score = Σ (dimension_score × peso)`. El script deriva el escenario de diagnóstico y las
alertas automáticas a partir del score y los umbrales.

---

## Catálogo de KPIs (15+)

Detalle completo (fórmula, fuente, umbrales) en `kpi_definitions.md`.

1. Headcount total y FTE
2. Rotación voluntaria (%)
3. Rotación involuntaria (%)
4. Rotación total (%)
5. Antigüedad media (meses)
6. Nuevas incorporaciones (mes)
7. Bajas (mes)
8. Time-to-fill (días)
9. Distribución por departamento
10. Distribución por ubicación
11. Tasa de ausencia (%)
12. Posiciones abiertas
13. Tasa de promoción (%)
14. Movilidad interna (%)
15. Ratio manager:empleado
16. Coste medio por empleado (si disponible en Odoo)

**Rotación anualizada** (regla clave): `(bajas_periodo / headcount_medio) * (12 / meses_periodo) * 100`.
No confundir con rotación mensual sin anualizar.

---

## Estructura del informe (8 secciones)

Plantilla en `templates/informe_hr.md`. Generar siempre desde el JSON calculado.

1. **Dashboard HR** — Score global, KPIs principales, estado por dimensión, alertas activas.
2. **Headcount y Organización** — Headcount, FTE, distribución por departamento y ubicación (tablas), ratio manager:empleado.
3. **Análisis de Rotación** — Voluntaria/involuntaria, tendencia 3 meses, departamentos con mayor rotación, motivos de baja (si disponibles).
4. **Antigüedad y Seniority** — Media global y por departamento, bandas (<1a, 1-3a, 3-5a, 5-10a, >10a), empleados en periodo de prueba.
5. **Staffing** — Posiciones abiertas por departamento, time-to-fill, vacantes críticas (>60 días), gap vs objetivo de plantilla.
6. **Resumen de Ausencias** — Tasa global y por departamento, tipos de ausencia, días perdidos.
7. **Detalle por Departamento** — Bloque por departamento: headcount, rotación, antigüedad, posiciones abiertas, alertas.
8. **Plan de Acción** — Acciones inmediatas, a corto plazo y estratégicas según el escenario diagnosticado.

---

## Reglas de generación (formato y RGPD)

- Métricas siempre **AGREGADAS** (por departamento, ubicación). Nunca datos individuales.
- Departamentos con <3 empleados se agrupan como "Otros" para proteger la anonimidad.
- Si Odoo no está disponible, generar informe solo con HiBob y avisar claramente.
- Costes en EUR con 2 decimales. Porcentajes con 1 decimal. Fechas DD/MM/YYYY. Horas HH:MM.
- Usar siempre datos reales del JSON, nunca inventar nombres ni cifras.
- RGPD: nunca listar empleados individuales con datos de rendimiento, NIF/NIE, salarios,
  datos bancarios ni datos de salud.
- Guardar como `informe_hr_{periodo}.md` (ej. `informe_hr_2026-02.md`).
- Los JSON intermedios (`hr_data.json`, `hr_health.json`) son temporales y no deben persistir.
