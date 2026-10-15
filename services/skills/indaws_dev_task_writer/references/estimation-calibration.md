# Calibración de Estimación (feedback loop + banda de riesgo)

La estimación (cifra única, con factor IA) sale de `odoo-effort-estimator`.
Esta referencia añade dos cosas que hacen la estimación **mejor con el tiempo** y **más honesta**
en tareas que explotan.

---

## 1. Feedback loop — leer histórico antes de estimar

**Problema.** Sin memoria, cada tarea se estima en frío y se repiten los mismos errores.

`tl-assistant` mantiene un `estimation_history` (estimado vs real, tareas reworkadas, gotchas que
aparecieron en ejecución). **Si existe, consultarlo antes de fijar el número:**

- Ruta esperada: registro `estimation_history` de `tl-assistant` (memoria del usuario / su modelo
  de datos). Si no está accesible en esta sesión → **degradar con elegancia**: estimar como
  siempre y **dejar una nota** "sin histórico de calibración disponible" (no inventar ajuste).
- Qué mirar:
  - **Sesgo por tipo**: si tareas de integración/migración han ido sistemáticamente un X% por
    encima de lo estimado → aplicar ese sesgo, no volver a estimar optimista.
  - **Gotchas recurrentes**: si un gotcha apareció en ejecución en tareas parecidas y NO estaba
    precargado → añadirlo a la sección de gotchas de esta tarea (cierra el hueco).
  - **Reworks**: si un tipo de tarea se reworkó por criterio de aceptación ambiguo → endurecer
    los criterios aquí.

Regla: **el histórico ajusta el número y la precarga de gotchas; nunca lo maquilla a la baja.**

Tras entregar, si se conoce el real → devolverlo a `tl-assistant` para que el histórico aprenda.

---

## 2. Banda de riesgo (no solo un número)

Un número puntual miente en las tareas que más se desvían: **custom no trivial, integraciones y
migración de datos**. Para esas, emitir una **banda**, no un punto.

| Tipo de tarea | Salida de estimación |
|---------------|----------------------|
| Trivial / campo / vista / cosmético | número puntual (basta) |
| Estándar acotada | número puntual + factor riesgo del estimator |
| **Custom no trivial / integración / migración de datos** | **banda: mejor / probable / peor** |

- **Probable** = la estimación base × multiplicador de riesgo del `odoo-effort-estimator`.
- **Peor** = probable + la contingencia del punto que más incertidumbre tiene (normalmente la
  integración externa o el volumen de datos a migrar).
- **Mejor** = probable sin las contingencias que podrían no materializarse.
- La cifra que se comunica se ancla en *probable* (o *peor* si el cliente exige compromiso
  cerrado). Todas las cifras de la banda llevan el factor IA aplicado. La banda se explica en la
  ficha, no se esconde.

### Fase abierta (scope dependiente de medición)

Si parte del alcance **solo se dimensiona tras un paso previo** (p.ej. medir coverage antes de
saber cuántos tests faltan, o inventariar registros antes de saber el coste de migración):

- Cerrar y numerar las fases **fijas**.
- Dejar la fase dependiente **explícitamente abierta**: *"Fase C escala con lo que aparezca en
  Fase B — se re-estima al cerrar B"*.
- **NUNCA** inventar un número cerrado para una fase cuyo tamaño aún no se puede medir.

---

## Salida en la ficha

- Línea de estimación: si es banda → `mejor Xh · probable Yh · peor Zh`.
- Si hay fase abierta → decirlo en el alcance/estimación, no enterrarlo.
- Si se aplicó sesgo del histórico → una línea: "ajustado +X% por histórico de <tipo>".
- Si no había histórico → "sin calibración histórica disponible".
