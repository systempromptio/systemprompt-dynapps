# Plantilla: Informe de Pipeline de Seleccion

**CONFIDENTIAL - HR TEAM ONLY**

Informe generado automaticamente por `pipeline_analyzer.py`.

---

## Periodo: {fecha_inicio} - {fecha_fin}
## Departamento: {departamento o "Todos"}

---

## 1. Dashboard de Seleccion

| Indicador | Valor | Objetivo | Estado |
|-----------|-------|----------|--------|
| Posiciones abiertas | {n_posiciones} | - | {info} |
| Candidatos activos en pipeline | {n_candidatos_activos} | - | {info} |
| Contrataciones del periodo | {n_contratados} | {objetivo_contrataciones} | {estado} |
| Time-to-hire medio | {tth_medio} dias | 30 dias | {estado} |
| Ofertas aceptadas | {pct_ofertas_aceptadas}% | 80% | {estado} |
| Coste medio por contratacion | {coste_medio} EUR | {objetivo_coste} | {estado} |

---

## 2. Funnel de Seleccion

| Etapa | Candidatos | % del Total | Conversion a Siguiente |
|-------|-----------|-------------|----------------------|
| Aplicacion | {n} | 100% | - |
| Screening | {n} | {pct}% | {conv}% |
| Entrevista RRHH | {n} | {pct}% | {conv}% |
| Entrevista Tecnica | {n} | {pct}% | {conv}% |
| Prueba / Caso | {n} | {pct}% | {conv}% |
| Entrevista Final | {n} | {pct}% | {conv}% |
| Oferta | {n} | {pct}% | {conv}% |
| Contratado | {n} | {pct}% | {conv}% |

**Conversion total (aplicacion a contratacion)**: {conv_total}%

---

## 3. Cuellos de Botella

| Etapa | Conversion | Benchmark | Diagnostico | Recomendacion |
|-------|-----------|-----------|-------------|---------------|
| {etapa_1} | {conv}% | {bench}% | {diagnostico} | {recomendacion} |

---

## 4. Time-to-Hire por Departamento

| Departamento | Time-to-Hire (dias) | Objetivo | Desviacion | Posiciones |
|-------------|--------------------|---------|-----------| ----------|
| {dept_1} | {tth_1} | 30 | {desv_1} | {n_pos_1} |

### Time-to-Hire por Etapa

| Etapa | Dias Medio | % del Total TTH |
|-------|-----------|----------------|
| {etapa_1} | {dias_1} | {pct_1}% |

---

## 5. Efectividad de Fuentes

| Fuente | Candidatos | Contratados | Conversion | Coste | Coste/Contratacion |
|--------|-----------|-------------|-----------|-------|-------------------|
| Portal empleo | {n} | {contratados} | {conv}% | {coste} | {cpc} |
| LinkedIn | {n} | {contratados} | {conv}% | {coste} | {cpc} |
| Referidos | {n} | {contratados} | {conv}% | {coste} | {cpc} |
| Directo | {n} | {contratados} | {conv}% | {coste} | {cpc} |
| Agencia | {n} | {contratados} | {conv}% | {coste} | {cpc} |
| Otro | {n} | {contratados} | {conv}% | {coste} | {cpc} |

**Fuente mas efectiva**: {fuente_top} ({conv_top}% conversion)

---

## 6. Posiciones Abiertas (Aging)

| Puesto | Departamento | Dias Abierta | Candidatos Activos | Estado |
|--------|-------------|-------------|-------------------|--------|
| {puesto_1} | {dept_1} | {dias_1} | {cand_1} | {estado_1} |

**Posiciones criticas (> 60 dias)**: {n_criticas}
**Posiciones en warning (> 30 dias)**: {n_warning}

---

## 7. Plan de Accion

### Acciones Inmediatas (esta semana)
- {accion_1}
- {accion_2}

### Acciones a Corto Plazo (proximas 2 semanas)
- {accion_3}
- {accion_4}

### Acciones Estrategicas
- {accion_5}
- {accion_6}

---

**Informe generado**: {fecha_generacion}
**Generado por**: Skill hr-recruitment v1.0.0
