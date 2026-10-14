# Fórmulas de Estimación de Referencia

> Fuente: Artículo 2602 — Sección 13 + skill odoo-database-analyzer.
> USAR SIEMPRE estas fórmulas. NO inventar estimaciones.

> **Horas base, sin asistencia IA.** Aplicar los factores despues de estas formulas y **antes** de
> la contingencia: delega en la skill `odoo-effort-estimator`.

---

## 1-4. BD, iteraciones, salto de versión y contingencia

**Delega en la skill `odoo-effort-estimator`**: allí viven la fórmula de BD por volumen, las
iteraciones por tamaño, el multiplicador de salto de versión, la fórmula completa y la
contingencia. Fuente única — no duplicar aquí.

---

## 5. Estimación de código por componente

**Delega en la skill `odoo-effort-estimator`**: la matriz por componente con tramos de LOC y la
tabla gruesa por módulo viven allí. Fuente única — no duplicar aquí.

---

## 6. Ajustes por Análisis de Código (Deep Analysis)

Los ajustes por calidad de codigo (APIs deprecated, seguridad, rendimiento, OWL, mala calidad)
viven en la skill **`odoo-effort-estimator`**. Fuente unica — no duplicar los porcentajes aqui.

---

## 7-8. Fórmula completa y ejemplo

En la skill `odoo-effort-estimator`, con el ejemplo resuelto de 50 GB / proyecto mediano /
v16 → v18. No reproducir aquí.

---

## 9. Referencia Rápida — Migración Mínima

- **Migración mínima:** 50 horas (30h técnico + 20h consultoría) + horas de análisis previo
- **Traspaso simple** a nuestro cloud (sin cambio de versión): ~15 horas
