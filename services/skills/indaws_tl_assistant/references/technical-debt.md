# Deuda Técnica en Odoo — Guía TL

## 6 Tipos

### 1. Customización
Código custom donde estándar habría bastado. "Mega módulos" mezclando responsabilidades.
Impacto: cada upgrade requiere re-migrar todo. Prevención: pirámide anti-código.

### 2. Integración
Quick-fixes sin arquitectura, URLs hardcodeadas, sin retry/logging.
Impacto: fallos silenciosos, pérdida datos. Prevención: patrones estándar, módulo por integración.

### 3. Conocimiento
Tribal knowledge — solo 1 persona sabe por qué algo se configuró así.
Impacto: bloqueo si ausente, riesgo migraciones. Prevención: ADRs, documentar el "por qué".

### 4. Testing
Sin tests, confianza en testing manual, sin CI/CD.
Impacto: upgrades = aventuras, cada fix puede romper. Prevención: tests obligatorios por PR.

### 5. Configuración
Workarounds acumulados: automated actions que parchean, cron jobs limpieza.
Impacto: impredecible, no migrable. Prevención: resolver root cause en código.

### 6. Datos
Duplicados, huérfanos, datos inconsistentes.
Impacto: reportes incorrectos. Prevención: validaciones modelo, constraints SQL, limpieza periódica.

## Métricas

| Métrica | Alarma |
|---------|--------|
| Ratio custom/total módulos | > 30% |
| Cobertura tests | < 40% |
| Módulos sin migrar >2 versiones | > 0 |
| Issues migración por módulo custom | > 5 |
| Onboarding técnico | > 2 semanas |

## Estrategia Reducción

- **Regla 20/80**: 20% sprint a deuda técnica, priorizar lo que bloquea próxima migración
- **Oportunidad migración**: antes de migrar módulo custom → "¿sigue siendo necesario?" Eliminar > Simplificar > Migrar
- **Boy Scout**: cada PR que toca módulo con deuda, mejorar algo (sin scope creep)

> **Template**: `templates/technical-debt-report.md`
