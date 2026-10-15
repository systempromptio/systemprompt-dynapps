# Plantilla: Módulos y cores del parque

Inventario y rankings de módulos/cores a nivel de parque. **Datos:** receta "Módulos / cores más extendidos"
(`database.env.module`, `env_count` almacenado) + catálogo de categorías. Para un entorno concreto, usa la receta
"Inventario de módulos de un entorno". Reglas en `report-rules.md`.

## Formato de salida

```
# 🧩 Módulos y cores del parque — [DD/MM/AAAA]

## Cores inDAWS más extendidos
| Core | Categoría | Entornos |
|---|---|---|
| indaws_base | inDAWS Core | 53 |
| indaws_mail | inDAWS Core | 52 |
| indaws_security | inDAWS Core | 51 |

## Distribución por categoría
| Categoría | Módulos distintos |
|---|---|
| inDAWS Core | 144 |
| inDAWS Core Enterprise | 14 |
| inDAWS Connectors | 5 |
| Inbuilt | 17 |
| Odoo Enterprise | 350 |
| Odoo Oficial | 438 |
| OCA | 72 |
| Terceros | 0 |

## Módulos custom más grandes (líneas de código)
| Módulo | Versión | LOC |
|---|---|---|
| indaws_base | 19.0.2.1.0 | 12.400 |
| indaws_mail | 19.0.1.4.0 | 6.100 |
| some_oca_module | 18.0.1.0.3 | 3.200 |
```
Solo incluye esta sección si la petición pide tamaño/LOC explícitamente o si ya la estás mostrando junto a otro
ranking. Nunca aparecerán aquí Odoo Enterprise/Oficial: Cloc no los cuenta por diseño.

## Reglas de contenido
- "Más extendidos" = `database.env.module` ordenado por `env_count` desc (NO uses `.version`: su `env_count` es
  computado y no ordenable).
- "Cores inDAWS" = categorías 1–4 (inDAWS Core / Core Enterprise / Connectors / Inbuilt). El resto no son cores.
- "Más grandes (LOC)" = `database.env.module.version` ordenado por `loc_total` desc, filtrando `loc_total > 0`
  (receta "Módulos custom más grandes del parque"). Un mismo módulo puede salir varias veces si distintas versiones
  tienen LOC distinto — indícalo si pasa. Es tamaño de código, no popularidad: no lo mezcles con `env_count`.
- Para "módulos del cliente X" cambia al formato de la Ficha de entorno o lista por categoría ese entorno.

**Cierre:** ofrece filtrar (p. ej. "qué entornos NO tienen indaws_security", "módulos OCA más usados", "cores de
cliente X", "los módulos custom más grandes por líneas de código").
