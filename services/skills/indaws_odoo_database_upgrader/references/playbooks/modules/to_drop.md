# Playbook: TO_DROP — eliminar módulo definitivamente

Acción para módulos que el cliente confirma no usar y de los que no hay
necesidad de preservar datos (vs `NEUTRALIZE` que sí preserva).

## ⚠ Timing — siempre preferible ANTES del Upgrade Service

Si tienes acceso a la BD origen v_X o un staging v_X:

1. Antes de generar el dump para el Upgrade Service, desinstala el módulo
   en v_X vía UI (`button_immediate_uninstall`) o XML-RPC.
2. Odoo v_X ejecuta el `_uninstall_hook` propio del módulo, en su propia
   versión: libera FKs, limpia `ir_model_data` y `ir_attachment` correctamente.
3. Re-dump. El módulo ya no existe en la BD que llega al Upgrade Service:
   - Cero scripts de migración intermedios v_X→v_(X+1)→...→v_destino sobre él.
   - Upgrade más rápido y con menos errores.
   - BD post-upgrade limpia (ni tablas residuales, ni FKs colgando).

Si la BD ya está post-upgrade (no se puede volver atrás), TO_DROP es la
única opción. Es más sucio porque hay que limpiar a mano lo que el
`_uninstall_hook` v_X habría hecho automáticamente.

## Aplicabilidad (criterios fail-fast)

El helper `DropModules` aborta si NO se cumple:

1. **Dependencias**: ningún módulo `installed`/`to upgrade` puede depender
   del que se va a dropear. Query `ir_module_module_dependency`.
2. **FKs externas**: ninguna tabla de OTRO módulo puede tener FK apuntando
   a una tabla del módulo target. Query `pg_constraint`. Bypaseable con
   `allow_cascade=True` (DROP TABLE … CASCADE).
3. **Volumen de datos**: si alguna tabla tiene >0 filas, requiere flag
   `allow_data_loss=True`.

## Flujo

### Fase A — reversible parcial
1. Uninstall vía XML-RPC (`button_immediate_uninstall`).
2. DELETE FROM ir_model_data WHERE module = target.
3. UPDATE ir_module_module SET state='uninstalled'.

Hasta aquí las tablas siguen vivas. Si te arrepientes, el `original.dump`
es la red.

### Fase B — destructiva
4. DROP TABLE de las tablas creadas por el módulo (FROM ir_model_data
   type='ir.model'), en orden inverso de FKs si `allow_cascade=False`.

**Después de fase B, los datos están perdidos.**

## Diferencia con otras acciones

| Acción | Stub fs | Archive UI | ir_model_data | Tablas | state | Datos |
|---|---|---|---|---|---|---|
| NEUTRALIZE | Sí | active=false | Conservado | Conservadas | installed | Conservados |
| REMOVE | No | — | DELETE | Conservadas | uninstalled | Sólo schema |
| TO_DROP | No | — | DELETE | DROP TABLE | uninstalled | **PERDIDOS** |

## Cuándo elegir TO_DROP vs NEUTRALIZE vs REMOVE

- **TO_DROP** si: cliente confirma que el módulo no se usaba, 0 registros funcionales,
  y quieres una BD post-migración limpia (sin ruido para migraciones futuras).
- **NEUTRALIZE** si: el módulo tenía datos legales/contables/históricos que hay
  que poder consultar aunque sin UI.
- **REMOVE** si: quieres que el módulo deje de aparecer pero conservar las
  tablas por si en el futuro se reinstala o se importan los datos a otra parte.

## Helpers a invocar

- `indaws_migration_kit.helpers.odoo.drop_module.DropModules`

## Params recomendados por escenario

```yaml
# Módulo nunca configurado, 0 registros, sin FKs externas
modules:
  account_mass_reconcile:
    action: TO_DROP
# Compiler genera:
- id: drop_modules
  params:
    confirm_destructive: false   # cambia a true cuando estés listo
    allow_data_loss: false        # 0 registros → no necesario
    allow_cascade: false          # sin FKs externas → no necesario
    phase: both
```

```yaml
# Módulo con tablas residuales pero datos sin valor
- id: drop_modules
  params:
    confirm_destructive: true
    allow_data_loss: true         # confirma pérdida explícita
    allow_cascade: false
    phase: both
```

```yaml
# Módulo con FKs colgando que el cliente confirma cortar
- id: drop_modules
  params:
    confirm_destructive: true
    allow_data_loss: true
    allow_cascade: true           # DROP TABLE … CASCADE
    phase: both
```

## Pitfalls

- **No mezclar TO_DROP con módulos que tienen `migrate.py` en versiones intermedias.**
  Si el módulo legacy tuvo un script de migración a v(X+1) y el Upgrade Service
  ya lo procesó, hay datos huérfanos que el `_uninstall_hook` original no preveía.
  TO_DROP los borrará igualmente, pero conviene auditar antes.
- **No usar TO_DROP en módulos `installed` con `module_dependency` desde apps activas.**
  El pre-check del helper lo detecta y aborta.
- **No olvidar el flag explícito** `confirm_destructive=True` para producción.
  Por defecto solo se ejecuta `plan()`.
