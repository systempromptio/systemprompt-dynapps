# Diseño de Solución — {título}

**Necesidad**: {1 frase de negocio} | **Odoo**: {X.0} | **Arquitecto**: {nombre} | **Fecha**: {fecha}
**Clasificación**: {Custom / Mejora Core} (confirmada por dev-assessment) | **Módulo**: `{técnico}`

## Contexto de negocio (POR QUÉ)

{Quién necesita qué y para qué. Situación actual. Mejora esperada.}

## Mecanismo central (tracer bullet) — verificado

- **Mecanismo**: `{método/campo/constraint}` en `{addon/file.py:line}` (Odoo {X.0})
- **Cómo se verificó**: {grep en fuente / probe con rollback vía odoo-flow-tester — resultado}
- **Por qué todo depende de él**: {1-2 líneas}

## Decisión de arquitectura (ADR)

- **Decisión**: {qué se construye + patrón: herencia / delegación / mixin / wizard / bridge}
- **Fronteras de módulo**: {SRP: qué problema único; core intacto; bridge si aplica}
- **Alternativas descartadas**:
  - {alternativa A} — descartada porque {motivo}
  - {alternativa B} — descartada porque {motivo}
- **Consecuencias**: {qué se gana / qué deuda o límite se asume / qué queda fuera}
- **Puerta**: {one-way (irreversible, analizado a fondo) / two-way (reversible)}

## Descomposición (slices verticales, INVEST)

| # | Slice (valor end-to-end) | Riesgo | Contiene tracer bullet |
|---|--------------------------|--------|------------------------|
| 1 | {…} | Alto/Medio/Bajo | Sí/No |
| 2 | {…} | | |

**Walking skeleton**: {la primera línea end-to-end que debe andar}.

## Dependencias

- **`depends` (verificadas)**: `{módulo}` — {por qué es imprescindible}
- **Se integra con**: {modelo/método core extendido, eventos}

## Impacto Odoo (delegado a checklist de dev-task-writer)

{Puntos que aplican: registros existentes/migración, multi-company, multi-currency, seguridad/ACL,
upgrade-safety, rendimiento. Cada uno con la instrucción para el dev.}

## Riesgos y contingencia

- {riesgo} → {mitigación}
