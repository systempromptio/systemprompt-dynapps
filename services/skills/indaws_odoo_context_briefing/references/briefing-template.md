# Briefing Template

Plantilla del Markdown final que se renderiza al usuario tras el pipeline.

`render_briefing.py` genera este output a partir de los 4 ficheros de `.ai-memory/inventory/`.

## Estructura fija (4 secciones)

```markdown
# Contexto del proyecto Odoo — {PROJECT_NAME}

**Fecha del scan**: {SCAN_DATE}
**Modo**: {MODE}
**Odoo**: {ODOO_VERSION} ({DB_NAME})

---

## 1. Resumen ejecutivo

| Tipo | Total | Delta vs scan anterior |
|------|-------|------------------------|
| standard | {N_STD} | {DELTA_STD} |
| oca | {N_OCA} | {DELTA_OCA} |
| indaws_core | {N_CORE} | {DELTA_CORE} |
| custom | {N_CUSTOM} | {DELTA_CUSTOM} |
| studio | {N_STUDIO_ENTRIES} entradas ({N_FIELDS} campos, {N_ACTIONS} acciones) | {DELTA_STUDIO} |
| **Total instalados** | **{N_TOTAL}** | |

**Ultima actualizacion**: {LAST_SCAN}
**Hash global**: `{GLOBAL_HASH_SHORT}`

{IF_FIRST_SCAN}
> ⚠️ Primera ejecucion en este proyecto — baseline establecida.
{END_IF}

{IF_DELTA_NOT_EMPTY}
### Cambios desde ultimo scan
- **Nuevos**: {NEW_LIST_OR_ZERO}
- **Actualizados**: {UPGRADED_LIST_OR_ZERO}
- **Desinstalados**: {REMOVED_LIST_OR_ZERO}
{END_IF}

---

## 2. Modulos relevantes

{IF_MODE_TARGET}
### Objetivo: `{TARGET_MODULE}`

**Tipo**: {TARGET_TYPE} | **Version**: {TARGET_VERSION} | **Risk**: {TARGET_RISK}
**Proposito**: {TARGET_PURPOSE}

**Modelos extendidos**: {TARGET_EXTENDS_LIST}
**Modelos nuevos**: {TARGET_NEW_LIST}
**Hooks**: cron={TARGET_CRON} | server_actions={TARGET_SA} | automated={TARGET_BA}

#### Dependencias inversas (quien depende de este modulo)
{REVERSE_DEPS_TABLE}

#### Deep-dive
Ver `.ai-memory/inventory/deep/{TARGET_MODULE}.md` para detalle completo.
{END_IF}

### Top-{N} modulos por riesgo

| # | Modulo | Tipo | Risk | Modelos clave | Notas |
|---|--------|------|------|---------------|-------|
{TOP_RISK_ROWS}

---

## 3. Colisiones detectadas

{IF_COLLISIONS_EMPTY}
✅ No hay modelos extendidos por mas de un modulo no-estandar.
{ELSE}

| Modelo | Severity | Extendido por | Notas |
|--------|----------|---------------|-------|
{COLLISIONS_ROWS}

{IF_TARGET_HAS_COLLISIONS}
### ⚠️ Tu objetivo `{TARGET_MODULE}` participa en {N_COLL_TARGET} colisiones

{TARGET_COLLISIONS_DETAIL}
{END_IF}

{END_IF}

---

## 4. Recomendaciones

{IF_DEV_ASSESSMENT_RAN}
### Evaluacion de la funcionalidad solicitada

{DEV_ASSESSMENT_OUTPUT}

{ELSE}
{IF_MODE_TARGET}
### Antes de modificar `{TARGET_MODULE}`:

{TARGET_RECOMMENDATIONS}
{END_IF}
{END_IF}

### Studio relevante para tu objetivo

{IF_STUDIO_TOUCHES_TARGET_MODELS}
| Modelo | Campos Studio | Server actions | Automations |
|--------|---------------|----------------|-------------|
{STUDIO_TARGET_ROWS}

⚠️ Studio toca modelos que tu modulo extiende. Coordinar cambios.
{ELSE}
✅ Studio no toca los modelos del objetivo.
{END_IF}

---

## Comandos utiles

```bash
# Re-scan rapido (solo inventario)
/odoo-context-briefing quick

# Deep-dive de un modulo concreto
/odoo-context-briefing target <nombre_modulo>

# Re-baseline completo
/odoo-context-briefing full

# Ver inventario sin re-escanear
cat .ai-memory/inventory/SNAPSHOT.md

# Buscar modulos que extienden un modelo
python scripts/read_inventory.py --extends sale.order

# Ver detalle de un modulo
cat .ai-memory/inventory/deep/<nombre>.md
```

---

*Generado por `odoo-context-briefing` el {SCAN_DATE}. Inventario en `.ai-memory/inventory/`.*
```

## Reglas de renderizado

### Recomendaciones automaticas (sin dev-assessment)

Si modo=target y `{TARGET_MODULE}` tiene risk=high o participa en colisiones high, el bloque `{TARGET_RECOMMENDATIONS}` se rellena con:

- Si toca modelo critico extendido por Studio: "⚠️ Coordinar con responsable Studio antes de tocar `{model}`."
- Si tiene >=3 modulos no-estandar tocando el mismo modelo: "Considera extraer la logica comun a un bridge module."
- Si no tiene tests (`tests/` ausente): "Crear tests basicos antes de modificar."
- Si version installed != latest: "Modulo desactualizado vs codigo. Lanzar `-u {module}` antes de cambios."
- Si depende de un modulo con risk=high: "Cadena de dependencias riesgosa: revisar `{dep_chain}`."

### Top-N riesgos

`N` se ajusta por modo:
- `quick`: top 5
- `target`: top 5 + objetivo (si no esta en top)
- `full`: top 10
- `studio-only`: no se muestra (esta seccion se omite)

Ordenacion: `risk_score` desc, luego LOC desc.

### Colisiones rendering

Para cada colision en `collisions.yaml`:

```
| sale.order | high | indaws_sale (medium), indaws_sale_rappel (low), cliente_x_custom (medium), studio (high) | Studio + 3 modulos sobre modelo critico |
```

Severity emoji prefix: `low` = ⚪, `medium` = 🟡, `high` = 🔴.

### Comandos en bloque final

Adaptar la ruta `python scripts/...` a la ubicacion real de la skill (el directorio `scripts/` de esta skill). Hardcodear ruta absoluta NO — usar la cwd o variable de entorno.

## Tono

- Conciso, factual, sin emojis salvo los semaforos de severity (⚪🟡🔴) y warnings (⚠️) y checks (✅).
- Frases breves. Tablas siempre que se pueda.
- Nada de "Genial!", "Aqui tienes!", etc.
- Al estilo de Toni: directo, sin floritura.
