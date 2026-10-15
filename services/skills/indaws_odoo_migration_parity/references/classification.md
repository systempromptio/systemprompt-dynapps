# Clasificación 3-vías, atribución de dueño, triage y baseline

## Atribución de dueño (genérica, sin prefijo hardcodeado)

`diff_inventory.py` clasifica el dueño de cada elemento con esta prioridad
(`owner_of` / `classify_owner`):

1. **studio** — módulo `studio_customization`.
2. **custom** — módulo en el inventario explícito `--custom-modules`
   (del `migration.yaml` / `indaws-migrate modules list`).
3. **custom** — módulo que empieza por un prefijo de `--prefixes` (fallback,
   default `xx_`).
4. **oca** — autor contiene "Community Association" / "(OCA)" o website OCA.
5. **core** — autor Odoo (`odoo`, `Odoo S.A.`, `Odoo SA`, `Odoo Inc`).
6. **vendor** — cualquier otro autor (in-house o tercero sin metadata Odoo/OCA).
7. **standard** — elemento sin módulo cuyo `state` no es `manual` (p.ej. campos
   base): se trata como estándar.

**Regla clave**: solo **core / oca / standard** pueden estar en un oráculo
vanilla → solo ellos pueden atribuirse a un cambio de Odoo. **custom / studio /
vendor** son NO estándar: el oráculo nunca los contiene, así que su pérdida es
**regresión** (o **CUSTOM_DROPPED** si el módulo entero desapareció), jamás
INTER_VERSION.

> Cuanto mejor sea el inventario `--custom-modules`, más precisa la separación
> custom/vendor. Sin él, el fallback por autor es **conservador**: prefiere
> marcar como no estándar (regresión) antes que asumir "cambio de Odoo".

## Tabla de decisión (elemento en ORIGEN, ausente en MIGRADO)

| dueño | ¿oráculo? | resultado | etiqueta |
|---|---|---|---|
| custom/studio/vendor, módulo presente en migrado | — | port incompleto | **REGRESSION** |
| custom/vendor, módulo ausente en migrado | — | ¿drop o port? | **CUSTOM_DROPPED** |
| core/oca/standard | en oráculo | la migración lo dañó | **REGRESSION** |
| core/oca/standard | NO en oráculo | Odoo lo quitó | **INTER_VERSION** |
| core/oca/standard | sin oráculo, matchea KB | Odoo lo quitó | **INTER_VERSION** |
| core/oca/standard | sin oráculo, no matchea KB | a triage | **UNKNOWN** |

Casos sobre elementos presentes en ambos:
- vista `active` en origen e inactiva en migrado → **DEACTIVATED** (regresión).
- filtro con `blank_domain` en migrado y no en origen → **REQUIRED_BLANKED**.
- **modelo** con `rows > 0` en origen y tabla ausente / `rows ≤ 0` en migrado →
  **DATA_LOSS_RISK** (la tabla se vació).

Elemento en MIGRADO y no en ORIGEN → **NEW_IN_TARGET** (informativo).

## Pérdida de datos en modelos (categoría `models`)

La desaparición de un modelo puede llevarse **datos**. Sobre la clasificación
base (regresión / cambio de versión / dropped), la categoría `models` aplica una
dimensión adicional con el conteo `rows`:

| situación | etiqueta |
|---|---|
| modelo desaparece, `rows != 0` (tenía datos) | **DATA_LOSS_RISK** (bloqueante) |
| modelo desaparece, tiene sucesor en migrado (`model_renames`) | **RENAMED** — Odoo lo renombró/fusionó; **verificar que el dato migró** (comparar conteos con el sucesor) |
| modelo desaparece, `rows == 0` o abstracto | clasificación base (sin datos en juego) |
| modelo en ambos, origen `rows>0` y migrado vacío | **DATA_LOSS_RISK** |

`DATA_LOSS_RISK` es **bloqueante**: obliga a verificar dónde fue el dato antes de
entregar (¿migró a un sucesor?, ¿un script de migración lo movió?, ¿se descartó a
propósito?). El conteo `rows` es aproximado (`reltuples`): haz `ANALYZE` de origen
y migrado antes del gate para que sea fiable; verifica los conteos exactos de los
casos marcados. Los **renames/fusiones** conocidos de Odoo se declaran en
`known-version-changes.json → model_renames` (`{modelo_viejo: modelo_sucesor}`)
para no marcarlos como falsa pérdida.

## Triage de los UNKNOWN

Un UNKNOWN es un elemento **estándar perdido** que ni el oráculo ni la KB
resolvieron. Procedimiento recomendado (Paso 3 del workflow):

1. Preferir siempre **construir el oráculo** (`oracle-setup.md`): elimina casi
   todos los UNKNOWN de golpe y sin criterio humano.
2. Para los que queden, un `Agent(model:sonnet)` por lote consulta
   `odoo-version-api-reference` (diffs de la versión) y el enterprise local de la
   versión destino: ¿el elemento existe en el estándar de destino?
   - existe → **REGRESSION** (la migración lo perdió).
   - no existe → **INTER_VERSION**; **añádelo a la KB** para no repetir el triage.
3. Lo que el equipo confirme como aceptable se lleva al **baseline** (abajo).

## KB de cambios de versión (`known-version-changes.json`)

Clasificador **fallback** cuando no hay oráculo. Formato: por categoría, lista de
**regex** que matchean `id`s que Odoo quitó/renombró entre versiones. Genérico y
extensible; se enriquece con cada triage. Un match ⇒ INTER_VERSION.

```json
{ "fields": ["::uom_po_id$"], "views": ["\\.some_removed_view$"], "*": ["patrón-cualquier-categoria"] }
```

⚠ La KB puede ocultar una regresión si un patrón es demasiado amplio: mantenerla
**específica**. El oráculo es la vía precisa; la KB es el atajo para lo conocido.

## Baseline de aprendizaje (`baseline.json`)

Cierra el bucle: los diffs que el equipo revisa y **acepta** se registran y dejan
de aparecer como problema en la siguiente migración.

```json
{ "accepted": [ ["views", "some_module.some_view"], ["fields", "res.partner::mobile"] ] }
```

Un `(categoria, id)` en `accepted` se re-etiqueta **ACCEPTED** (info) y no cuenta
como bloqueante. Con el tiempo el informe converge a **solo regresiones reales**.
Mantén el baseline por proyecto (o uno común para cambios de Odoo universales).
