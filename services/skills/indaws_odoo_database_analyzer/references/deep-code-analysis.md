# Deep Code Analysis Patterns

When enabled, search for these patterns to adjust estimates.

**Automation:** Use **odoo-module-scanner** skill to automate these searches and classify findings by type. For deprecated API patterns by version, see **odoo-version-api-reference** skill.

**Este fichero dice COMO detectar, no CUANTO ajustar.** Los porcentajes por hallazgo, la
contingencia por tamaño y los factores IA viven en la skill `odoo-effort-estimator`.

## Legacy/Deprecated APIs
```bash
# Search patterns (or use odoo-module-scanner)
# REMOVED in modern Odoo (v14+) -> cause real failures, must be rewritten:
grep -r "@api.one" --include="*.py"        # removed
grep -r "fields.function" --include="*.py"  # removed (use fields.* compute=)

# LEGACY in v18 (api.py:359) but HARD BREAK if the target is v19:
grep -r "@api.returns" --include="*.py"
grep -r "@api.model_create_single" --include="*.py"   # same story, and MORE likely in old custom code
```
**Note:** `@api.returns` **depends on the target version, and that flips the verdict**:

- **target v18** → legacy-style smell, not a runtime failure. Still defined
  (`odoo/api.py:359`, `returns` — firma `returns(model, downgrade=None, upgrade=None)`) and used by 21 places in
  core addons.
- **target v19** → **removed, it breaks**. There is no `def returns` anywhere in `odoo/`, `returns` is
  not among the names re-exported by `odoo/api/__init__.py`, and core addons use it **0 times**
  (21 in v18). Counting it as "just a smell" when quoting a v19 migration **underestimates the
  work**: every occurrence has to be removed.
  (Precision, verified 2026-08-07: the ORM did move to `odoo/orm/`, but do **not** justify this with
  "`odoo/api.py` no longer exists" — `odoo/api/` exists in v19 as a re-export **package**. The
  evidence that kills `@api.returns` is the missing `def returns` and its absence from that package's
  export list, not the missing file.)

**`@api.returns` no viaja solo: v19 se llevó por delante seis nombres de `odoo.api`.** Medido
2026-08-17 comparando los `def` públicos de `odoo18/odoo/api.py` con la lista de re-exports de
`odoo/api/__init__.py:4-21`, `NewId` el primero de la lista (v19): todo el fichero son re-exports desde `odoo.orm.*`:

| Nombre de `odoo.api` en v18 | En v19 | Qué significa al analizar un módulo |
|---|---|---|
| `returns` | **fuera** | ver arriba |
| `model_create_single` | **fuera** (`odoo/api.py:477`, `def model_create_single` en v18, `:413` en v17; **0 ocurrencias en todo CE19**) | **el que más va a aparecer en custom viejo** |
| `downgrade`, `propagate` | **fuera** | internos, raros en custom |
| `call_kw` | movido a `odoo/service/model.py:74`, `call_kw` | sigue existiendo, cambia el import |
| `attrsetter` | movido a `odoo/orm/decorators.py:73`, `attrsetter` | idem |
| — | **`deprecated` es NUEVO** (`odoo/orm/decorators.py:18`, `deprecated` — shim de `warnings.deprecated`) | no existía en v18 |

> **Ojo con `@api.model_create_single` al cotizar un v19.** Es el hermano de `@api.returns` y tiene
> **más probabilidad de estar** en código de cliente: es a lo que migró en su día el `create` de un
> solo dict, así que vive en módulos viejos que nunca pasaron a `model_create_multi`. Core no lo usa
> (0 ocurrencias en v17, v18 y v19), o sea que **grepear el core no te avisa**: hay que grepear el
> custom. Y como `odoo.api` es en v19 un paquete de re-exports, el fallo es
> `AttributeError: module 'odoo.api' has no attribute 'model_create_single'` **al importar**, no en
> runtime — revienta la carga del módulo entero.

`@api.one` and `fields.function` ARE removed in both.

## Security Risks
```bash
grep -r "\.execute.*%" --include="*.py"  # SQL injection
grep -r "\.sudo\(\)" --include="*.py"    # Excessive sudo
```

## Performance Issues
```bash
grep -r "for.*in.*search\(" --include="*.py"  # N+1 queries
grep -r "search\(\[\]\)" --include="*.py"     # Unlimited search
```
