# Tarea para Desarrollador

**Titulo**: {titulo conciso} | **Proyecto**: {nombre} | **Asignado**: {dev}
**Nivel dev**: Junior/Mid/Senior | **Prioridad**: Urgente/Alta/Media/Baja
**Estimacion**: {h con factor IA} | **Odoo**: {X.0} | **Limite**: {fecha}
{Si custom no trivial / integración / migración de datos → banda: **probable {Y}h · peor {Z}h**. Si hay fase que depende de una medición previa → marcarla ABIERTA: "Fase C escala con Fase B".}
**Nivel de entrega**: {trivial / estandar / complejo} — determina qué exige el estándar (ver `references/shared/delivery-standard.md`; cortes en `delivery-thresholds.json`)

## Contexto de Negocio
{POR QUE: "{Quien} necesita {que} para {por que}. Actualmente {situacion}. Con esto {mejora}."}

## Objetivo
{QUE: comportamiento esperado, NO implementacion. "En {donde}, {que se ve/hace}."}

## Alcance
**Incluido**:
- {item 1}
- {item 2}

**Excluido** (evita scope creep):
- {item NO 1}
- {item NO 2}

## Gotchas conocidos que aplican

> Consultar ANTES de codificar (evita repisar minas ya documentadas). El redactor los precarga
> desde `odoo-deep-knowledge` + el `project-knowledge` del cliente para los modelos/flujo de esta tarea.

- {ID gotcha — comportamiento a tener en cuenta} (p.ej. `MCG23`, `STKG74`)
- {— o "ninguno conocido para estos modelos" —}

## Punto de partida verificado (no re-investigar)

> Solo si el redactor YA exploró el código. Lista lo confirmado (con evidencia) para que el dev
> no repita el análisis. Separa "ya confirmado" de "a verificar". Omitir si no se exploró.

- **Ya confirmado**: {hallazgo — evidencia `file:line` / resultado de probe}
- **A verificar por el dev**: {lo que queda abierto}

## Mecanismo central (fuente)

> El método/campo/constraint del que depende TODO el diseño, trazado a la versión objetivo.
> Obligatorio para config-only y specs de stock/UoM/FSM/pagos/contabilidad (NEVER #8 / gate Step 2.5).

- **Mecanismo**: `{método/campo}` en `{addon/file.py:line}` (Odoo {X.0}) — {qué hace y por qué el diseño depende de él}
- {— o "verificado por probe con rollback vía odoo-flow-tester: {resultado}" —}

## Impacto Odoo

> Solo los puntos que APLICAN, cada uno con la instrucción concreta para el dev
> (`references/odoo-impact-checklist.md`). Omitir en nivel trivial.

- {p.ej. **Registros existentes**: recalcular `margin_percent` en post-init para las N líneas ya cargadas}
- {p.ej. **Seguridad**: el campo muestra costes → `groups="grupo_ver_costes"` en campo y vista}
- {— o "sin impactos transversales relevantes para estos modelos" (solo si de verdad ninguno aplica) —}

## Guia de Implementacion

> Orienta, no dicta. Si encuentras camino mejor, proponlo.

{Adaptar detalle al nivel del dev — ver `references/level-scaffolding.md`:}
{- Junior: pasos numerados con archivos, modelos, refs concretas}
{- Mid: enfoque sugerido + constraints + "propon antes de codificar"}
{- Senior: solo constraints + "presenta propuesta tecnica"}

### Paso 1: {Titulo}
{Que modelo/vista tocar, que patron seguir, que documentacion, que trampas}

### Paso 2: {Titulo}
{Idem — solo para Junior/Mid. Senior no necesita pasos.}

## Criterios de Aceptacion

Verificables tipo "Cuando X, entonces Y" con valores concretos — **formato ejecutable**,
enlazable con `odoo-flow-tester`/`odoo-test-generator` (no prosa vaga como "funciona bien"):

- [ ] {Cuando X (con valores concretos), entonces Y}
- [ ] {Cuando edge case Z, entonces W}
- [ ] Tests unitarios cubren logica principal
- [ ] Code review aprobado por TL
- [ ] **Prueba de flujo funcional** end-to-end pasada (escenario ejecutable) — obligatorio salvo nivel trivial
- [ ] **Manual/README** de uso-configuración en el repo cliente — solo nivel complejo

## Contexto de origen

| Recurso | Enlace | Que aporta |
|---------|--------|------------|
| Registro origen | {modelo}/{id} — {url} | Peticion original y conversacion |
| {nombre adjunto} | adjunto #{id} | {que aporta} — marcar **DECISIVO** si cambio el veredicto |

> Los adjuntos DECISIVOS van re-adjuntados a esta tarea. El resto, enlazados arriba.

## Acceso al codigo

- **Repo**: `indawsodoo/{repo}` · **Rama**: `{rama}` · **Version del modulo**: `{x.y.z}`
- **Como bajarlo** (usa la via que tengas disponible):
  - `gh repo clone indawsodoo/{repo} -- --depth=1`
  - `git clone --depth=1 git@github.com:indawsodoo/{repo}.git`
  - ZIP: `https://github.com/indawsodoo/{repo}/archive/refs/heads/{rama}.zip`
- **Pieza a extender**: `{modulo}` → `{modelo / campo / xmlid exacto}` — {por que esa}

## Referencias

> Verificar cada ref antes de citarla: existe, está a la versión objetivo, patrón vigente.
> **Nunca una ruta local** (`/opt/…`, `~/…`, un contenedor o una BD de tu máquina): el dev no tiene
> tu disco. Cada recurso va como **repo + rama + ruta relativa al repo**, o como **adjunto de la
> tarea** si no está en git (specs del cliente, ficheros de ejemplo, planes de prueba). El entorno
> se describe por lo que hace falta —versión, addons, skill que lo levanta—, no por dónde está el tuyo.

| Recurso | Dónde (repo+rama / adjunto) | Versión | Para que |
|---------|-----------------------------|---------|----------|
| {modulo ref} | {repo `org/repo`, rama `X.0` → `ruta/relativa`} | {X.0} | {patron a seguir} |
| {spec / fichero de ejemplo} | **Adjunto**: `{nombre-fichero}` | — | {para qué} |

## Restricciones

- {restriccion + POR QUE — nunca restriccion sin motivo}

## Preguntas Abiertas

- {confirmar antes de empezar — si >3, tarea no esta lista}

## Ayuda

- Timer: pide ayuda si llevas {Junior: 1.5h | Mid: 4h | Senior: 1 dia} atascado sin progreso
- Contacto: {nombre del redactor/TL} por {canal}

## Pruebas funcionales realizadas
*(rellena el DEV al entregar — es lo que el consultor re-ejecuta con su Claude)*

- **Flujo probado**: {de principio a fin, en lenguaje de negocio}
- **Datos usados**: {registros/valores reales del cliente aportados en la tarea}
- **Escenario ejecutable**: {ref/enlace al escenario de `odoo-flow-tester` — NO prosa}
- **Resultado**: {OK / desviaciones}

## Entregables

> El PR lleva **el código**. También en una revisión o auditoría: para sacar el informe hay que tocar
> el código, así que el arreglo se entrega, no se aparca (NEVER #10). Informe y evidencias son el
> respaldo del diagnóstico, no el entregable — **el código manda sobre las evidencias**. Lo que de
> verdad no deba cambiarse todavía se nombra caso por caso y con motivo, nunca como norma general.

1. [ ] Rama `feature/{nombre}`
2. [ ] Tests
3. [ ] PR con descripcion **y con el código** (informe/evidencias, si los hay, en el mismo PR)
4. [ ] Staging validado
5. [ ] Escenario de flujo ejecutable (salvo trivial) + sección "Pruebas funcionales realizadas" rellena
6. [ ] README de uso/configuración en repo cliente (solo nivel complejo)
