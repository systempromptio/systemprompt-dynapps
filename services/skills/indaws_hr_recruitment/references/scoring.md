# Scoring de Candidatos (0-100) y Definicion del Funnel

**CONFIDENTIAL - HR TEAM ONLY**

Algoritmo de scoring ponderado, definicion de etapas del funnel y formula de
time-to-hire. Fuente de verdad de los pesos: `config/scoring_config.json` y
`lib/config.py` (`CANDIDATE_SCORE_WEIGHTS`). No modificar sin aprobacion de RRHH.

> El scoring lo calcula SIEMPRE `scripts/candidate_scorer.py`. Esta referencia
> documenta el algoritmo; nunca se calcula a mano.

---

## Algoritmo de Scoring (0-100)

Puntuacion total = suma ponderada de 5 dimensiones (los pesos suman 1.0):

| Dimension | Peso | Que mide |
|-----------|------|----------|
| `experience_match` | 30% | Anos en rol + relevancia de industria (ERP/Odoo) |
| `skills_match` | 25% | % de skills requeridas cubiertas + bonus nice-to-have |
| `culture_fit` | 20% | Media del scorecard de entrevista (dimensiones culturales) |
| `education` | 15% | Match de titulacion + bonus certificaciones |
| `availability` | 10% | Fecha de incorporacion + flexibilidad de ubicacion |

Dimension sin datos = neutral 50/100. `total_score = round(Σ score_dim × peso_dim)`.

### Escalas por dimension (resumen)

- **experience_match**: years_in_role (excede 2+ =100, cumple =80, ±1 año =60, ±2 =40, por debajo =20); industry_relevance (mismo ERP/Odoo =100, tech relacionada =75, transferibles =50, distinta =25).
- **skills_match**: % skills requeridas (100% =100, 75-99% =80, 50-74% =60, 25-49% =40, <25% =20); bonus +5 por nice-to-have (max +20).
- **culture_fit**: media del scorecard 1-4 → 4/4 =100, 3/4 =75, 2/4 =50, 1/4 =25.
- **education**: degree_match (exacto =100, relacionado =75, cualquiera =50, sin titulo+experiencia =40, sin titulo ni experiencia =10); bonus certs (Odoo cert +15, industria +10, +5 por extra, max +25).
- **availability**: start_date (inmediata =100, 2 sem =90, 1 mes =75, 2 meses =50, >2 meses =25); location_flexibility (cumple =100, dispuesto a mudarse =75, remoto-cuando-oficina =50, sin flex =25).

### Clasificacion / recomendacion

| Rango | Etiqueta | Significado |
|-------|----------|-------------|
| 75-100 | **Recomendado** | Candidato fuerte, alineado con el perfil |
| 50-74 | **Considerar** | Potencial; requiere evaluacion adicional |
| 0-49 | **No recomendado** | Desalineamiento significativo |

### Umbrales operativos (`thresholds`)

- `auto_reject_below`: 30 — descarte automatico (siempre con validacion humana).
- `fast_track_above`: 85 — fast-track del proceso.
- `requires_additional_interview_below`: 60 — entrevista adicional obligatoria.

> El scoring es **herramienta de apoyo**, nunca decision automatica: toda accion
> sobre el candidato requiere validacion humana.

---

## Funnel de Seleccion (etapas)

Etapas estandar (configurables por proceso en `hr.recruitment.stage` de Odoo):

`Aplicacion → Screening → Entrevista 1 → Entrevista 2 → Oferta → Contratado`

`pipeline_analyzer.py` calcula por etapa: volumen, % del total, conversion entre
etapas y tiempo medio. Cuello de botella = etapa con conversion < 30%.

> Las etapas reales pueden variar por proceso/departamento. Leer las etapas
> configuradas en Odoo; no asumir el set estandar.

---

## Time-to-Hire

Formula: `Fecha de contratacion - Fecha de apertura de la posicion`.

- Objetivo: **30 dias** (`TIME_TO_HIRE_TARGET_DAYS`). Warning > 30, critico > 60.
- Definir explicitamente la **fecha de inicio** del computo (sourced / applied /
  apertura de `hr.job`). Por defecto en Indaws: apertura de la posicion.
- SLAs por fase y metricas de seguimiento detalladas en `references/hiring_process.md`.
