# Generacion del informe

Base: `assets/report_template.md`. Adapta longitud y nivel tecnico a lo pedido.

## Longitudes

| Longitud | Contenido |
|----------|-----------|
| Resumen | Veredicto + Core Compliance Score + tabla de bloqueantes (🔴) + top 3 acciones. 1 pagina. Ideal para gate/PR. |
| Estandar | Resumen + un apartado por bloque (A–E) con hallazgos y plan de remediacion priorizado. Por defecto. |
| Extenso | Estandar + evidencia (`file:line` + fragmento) por hallazgo + roadmap por fases + apendice de delegaciones. |

## Niveles tecnicos

- **Alto**: terminologia Odoo directa, dirigido al desarrollador del PR.
- **Medio**: explica el "por que" de filosofia Core en cada hallazgo.
- **Bajo**: para Producto/consultoria; traduce a impacto de negocio
  (reutilizacion, coste de mantenimiento, riesgo de migracion).

## Reglas

1. **Accionable o no se reporta.** Cada hallazgo: que esta mal, principio Core
   roto, ubicacion `file:line`, y arreglo concreto.
2. **Veredicto arriba.** Lo primero es ✅/🟠/🔴 y el score; el desarrollador
   debe saber en 5 segundos si su PR pasa.
3. **Ordena por severidad**, no por fichero. Primero 🔴, luego 🟠, 🟡, 🔵.
4. **Trazabilidad de origen** en cada hallazgo: (propio) /
   (classification-guide) / (code-auditor).
5. **Documentacion** = solo README/manifest del modulo; la ficha en
   Conocimiento NO se audita ni se menciona como pendiente.
6. Si una delegacion no se ejecuto, dilo; no des bloques por aprobados.
7. Ortografia es_ES correcta (tildes, enes) en todo el informe.

## Modo gate (invocado por quality-gate / git-pr)

Devuelve compacto y parseable:

```
CORE COMPLIANCE: <score>/100 — <APTO|APTO CON RESERVAS|NO APTO>
BLOQUEANTES (<n>):
- 🔴 [Bloque A] <hallazgo> — <file:line> — <fix>
ACCIONES RECOMENDADAS:
- 🟠 <hallazgo> — <fix>
```

Sin narrativa larga: el skill invocante decide si frena el merge.
