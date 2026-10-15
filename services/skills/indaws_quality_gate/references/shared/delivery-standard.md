# Estándar de entrega — "Desarrollador 2.0" (shared)

Qué se exige **antes de dar por terminada** una tarea de desarrollo Odoo. Fuente única; la referencian
las skills de generación (que la aplican), `dev-task-writer` (que instrumenta la tarea) y los gates
(`git-precheck`/`quality-gate`, que la hacen cumplir). No dupliques la regla — enlaza aquí.

> Filosofía: cada tarea no solo entrega código, entrega **evidencia de que funciona** y **conocimiento
> reutilizable**. El tiempo que "sobra" al generar rápido con Claude se reinvierte en solidez, no en
> devolver la tarea a medias para que el consultor la testee.
>
> **Piedra angular.** Terminar un desarrollo no es que compile: es que pase **control de calidad** — que
> quede **demostrado que el módulo hace lo que dice dentro del flujo real de negocio**, no que "da verde"
> ni que "no peta". Ese es el objetivo del flujo end-to-end; todo lo demás de este estándar lo sirve.

## 1. Prueba de flujo funcional = control de calidad (OBLIGATORIA)

No basta con unit tests: el control de calidad de una tarea es probar el **flujo de negocio de principio
a fin** y **asertar lo que el módulo aporta** en él (campo calculado, estado, importe, línea creada), no
que la cadena corrió sin excepción.

- **Escenario EJECUTABLE, no prosa.** Se deja como escenario de `odoo-flow-tester` (no un texto
  "he probado X"), para que el consultor / su Claude lo **re-ejecute y verifique**, no solo lo lea.
- **Con datos reales del cliente.** El flujo se prueba con los datos que aporta la tarea (ver §4).

## 2. Registro en la tarea — "Pruebas funcionales realizadas"

Toda tarea entregada incluye una sección con: qué flujo se probó, con qué datos, resultado, y el
**enlace/ref al escenario ejecutable**. Es lo que el consultor abre para revisar con su Claude.
(La instrumenta `dev-task-writer` como sección obligatoria de salida.)

## 3. Manual / README por complejidad

Para flujos con cierta complejidad (ver umbral, §5), la entrega incluye un **manual de uso/configuración**
generado con `odoo-module-documenter`, versionado como **README en el repo del cliente** (no solo pegado
en la tarea). "Añade un campo" → una línea basta; flujo cross-módulo → manual completo.
Este README es además la **semilla de `project-knowledge`** (evolución futura): no es trabajo tirado.

## 4. Consumir conocimiento al ABRIR (no solo producir al cerrar)

Antes de escribir código, la tarea declara los **gotchas conocidos que aplican** a los modelos/flujo que
se van a tocar (consulta a `odoo-deep-knowledge` + `project-knowledge` del cliente). Cierra el flywheel:
el conocimiento no solo se produce, se **consume en el momento justo** para no repisar minas ya documentadas.

## 5. Umbral de complejidad — cómo se calcula y se calibra

Qué tareas obligan a §1 completo + §3 manual, y cuáles solo a una nota mínima. Tres niveles:

| Nivel | Criterio (resumen) | Exige |
|-------|----------|-------|
| Trivial | 1 campo / 1 vista / cambio cosmético | Nota de 1 línea + smoke test |
| Estándar | 1 modelo / 1 flujo dentro de un módulo | §1 escenario + §2 registro + §4 gotchas |
| Complejo | cross-módulo / toca modelo base o compartido / migración / seguridad / >N modelos / >X h | §1 + §2 + §3 manual + §4 gotchas + blast-radius (`odoo-impact-analyzer`) |

**Los cortes NO viven aquí — viven en `common:references/shared/delivery-thresholds.json`.** Ajustar el
umbral = cambiar un valor en ese JSON (PR de una línea), sin reescribir esta norma.

**Cálculo automático, no juicio manual.** El nivel se deriva de señales que ya existen, con la misma
función en dos momentos:
- al **definir** la tarea → horas estimadas de `dev-assessment`;
- en el **gate** → diff real + `odoo-impact-analyzer` (modelos tocados, base/compartido, cross-módulo,
  migración, seguridad).

**Override con justificación.** El humano puede subir/bajar el nivel propuesto con una línea
`NIVEL: <nivel> — <razón>`. Cada override es un dato de calibración.

**Calibración ágil (parámetro vivo).** Se revisa con la telemetría (Paso 4): se **mueve el corte** en el
JSON cuando una tarea de nivel inferior rompió en prod, cuando los overrides van siempre en el mismo
sentido, o cuando el % de 'complejo' es desproporcionado. Arranque **conservador**: solo lo claramente
complejo obliga a manual+flujo; rodar 2-4 semanas y apretar con datos.

## 6. Requisito para consultoría (input real)

Para que el flujo sea ejecutable con datos reales, la **tarea debe traer esos datos** (registros,
casos, valores esperados). Es parte de la definición de "tarea lista"; sin ellos, la tarea se devuelve
a consultoría, no se entrega a medias. `consultant-assistant` (validaciones) lo exige al detallar la tarea.

## Enforcement

- **Skills de generación**: aplican §1-§4 al entregar y lo exponen como regla `ALWAYS`.
- **Gate** (`git-precheck` / `quality-gate`): según nivel (§5), verifica que existan el escenario
  ejecutable + registro en tarea + README. **Sin gate esto es una recomendación, no un estándar** —
  el gate es lo que le da dientes.
- El "escenario ejecutable" permite además convertir la batería acumulada en **regresión viva por
  cliente** (evolución): se re-ejecuta en cada cambio/migración futura.
