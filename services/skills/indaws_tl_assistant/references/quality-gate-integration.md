# Integracion Quality Gate en el flujo del TL

## Objetivo

Automatizar la primera pasada de code review invocando `/quality-gate`
antes del review manual del TL. Reduce tiempo de review y estandariza criterios.

## Cuando invocar quality-gate

| Situacion | Accion |
|-----------|--------|
| Dev marca tarea como "En review" | Sugerir al TL: "Ejecuto /quality-gate?" |
| TL pide review de un modulo | Ejecutar quality-gate primero |
| Pre-merge de PR | Obligatorio si no se ha hecho |
| Post-fix de hallazgos | Re-ejecutar para validar |

## Flujo integrado

```
1. Dev termina tarea → Stage "En Review"
2. TL recibe notificacion (standup o check)
3. LLM sugiere: "Hay {N} tareas en review. Ejecuto /quality-gate?"
4. Si TL acepta → ejecutar quality-gate sobre el codigo
5. Si pasa (0 blockers, 0 critical) → TL hace review rapido (focus en logica negocio)
6. Si no pasa → devolver al dev con hallazgos antes de que el TL pierda tiempo
```

## Criterios de auto-rechazo (devolver sin review manual)

- Cualquier hallazgo **Blocker** (seguridad, data loss, crash)
- >3 hallazgos **Critical**
- Tests no pasan
- Sin tests para logica nueva

## Criterios de review rapido (TL solo revisa logica)

- 0 Blockers, 0 Critical
- <=3 Major (estilo, convenciones)
- Tests pasan

## Como usar desde tl-assistant

Cuando el TL pida code review (WF4), antes de aplicar el checklist manual:

1. Preguntar: "Quieres que ejecute /quality-gate como pre-check?"
2. Si acepta → invocar `/quality-gate` con la ruta del modulo
3. Presentar resultado resumido
4. Si hay blockers → "Recomiendo devolver al dev con estos hallazgos"
5. Si limpio → proceder con WF4 manual (focus en negocio, arquitectura)

## Beneficios

- TL no pierde tiempo en issues mecanicos (formato, convenciones, imports)
- Dev recibe feedback mas rapido (antes de que el TL revise)
- Criterios consistentes entre reviews
- Reduce "Override Festival" al detectar anti-patrones temprano

## Limitaciones

- quality-gate no entiende contexto de negocio → TL siempre necesario
- No valida UX ni flujo funcional
- Puede dar falsos positivos en codigo legacy (filtrar con criterio)
