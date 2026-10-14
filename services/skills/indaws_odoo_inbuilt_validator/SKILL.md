---
name: indaws-odoo-inbuilt-validator
description: >-
  Valida el vertical de obra inbuilt en una instancia Odoo recorriéndolo SOLA y con navegador:
  importación FIEBDC-3, proyecto, material, certificación por mediciones y los nueve modos de
  facturación a origen. Pilota Chrome o Playwright y guarda captura por camino. Trae 83 caminos
  probados con sus trampas y defectos conocidos, y barre las pantallas e informes PDF del vertical
  buscando los rotos. Usar cuando pidan "valida el vertical de obra", "probar inbuilt", "comprobar
  las certificaciones", "la factura de certificación sale mal", "revisar la importación de BC3".
version: 1.0.2
effort: high
model: opus
allowed-tools: [Read, Write, Bash, Glob, Grep, Agent, Skill, AskUserQuestion, common-skills:odoo-flow-tester]
---

# Odoo inbuilt Validator v1.0

## When to Use This Skill

Cuando hay que **decidir si el vertical de obra funciona** en una instalación:
tras implantarlo, tras migrar de versión, o cuando el cliente dice que los números
de una certificación o una factura no cuadran.

**Frente a las que se le parecen:** `odoo-flow-tester` y
`odoo-flow-coverage-planner` trabajan por API y derivan lo visual a una persona;
`questionnaire-generator` es de preventa. **Esta automatiza el recorrido visual
ella misma**: la mitad de los fallos de inbuilt solo se ven en pantalla.

## Workflow

### Paso 0 — Entrada [DETENTE dos veces]

**Pregunta poco: casi todo se detecta.**

**0a · Conexión.** URL, base, usuario y clave o API key
(`references/shared/odoo-connection.md`), por entorno (`ODOO_URL`,
`ODOO_DB`, `ODOO_USER`, `ODOO_KEY`) o `config.json` — **nunca escritas en un
fichero de la skill**. **DETENTE hasta tenerlas.**

**0b · Reconocer.** `scripts/reconocer_instancia.py` detecta versión, edición,
compañía, divisa, módulos y **qué bloques aplican**. Enséñaselo al usuario: si no
es lo que esperaba, el problema es la conexión.

**0c · Tres preguntas**, juntas, con `AskUserQuestion`: **¿staging o
producción?** (no se detecta; nunca escribas en producción), **¿hay un `.bc3`
real?** (uno fabricado valida menos, y eso llega al informe) y **¿Chrome o
Playwright?** — Chrome lo pilota Claude y **el usuario inicia sesión**; Playwright
lo pilota el script y es la **vía por defecto**. **DETENTE y espera.**

### Paso 1 — Preparación [sonnet · script]

`scripts/comprobar_preparacion.py`: las 14 condiciones sin las que los caminos
mueren sin explicación (`references/preparacion.md`). **Si falla, para**: un camino
que muere por falta de dato se reporta como defecto falso.

Fija aquí el **alcance**: los bloques que 0b marcó aplicables y **por escrito los
descartados con su motivo** — eso distingue después un `bloqueado` de un defecto.
Ojo a los matices del reconocedor: cambian el resultado esperado de un bloque que
sí aplica.

### Paso 3 — Recorrido visual [opus · principal]

**El grueso del trabajo, no un anexo.** Regla de canal: **si un usuario lo vería
en pantalla, se recorre con el piloto** — de los 14 defectos del primer recorrido,
13 se veían en pantalla. La API corrobora cifras; no sustituye a haberlo visto.

Con **Playwright**, `scripts/recorrido_visual.py`, un camino por invocación y
**siempre con `--esperar`**: sin él, si la navegación no llega, el piloto
fotografía la pantalla anterior y la da por buena. Con **Chrome lo conduces tú**,
sin script. Una **captura por camino**, con su id en el nombre; el montaje de
datos va siempre por API (`references/automatizacion-visual.md`).

### Paso 4 — Corroboración por API [sonnet · script]

Cuadrar el céntimo de los caminos `visual+api` —si captura y dato discrepan, una
de las dos miente y eso ya es un hallazgo— y recorrer los pocos `api` puros.

### Paso 4b — Barrido de pantallas e informes [script + piloto]

Los caminos siguen el proceso: solo pisan las pantallas del recorrido feliz.
Medido en una instalación real, **59 de las 119 vistas del vertical estaban en
modelos que ningún camino abre**, 40 heredadas (`references/barrido-pantallas.md`).

```
python3 scripts/barrer_pantallas.py          # 1. API: todos los pares (modelo, tipo)
python3 scripts/barrer_pantallas.py --plan   # 2. órdenes del piloto: lo que tiene menú
python3 scripts/barrer_informes.py           # 3. plan de los informes PDF
```

La lista sale **de la instancia**, nunca a mano. El nivel 2 no llega a asistentes
ni diálogos: **el informe debe decir qué quedó cubierto solo por API**.

### Paso 5 — Entrega [opus · principal]

`templates/resultados.yaml`, luego `scripts/verificar_cobertura.py` —que
**rechaza** el informe si queda un camino sin recorrer o sin captura— y se redacta
con `templates/parte-de-defectos.md`.

## Rules and Guidelines

**NEVER.** Marcar `ok` sin haber visto el resultado —si no se pudo comprobar es
`bloqueado`, y se dice por qué—. Teclear contraseñas en un formulario. Ejecutar
sobre producción los caminos que escriben.

**ALWAYS.** Empezar por `comprobar_preparacion.py`. Captura de cada camino
visual, con su id. Ante la duda de canal, **elegir el piloto**.

Las seis de cada lista, con su porqué, en `references/reglas.md`.

## GOTCHAS

Las dos que más tiempo cuestan; las trece —incluidas las del propio recorrido,
que producían informes falsos— en `references/gotchas.md`.

1. **El cron de importación se apaga solo** tras la primera ejecución y es
   invisible a un `search` normal. Sin él todo queda encolado y parece que la
   importación falla.
2. **Antes de fiarte de un barrido, comprueba que puede fallar.** Un aprobado que
   nadie ha comprobado es peor que no barrer.

## Success Criteria

Todo camino aplicable con estado y `verificar_cobertura.py` en 0; captura de cada
camino visual; cada `defecto` con reproducción y evidencia; y los ya conocidos
separados de los nuevos.

## Delegates to

`odoo-flow-tester` (common), vía `Agent()`, para el canal API si el volumen lo pide.

## Consumed by

Consultores en implantación o migración de un cliente con vertical de obra.

## References

| Fichero | Contenido | Cargar cuando |
|---|---|---|
| `references/shared/odoo-connection.md` | Patrón de conexión | Paso 0 |
| `references/automatizacion-visual.md` | Chrome vs Playwright, sesión, capturas | Pasos 0 y 3 |
| `references/preparacion.md` · `catalogo-caminos.md` | Las 14 condiciones previas; los 83 caminos con canal y resultado esperado | Pasos 1, 3 y 4 |
| `references/barrido-pantallas.md` | Por qué es un barrido y no un camino | Paso 4b |
| `references/mensajes-error.md` · `defectos-conocidos.md` · `gotchas.md` · `reglas.md` | Literales de error, defectos ya vistos, las trece trampas y las reglas completas | Paso 5 y cualquiera |
| `scripts/` · `templates/` | Reconocimiento, preparación, piloto, barridos, cobertura; y las plantillas de salida | Todos |
