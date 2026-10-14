# Trampas del vertical y del recorrido

Lo que hace perder una tarde. Las cinco primeras están también en `SKILL.md`
porque cambian lo que decides; el resto vive aquí para no cargar el índice.

1. **El cron de importación se apaga solo** tras su primera ejecución, y es
   invisible a un `search` normal por estar inactivo. Sin él todo queda encolado
   y parece que la importación falla.
2. **Pasarse del 100 % al certificar NO es un fallo**: los módulos de exceso
   quitan el tope a propósito. Que salte el error sí lo sería.
3. **La configuración de certificación se congela** con la primera certificación:
   cada modo de facturación necesita **su propio proyecto**.
4. **Duplicar un presupuesto arrastra el proyecto** y contamina la obra original
   con tareas duplicadas. Vaciar `project_id` en la copia antes de nada.
5. **El asistente «Añadir líneas» exige secciones**. Sin capítulos ofrece cero
   líneas, en silencio.
6. **Capturas y render**: una captura demasiado pronto sale en blanco, y el
   renderizador se congela. Detalle y remedios en
   `references/automatizacion-visual.md`.
7. **En 17.0 la entrada es `/web`**, no `/odoo`.
8. **`recorrido_visual.py` no pilota Chrome y nunca lo hará.** Un script no puede
   conducir la extensión: eso lo hace Claude. Si se eligió Chrome, no busques un
   script — sigue el procedimiento de la referencia.

---

## Del recorrido, no del vertical

Aprendidas cometiéndolas. Todas producían **informes falsos**, que es peor que
no informar.

10. **Un total esperado escrito a mano miente en cuanto cambia el fichero.** Se
    lee de la cabecera del `.bc3`, no de una constante. Un descuadre de 271,50 €
    salió de aquí, y no existía.
11. **Localizar el trabajo de importación como «el último con id mayor» cruza
    recorridos.** Con dos procesos a la vez, cada uno mide el presupuesto del
    otro. El trabajo se ata a **su** fichero y a su pedido de destino.
12. **Una comprobación que no puede fallar es peor que ninguna.** Anotar `ok` sin
    comparar con lo esperado, o comparar 0,00 con 0,00 porque tocó una línea de
    precio cero. Antes de fiarte de un barrido, comprueba que **protesta** ante
    un caso malo conocido.
13. **Clasificar por olfato de texto se equivoca en las dos direcciones.**
    Deducir «no aplica» buscando la palabra «falta» en un motivo marcó como
    inexistentes seis caminos cuyo motivo era «hacen falta tres periodos». Las
    listas de clasificación se declaran, no se adivinan.
