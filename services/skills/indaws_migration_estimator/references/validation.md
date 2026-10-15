# Validación paramétrica (INTERNA) — LOC + base por tamaño de BD

Contraste obligatorio de cada estimación contra un modelo paramétrico calibrado con las
migraciones Dynapps ya ejecutadas. Es un **control interno**: detecta estimaciones
infladas o cortas antes de generar entregables. **No aparece nunca** en el informe PDF,
en el `sale.order` ni en ningún texto que vea Dynapps o su cliente (ver §Confidencialidad).

## La fórmula

```
Referencia (h) = Base(tamaño BD) + LOC × 0,20 € / 45 €/h
```

| Tamaño BD | Rango | Base |
|---|---|---|
| S | < 20 GB | 60 h |
| M | 20–60 GB | 70 h |
| L | > 60 GB | 90 h |

- La **base** representa BD (4 iteraciones) + unit tests + una ronda de correcciones.
- **LOC** = líneas `.py` + `.xml` + `.js` de los **módulos en scope** (los del bucket 1:
  custom in-house/vendor + OCA sin port/core), **sin** `tests/`, `i18n/`, `migrations/`,
  `upgrades/` ni `static/**/lib`. Todas las líneas pesan igual (sin ponderar por lenguaje).
  Nunca contar OCA/terceros excluidos (casos 1 y 2 de `calibration.md`): inflan el LOC sin
  ser trabajo (p. ej. un repo de 84k LOC con 0,3k propias).

## Cómo se aplica (paso 4 del workflow)

```bash
python3 <skill>/scripts/reference_check.py --db-gb <GB> --estimate <total_1+2+3+4> <módulo_en_scope> [...]
# o, si ya tienes el recuento:  --loc <N>
```

| Veredicto | Desvío vs referencia | Qué hacer |
|---|---|---|
| `OK` | ±15 % | Seguir. |
| `ABOVE` | > +15 % | Revisar inflado: módulos por encima del cap, bucket 2 alto para el tamaño, horas de validación humana colada. Recortar lo que sea sobreestimación. Si el exceso es volumen/complejidad real (OWL rewrite, conectores, saltos grandes), mantener y justificarlo en la sección 04 **con argumentos técnicos**, nunca citando la referencia. |
| `BELOW` | < −15 % | Revisar infraestimación: módulos complejos con 1–2 h, bucket 2 en el floor con BD grande, OCA caso 3 sin estimar. Subir lo que esté corto. Si tras revisar sigue por debajo con motivo (código muy mecánico, salto de 1 versión), mantener. |

- Informa al usuario **en el chat** (no en el PDF) de la línea `REFERENCIA / ESTIMADO /
  DESVÍO / veredicto` y de qué ajustaste o por qué mantuviste el total.
- Guarda el resultado (`--json`) en `docs/estimate_check.json` del proyecto: sirve para
  comparar después estimado vs consumido y recalibrar la fórmula.
- **No sustituye** a la calibración por buckets ni cambia la estructura del presupuesto
  (4 buckets, 45 €/h, solo horas en el PDF). Es un contraste, no el cálculo del precio.

## Confidencialidad — regla dura
- **NUNCA** mencionar en el informe PDF, en el `sale.order` ni en su descripción: la
  fórmula, "referencia paramétrica", "LOC × 0,20", "€/LOC", la talla S/M/L como base de
  precio, ni el desvío. Tampoco el recuento total de LOC como base de precio.
- Las LOC **por módulo** pueden seguir citándose en la sección 02/04 como dato técnico de
  complejidad (como hasta ahora), pero no relacionadas con el precio.

## Calibración (de dónde salen los números)
Contraste con las migraciones ejecutadas con el proceso de gates (horas reales imputadas):
BDK (2,8k LOC, S), ZNU (14,2k LOC, S), Horafrost (15,5k LOC, S) → referencia total
dentro de −6 % del consumo real. Revisar trimestralmente con `docs/estimate_check.json`
vs horas consumidas de los proyectos cerrados y ajustar base/tarifa si el desvío
agregado sale de ±10 %.
