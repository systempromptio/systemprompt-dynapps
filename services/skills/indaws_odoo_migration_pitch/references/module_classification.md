# Clasificación de Módulos para Migration Pitch

## Categorías de Módulos (Detección en Odoo)

Cada módulo instalado en `ir.module.module` se clasifica automáticamente en una de estas 4 categorías según sus metadatos:

- **Odoo Standard**: campo `author` contiene "Odoo", "Odoo S.A." u "Odoo SA" → Migración automática por Odoo. No requiere acción específica.
- **OCA**: campo `author` contiene "Odoo Community Association" o campo `website` contiene "github.com/OCA" → Requiere evaluación (puede o no existir versión para la versión destino).
- **Indaws Core**: campo `name` empieza por `indaws_` o campo `author` contiene "Indaws" o "inDAWS" → Migración garantizada por Indaws. Compatibilidad asegurada.
- **Custom/Terceros**: todo lo que no encaje en las categorías anteriores → Requiere análisis de código para determinar la acción.

---

## Árbol de Decisión (Análisis de Código)

Para módulos **Custom/Terceros** y **OCA**, aplicar la clasificación en este ORDEN EXACTO de prioridad. En cuanto un nivel aplica, se asigna esa acción y no se evalúan los siguientes.

### Nivel 1: ¿Lo cubre Indaws Core?

Buscar en los 6 repositorios de Indaws Core si existe un módulo que cubra la misma funcionalidad:

| Repositorio | Contenido |
|-------------|-----------|
| `indaws-odoo` | Core CE (módulos base Indaws) |
| `indaws-odoo-enterprise` | Core EE (módulos Enterprise Indaws) |
| `vertical-odoo-inbuilt` | Vertical obras/instaladores |
| `connector-sale-odoo` | Conectores eCommerce/Marketplaces |
| `indaws-edicom` | Conector EDI |
| `connector-stock-odoo` | Conectores stock |

**Cómo buscar** (en repos ya clonados por `clone_indaws_repos.sh`):
- Patrones de nombre de módulo similares
- Nombres de modelo (en ficheros Python: `_name = '...'`, `_inherit = '...'`)
- Nombres de campo relevantes
- Keywords en `__manifest__.py` (description, summary, category)

Si se encuentra un módulo Core equivalente → **REEMPLAZAR POR INDAWS CORE**
Indicar qué módulo(s) Core concretos cubren la funcionalidad.

### Nivel 2: ¿Es suficientemente simple para Studio?

Analizar el código Python del módulo:
- Si contiene **<50 líneas de lógica real** (excluyendo imports, `__manifest__.py`, `__init__.py`, comentarios y líneas en blanco)

**Candidatos típicos para Studio:**
- Campos personalizados (fields)
- Campos calculados simples (computed fields con lógica trivial)
- Automatizaciones básicas (base.automation)
- Modificaciones de vistas (ir.ui.view)
- Workflows de aprobación simples

Si aplica → **RECONSTRUIR EN STUDIO**

### Nivel 3: ¿Lo cubre Odoo estándar en la versión target?

Consultar las release notes de la versión destino (`references/odoo-versions/ODOO_[VERSION]_RELEASE_NOTES.md`):
- ¿La funcionalidad del módulo es ahora nativa en la nueva versión?

**Ejemplos comunes:**
- Custom audit trails → `mail.thread` (nativo)
- Custom sequences → `ir.sequence` (nativo)
- Custom email sending → `mail.template` (nativo)

Si la funcionalidad es ahora estándar → **ELIMINAR (ahora es estándar)**

### Nivel 4: ¿El módulo sigue siendo necesario?

- ¿Es obsoleto? (funcionalidad que ya no se usa)
- ¿Es redundante? (otro módulo ya instalado cubre lo mismo)
- ¿Está sin uso real? (instalado pero sin datos asociados)

Si no es necesario → **ELIMINAR**

### Nivel 5: Migrar

Si ninguno de los niveles anteriores aplica → **MIGRAR**

---

## Reglas de Oro (OBLIGATORIAS - anulan clasificación normal)

Estas reglas tienen prioridad ABSOLUTA sobre el árbol de decisión. Se evalúan ANTES de los niveles:

| Funcionalidad detectada | Acción obligatoria | Módulo/Solución |
|-------------------------|-------------------|-----------------|
| **Import/Export de datos** (Excel, CSV, conectores, API integrations) | SIEMPRE Indaws Core | `indaws_data_import` / `indaws_data_export` |
| **Dashboards / KPIs / Business Intelligence** | SIEMPRE Odoo Standard | Funcionalidad nativa de Spreadsheets |

---

## Complejidad del Módulo (para la tabla comercial)

Clasificación simplificada para presentación comercial (NO técnica):

| Complejidad | Criterio | Descripción |
|-------------|----------|-------------|
| **Baja** | <500 líneas de código | Funcionalidad simple: campos, vistas, configuraciones |
| **Media** | 500-2000 líneas | Lógica de negocio moderada, flujos personalizados |
| **Alta** | >2000 líneas | Integraciones externas, lógica compleja, múltiples modelos |

---

## Score de Complejidad Global

Fórmula para calcular la complejidad global de la migración en una escala 0-100:

```
raw = (
  custom_modules * 15 +
  oca_sin_version_target * 20 +
  studio_fields * 0.3 +
  studio_views * 0.5 +
  automated_actions * 2 +
  custom_reports * 5 +
  versiones_a_saltar * 10
)

score = min(round(raw / 200 * 100), 100)

Clasificación:
- 0-25:  Baja complejidad (migración ágil)
- 26-50: Media complejidad (migración estándar)
- 51-75: Alta complejidad (migración con riesgos)
- 76-100: Muy alta complejidad (migración crítica)
```

**Nota sobre factor de normalización (200):** Un caso típico de alta complejidad (10 custom × 15 + 5 OCA × 20 + 80 campos × 0.3 + 40 vistas × 0.5 + 10 automations × 2 + 5 reports × 5 + 3 versiones × 10 = 339 raw) queda correctamente escalado. Para casos extremos, `min(..., 100)` asegura que nunca exceda la escala.

---

## Tabla de Acciones (Formato de Salida)

La tabla de salida agrupa módulos por acción con cabeceras de color:

1. **ELIMINAR** (cabecera verde) — Cubierto por estándar en la versión destino o ya no es necesario
2. **REEMPLAZAR** (cabecera azul/verde) — Cubierto por Indaws Core o reconstruible en Studio
3. **EVALUAR** (cabecera amarilla) — Módulos OCA que requieren evaluación de disponibilidad
4. **MIGRAR** (cabecera morada) — Módulos custom que deben migrarse (último recurso)

Cada fila de la tabla:

| Columna | Contenido |
|---------|-----------|
| **Módulo** | Nombre técnico del módulo |
| **Tipo** | OCA / Custom / Tercero |
| **Acción** | ELIMINAR / REEMPLAZAR / EVALUAR / MIGRAR |
| **Motivo** | Explicación en lenguaje de negocio (NO técnico) |
| **Complejidad** | Baja / Media / Alta |

---

## Anti-Hallucination Rules

- **NUNCA** clasificar un módulo sin haber analizado realmente su código
- **NUNCA** asumir que Indaws Core cubre algo sin haber encontrado una coincidencia real en los repositorios
- **NUNCA** inventar módulos que no existan en la instalación del cliente
- Si hay duda sobre una clasificación → por defecto **EVALUAR** (no MIGRAR)
- Módulos OCA sin versión disponible para la versión destino → **EVALUAR** (no MIGRAR automáticamente)
