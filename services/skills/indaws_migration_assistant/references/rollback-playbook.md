# Rollback Playbook — Protocolo de Abort en Go-Live

> **Principio clave:** La decisión de abortar es EXCLUSIVAMENTE del TL.
> La pregunta clave NO es "¿cuánto tarda?" sino "¿existe solución?".

---

## 1. Tres niveles de abort

| Nivel | Nombre | Criterio | Acción |
|-------|--------|----------|--------|
| 1 | Fix-Forward | Error aislado que no afecta flujos críticos del cliente | Aplicar fix en caliente, continuar go-live |
| 2 | Pause + Fix | Error en flujo crítico pero existe camino a resolución (aunque tarde horas) | Pausar go-live, informar al cliente, aplicar fix, validar, continuar |
| 3 | Full Rollback | Error sin solución disponible: upgrade imposible, caída de infraestructura, o impacto directo sin camino a resolución | Restaurar backup pre-migración completo |

**Principio de clasificación:**
Si hay solución, aunque tarde 5+ horas, es Nivel 2 (Pause + Fix). El Nivel 3 es SOLO cuando no hay camino a resolución. La diferencia no es el tiempo sino la existencia de una solución viable. En Nivel 2, lo crítico es la comunicación continua con el cliente.

---

## 2. Árbol de decisión

```
¿Error detectado durante go-live?
│
├─ ¿Existe una solución técnica viable (aunque tarde horas)?
│  │
│  ├─ NO (upgrade imposible, caída infra, sin camino a resolución)
│  │  └─ NIVEL 3: Full Rollback
│  │
│  └─ SÍ → ¿Afecta a flujo crítico del cliente (venta, compra, contabilidad)?
│     │
│     ├─ SÍ → NIVEL 2: Pause + Fix
│     │  └─ Informar al cliente INMEDIATAMENTE, dar ETA, mantener contacto continuo
│     │
│     └─ NO → NIVEL 1: Fix-Forward
│        └─ Resolver en paralelo al go-live, informar al cliente si es visible
│
En TODOS los niveles: mantener comunicación continua con el cliente
```

---

## 3. Ejemplos concretos por nivel

### Nivel 3 — Full Rollback (sin solución disponible)

- Error de Odoo que impide subir la BD al proceso de upgrade
- Caída de servidores (no disponibilidad de infraestructura)
- Corrupción de datos sin camino de recuperación
- Cualquier impacto directo al cliente sin solución disponible

### Nivel 2 — Pause + Fix (la mayoría de casos)

- Flujo de venta roto pero con fix identificado (aunque tarde 5h)
- Integración externa fallando pero con workaround temporal
- Rendimiento degradado pero optimizable
- Permisos/seguridad incorrectos pero corregibles

### Nivel 1 — Fix-Forward (errores menores)

- Vista mal alineada, traducción incorrecta
- Error cosmético no visible al cliente
- Funcionalidad menor no crítica con fix trivial

---

## 4. Protocolo de ejecución por nivel

### Nivel 1: Fix-Forward

| Paso | Responsable | Acción |
|------|-------------|--------|
| 1 | Dev | Identifica el error y evalúa impacto |
| 2 | Dev | Aplica fix en caliente |
| 3 | Dev | Ejecuta smoke test del flujo afectado |
| 4 | TL | Valida que el fix es correcto |
| 5 | — | Continuar go-live |
| 6 | BC | Informar al cliente si el error era visible |

### Nivel 2: Pause + Fix

| Paso | Responsable | Acción |
|------|-------------|--------|
| 1 | TL | Decide pausa del go-live |
| 2 | BC | Informa al cliente con ETA (plantilla abajo) |
| 3 | Dev | Aplica fix |
| 4 | BC | Actualiza al cliente periódicamente (cada 30-60 min) |
| 5 | Dev | Re-ejecuta smoke tests de flujos afectados |
| 6 | TL | Valida resultado |
| 7 | BC | Confirma con cliente que todo OK |
| 8 | — | Continuar go-live |
| 9 | — | Si se pierde el camino a resolución → escalar a Nivel 3 |

### Nivel 3: Full Rollback

| Paso | Responsable | Acción |
|------|-------------|--------|
| 1 | TL | Anuncia abort al equipo |
| 2 | PM/BC | Informa al cliente inmediatamente (plantilla abajo) |
| 3 | Dev | Restaura backup pre-migración (BD + filestore) |
| 4 | Dev | Verifica BD restaurada (integridad, accesos) |
| 5 | Dev | Reabre accesos con versión anterior |
| 6 | BC | Confirma operación normal con cliente |
| 7 | PM | Comunica nuevo timeline al cliente |
| 8 | TL | Convoca post-mortem en 48h máximo |

---

## 5. Criterios de clasificación de errores

| Categoría | Ejemplos concretos | ¿Tiene solución? | Nivel |
|-----------|-------------------|-------------------|-------|
| Upgrade imposible | Odoo no puede procesar la BD, error irrecuperable en el proceso de upgrade | No | 3 |
| Caída de infraestructura | Servidor caído, sin acceso a la máquina, proveedor cloud con incidencia | No (depende de terceros) | 3 |
| Corrupción de datos sin recuperación | Saldos descuadrados sin trazabilidad del error, registros borrados | No | 3 |
| Flujo crítico roto | Venta no funciona, facturación bloqueada | Sí (fix identificable) | 2 |
| Rendimiento degradado | Vistas principales lentas, timeouts | Sí (optimizable) | 2 |
| Integración externa fallando | API, pasarela de pago, EDI | Sí (workaround o fix) | 2 |
| Error de seguridad/permisos | Accesos incorrectos, datos visibles a rol equivocado | Sí (corregible) | 2 |
| Error cosmético/menor | Vista mal alineada, traducción incorrecta, error no visible al cliente | Sí (trivial) | 1 |

---

## 6. Comunicación — El eje central del protocolo

La comunicación continua con el cliente es **OBLIGATORIA** en cualquier nivel. No es un paso final sino un flujo constante.

### Frecuencia de comunicación por nivel

| Nivel | Frecuencia |
|-------|-----------|
| 1 | Informar al cliente al cierre del go-live si el error era visible |
| 2 | Informar al inicio, actualizar cada 30-60 min, confirmar al resolver |
| 3 | Informar inmediatamente, actualizar cada 15-30 min hasta completar rollback |

### Plantillas de mensaje

#### Al cliente (vía BC) — Nivel 2 (inicio)

> "Hemos detectado una incidencia en [flujo]. Nuestro equipo ya está trabajando en la solución. Estimamos resolverla en aproximadamente [ETA]. Te mantenemos informado."

#### Al cliente (vía BC) — Nivel 2 (actualización periódica)

> "Seguimos trabajando en la incidencia. [Progreso]. Nuevo ETA: [tiempo]. Próxima actualización en [30/60 min]."

#### Al cliente (vía BC) — Nivel 3

> "Hemos detectado una incidencia que nos impide completar la migración de forma segura. Para proteger vuestros datos, vamos a restaurar la versión anterior. El servicio estará disponible en [ETA]. Os comunicaremos la nueva fecha de migración en los próximos días."

#### Al equipo interno (Slack/chat)

> "[NIVEL X] — {descripción breve}. Acción: {qué se está haciendo}. Responsable: {quién}. ETA: {cuándo}."

#### Al COO/CTO (solo Nivel 3)

> "Rollback en curso en proyecto {nombre}. Causa: {motivo}. ETA restauración: {hora}. Post-mortem en 48h."

---

## 7. Tiempos críticos

Se miden durante el rehearsal de Fase 5 y se registran en `templates/rollback-decision.md`.

| Paso del rollback | Tiempo medido (Fase 5) | Tiempo real (Go-Live) |
|-------------------|-----------------------|----------------------|
| Cierre de accesos | ___ min | ___ min |
| Restauración backup BD | ___ min | ___ min |
| Restauración filestore | ___ min | ___ min |
| Verificación BD restaurada | ___ min | ___ min |
| Reapertura accesos | ___ min | ___ min |
| **TOTAL ROLLBACK** | ___ min | ___ min |

---

## 8. Precondiciones (verificar ANTES del go-live)

Checklist de precondiciones que deben cumplirse para que el rollback sea viable:

- [ ] Backup producción validado (dump + filestore) en almacenamiento accesible
- [ ] Backup probado restaurándolo al menos 1 vez (en rehearsal)
- [ ] Tiempo de restauración medido y dentro de ventana
- [ ] Acceso SSH/admin al servidor de producción confirmado
- [ ] Equipo de guardia con disponibilidad confirmada
- [ ] Canal de comunicación de emergencia establecido (Slack, teléfono)

---

## 9. Post-rollback / Post-mortem

Protocolo a seguir tras ejecutar un rollback Nivel 2 o Nivel 3:

1. **Reunión post-mortem en 48h máximo** — convocada por el TL
2. **Root cause analysis documentado** — qué falló, por qué, cómo se detectó
3. **Plan de remediación** — acciones correctivas con fecha de nuevo intento
4. **Comunicación al cliente** — nuevo timeline comunicado por PM/BC
5. **Actualización del plan de migración** — lecciones aprendidas incorporadas
