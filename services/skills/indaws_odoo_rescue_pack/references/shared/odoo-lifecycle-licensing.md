# Ciclo de vida y licenciamiento de Odoo (SSOT)

Fuente de verdad única para dos hechos de plataforma/política comercial de Odoo que se citan en pitches de
migración, presupuestos (Rescue Pack), planificación de proyectos y en el monitor del parque:

- **(A) Calendario de retirada de Odoo.sh** — cuándo cada versión deja de estar disponible en Odoo.sh.
- **(B) Recargo del 25% en renovación de licencia Enterprise** — a quién y cuándo se le aplica.

> Este fichero es el **único sitio donde viven estos datos**. Los skills consumidores **apuntan aquí**, no copian
> los números. Para actualizar la política, edita **solo este fichero**.
>
> ✅ **Confirmado contra documentación pública de Odoo el 2026-07-01** (ver §Fuentes). El mes exacto en que cada
> cliente entra en el +25% depende de su fecha de renovación real → confírmala por cliente.

**Consumido por:** `sales-skills:odoo-migration-pitch` (argumentario), `sales-skills:quotation-builder`
(comparativa económica / Rescue Pack), `common-skills:indaws-env-monitor` (marca clientes afectados),
`common-skills:odoo-database-analyzer` (coste/urgencia en el informe de migración), `developer-skills:odoo-rescue-pack`
(salvedad de la Opción 1), `consultant-skills:migration-project-creator` (deadline de Go-Live),
`common-skills:migration-assistant` (driver de urgencia). Puntero técnico desde `common-skills:odoo-version-api-reference`.

---

## Modelo de ciclo de vida (3 tramos por versión mayor)

Odoo aplica tres tramos, todos referidos a la **fecha de lanzamiento** de la versión mayor:

| Tramo | Cuándo | Qué implica |
|---|---|---|
| **Soporte estándar** | las **3 versiones mayores más recientes** (≈ primeros 3 años) | helpdesk, bugfixes y actualizaciones de seguridad incluidos, sin sobrecoste |
| **Soporte extendido / +25%** | a partir de **~3 años 6 meses** desde el lanzamiento (la versión sale del top-3) | para seguir con soporte se paga **+25%** sobre la suscripción anual (ver §B) |
| **Retirada de Odoo.sh** | a los **6 años**, el **31 de octubre** de ese año | la versión se elimina de Odoo.sh; cualquier BD que siga en ella queda **bloqueada permanentemente, sin excepciones** (ver §A) |

---

## (A) Calendario de retirada de Odoo.sh

Aplica **solo a hosting Odoo.sh** (no a servidores propios ni a hosting Indaws/Dynapps). Deadline técnico duro.

| Versión | Lanzamiento | Sale de Odoo.sh |
|---|---|---|
| v14 | Oct 2020 | **31 oct 2026** |
| v15 | Oct 2021 | **31 oct 2027** |
| v16 | Oct 2022 | **31 oct 2028** |
| v17 | Nov 2023 | **31 oct 2029** |
| v18 | Oct 2024 | **31 oct 2030** |
| v19 | Sep 2025 | **31 oct 2031** |

**Patrón:** retirada = 31 de octubre, 6 años después del lanzamiento. Tras esa fecha la BD queda bloqueada como
producción, sin excepciones. Fuente: FAQ oficial de Odoo.sh (se actualiza dinámicamente; revísala al citar).

---

## (B) Recargo del 25% en renovación de licencia Enterprise

**Regla oficial (confirmada):** +25% sobre la suscripción anual para mantener soporte en una versión **fuera de las
3 más recientes** (equivale a ≳ 3 años 6 meses desde el lanzamiento).

- **Facturación desde abril 2026.** Actualizar a una versión cubierta **antes de abril 2026** lo evita.
- **Corte del 4 de julio de 2025** (fecha exacta):
  - Renovado **antes** del 4-jul-2025 → **NO afectado hasta que termine el contrato**, aunque siga en versión antigua
    (típico contrato multianual que acaba p. ej. en 2028). Al vencer, se reevalúa.
  - Renovado **después** del 4-jul-2025 → **+25%** sobre el precio anualizado si está en versión no cubierta.
- **Multianual:** precio anualizado = precio total ÷ nº de años; el +25% se aplica a esa cifra **cada año** hasta que
  se actualice.
- El recargo **cesa** en cuanto se actualiza a una versión cubierta.
- **Hosting:** afecta a **Odoo.sh** y **On-Premise**. **Odoo Online** no (se actualiza solo, nunca queda en versión
  antigua).

### Entrada en zona de +25% por versión (aproximada)

| Versión | Lanzamiento | Entra en zona +25% (~lanzamiento + 3 a 6 m) | Retirada Odoo.sh |
|---|---|---|---|
| v14 | Oct 2020 | ya aplica | 31 oct 2026 |
| v15 | Oct 2021 | ya aplica | 31 oct 2027 |
| v16 | Oct 2022 | **~abr 2026** | 31 oct 2028 |
| v17 | Nov 2023 | **~may 2027** | 31 oct 2029 |
| v18 | Oct 2024 | ~abr 2028 | 31 oct 2030 |
| v19 | Sep 2025 | ~mar 2029 | 31 oct 2031 |

> El **mes exacto** de entrada en el +25% depende de la **fecha de renovación** del cliente y del cálculo oficial de
> Odoo; usa esta columna como orientación y confirma por cliente.

### Árbol de decisión

```
¿Enterprise?                                   no → no aplica
  └ sí → ¿version fuera del top-3 (≳3a6m)?     no → no aplica (version cubierta)
      └ sí → ¿renovacion posterior al 4-jul-2025?
              ├ no (pre-4-jul-2025, p.ej. multianual que acaba 2028) → NO afectado hasta que venza el contrato
              └ sí → +25% sobre el precio anualizado, cada ano, hasta actualizar a version cubierta
```

### Cálculo de impacto (para argumentario/presupuesto)

```
Coste anual de licencias × 0.25 = Sobrecoste anual por quedarse
```

Ejemplo: 50 usuarios en el plan Personalizado a 27,90 €/usuario/mes (pago anual, precio de lista, España,
oct-2026) → 50 × 27,90 × 12 = 16.740 €/año → sobrecoste 25% = **4.185 €/año** recurrentes evitables
actualizando a versión cubierta. Los Light Users (4,90 €/usuario/mes) van en línea aparte y también suben.

### Casos peliagudos (los "intríngulis")

- Renovó en **feb 2026** y actualiza en **jul 2026** → paga igualmente el +25% del ciclo de **abril 2026** (la
  renovación es posterior al 4-jul-2025 y en abril seguía en versión antigua).
- Renovación **multianual a 5 años** posterior al 4-jul-2025 → +25% **cada año** hasta que actualice.
- Renovación **anterior** al 4-jul-2025 → blindado hasta que el contrato venza, aunque cruce abril 2026 en versión
  antigua.

---

## Cómo cruzar ambos hechos por cliente

- **En Odoo.sh + versión antigua** → tiene fecha de expulsión de Odoo.sh (A) → argumento de deadline forzado para
  venir a servidores Indaws/Dynapps.
- **Enterprise + versión fuera del top-3 + renovación post-4-jul-2025** → paga +25% (B) → argumento económico
  recurrente.
- Ambos a la vez = caso de máxima urgencia comercial (candidato prioritario a Rescue Pack + migración).

`common-skills:indaws-env-monitor` puede **marcar** qué clientes cumplen (A) y el candidato de (B), pero no conoce la fecha
de renovación del contrato: para (B) confirma siempre la fecha de renovación con comercial antes de afirmar el +25%.

---

## Fuentes

- **Odoo.sh FAQ** — calendario de retirada + regla de 3 tramos: https://www.odoo.sh/faq
- **Odoo 19 · Standard and extended support** (doc oficial): https://www.odoo.com/documentation/19.0/administration/standard_extended_support.html
- **Ventor.tech · "Will your Odoo Enterprise contract be surcharged 25%? Tricky cases explained"** — corte del
  4-jul-2025, multianuales y casos peliagudos: https://ventor.tech/odoo/will-your-odoo-enterprise-contract-be-surcharged-25-percent/

## Provenance

- (A) y (B) verificados contra documentación pública de Odoo el 2026-07-01.
- Creado: 2026-07-01.
