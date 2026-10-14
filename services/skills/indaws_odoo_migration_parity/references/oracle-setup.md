# Levantar los entornos y construir el oráculo

La skill necesita **dos BDs sí o sí** (origen y migrada) y **una tercera muy
recomendable** (el oráculo). Para la Capa 1 basta con las BDs — NO hace falta
arrancar Odoo. Para la Capa 2 (comportamiento) sí se levanta runtime.

## 1. BD origen (versión antigua)

El dump de producción **pre-upgrade**, restaurado en local. Suele ser el
`original.dump` del kit o la BD `${CUSTOMER}_v${ORIGEN}`. Restaurar en un
Postgres local/Docker (ver `common-skills:docker-deployment`). **Neutralizar** si se va
a arrancar (Capa 2); para Capa 1 (solo SELECT) no es imprescindible pero se
recomienda por higiene.

## 2. BD migrada (versión destino)

La salida de la migración: `${CUSTOMER}_upgraded_base` tras el port de código y
el `-u all`. Es la que se valida.

## 3. Oráculo — vanilla de la versión destino (la tercera vía)

Una BD **limpia** de la versión destino con **exactamente el mismo set de
módulos estándar + OCA** que la migrada, pero **sin el código custom**. Es lo que
permite decir "esto lo quitó Odoo" vs "esto lo perdió la migración".

### Cómo construirlo

1. Obtener la lista de módulos **estándar+OCA** installed en la migrada
   (excluyendo custom/studio/vendor). El propio inventario ayuda:
   ```bash
   python3 scripts/extract_inventory.py --dsn "$MIGRATED_DSN" --label migrated --out migrated.json
   # los módulos con autor Odoo/OCA de migrated.json son el set del oráculo
   ```
2. Inicializar una BD nueva en la versión destino instalando ese set (core + EE
   + esos OCA), **sin** los addons custom en el addons-path:
   ```bash
   odoo-bin -d oracle_v<destino> -i base,<lista_estandar_y_oca> --stop-after-init \
     --without-demo=all
   ```
   (o vía `common-skills:docker-deployment` con una imagen limpia de la versión destino
   y el addons-path solo con core/EE/OCA.)
3. Extraer su inventario:
   ```bash
   python3 scripts/extract_inventory.py --dsn "$ORACLE_DSN" --label oracle --out oracle.json
   ```

### Requisito de paridad de módulos

El oráculo debe tener el **mismo** set estándar+OCA que la migrada. Si un módulo
OCA está en la migrada pero no en el oráculo, sus elementos se leerían como
"Odoo los quitó" (falso). Verifica que el nivel `modules` no tenga estándar/OCA
solo-en-migrado antes de fiarte del oráculo.

### Coste / reutilización

- El oráculo depende solo de (versión destino + set de módulos), **no del
  cliente**: una vanilla por combinación se **reutiliza** entre migraciones con
  el mismo set. Cachea el `oracle.json`.
- Sin oráculo, la skill funciona en **modo KB** (menos preciso): los estándar
  perdidos no conocidos quedan UNKNOWN para triage. Construir el oráculo es la
  inversión que más UNKNOWN elimina.

## Modo sin runtime (Capa 1)

Como la extracción es solo SQL, puedes correr las tres extracciones apuntando a
las tres BDs en el mismo Postgres (distintos `--db`/`--dsn`) sin arrancar ningún
Odoo. Es lo más rápido y es suficiente para el informe de decisiones de la
Capa 1.
