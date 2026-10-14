# Estimación de Esfuerzo

**Tarea**: {nombre} | **Solicitante**: {nombre} | **TL**: {nombre}
**Odoo**: {X.0} | **Fecha**: {YYYY-MM-DD}

## Descripción
{Resumen funcional}

## Desglose

| Componente | Descripción | H base | Factor | H ajustadas |
|-----------|------------|--------|--------|------------|
| Análisis | {qué} | {h} | x{f} | {h} |
| Modelos Python | {qué} | {h} | x{f} | {h} |
| Vistas XML | {qué} | {h} | x{f} | {h} |
| JavaScript/OWL | {qué} | {h} | x{f} | {h} |
| Seguridad | {qué} | {h} | x{f} | {h} |
| Tests | {qué} | {h} | x{f} | {h} |
| Integración | {qué} | {h} | x{f} | {h} |
| Review + deploy | {qué} | {h} | x1.0 | {h} |
| **TOTAL** | | **{h}** | | **{h}** |

## Factores Riesgo

| Factor | Aplica | Mult | Justificación |
|--------|--------|------|--------------|
| Módulo nuevo | Sí/No | x1.3 | {razón} |
| Integración externa | Sí/No | x1.5-2.0 | {razón} |
| JS/OWL | Sí/No | x1.3-1.5 | {razón} |
| Sin spec clara | Sí/No | x1.5 | {razón} |
| Dev junior | Sí/No | x1.3-1.5 | {razón} |
| Impacto datos | Sí/No | x1.3 | {razón} |

## Estimación Final

| Escenario | Horas | Nota |
|-----------|-------|------|
| Optimista | {h} | Sin imprevistos |
| **Esperado** | **{h}** | **Con factores riesgo** |
| Pesimista | {h} | Esperado × 1.5 |

## Propuesta Económica al Cliente

| Concepto | Horas |
|----------|-------|
| Análisis + desarrollo + tests | {h} |
| Consultoría: validación del flujo con el cliente | {h} |
| **Total propuesta** | **{h}** |

**Comunicar al cliente**: rango {h-h}, presentando {h} como escenario esperado.

## Dependencias

| Dependencia | Estado | Impacto |
|------------|--------|---------|
| {qué} | Resuelto/Pendiente/Bloqueado | {consecuencia} |

---

## Ejemplo: Integración API Envialia

**Tarea**: Integración transportista Envialia | **Odoo**: 18.0

| Componente | Descripción | H base | Factor | H ajustadas |
|-----------|------------|--------|--------|------------|
| Análisis | API docs, sandbox, auth | 4h | x1.0 | 4h |
| Modelos | delivery.carrier + stock.picking + service class | 12h | x1.5 | 18h |
| Vistas | Config carrier, campos picking, wizard etiquetas | 4h | x1.0 | 4h |
| Seguridad | ACLs config carrier | 1h | x1.0 | 1h |
| Tests | Mock API, test envío/cancel/tracking | 6h | x1.3 | 8h |
| Integración | REST calls, parseo, errores | 8h | x2.0 | 16h |
| Review+deploy | 2 rondas + docs | 4h | x1.0 | 4h |
| **TOTAL** | | **39h** | | **55h** |

Optimista: 39h | **Esperado: 55h** | Pesimista: 80h → ~2 semanas dedicación completa.
Bloqueante: credenciales sandbox Envialia (pendiente).
