# Rescue Pack — {CLIENTE}

> **PDF entregable al cliente.** Identidad visual y verbal con `dynapps-brand`; render con
> `pdf-renderer`. Estructura fija: Portada + 6 secciones. Núcleo = estado actual (2-3);
> valoraciones (4, 6): solo importe € (horas internas, nunca en el PDF) vía `odoo-effort-estimator`.

---

## Portada
_(dynapps-brand: logo, título "Rescue Pack — Diagnóstico técnico", cliente, fecha, versión Odoo
origen, autor Dynapps. Paleta y tipografía de marca.)_

## 1. Introducción
_(Breve presentación del cliente y de su instalación Odoo: qué es, a qué se dedica, versión y
edición, alcance de lo analizado. 1-2 párrafos, de la sesión de consultoría / Fase 0.)_

## 2. Estado de la base de datos
_(Volumen real: nº de registros de las tablas principales — salida de `scripts/db_state.sh`.
Lectura: qué flujos están vivos vs. vacíos.)_

| Tabla | Concepto | Registros |
|-------|----------|----------:|
| | | |

## 3. Estado de salud del código
_(Fase 2. Una subsección por clase. Salud 0-10 por módulo; Python/JS/XML.)_

> _(Si se genera el anexo técnico —`{detalle}`=completo—, añadir al inicio: "El detalle funcional de
> cada módulo, el análisis de flujos y las referencias a código se encuentran en el **anexo técnico**
> (documento aparte)." Y en la columna Nota de cada módulo con salud media/baja, la coletilla
> "→ ver anexo técnico".)_

> **Según el escenario (Fase 0), esta sección cambia de contenido pero NO de forma:**
> - **Caso 1 (con fuente):** auditoría real → tabla de salud 0-10 de abajo.
> - **Caso 2 (sin fuente, BD local):** salud **parcial** desde estático `.pyc` (imports/integraciones,
>   SQL, overrides, módulos licenciados) + **reconstrucción funcional** con niveles de certeza
>   (datos de `odoo-reverse-engineering`: `consolidated.json`). Sin puntuación 0-10 de calidad.
> - **Caso 3 (producción):** "**No evaluable sin fuente**"; el peso recae en la reconstrucción
>   funcional observada (menor certeza). Declararlo explícitamente.
> En casos 2/3, sustituir la tabla por la síntesis funcional por módulo + nivel de certeza.

- **Custom** _(auditoría exhaustiva: calidad/deuda, seguridad, rendimiento, convenciones)_
- **Studio** _(módulo `odoo_studio`: campos, server actions, automatizaciones — auditado igual)_
- **OCA** _(estado y mantenimiento; continuidad → sección 6)_

| Módulo | Clase | LOC | Salud 0-10 | Certeza | Nota |
|--------|-------|-----|:----:|:----:|------|
| | custom/studio/OCA | | | confirmado/observado/aprox./n.d. | |

## 4. Mejoras a realizar (valorado — dos niveles)
_(Valorado SOLO en importe — **nunca horas en el PDF**; el desglose de horas es interno.)_

### Críticas · bloqueantes (para iniciar la relación)
| Mejora crítica | Riesgo si no se hace | Importe |
|----------------|----------------------|--------:|
| | | |

### Recomendadas · no críticas (mejora de calidad, no bloqueante)
| Mejora recomendada | Beneficio | Importe |
|--------------------|-----------|--------:|
| | | |

_(Lo que dependa de una decisión funcional del cliente → sección 5; deuda de versión → sección 6.)_

## 5. Preguntas para la sesión de consultoría
_(Hallazgos NO críticos que requieren confirmación funcional del cliente antes de decidir.
Una pregunta por hallazgo ambiguo.)_

1. …

## 6. Propuesta — cómo seguir (valorado, value-based)

### 6.0 Racionalización del desarrollo a medida (necesidad funcional real)
_(Análisis de cada custom desde el negocio: ¿justifica código a medida o se resuelve con estándar /
Indaws Core / consolidando código? Es la foto que el decisor necesita para elegir qué mantener,
sustituir o eliminar. Cuatro grupos + tabla de veredictos.)_

- **Núcleo — mantener a medida:** los módulos que aportan el valor real y no tiene el estándar/Core.
- **Sustituir por Core/OCA:** con el paquete concreto (ver 6.3) y su precio.
- **Consolidar/simplificar en código (NO Studio):** micro-módulos de solo-campos → fusionar en código
  versionado o junto al módulo que los consume. **No a Studio** (los volvería opacos/no versionables,
  lo contrario a `odoo_studio`, y desacopla campos que alimentan lógica). Ver `gotchas.md`.
- **Eliminar por riesgo / no-necesidad.**

| Módulo | Necesidad de negocio | Veredicto | Alternativa |
|--------|----------------------|-----------|-------------|
| … | … | mantener / sustituir→Core / consolidar en código / eliminar | … |

_Cerrar con las **preguntas que más condicionan el alcance** (las que se llevan a la sesión de análisis)._

### 6.1+ Propuesta value-based
_(Valorado SOLO en importe — **nunca horas en el PDF**. Presupuesto formal → `quotation-builder`.
**Puerta de Fase 0:** si `{ofrecer_migracion}`=no, incluir **solo la Opción 1** y omitir la Opción 2
y la sección 7. Si es sí, las dos opciones.)_

### Opción 1 — Estabilizar y mantener (sin migrar) — "¿nos hacemos cargo del entorno?"
| Concepto | Alcance | Importe |
|----------|---------|--------:|
| Arreglar puntos críticos | ver sección 4 | € |
| Traspaso a cloud | **si es a deshoras, especificar recargo** | € |
| Indaws Cloud (hosting gestionado) | cuota | €/mes |
| Mantenimiento | bolsa de soporte | €/mes |

_Salvedad temporal (si aplica): mantener Odoo {VERSION_ORIGEN} tiene fecha límite y coste. En hosting Odoo.sh esta versión se retira el {FECHA_RETIRADA_SH} y después la base de datos queda bloqueada. En Enterprise fuera de las tres versiones más recientes, la renovación puede incrementarse un 25% (~{IMPORTE_SOBRECOSTE_ANUAL}/año). Ambos factores refuerzan la Opción 2._

### Opción 2 — Migración a Odoo {VERSION_OBJETIVO}
| Concepto | Alcance | Importe |
|----------|---------|--------:|
| Migración del código + saneamiento | … | € |
| Actualización de la base de datos | 4 iteraciones | € |
| **Formación** | N sesiones (1 sesión = 2 h) | € |
| **Soporte en arranque (1 mes)** | = **10 % del importe de la migración** | € |
| Gestión de proyecto | … | € |
| **Total Opción 2** | | **€** |

**Migraciones siguientes (mantenerse en versión soportada)**

| Concepto | Alcance | Importe |
|----------|---------|--------:|
| Migración de versión, cada {CADA_N_ANIOS} años | {PCT}% de lo que costaría implantar hoy este mismo sistema desde cero ({COSTE_IMPLANTACION_CERO}) | {IMPORTE_MIGRACION_RECURRENTE} |

_Este importe **se mantiene mientras no se amplíe significativamente la base instalada** (más de un
{UMBRAL_AMPLIACION}% en módulos a medida o integraciones). Si el alcance crece, se recalcula._

_La cifra de referencia ({COSTE_IMPLANTACION_CERO}) es una **reconstrucción del alcance actual** a partir
de lo analizado —módulos a medida, personalizaciones en la base de datos, integraciones y volumen de
datos—, no una oferta de implantación. {NOTA_SI_ESTIMADA}_

> `{NOTA_SI_ESTIMADA}` solo cuando no hubo datos para calcularla: "En este caso la cifra es **una
> estimación aportada por el equipo**, al no disponer del código fuente." Si se calculó, se omite.

_(Cuando se ofrece migración, es **la Opción 2**; estabilizar/mantener es la Opción 1. Si en Fase 0 se
decidió no ofrecerla, esta Opción 2 no aparece.)_

_Nota: importes orientativos, IVA no incluido. Continuidad OCA verificada contra ramas reales._
