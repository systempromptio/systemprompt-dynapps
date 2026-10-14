# Golden Examples — odoo-classification-guide

**Ancla de regresión: antes de modificar las reglas, confirma que estos ejemplos siguen dando el mismo resultado. Si cambian a propósito, actualiza el ejemplo.**

Casos canónicos input→clasificación esperada. Todos en contexto **"Evaluación de nueva
funcionalidad"** (prioridad `Estándar → Core → Studio → Custom`). Derivados de
`golden-rules.md`, `classification-rules.md` y `complexity-thresholds.md`.

---

## Ejemplo 1 — Cuatro funcionalidades, una de cada clasificación

### INPUT (Odoo 18, contexto: evaluación de nueva funcionalidad)
1. **Standard**: "Pipeline de oportunidades de CRM con etapas estándar." Funcionalidad
   nativa de Odoo en la versión destino, sin extensiones.
2. **Studio**: "Añadir 3 campos personalizados a la ficha de cliente y un email automático
   al cambiar de etapa." 0 modelos nuevos, 0 Python complejo (<30 líneas en una automated
   action), ≤20 campos.
3. **Core**: "Importar contactos desde Excel." → activa **Golden Rule 1 (DATA & INTEGRATIONS)**.
4. **Custom**: "Calcular comisiones de vendedor con 2 modelos nuevos, ~400 líneas Python
   y lógica de escalado." No existe en estándar ni en Core; excede umbrales Studio.

### OUTPUT ESPERADO
| # | Funcionalidad | Clasificación | Por qué |
|---|---------------|---------------|---------|
| 1 | Pipeline CRM estándar | 🟢 **Standard** | Existe 100% nativa en v18 |
| 2 | 3 campos + email automático | 🟡 **Studio** | ≤20 campos, 0 modelos, automación simple <30 líneas |
| 3 | Importar contactos desde Excel | 🟣 **Core** (`indaws_data_import`) OBLIGATORIO | Golden Rule 1, evaluado ANTES que todo |
| 4 | Comisiones (2 modelos, ~400 LOC) | 🔴 **Custom Estándar** | 200-800 LOC, 1-3 modelos (complexity-thresholds STANDARD) |

**Invariantes:**
- Cada fila lleva emoji 🟢🟡🟣🔴 (regla anti-alucinación 4).
- La importación Excel SIEMPRE es Core en contexto evaluación (Golden Rule 1), nunca Custom.
- El custom de comisiones cae en **STANDARD** (16-40h), no MINOR ni COMPLEX, por las 400 LOC + 2 modelos.

---

## Ejemplo 2 — Golden Rule 2 (Dashboard) y precedencia de orden

### INPUT (Odoo 18 Enterprise, contexto: evaluación de nueva funcionalidad)
- **A**: "Dashboard de ventas con gráficos y KPIs por comercial."
- **B**: "Dashboard de ventas" pero en contexto **Migración 1:1** (directa) del módulo
  custom existente del cliente.

### OUTPUT ESPERADO
| Caso | Contexto | Clasificación | Por qué |
|------|----------|---------------|---------|
| A | Evaluación nueva | 🟢 **Standard** (Spreadsheets nativos) OBLIGATORIO | Golden Rule 2, evaluada PRIMERO |
| B | Migración 1:1 | 🔴 **Custom** (migrar tal cual) | En 1:1 solo Standard/Custom; Golden Rule da Custom |

**Invariantes:**
- El mismo requisito (dashboard) cambia de clasificación según el **contexto** — la estrategia
  manda (regla anti-alucinación 3).
- En 1:1 están **PROHIBIDAS** Studio y Core; solo Standard o Custom.
- Las Golden Rules se evalúan ANTES que los niveles Standard/Core/Studio/Custom.
