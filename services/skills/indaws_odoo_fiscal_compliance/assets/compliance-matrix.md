# Matriz de no-regresión fiscal — {{MÓDULO}} ({{VERSIÓN}} · {{JURISDICCIÓN}})

> Este informe **no certifica legalidad**. La ley es el oráculo de "correcto". Verifica, mediante
> un diferencial A/B (sin módulo vs con módulo), que el cambio fiscal está acotado a su objetivo y
> deja el resto legalmente correcto e idéntico. Lo marcado "requiere firma de asesoría" es criterio profesional.

## Resumen
- Cómputo(s) fiscal(es) alterado(s): {{base / cuota / total / posición / retención / numeración / inmutabilidad}}
- Subconjunto objetivo (lo que DEBE cambiar): {{p.ej. ventas con posición fiscal REAV}}
- Batería no-objetivo (lo que DEBE quedar intacto): {{venta 21% / otro tipo / abono / compra}}
- Casos evaluados: {{N}} · SCOPED-OK {{n}} · REGRESIÓN {{n}} · TARGET-WRONG {{n}} · NO-OP {{n}} · RUNTIME-N/A {{n}}
- **Radio de impacto**: {{qué operaciones del sistema quedan afectadas — deseadas y no deseadas}}
- Veredicto global: {{acotado / regresión detectada / requiere firma}}

## Matriz de regresión
| # | Caso | Tipo | Salida legal esperada (cita) | DB-A (sin módulo) | DB-B (con módulo) | Veredicto |
|---|------|------|------------------------------|-------------------|-------------------|-----------|
| 1 | {{venta nacional 21%}} | no-objetivo | {{base=precio, cuota=21%, total=base+cuota — arts. 78/90 LIVA}} | {{base/cuota/total}} | {{base/cuota/total}} | SCOPED-OK / REGRESIÓN |
| 2 | {{venta REAV}} | objetivo | {{base=margen/1,21, cuota comprendida — arts. 142/145}} | {{cómo sale sin régimen}} | {{cómo sale con régimen}} | SCOPED-OK / TARGET-WRONG / NO-OP |
| 3 | {{abono / otro tipo / compra}} | no-objetivo | {{…}} | | | |
| … | | | | | | |

## REGRESIÓN / TARGET-WRONG — correcciones concretas
- {{caso}} → {{delta A vs B / número mal}} → {{causa raíz (filtro, base_line, auto_install, onchange…)}} → {{fix de acotamiento}}.

## Filtro del override (causa raíz verificada)
- {{qué selecciona el `filtered(...)`/dominio sobre la batería: subconjunto correcto vs vacío/global}}.

## UNVERIFIED / RUNTIME-N/A
- {{regla sin fuente o invariante no ejecutado}} → {{qué falta}}.

## Requiere firma de asesoría (criterio profesional)
- {{decisión de representación / interpretación}} → {{por qué no lo decide la skill}}.

## Reproducibilidad
- Entorno: {{DB-A / DB-B / ports}} · batería, flujos y snippets usados en el diferencial.
