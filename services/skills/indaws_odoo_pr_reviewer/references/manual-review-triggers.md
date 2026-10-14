# Disparadores de Revisión Manual (guía viva)

**Principio**: la automatización (tests, `/review`, `quality-gate`, Docker) pesca mucho, pero
**un test verde NO prueba que la decisión sea correcta**, solo que el código hace lo que el test
dice. Hay cosas que exigen ojo humano sí o sí. Este fichero es la lista de esos disparadores y
**crece con cada review** (ver "Cómo crece" al final).

Regla: si el diff toca cualquiera de estos → marcar **REVISIÓN MANUAL OBLIGATORIA** en el feedback,
indicando **quién** debe mirarlo (consultor / contable / CTO / seguridad / el propio arquitecto).
No se aprueba solo porque los tests pasen.

> **Recall vs precisión.** Añadir filas sube la **cobertura** (menos cosas se escapan). La
> **precisión** (que lo que marcamos de verdad haga falta) NO sube apilando reglas — sube con el
> **scope preciso** de cada disparador y con la **calibración por resultado** de abajo. Sin eso, la
> guía se vuelve ruido y todo acaba en "revisión manual" → fatiga de alertas. Crecer ≠ mejorar.

---

## Tabla de disparadores (con scope preciso)

El **scope** es lo que da precisión: no "toca account.move", sino la condición estrecha que de
verdad exige ojo humano. Estrechar el scope es cómo baja el falso positivo.

| # | Disparador | Scope preciso (cuándo dispara) | Por qué el test no basta | Quién |
|---|-----------|-------------------------------|--------------------------|-------|
| 1 | Contabilidad / impuestos | Cambia **cálculo** de impuesto/asiento/conciliación, no un label | Verde ≠ correcto legal/fiscal | Contable |
| 2 | Dinero / redondeos | Opera importes **Monetary** o redondeo/multi-currency | Descuadre de céntimos a escala | Contable/arq. |
| 3 | Migración de datos reales | Script pre/post/end que **transforma** datos existentes | Corre en verde pero convierte mal | Humano (muestras dump) |
| 4 | Seguridad ACL/rules | Añade/edita `ir.model.access` o `record rule` o dominio de una | Puede pasar y exponer al grupo erróneo | Seguridad/arq. |
| 5 | `sudo()` / bypass | Aparece `sudo()` o `with_user` que salta permisos | Salta el modelo de seguridad | Arquitecto |
| 6 | Valoración stock | Toca quants/layers/coste, no un campo informativo | Efecto coste/contable no cubrible en unit | Logística/contable |
| 7 | Integración externa | Llama API/webhook real (no mock) | El test mockea; el real falla distinto | Humano (vs sistema ext.) |
| 8 | Rendimiento a escala | computed **stored** en tabla grande, query en bucle, cron batch | Test con 3 registros no ve el N+1 | Arquitecto |
| 9 | Concurrencia/locks | Escritura concurrente al mismo registro, idempotencia | No reproducible en test determinista | Arquitecto |
| 10 | Cascada/`ondelete` | `ondelete` o unlink masivo | Pérdida de datos irreversible | Humano |
| 11 | `xpath` frágil | Herencia de vista por posición, no por atributo | Rompe en otra config/módulo | Humano (varias configs) |
| 12 | Cron / server action | Nueva/modificada acción automática en background | El unit test no la dispara de verdad | Arquitecto |
| 13 | UX / traducciones | Texto de cara al usuario, término de negocio | No lo cubre un unit test | Consultor |
| 14 | Multi-company | Regla/default/dominio con `company_id` | Fuga entre compañías | Arquitecto |
| 15 | Emails / message_post | Contenido, destinatarios, adjuntos de correo | Se ven, no se asertan bien | Humano |

---

## Cómo usarla durante el review

1. Cruzar los ficheros/modelos tocados con la columna **scope preciso** (no con el título): solo
   dispara si se cumple la condición estrecha, no por rozar el modelo.
2. Por cada disparador que aplique → entrada en **"Revisión manual obligatoria"** del feedback
   (`templates/pr-review-feedback.md`), con el porqué y **quién** firma.
3. **Sin firma humana → bloquea el merge**, aunque lo automático esté verde.

## Calibración por resultado (esto es lo que sube la precisión)

Cada vez que un disparador se activa, se anota el **resultado real** en el ledger de abajo. Con eso
se decide mantener / estrechar / podar. Es el bucle que distingue señal de ruido.

Ratio de utilidad = `veces útil / veces disparado`.

- **Útil** = la revisión manual encontró un problema real (o confirmó un riesgo que el test no cubría).
- **No útil** = la revisión no encontró nada; el flag fue ruido.

Reglas de ajuste (aplicar en el Step 6 al cerrar):
- Ratio **alto** (pesca a menudo) → mantener tal cual.
- Ratio **bajo tras ≥5 disparos** → **estrechar el scope** (¿qué tenían en común las veces que SÍ
  pescó? esa es la condición nueva). Estrechar > borrar.
- Ratio **≈0 tras ≥8 disparos** aun con scope estrecho → **podar** (degradar a "opcional" o retirar).
- Disparador nuevo descubierto en un review → añadir fila con **scope lo más estrecho posible desde
  el día 1** (no empezar ancho). Si es gotcha Odoo general → proponerlo a `odoo-deep-knowledge`.

## Ledger de calibración (append-only)

<!-- Una línea por activación real. Formato:
     {fecha} · PR {url} · disparador #{n} · resultado: útil|no-útil · nota -->

| # | Disparado | Útil | Ratio | Última vez útil | Acción vigente |
|---|-----------|------|-------|-----------------|----------------|
| 1 | 0 | 0 | — | — | mantener |
| … | | | | | |

<!-- Activaciones (histórico crudo, para reconstruir la tabla): -->
- {aún sin entradas — se rellena con los reviews reales}
