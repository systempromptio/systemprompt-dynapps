---
name: indaws-odoo-verify-green
version: 1.4.0
effort: low
model: opus
description: >
  Comprueba si un "está verde" de Odoo es REAL antes de darlo por hecho: runs que no ejecutaron
  nada, tours que pasan sin hacer nada, verdes heredados de un documento, y arreglos que ningún
  test protege. Enseña a verificar EN ROJO (neutralizar y ver qué cae) y trae 40 falsos verdes
  medidos, más 8 rojos inútiles o falsos. NOT revisión de calidad pre-merge → quality-gate. NOT escribir tests → odoo-test-generator.
  Triggers: "ya está hecho", "está en verde", "los tests pasan", "verifica esto", "¿seguro que
  funciona?", "dalo por bueno", "0 fallos", "antes de cerrar la tarea", "/verify-green".
---

# Verificación honesta en Odoo — ¿tu verde es real?

Existe porque **en Odoo un test puede pasar sin haber probado nada**, y el modo de fallo no es un
rojo: es un **verde**. Un rojo lo mira todo el mundo; un verde no lo mira nadie. Los 48 casos de
`references/false-greens.md` (40 verdes falsos + 8 rojos falsos o inútiles) están **medidos**, no imaginados.

## Cuándo dispara

Cuando alguien —tú incluido— está a punto de **declarar algo hecho**: «los tests pasan», «0 fallos»,
«ya está», «esa deuda es benigna». No en cada edición: en el momento de dar por bueno.

## El procedimiento, en cuatro preguntas
Contéstalas en orden. La primera que falle detiene el «hecho».

### 1. ¿El run EJECUTÓ algo?
Un exit code 0 no dice nada. **Se cuenta, no se infiere.**

```bash
grep -c "odoo.tests.result" run.log     # 0 = NO HA CORRIDO, diga lo que diga el exit code
grep -c "odoo.tests.stats"  run.log     # ¿aparecen TODOS los módulos que esperabas?
```

Un módulo que desaparece de `odoo.tests.stats` es un verde falso. Causas medidas: variable de
módulos vacía por el `cwd`, puerto ocupado por un run zombi, y un módulo cuyo nombre acaba en `py`
(bug del selector de tags de Odoo). Ver `references/false-greens.md` §A.

### 2. ¿Es de una base LIMPIA — y la has BORRADO al acabar?
`-u` con `--test-enable` da **rojos falsos**: siempre `-i` en base nueva. Un modelo nuevo del kernel
no tiene tabla donde el módulo ya estaba: hace falta `-u <kernel>` **sin** `--test-enable`. Y **la
base se borra en cuanto se lee su resultado**: rojo+verde son dos por arreglo, se acumulan a docenas
y acaban en trampa — nadie sabe cuál mirar, o se reutiliza una y el verde no vale. §B.

### 3. ¿Lo has verificado EN ROJO?
**El paso que más falsos verdes caza, y el que más se salta.** Neutraliza el arreglo y comprueba que
sus tests caen:

1. copia el fichero;
2. rompe la línea del arreglo a propósito (`return []`, `if False:`, invertir el orden…);
3. corre **sólo esos tests**;
4. **caen los que deben, y por el motivo que deben** (lee el mensaje, no el contador);
5. restaura y confirma que no quedan marcas.

Si no cae ninguno: o el test es teatro, o el código no hacía nada. **Las dos cosas son hallazgo.**
Y si cae uno que no esperabas, tu modelo mental del cambio está mal.

> Caso real: dos ramas del mismo largo hacían pasar un test de join AND **con y sin** el AND. Sólo
> discriminaba con ramas de largos distintos. El test llevaba horas en verde sin afirmar nada.

### 4. ¿Lo has comprobado en la capa donde falla?
La trampa más cara. Cada capa se verifica en la suya:

| Lo que cambiaste | Dónde se verifica | Dónde NO |
|---|---|---|
| Modelo / lógica | suite con `-i` en base nueva | — |
| Vista, widget, JS | **tour** (`-standard`: la suite NO los corre) | la suite |
| Un botón, un clic | tour + aserción **en Python sobre lo GUARDADO** | «el paso pasó» |
| Instancia viva tras `-u` | **HTTP contra el proceso vivo** + `docker restart` | `odoo-bin shell` |
| Una deuda «benigna» | medirla | leerla |

## Enrutado, y dónde se para

Contar no es juzgar, así que el reparto no es uniforme:

| Parte | Modelo | Por qué |
|---|---|---|
| Preguntas 1-2 — contar la línea de resultado, los módulos en `stats`, cómo se lanzó el run | **sonnet**, o inline si es un `grep` | mecánico y verificable; un modelo caro no acierta más aquí |
| Preguntas 3-4 — decidir si el verde significa algo y en qué capa | **opus**, principal | es juicio: hay que leer el mensaje del test y el cambio, no un contador |

Con volumen la parte mecánica puede ir a un `Agent(model: sonnet)` —**condicional, nunca
obligatorio**: hay configuraciones que prohíben el fan-out, y un paso obligatorio que no se ejecuta
es otro verde falso. Sin fan-out, el mismo `grep` inline y se dice.
Y dos paradas que son de verdad:

1. La pregunta 3 **modifica código a propósito**. Si el árbol no es tuyo o tiene cambios sin
   commitear, **pregunta antes** de tocar nada, o hazlo sobre una copia.
2. Si no puedes verificar en rojo (sin entorno, sin datos), **no lo declares hecho**: entrega el
   «hecho» con la salvedad escrita. Esperar entorno sale más barato que un verde falso ante el cliente.

## GOTCHAS

1. **Un paso de tour puede PASAR sin hacer nada.** Un `<option>` de un `<select>` cerrado no es
   visible, así que una aserción sobre él **jamás** puede pasar; y `run: "select X"` casa por VALOR,
   no por etiqueta — si no existe, no cambia nada y el paso sigue. Afirma sobre el `<select>`, expón
   lo que necesites en un `data-`, y **el texto se afirma en Python**.
2. **Un `HttpCase` falla ante cualquier error JS no capturado aunque TODOS los pasos pasen.** En el
   log sale `TOUR ... SUCCEEDED` y el test rojo a la vez. No es contradictorio: es la señal.
3. **`odoo-bin shell` monta su propio registro fresco**, así que no verifica el proceso que sirve la
   web. Tras un `-u` con `--stop-after-init`, el servidor vivo se queda con el registro viejo:
   `docker restart` y comprobar por HTTP.
4. **`assertRaises` de Odoo abre un savepoint y lo revierte**, así que borra la traza que ibas a
   comprobar. Usa `try/except` a mano.
5. **`search_count` esconde los `active=False`.** Cualquier medida sobre un modelo con `active` va
   con `.with_context(active_test=False)` o el «+0» es mentira.
6. **Un test `at_install` ve el registro A MEDIAS.** Todo lo que razone sobre el registro entero va
   `@tagged("post_install")` o pasa en verde con la fuga abierta.
7. **Una puerta de CI que no se ha visto en rojo no está verificada.** Córrela contra código que
   DEBE fallar antes de creerte su verde.
8. **Un `raise` dentro de un motor que aísla errores no avisa a nadie.** Colas, crons y motores de
   integración lo capturan (`except Exception`) y lo anotan en un modelo técnico: el test que afirma
   `state == 'error'` da verde afirmando **lo que deja el `except`**, no que el usuario reciba el
   aviso. Y `assertTrue(rec.error_message)` no dice CUÁL: afirma el contenido (C7, C8).

## NEVER
- Declarar hecho algo verificado en **otra** sesión o heredado de un documento. Se reejecuta aquí.
- Creerse un contador sin leer el mensaje. Llamar benigna a una deuda sin medirla.
- Tumbar un test que falla tras tu cambio sin **leer por qué lo escribieron**.

## ALWAYS
- Contar la línea de resultado y los módulos en `stats`. Verificar en rojo lo arreglado.
- **Borrar la base de cada pasada al leer su resultado.** Dejarlas es dejar trampas.
- Decir qué NO has verificado: un «hecho» con salvedad escrita vale, uno sin ella es sorpresa aplazada.

## Delegates to
- `odoo-test-generator` (developer) — cuando el hallazgo es «esto no tiene test».
- `odoo-pilot` (common) — comprobar contra el proceso vivo por HTTP. El contrato de conexión (MCP
  primario / odoo-pilot fallback) está en `references/shared/odoo-connection.md`: **no lo
  reimplementes**, y nada de curl/xmlrpc ad-hoc — así se acaba «verificando» contra el proceso
  equivocado. El listón de qué se puede llamar entregado está en
  `references/shared/delivery-standard.md`; esto es el paso ANTERIOR a ese veredicto.

## Consumed by
Todas cierran con un veredicto, y este es el paso que decide si el veredicto vale.

| Skill | Suite | Para qué |
|---|---|---|
| quality-gate | common | Gate 2 — un «los tests pasan» sin verificar puntúa BLOCKED, no PASS |
| odoo-code-auditor | developer | Dimensión Code Quality — tests que nunca se han visto caer no son cobertura |
| odoo-test-generator | developer | Paso final: neutralizar el código y ver caer los tests antes de entregarlos |
| git-precheck | developer | Cuestionar el «pasan» del dev sin correr la suite (tours no ejecutados) |
| git-review-pr | developer | Auditar el verde del pipeline antes de contarlo como CI superada |

## REFERENCES
- `references/false-greens.md` — los 48 casos medidos (40 verdes falsos + 8 rojos falsos o inútiles,
  marcados `[ROJO FALSO]` / `[ROJO INÚTIL]`), con síntoma, causa y cómo se pilla.
- `references/verify-in-red.md` — el procedimiento paso a paso, con dos casos reales.
