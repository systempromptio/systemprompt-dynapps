# Auto-feedback al workdir (skill → migration.yaml)

A partir del kit v2.2.0 la skill puede **proponer y aplicar fixes al workdir
cliente automáticamente** tras una iteración fallida — sin que tengas que
editar el yaml a mano.

## Flujo automático

```
iterate (-u all falla)
   │
   ▼
parse.json — el kit clasifica los errores en patrones conocidos
   │
   ▼
indaws-migrate suggest --workdir .
   │
   ▼
Muestra propuestas:
  [1] FIX_FLAG | conf=0.7 | external_id_not_found
       target: fixes.post_restore_validator
       razón: xml_id huérfano. delete_unowned_views (P5) suele resolverlo.
   │
   ▼
indaws-migrate suggest --workdir . --apply [--commit]
   │
   ▼
- Edita migration.yaml preservando comentarios (ruamel.yaml round-trip)
- Appendea a playbook.md (bitácora + TODOs si procede)
- Opcional: auto-commit en git con mensaje descriptivo
```

## Tipos de sugerencias

| Action type | Significa | Auto-aplicable |
|---|---|---|
| `FIX_FLAG` | Activar/actualizar flag en `fixes:` | Sí — alta confianza |
| `SQL_TEMPLATE` | Crear archivo `sql/NN_*.sql` template + referencia en `steps_custom:` | Parcial — la entrada se añade, el SQL queda como TODO |
| `MODULE_CHANGE` | Cambiar action/notes de un módulo | Sí, si el dev confirma |
| `MANUAL_REVIEW` | Sin auto-fix viable, requiere análisis humano | No — solo informa |

## Confidence

Cada sugerencia trae una `confidence` (0.0-1.0). El threshold por defecto
para auto-aplicar es `0.5`. Patrones con confianza menor se reportan pero
no se aplican (requieren validación humana).

| Threshold | Comportamiento típico |
|---|---|
| `0.0` | Muestra absolutamente todo |
| `0.5` (default) | Aplica solo lo razonablemente seguro |
| `0.8` | Solo lo casi-certero (alta confianza) |

## Cómo se usa en sesión

```bash
# 1. Lanzar iterate normal
indaws-migrate iterate --dump $DUMP_PATH/original.dump

# 2. Si falla, ver propuestas (no escribe nada)
indaws-migrate suggest --workdir .

# 3. Si te convencen, aplica + commit automático
indaws-migrate suggest --workdir . --apply --commit

# 4. Repetir iterate
indaws-migrate iterate --dump $DUMP_PATH/original.dump
```

## Qué pasa internamente

1. `suggest` lee `iterations/<latest>/parse.json` (lo escribió `iterate`)
2. Por cada error, busca su factory en el catálogo de sugerencias
3. Deduplica (no propone 10 veces el mismo flag por 10 ocurrencias del mismo patrón)
4. Filtra por confidence threshold
5. Con `--apply`: usa `yaml_auto_edit` (ruamel.yaml) para modificar el yaml
   preservando 100% comentarios + formato + orden de claves
6. Notifica al `PlaybookHook` que appendea entradas a `playbook.md`:
   - "💡 Sugerencias automáticas: N propuestas (aplicadas: X, descartadas: Y)"
   - "📝 Edit automático yaml: <lista de cambios>"
7. Con `--commit`: hace `git add` + `git commit` con mensaje
   `chore: auto-suggest aplica fixes (pid1,pid2) iter=NNN`

## Cuándo NO usarlo

- **Decisión funcional de un módulo** (KEEP / REPLACE / STUDIO): nunca auto-aplicar.
  La skill puede sugerir el cambio, pero requiere `AskUserQuestion` antes de aplicar.
- **Cambios destructivos**: el catálogo SOLO propone fixes opt-in y conservadores.
  No verás sugerencias de `DROP TABLE` automáticas.
- **Cuando el patrón es desconocido**: si `parse.json` tiene errores sin pattern_id
  reconocido, `suggest` no propone nada — quedan como TODO manual.

## Extender el catálogo

Si descubres un patrón nuevo durante una migración que se repetirá:

1. Añade el `ErrorPattern` en `indaws_migration_kit/migration/iterate.py`
2. Añade la factory en `indaws_migration_kit/catalogs/suggestions.py`
3. Tests en `tests/catalogs/test_suggestions.py` (mínimo: factory devuelve
   Suggestion correcta + caso real del log)
4. PR al kit

A partir de la siguiente sesión, todos los clientes se benefician.

## Auto-log de hitos en playbook

`PlaybookHook` también escribe automáticamente cuando ocurre:

- 🟢 Apertura de sesión
- ✓/✗ Step completado en runner
- ✓/✗ Iteración terminada (éxito vs nº errores)
- 💡 Sugerencias propuestas/aplicadas
- ✓ Gate ACK
- 🚀 Go-live (con duración)
- ⚠ Smoke fallido (modelo + error) → además crea TODO
- 📝 Edit automático yaml
- 🎯 Fase completada

Resultado: el `playbook.md` crece solo con un timeline auditable de la
migración. El dev solo añade decisiones humanas.
