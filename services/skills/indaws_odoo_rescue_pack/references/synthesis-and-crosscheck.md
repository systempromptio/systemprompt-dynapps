# Fases 4-5 — Cruce funcional↔código y gap de versión

## Sección 06 — "Lo que nos contaste vs. lo que confirma el código/BD"

Es el corazón diferencial del Rescue Pack: contrastar la **percepción** del cliente
(sesión de consultoría, Fase 0) con la **evidencia** (Fases 2 y 3). Para cada afirmación
funcional relevante, una fila:

| Lo que nos contaste | Lo que confirma el código/BD | Veredicto |
|---------------------|------------------------------|-----------|
| "Tenemos automatizado X" | `base.automation` inexistente o desactivada (extracción) | ❌ No existe |
| "Tenemos un desarrollo a medida de X" | Módulo `x_custom` con N modelos (auditoría) | ✅ Confirmado, deuda alta |
| "Todo está integrado" | Solo `ir.cron` de la integración Y activo | ⚠️ Integración parcial |

Reglas:
- **Hecho** = sale de BD o código. **Hipótesis** = sale de la consultoría. No mezclar.
- Donde haya conflicto, gana la evidencia, y se explica el porqué con datos.
- Este cruce es lo que justifica la propuesta posterior (sección 08).

## Sección 07 — "Coste de migrar de versión" (EXTRA opcional)

**Solo si `{ofrecer_migracion}`=sí** (decidido en Fase 0). NO es el objetivo del Rescue Pack (que es
el estado actual). Si fue "no", **omitir esta sección entera**. Gap entre la versión origen y la
objetivo (por defecto la última conocida).

- Usar `odoo-database-analyzer` para el esfuerzo de migración por módulo y `odoo-version-api-reference`
  para los breaking changes concretos (vistas, ORM, OWL/JS, deprecaciones).
- Enfocar a negocio: qué gana el cliente (funcionalidad nativa que hoy es custom, fin de
  deuda, soporte), no solo el diff técnico.
- Cruzar con la Fase 2: deuda/seguridad que la nueva versión resuelve de serie, y custom
  que puede sustituirse por estándar o por Indaws Core (`odoo-classification-guide`).

## Sección 06 (Fase 4) — Propuesta value-based, DOS opciones

**Puerta de Fase 0:** la Op.2 y la sección 07 existen **solo si `{ofrecer_migracion}`=sí**. Si en el
arranque se respondió **no**, la propuesta lleva **solo la Op.1** y no se genera la sección 07 ni se
ejecutan `odoo-database-analyzer`/`odoo-version-api-reference`. Cuando sí se ofrece, la migración es
la Op.2. Solo importes € en el PDF (nunca horas/tarifa); presupuesto formal → `quotation-builder`.
Valorar con `odoo-effort-estimator`.

- **Op.1 — estabilizar + mantener** ("¿nos hacemos cargo del entorno?"): arreglar los puntos
  **críticos/bloqueantes** · traspaso a cloud (si es a deshoras, **especificar recargo**) · **Indaws
  Cloud** (hosting gestionado) · mantenimiento.
- **Op.2 — migrar** de versión: coste de migración + **formación** (N sesiones de 2h) + **soporte de
  arranque** 1 mes = **10% de la migración** + **migraciones recurrentes** (abajo).

### Migraciones recurrentes — el coste de NO quedarse anclado

Odoo saca versión cada año y el cliente no se queda en la que migra hoy: **el salto se repite**. Esta
línea existe para que vea el coste de mantenerse al día **antes** de firmar, no dos años después. Va
dentro de la Op.2, con su propia fila.

```
importe por salto = coste_implantacion_cero × {pct_migracion_recurrente}
cadencia          = un salto cada `cada_n_anios` años        (config.json → proposal.recurring_migration)
```

**Lo que se pregunta en Fase 0 es el PORCENTAJE**, y solo para poder cambiarlo: por defecto el de
`config.json` (15 %). El **coste de implantación desde cero se CALCULA**, no se pregunta.

#### Cómo se calcula el "implantarlo desde cero con nosotros"

Es el mismo alcance que el rescate ya tiene medido, valorado como si lo construyéramos hoy. Se apoya en
lo que las fases anteriores dejan sobre la mesa, y se valora con `odoo-effort-estimator`:

| de dónde sale | qué aporta |
|---|---|
| Fase 1 triaje | módulos **CUSTOM** instalados = funcionalidad a medida que habría que construir |
| Sección 6 (mapeo a Core) | los custom que **ya tienen paquete Indaws Core con precio de catálogo**: ese precio entra tal cual, no se re-estima |
| Fase 3 Studio/acciones | campos, modelos `x_`, automatizaciones, vistas e informes hechos a mano |
| Fase 3 `db_intelligence.sh` | dimensionado: dependencias, complejidad, actividad — y **fill-rate**, que descarta lo que nadie usa |
| Sección 2 `db_state.sh` | **volumen de datos**: dimensiona la implantación funcional y la migración de datos |
| Fase 2 | integraciones y conectores detectados en el código |

**Lo que NO se cuenta:** módulos estándar (no se implanta lo que viene de serie, sí su parametrización),
OCA instalado y funcionando, todo lo que el fill-rate diga que está a cero —un modelo sin filas o un campo
al 0 % no se habría construido— y **los usuarios: su licencia se le paga a Odoo, no a nosotros**, así que
no mueve lo que cuesta implantar.

**Es una reconstrucción, y se dice así.** No es un presupuesto de implantación ni una oferta: es el
tamaño del sistema que hoy tiene, puesto en euros de hoy, para poder calcular sobre él. Presupuesto
formal → `quotation-builder`.

#### Cuando no hay datos para calcularlo

Casos 2/3 sin fuente o dump-only: **entonces sí se pregunta** al consultor y entra como **cifra
estimada**, dicho en el PDF con esas palabras. Lo que no vale es tomar un número del consultor y
presentarlo como calculado, ni al revés.

**La condición va PEGADA al importe, no en una nota al pie.** Vale mientras **no se amplíe
significativamente la base instalada**: si el alcance crece más del `umbral_ampliacion_alcance_pct`
(por defecto 20 % en módulos custom o integraciones; **los usuarios no cuentan**), se recalcula. Sin esa
frase el cliente lee un precio cerrado para siempre, y no lo es.

**Qué NO es:** no sustituye a la Op.1 ni al mantenimiento; es el coste de seguir en versión soportada.
Si preguntan el equivalente anual, es el importe partido por `cada_n_anios` — dilo como cálculo, no
como cuota: no se factura cada año.

**Dónde vive: en el PDF y solo en el PDF** (decidido el 2026-08-19). No baja al presupuesto de
`quotation-builder`: allí la migración es un rango puntual (`x_studio_migration`) y el mantenimiento una
cuota mensual, y un importe **cada dos años** no es ninguna de las dos. Meterlo exigiría decidir cómo se
representa en el `sale.order`, y eso se ha dejado fuera a propósito. Está escrito para que la próxima
sesión no lo tome por un descuido y lo "arregle".

- **Salvedad de coste/plazo de quedarse** (refuerza la Op.2): si el hosting es **Odoo.sh** o es
  **Enterprise en versión fuera de las 3 más recientes**, añadir en Op.1 la fecha de retirada de
  Odoo.sh + posible **+25% en renovación**. Fechas y regla en
  `references/shared/odoo-lifecycle-licensing.md` — **nunca a fuego**.
- Importes/parámetros (formación, %, Indaws Cloud €/mes) en `config.json` (`proposal`), no inventar.

## Entrada a la síntesis (Fase 5)

El informe final (secciones 01-07) debe dejar listo para el handoff:
- Esfuerzo estimado por módulo (de `odoo-database-analyzer`).
- Riesgos técnicos priorizados (de `odoo-code-auditor`).
- Personalizaciones a migrar (Studio/acciones extraídas → módulo `odoo_studio`).
- Recomendación de estrategia (reimplantar limpio / migrar 1:1 / refactor + Core).

Eso es lo que recibe `quotation-builder` para construir 08-10.
