# Risk Scoring

Algoritmo para asignar `risk: low | medium | high` a cada modulo no-estandar.

Modulos `standard` y `oca` no se evaluan (siempre `risk: low` por defecto, no aporta).

## Inputs

| Input | Origen | Peso |
|-------|--------|------|
| LOC (lineas Python sin comentarios/blank) | filesystem o `odoo-module-scanner` | 1pt cada 500 LOC, max 3pt |
| Modelos extendidos | `extends_models` count | 1pt cada 3 modelos, max 3pt |
| Modelos nuevos | `new_models` count | 1pt cada 5 modelos, max 2pt |
| Hooks activos | `has_cron` + `has_server_actions` + `has_automated_actions` | 1pt cada uno, max 3pt |
| Modelos criticos tocados | sale.order, account.move, stock.picking, mrp.production, res.partner, product.template, account.payment, stock.move | 2pt si toca >=1 |
| Sin tests | `tests/` ausente o vacio | 2pt |
| Bridge module | `purpose` empieza por "Bridge:" o auto_install=True con depends >2 | 1pt |
| Sin acceso al codigo | `scan_source: remote_only` | +3pt forzados |
| Hash unknown | `hash: unknown` | +3pt forzados |
| Antiguedad sin actualizar | `latest_version` con `installed_version != latest_version` | 1pt |

## Mapeo a etiqueta

| Suma | risk |
|------|------|
| 0-3 | low |
| 4-7 | medium |
| 8+ | high |

## Casos forzados (override)

| Condicion | risk forzado |
|-----------|--------------|
| Modulo toca >=2 modelos criticos | minimo `medium` |
| Modulo toca >=2 modelos criticos + sin tests | `high` |
| Bridge module entre indaws_core y custom | minimo `medium` |
| `scan_source: remote_only` | minimo `medium` |
| Modulo es Studio export (`studio_customization`) | siempre `high` (suele acumular sin governance) |

## Modelos criticos (por dominio)

Lista mantenida en `risk-scoring.md` (este archivo). Si tu proyecto tiene modelos extra criticos (ej: `x_proyecto_obra` en construccion), añadirlos a la lista local del proyecto en `.ai-memory/inventory/.config.yaml`:

```yaml
critical_models_extra:
  - x_proyecto_obra
  - cliente.contrato
```

## Ejemplo de calculo

Modulo `indaws_sale_rappel`:
- LOC: 1245 -> 2pt
- extends_models: [sale.order, sale.order.line] = 2 -> 0pt
- new_models: [rappel.line] = 1 -> 0pt
- has_cron: true, has_server_actions: false, has_automated_actions: false -> 1pt
- Modelos criticos tocados: sale.order -> 2pt
- Sin tests (asumimos) -> 2pt
- Bridge: false -> 0pt
- Hash ok: 0pt
- Sin remote: 0pt

**Total**: 7pt -> `risk: medium`

(Si tuviera tests: 5pt -> `medium`. Si ademas no tocara sale.order: 3pt -> `low`.)

## Risk contribution para colisiones

En `collisions.yaml`, cada extensor reporta su `risk_contribution` (low/medium/high) que es:

- `risk_contribution = risk del modulo` SI extends_models contiene el modelo en colision
- Studio siempre contribuye `high` (no hay governance del codigo, opaco para review).

La `severity` de la colision es `max(risk_contribution)` ajustada por count:
- `low` si todos contribuyen `low` y count == 2
- `medium` si algun `medium` o count >= 3
- `high` si algun `high`, o Studio presente, o el modelo es critico

## Output del scoring

Cada entrada en `modules.yaml` debe incluir:

```yaml
risk: medium
risk_score: 7                    # numerico, util para sort
risk_factors:                    # lista de factores que sumaron
  - "LOC: 1245 (+2)"
  - "Cron activo (+1)"
  - "Toca sale.order critico (+2)"
  - "Sin tests (+2)"
```

`risk_factors` permite al briefing explicar al usuario POR QUE un modulo tiene su risk, sin necesidad de re-calcularlo.
