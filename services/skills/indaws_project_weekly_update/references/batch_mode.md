# Modo Batch: procesar varios proyectos a la vez

## Regla de oro

**Este skill procesa UN proyecto por invocacion.** Nunca llames a
`project-weekly-update` con una lista de proyectos en la misma conversacion:
el contexto se contamina entre proyectos, los ficheros temporales colisionan
y el LLM termina redactando con datos mezclados o inventando campos.

Para procesar varios proyectos de golpe, usa **subagentes paralelos**: uno
por proyecto, cada uno con su propio contexto aislado.

## Nota sobre el Paso 4.5 (validacion del borrador) en modo batch

El Paso 4.5 del workflow exige validacion manual del HTML por parte del
usuario antes de escribir en Odoo (GOTCHA #9 de SKILL.md dice "NUNCA
saltar"). En modo batch esta validacion **no desaparece**: se delega al
orquestador. El orquestador es responsable de:

1. Validar la **lista de proyectos** con el usuario ANTES de lanzar los
   subagentes (esto reemplaza la validacion individual del borrador).
2. Instruir a cada subagente para que ejecute todo el workflow sin
   detenerse en Paso 4.5 (modo desatendido).
3. Confiar en `create_update.py` como red de seguridad final: aborta si
   los campos de salud estan vacios, si el nombre no es canonico, o si
   el HTML mezcla idiomas.

La combinacion de (validacion humana de la lista) + (fail-fast del script)
es equivalente en garantias a la validacion individual del borrador, sin
bloquear el procesamiento en paralelo.

## Patron con Agent() (recomendado)

Desde la conversacion principal (o desde un skill orquestador como
`consultant-daily-planner`), lanza un `Agent` por proyecto:

```
Agent(
  description: "Weekly update proyecto 1165",
  subagent_type: "general-purpose",
  prompt: "Ejecuta el skill project-weekly-update para el proyecto ID 1165
           (Hotel Portomar - v18) para la semana del 2026-04-06 al 2026-04-10.
           Credenciales Odoo: URL=..., DB=..., USER=..., API_KEY=...
           Sigue el workflow estricto: Paso 0 (recopilacion) -> Paso 1
           (project-health-monitor automatico) -> Paso 2 (fetch semanal) ->
           Paso 3 (prepare_update.py) -> Paso 4 (redactar HTML en espanol) ->
           Paso 4.5 NO aplicar (modo batch, autoaprobar) -> Paso 5
           (create_update.py). Devuelve el ID del project.update creado o
           actualizado. Si algun paso falla, aborta y reporta el error."
)
```

Lanza en **paralelo** enviando multiples tool calls `Agent` en el mismo
mensaje. Cada subagente:

- Tiene su propio contexto limpio (no ve datos de otros proyectos).
- Usa `/tmp/pwu_{project_id}_*.json` (no colisiona con otros).
- Ejecuta la cadena completa, incluyendo `project-health-monitor`.
- Devuelve un resultado compacto al orquestador principal.

## Recoleccion de resultados

El orquestador principal recopila los resultados:

```
Para cada proyecto en la lista:
  - proyecto_id, proyecto_nombre
  - update_id creado/actualizado
  - health_score resultante
  - accion (created/updated/skipped)
  - error (si lo hubo)

Genera una tabla resumen al final.
```

## Validacion del orquestador

Antes de lanzar los subagentes, el orquestador debe:

1. **Leer la lista de proyectos** (desde MD, CSV, o query a Odoo).
2. **Validar que cada proyecto tiene ID numerico** y nombre.
3. **Confirmar con el usuario la lista completa** antes de lanzar nada.
4. **Pasar credenciales unicas** a cada subagente (no reutilizar entre ellos).

## Modo batch NO permitido sin subagentes

Procesar varios proyectos en la conversacion principal (sin subagentes)
esta **prohibido** por estas razones empiricas documentadas:

- El LLM empieza a reutilizar datos del proyecto anterior.
- Salta pasos del workflow para "acelerar" (tipicamente
  `project-health-monitor` y `prepare_update.py`).
- Reescribe el nombre del registro (deberia ser canonico "Seguimiento
  Semanal ..." y termina como "Weekly Update ..." en ingles).
- La nota interna de salud sale vacia o con datos de otro proyecto porque
  no hay JSON de metricas propio.

El script `create_update.py` detecta estos sintomas y aborta, pero es mejor
prevenir con arquitectura (un subagente por proyecto) que depender del gate.

## Checklist pre-batch

Antes de lanzar un batch con N subagentes:

- [ ] La lista de proyectos esta validada y confirmada con el usuario.
- [ ] Las credenciales de Odoo son unicas y se pasan a cada subagente.
- [ ] Cada subagente tiene instruccion explicita de ejecutar
      `project-health-monitor` (no saltar).
- [ ] Cada subagente devuelve resultado compacto (ID + status + score).
- [ ] Hay un plan para procesar fallos parciales (retry manual).
