# Golden Examples — project-analysis
<!-- referenced from SKILL.md References table -->


**Ancla de regresión: antes de modificar las reglas, confirma que estos ejemplos siguen dando el mismo resultado. Si cambian a propósito, actualiza el ejemplo.**

Casos canónicos: requisito de cuestionario → categoría + peso esperados. Derivados de
`categorias_y_pesos.md` (categorías, guía de pesos) y `golden-rules.md` (prioridad Core
sobre custom). Datos sintéticos, sin cliente.

---

## Ejemplo 1 — Dos funcionalidades de un cuestionario de Ventas

### INPUT (Odoo 18, app Ventas)
1. **F1**: "Necesitamos campos personalizados en la ficha de cliente: 7 campos extra
   (segmento, zona, riesgo…) y que se rellene un campo según otro." Sin modelos nuevos,
   sin código Python complejo.
2. **F2**: "Calcular comisiones de vendedor con reglas de escalado por volumen", lógica
   de negocio en Python, ~1 modelo nuevo. No hay paquete Core que lo cubra.

### OUTPUT ESPERADO
| # | Funcionalidad | Categoría | Peso | Por qué |
|---|---------------|-----------|------|---------|
| F1 | 7 campos custom + autorrelleno | `[STUDIO/AUTO]` | **3** | 4-10 campos custom o automated action con lógica básica (criterios §[STUDIO/AUTO] peso 3) |
| F2 | Cálculo de comisiones (Python, 1 modelo) | `[DEV]` | **3** | Desarrollo trivial: < 100 líneas, 1 modelo simple (criterios §[DEV] peso 3) |

**Invariantes:**
- F1 es `[STUDIO/AUTO]` peso 3 (7 campos cae en el rango 4-10), NO `[CONFIG]` ni `[DEV]`.
- F2 es `[DEV]`; todo `[DEV]` va a sección 7 sin importar el peso.
- Todo peso ∈ 1-5; cada tarea con categoría de la lista cerrada `[ANLS]/[CONFIG]/[STUDIO/AUTO]/[CORE]/[DEV]/[MIGR]/[TEST]/[TRAIN]`.

---

## Ejemplo 2 — Prioridad Indaws Core sobre custom (Golden Rule)

### INPUT (Odoo 18)
1. **F3**: "Importación masiva de productos desde Excel (1.200 registros) con
   transformaciones de categorías y tarifas." Existe paquete Core (`indaws_data_import`).
2. **F4**: "Migrar 1.200 clientes históricos desde Excel a Odoo, con normalización de
   NIF y dirección." Es carga de datos al arrancar, no un conector recurrente.

### OUTPUT ESPERADO
| # | Funcionalidad | Categoría | Peso | Por qué |
|---|---------------|-----------|------|---------|
| F3 | Import/conector de productos (recurrente, Excel→Odoo) | `[CORE]` (`indaws_data_import`) | **3** | Golden Rule DATA: existe paquete Core → NO marcar `[DEV]` |
| F4 | Carga inicial de 1.200 clientes | `[MIGR]` | **3** | Migración 100-1000… 1.200 registros con transformaciones simples (§[MIGR] peso 3) |

**Invariantes:**
- F3: ANTES de marcar `[DEV]` se verifica `paquetes_indaws_core.md`; al existir paquete
  que lo cubre ⇒ `[CORE]`, nunca `[DEV]` (Golden Rule / gate Core-sobre-custom).
- F4 es `[MIGR]` (carga puntual al arranque), distinto de un conector/import recurrente `[CORE]`.
- Peso `[MIGR]` 3 corresponde a 100-1.000 registros con transformaciones simples; 1.200 está en el límite, se mantiene 3 salvo señal de complejidad.
