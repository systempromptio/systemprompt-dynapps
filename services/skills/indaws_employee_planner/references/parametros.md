# Parámetros: de la doctrina al script, sin pasar por aquí

Ni esta skill ni sus scripts llevan un solo valor de planificación. Los valores viven en la tabla **«Parámetros que lee la herramienta»** del artículo *Colocar las horas en los días*, y se leen en cada pasada:

```bash
# 1. docs_get del artículo → guarda su Markdown completo en un fichero
python3 scripts/leer_parametros.py articulo.md > params.json   # 2. extrae y valida la tabla
# 3. params.json["params"] va tal cual en la entrada de colocar.py
```

## Lo que hace `leer_parametros.py`

- Localiza la tabla por su cabecera (`| Parámetro | Valor | Admite |`), no por el título de la sección.
- **Para la pasada** si falta un parámetro, si un valor no es del tipo esperado, si no está entre los que la propia tabla admite, o si pide una variante de regla que `colocar.py` todavía no sabe aplicar. El mensaje dice cuál y por qué: se repite tal cual en la cabecera, y **no se sustituye por un valor por defecto**.
- Los parámetros de la tabla que el script no conoce se devuelven en `ignorados`: dilo en la cabecera, porque significa que alguien ha añadido a la doctrina una regla que la herramienta todavía no aplica.

## Quién usa cada parámetro

Casi todos los consume `colocar.py`. Tres los usa la propia skill:

- `imputacion_sin_tarea_dias` — al leer los hechos: cuántos días hacia atrás buscar a quien imputa sin tarea (`hechos-odoo.md`).
- `preguntar_previsiones` — si en el arranque se pregunta por proyectos en previsión (Paso 4).
- `reunion_interna_por_asunto` — si una reunión con solo asistentes internos se asigna por su asunto (`hechos-odoo.md`, gotcha 5).

`reparto_semanal`, `avisar_proyecto_a_cero`, `vencidas_dentro_del_reparto`, `hueco_corto` y `hueco_corto_nombre` los aplica `colocar.py`:

| Parámetro | Lo usa | Qué hace en el script |
| --- | --- | --- |
| `hueco_corto` | `colocar.py` | `proyecto_y_gestion`: un hueco menor que el bloque mínimo alarga el turno de trabajo contiguo, o va al proyecto y la tarea de la reunión de cliente contigua (dentro de su reparto), o a gestión interna. `sin_colocar`: se deja sin colocar y se mide como `capacidad_perdida_por_fragmentos`, como en la v2.2 |
| `hueco_corto_nombre` | `colocar.py` | Nombre (`label`) del turno interno que ocupa un hueco corto suelto. Se escribe tal cual en `name` | Con `reparto_semanal = por_fechas` el script se comporta como la v2.1 (sin reparto validado); `tests/test_colocar.py` lo comprueba contra esa versión.

## Qué puede cambiar la doctrina y qué no

- **Cambiar un valor** (el horizonte, el bloque mínimo, el reparto de desarrollo, la lista de departamentos): se edita la tabla y vale desde la siguiente pasada.
- **Elegir otra variante ya implementada** (holgura al principio del día, vencidas sin prioridad, vencida a medias planificada entera): igual, se edita la tabla.
- **Una regla nueva** (una variante que no está en la columna «Admite», o un parámetro nuevo): la pasada se para y lo dice. Hay que implementarla en `colocar.py` y declararla en `SCHEMA` de `leer_parametros.py`. Las reglas fijas del script están listadas en la cabecera de `colocar.py`.

## Contradicciones entre artículos

La tabla manda para la herramienta. Si el texto de otro artículo (Dedicación, Reglas de planificación) dice otra cosa, la pasada usa la tabla y **cita la contradicción en la cabecera** como hallazgo, para que alguien la resuelva en la doctrina.
