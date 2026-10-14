# Golden Examples — odoo-effort-estimator

**Ancla de regresión: antes de modificar las reglas, confirma que estos ejemplos siguen dando el
mismo resultado. Si cambian a propósito, actualiza el ejemplo.**

Datos sintéticos, sin cliente.

---

## Ejemplo 1 — Desarrollo peso 4, riesgo medio

### INPUT
- 1 tarea `[DEV]` de peso 4 (< 500 líneas, 1-2 modelos, lógica moderada).
- Estimada en bloque, sin desglose por componentes.
- Proyecto mediano → contingencia +18%.

### OUTPUT
| Concepto | Cálculo | Resultado |
|---|---|---|
| Base peso 4 | punto medio del rango 16-40h | 28h |
| Factor IA en bloque | tramo >25h → ×0.75 | 21h |
| Contingencia +18% | `ceil(21 × 1.18)` | **25h** |
| Testing (línea aparte) | 20-30% de 28h = 5.6-8.4h, × 0.35 | **2-3h** |

### INVARIANTES
- La base cae en 16-40h ANTES de factor y contingencia.
- El factor se aplica ANTES de la contingencia, nunca después.
- Testing es partida separada, nunca sumada al rango de dev.
- Ninguna cifra final baja de 1h.

---

## Ejemplo 2 — Implantación funcional: el factor NO aplica

### INPUT
- App Ventas estándar: presupuestos, pedidos, tarifas básicas.
- Sin tarifas escalonadas, intercompany, aprobaciones ni EDI.
- Proyecto pequeño → contingencia +10%.

### OUTPUT
| Concepto | Cálculo | Resultado |
|---|---|---|
| Ventas estándar | 2 semanas × 10h | 20h |
| Factor IA | configuración funcional → ×1.00 | 20h |
| Contingencia +10% | `ceil(20 × 1.10)` | **22h** |
| Configuración inicial | regla fija | +10h |

### INVARIANTES
- Ventas estándar = 2 semanas. Ni 3 ni 4 sin señal de complejidad.
- **Configuración, Studio y formación van a ×1.00.** El factor IA no las toca.
- Config inicial es invariablemente 1 sem / 10h, nunca 0.

---

## Ejemplo 3 — Tarea pequeña: manda el suelo

### INPUT
- Petición de cliente: añadir un campo calculado a una vista existente.
- Desglose: 1h de código, 0.5h de review y despliegue.

### OUTPUT
| Concepto | Cálculo | Resultado |
|---|---|---|
| Código | 1h × 0.70 | 0.7h |
| Review + deploy | 0.5h × 1.00 | 0.5h |
| Subtotal | | 1.2h |
| Suelo por tarea | `max(1.2h, 1h)` | **1.2h** |

### INVARIANTES
- El overhead (review, deploy, comunicación) nunca lleva factor.
- Ninguna tarea de cliente baja de 1h por aplicar factores.
- Con desglose, una tarea pequeña baja poco: el código es una fracción del coste.
