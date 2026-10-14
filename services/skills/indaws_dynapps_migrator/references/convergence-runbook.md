# Convergencia: del dump migrado al `-u all` limpio

Los fallos de esta fase **no los ve ningún análisis de código**: viven en la BD (datos del
cliente, vistas Studio, dominios guardados) o en el orden de carga en runtime. En migraciones
reales, la mayoría de los hallazgos que rompen esta fase **solo aparecen ejecutando el `-u all`
contra la BD real**, no antes.

Este es el orden que funciona. Cada paso está aquí porque saltárselo cuesta un ciclo.

```bash
#  0. copia desde el restore PRISTINO (no reutilizar una BD a medio upgrade)
dropdb --if-exists $DEV; createdb $DEV -T ${CUSTOMER}_upgraded_base

#  1. GATE de neutralizacion — DURO, aborta antes de arrancar Odoo
#     (ver dynapps-db-upgrade-runner/references/neutralization.md)

#  2. guardas de overrides que evaluan dominios almacenados  → §A
#  3. calcular dependencias nuevas a instalar                → §B
#  4. odoo-bin -d $DEV -i <nuevas> -u all --stop-after-init
#  5. restaurar las guardas del paso 2
#  6. re-neutralizar los conectores creados EN el paso 4     → §C
```

Los placeholders (`<prefijo>_`, `<flag_override>`, nombres de módulo) son genéricos; los
bloques marcados "ejemplo" ilustran el patrón con un caso concreto.

---

## §A. Overrides que evalúan dominios con campos que aún no existen

Un módulo OCA/vertical con override de `create`/`write` puede evaluar dominios guardados en
**cualquier** escritura. Si esos dominios referencian campos custom y un módulo core escribe
antes de que cargue el custom, el `-u all` muere a mitad:

```
# ejemplo
ValueError: Invalid field res.partner.<prefijo>_<campo_custom> in condition
            ('<prefijo>_<campo_custom>.ref', 'not ilike', '...')
```

Patrón típico: un módulo (p. ej. un `*_assortment` OCA) sobrecarga `res.partner.write()` y
evalúa un `partner_domain` guardado en `ir.filters`; un módulo core escribe partners **antes**
de que cargue el módulo custom dueño del campo.

**No siempre se arregla por `depends`**: si el `<prefijo>_base` depende de ese módulo core, el
custom carga necesariamente después. Se baja el flag que dispara el override durante el upgrade
y se restaura al terminar — solo un booleano, los dominios no se tocan:

```sql
-- antes (ejemplo con un flag booleano <flag_override> en ir_filters)
CREATE TABLE IF NOT EXISTS _guard_backup (filter_id integer PRIMARY KEY);
INSERT INTO _guard_backup SELECT id FROM ir_filters WHERE <flag_override> ON CONFLICT DO NOTHING;
UPDATE ir_filters SET <flag_override> = false WHERE <flag_override>;
-- despues
UPDATE ir_filters f SET <flag_override> = true FROM _guard_backup b WHERE b.filter_id = f.id;
DROP TABLE _guard_backup;
```

Cómo detectar candidatos antes de arrancar:

```bash
grep -rln "def write" <addons>/*/models/res_partner.py \
  | xargs grep -l "filtered_domain\|_get_eval\|search(\[" 
psql -d $DEV -tAc "select count(*) from ir_filters
  where domain like '%<prefijo>\_%' or partner_domain like '%<prefijo>\_%';"
```

---

## §B. `-u all` no instala módulos nuevos, y `-i <ya instalado>` es un no-op

Si en la versión destino el `<prefijo>_base` gana dependencias que **nunca estuvieron
instaladas**:

```
module <prefijo>_base: some depends are not loaded (<dep_a>, <dep_b>, <dep_c>), skipped
module <prefijo>_partner: some depends are not loaded (<prefijo>_base), skipped   ← y asi los demas
```

Y acto seguido `Transient module states were reset` los deja en **`installed` sin haberlos
actualizado**: el estado final miente y el resumen parece correcto.

`-i <prefijo>_all` **no** lo arregla: si `<prefijo>_all` ya está `installed`/`to upgrade`, Odoo
no dispara instalación alguna. Hay que pasar explícitamente las dependencias no activas:

```python
# dependencias declaradas por los custom que NO estan activas en la BD
deps = union(manifest["depends"] for manifest in parts/custom/*/__manifest__.py)
activos = SELECT name FROM ir_module_module WHERE state IN ('installed','to upgrade')
nuevas = sorted((deps - modulos_custom) - activos)
# -> odoo-bin -i <nuevas> -u all
```

Cuidado: en la BD recién restaurada esos módulos **no existen ni como fila** en
`ir_module_module` (Odoo aún no conoce los módulos nuevos), así que un
`WHERE name IN (...) AND state='uninstalled'` devuelve vacío. Hay que calcularlo por
diferencia contra los **activos**, no buscando los `uninstalled`.

Verificación de que de verdad se procesaron:

```bash
grep -c "some depends are not loaded" $LOG        # 0
grep "Transient module states were reset" $LOG    # sin salida
```

---

## §C. Conectores creados por el propio `-u all`

Los modelos de un stack nuevo (p. ej. un broker EDI con su credencial/clave) **no existen al
restaurar**: los crea el `-u all`. Si la neutralización solo se pasa antes, queda un hueco en
el conector potencialmente más peligroso (puede emitir de verdad). Por eso el SQL de extras se
pasa **dos veces**: paso 1 y paso 6. Ver la lista de conectores en
`dynapps-db-upgrade-runner/references/neutralization.md`.

---

## §D. Los datos custom SOBREVIVEN al Upgrade Service (y eso es bueno)

Patrón general: de los módulos activos en la v-origen, el Upgrade Service deja **una fracción
significativa en `to upgrade`** con su versión de origen (p. ej. `<prefijo>_base` → `to upgrade`
con su versión v-origen) y solo desinstala de verdad los que sí tenían código en los servidores
de Odoo. Los módulos sin código en Odoo **no se desinstalan**: se marcan y se conserva su
esquema (tablas, columnas `<prefijo>_*`, relaciones y datos).

Dos consecuencias:
1. El `-u all` reinstala los custom **encima de sus datos**; no hay que re-importar nada.
2. `state='to upgrade'` es lo que hace que **corran los scripts de migración** — siempre que
   la sub-versión del manifest supere la de la BD (ver
   `dynapps-code-migration/references/buildout-structure.md`).

La lista de `modules_to_remove` se calcula de aquí, no a mano:

```sql
-- modulos ACTIVOS (installed + to upgrade) sin codigo en el buildout v-destino
select name from ir_module_module where state in ('installed','to upgrade') order by name;
```
...y se cruza contra los módulos alcanzables por el `addons_path`. **Excluir siempre
`studio_customization`**: no tiene código en disco por diseño (lo genera Studio en la BD) y
stubarlo borra las personalizaciones del cliente.

---

## §E. Vistas Studio y renombrados de xmlid

Ver `odoo-module-upgrader/patterns/odoo18_to_19.md` → §S3. Resumen: si un módulo renombra el
xmlid de una vista (`*_view_tree` → `*_view_list`), la limpieza final
(`ir.model.data._process_end`) borra la vieja y **una vista Studio del cliente que heredaba de
ella puede tumbar el registry entero**, con todos los módulos ya cargados. El arreglo va en un
script de migración del `<prefijo>_all` (dentro del `-u all`, antes de `_process_end`), no en un
paso SQL previo: la vista nueva la crea el propio upgrade.

---

## §F. Traspaso de datos entre campos renombrados

Cuando un módulo se retira y su sustituto usa **otro nombre de campo**, los datos se quedan en
la columna vieja y nadie se queja. Ejemplo de un identificador de partner (un GLN) cuyo módulo
de origen se retira:

| Origen v-origen (retirado) | Destino v-destino | Quién lo usa |
|---|---|---|
| `res.partner.<campo_retirado>` | `res.partner.<campo_nuevo>` | stack nuevo del vendor |
| `res.partner.<campo_retirado>` | `res.partner.<campo_core>` | **core** (de donde un módulo estándar saca el dato para el UBL/EDI) |
| `stock.location.<campo_retirado>` | `stock.location.<campo_nuevo>` | módulo de stock del stack |
| `<modelo>.<campo_retirado>` | `res.company.<campo_nuevo>` | módulo base del stack |

Ojo al segundo caso: el destino puede ser un campo **estándar** que en la v-origen no existía, y
entonces la funcionalidad que el cliente tenía por custom pasa a ser nativa — pero **solo si el
dato llega**.

Cómo buscarlos de forma sistemática antes del `-u all`:

```bash
# columnas del modulo retirado que tienen datos
psql -d $DEV -tAc "select table_name||'.'||column_name from information_schema.columns
  where table_schema='public' and column_name like '%<prefijo_retirado>%';"
# y el campo equivalente en el modulo sustituto
grep -rn "= fields\." <sustituto>/models/*.py | grep -iE "gln|code|ref|identifier"
```

El traspaso va en un `post-migrate` del módulo que dependa del sustituto (así el campo destino
ya existe), idempotente y solo donde el destino esté vacío.
