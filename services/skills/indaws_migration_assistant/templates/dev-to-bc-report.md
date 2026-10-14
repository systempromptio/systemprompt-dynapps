# Template: Informe Dev → Consultoría (2x/semana)

> Usar cuando un Dev pide: "Plantilla de informe", "Informe para BC"
> El Dev publica este informe como comentario en la tarea principal de migración.
> Ver references/communication.md para protocolo completo.

---

## Informe Dev → BC — {fecha}

**Proyecto:** {nombre_proyecto}
**Migración:** v{version_origen} → v{version_destino}

### Fase actual
**Fase {fase_num}** — Progreso global: {modulos_completados}/{modulos_totales} módulos ({porcentaje}%)

### Completado esta semana
| Módulo | Estrategia | Detalle |
|---|---|---|
| {modulo_1} | {estrategia} | {descripcion_breve} |
| {modulo_2} | {estrategia} | {descripcion_breve} |

### En progreso
| Módulo | Estrategia | % Completado | Estimación restante |
|---|---|---|---|
| {modulo_3} | {estrategia} | {porcentaje}% | {horas}h |

### Bloqueado — requiere decisión BC
| # | Módulo | Descripción del bloqueo | Opciones propuestas |
|---|---|---|---|
| 1 | {modulo} | {descripcion_bloqueo} | A: {opcion_a} / B: {opcion_b} |

> ⚠️ BC debe responder en < 24h laborables. Si no hay respuesta, se escalará a TL → PM.

### Próxima semana
- {objetivo_1}
- {objetivo_2}

### Riesgos detectados
| Riesgo | Impacto | Mitigación propuesta |
|---|---|---|
| {riesgo} | {impacto} | {mitigacion} |

---

## Reglas de formato

- OBLIGATORIO: Dev publica 2x/semana (ej. martes y jueves)
- Cada sección debe estar presente aunque esté vacía (poner "Ninguno" si no aplica)
- Sección "Bloqueado" es la más importante — BC debe poder actuar sin pedir contexto adicional
- NO incluir detalles técnicos excesivos — BC necesita entender qué decidir, no cómo se implementa
