# Workflow Checklists

## PRE-Writing Checklist

Before writing any server action code:

- [ ] NO uso `self` para acceder al registro (uso `record`/`records`)
- [ ] NO uso asignacion directa `record.field = value` (uso `record.write({...})`)
- [ ] NO uso `import X` (statement bloqueado siempre)
- [ ] SI necesito importar modulos: verifico que tengo indaws_base y uso `wrap_module()`/`import_module()`
- [ ] NO uso `return` (asigno a variable `action` si necesito retornar)
- [ ] NO uso `_()` para traducciones (texto directo)
- [ ] NO uso `ValidationError`/`AccessError` (solo `UserError`)
- [ ] SI es cron: NO dependo de `record`/`records` (estan vacios)
- [ ] SI necesito regex SIN indaws_base: uso metodos string como alternativa
- [ ] SI necesito regex CON indaws_base: `re = wrap_module('re', ['findall', 'sub', ...])`
- [ ] SI hago SQL: uso parametros `%s` (no string format)
- [ ] SI uso `open()`: solo rutas seguras (/tmp/) y cierro con `with`
- [ ] Verifico que cada variable que uso existe en `eval-context.md`

## POST-Writing Validation

### Standard Checks (always)

| # | Check | Severity |
|---|-------|----------|
| G01 | No `record.field = value` (always `write()`) | CRITICAL |
| G02 | No `import X` statement (use `wrap_module`/`import_module` with indaws_base) | CRITICAL |
| G03 | No `_()` | MEDIUM |
| G04 | No `ValidationError`/`AccessError` | MEDIUM |
| G05 | If cron, no dependency on `record` | HIGH |
| G06 | Multi-record: iterate with `for rec in records` | MEDIUM |
| G07 | Return via `action = {}`, not `return` | MEDIUM |
| G08 | No `__dunder__` attribute access | CRITICAL |
| G09 | No N+1 in loops (use batch/filtered) | MEDIUM |
| G10 | No `cr.commit()` / `cr.rollback()` | CRITICAL |
| G11 | `sudo()` for restricted models | MEDIUM |
| G17 | No `self` for record data | CRITICAL |
| G19 | SQL with parameters, not string format | CRITICAL |
| G20 | No direct `env.context` modification | MEDIUM |

### indaws_base Additional Checks

| # | Check | Severity |
|---|-------|----------|
| G21 | `wrap_module` vs `import_module` correct per need | MEDIUM |
| G22 | `open()` only on safe paths (/tmp/) | CRITICAL |
| G23 | Files opened with `with` (context manager) | MEDIUM |
| G24 | `wrap_module` lists methods that actually exist | LOW |
| G25 | `import_module` with correct fromlist for submodules | LOW |
