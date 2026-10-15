# Contrato con /qualify: la línea «MVI:»

La sección 7 del formulario de `qualification-form` («Encaje MVI») deja en el resumen que el SDR
pega en el chatter del `crm.lead` una línea con un formato fijo. Este skill la lee con
`scripts/mvi_quote.py from-qualify`. **El formato es un contrato**: si cambia en el formulario,
cambia aquí y en `parse_qualify()`, y `tests/mvi_smoke.py` lo tiene que cubrir.

```
MVI: ELEGIBLE · lineas=o2c,p2p,r2r · condiciones=estandar,iteraciones,usuarios,estructura,idioma,sin_desarrollos,sin_fabricacion,solo_maestros · 4.500 € + 149 €/mes
MVI: NO ELEGIBLE · lineas=o2c,p2p,r2r,h2r · falla=usuarios>10,modulo_excluido,lineas>3
MVI: SIN EVALUAR
```

- `lineas` son ids de `config/mvi_offer.json` (`l2o`, `web`, `o2c`, `p2p`, `r2r`, `h2r`, `serv`);
  `-` si no hay ninguna.
- `condiciones` solo aparece si es elegible, y lista todas las confirmadas.
- `falla` lista los ids que no se cumplen, más `usuarios>N` si el número de usuarios de la
  sección 2 supera el límite, `modulo_excluido` si en la sección 5 está marcado «Fabricación
  (MRP)» o «eCommerce» (`limites.modulos_excluidos_qualify`), `lineas>N` si se marcan más líneas
  de las permitidas (`limites.max_lineas`), y `sin_lineas`.
- El importe es informativo: el skill recalcula siempre desde la configuración.

## Reglas de lectura

1. **Manda la última nota.** Si el SDR recualifica, la nota nueva va al final del chatter.
2. **Sin línea «MVI:», no hay oferta.** El lead no ha pasado por la sección 7: se le pide al SDR
   que la rellene. No se deducen las condiciones de otras secciones.
3. **`NO ELEGIBLE` no se discute aquí.** El skill no ofrece «forzarla»: el lead va a comercial
   (value-based). Si el SDR cree que el formulario se equivocó, lo corrige en el formulario y
   vuelve a pegar el resumen.
4. **El formulario no es la única comprobación.** Con la nota elegible, `check` vuelve a pasar
   las condiciones y el número de usuarios contra la configuración antes de tocar Odoo.

## Toma de datos: lead que ya está con comercial

Sin línea «MVI:» pero con toma de datos (`crm.lead.xx_data_collection`, HTML), cada condición va a
`true` **solo** si la toma lo dice de forma explícita; si no, se deja sin responder y la oferta
solo sale como borrador. Enséñale al SDR la tabla rellena antes de generar nada.

| Condición | Dónde mirarlo en la toma | Nota |
|---|---|---|
| `estandar` | «Acordado en la sesión», «Personalizaciones y desarrollos» | «vanilla», «estándar» |
| `iteraciones` | nunca aparece | siempre pendiente: se confirma con el cliente |
| `usuarios` | «Nº de usuarios previsto», «Empleados» | con el número; si es `[PREGUNTAR]`, pendiente |
| `estructura` | «Multicompañía», «Almacenes», «Bancos y monedas» | cualquier `[PREGUNTAR]` ahí, pendiente |
| `idioma` | «Idioma del proyecto» | español o inglés |
| `sin_desarrollos` | «Personalizaciones y desarrollos», «Integraciones» | la sincronización bancaria estándar no cuenta |
| `sin_fabricacion` | «Parte B: fuera de la propuesta» | fabricación en una fase posterior no bloquea la fase 1 |
| `solo_maestros` | «Migración de datos» | greenfield o solo maestros |

Caso real: Airvolut (lead 33080, 29-sep-2026): seis en `true`, `iteraciones` y `estructura`
pendientes.

