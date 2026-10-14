# Protocolo de Escaneo — Patrones Grep por Tipo de Dependencia

Patrones grep exactos para localizar referencias a `CAMPO`, `MODELO` o `METODO`.
Sustituir los placeholders en MAYÚSCULAS por el valor real.
`ROOT` = ruta raíz de búsqueda proporcionada por el usuario.

---

## Convenciones de uso

```bash
# Buscar en .py
grep -rn --include="*.py" 'PATRON' ROOT

# Buscar en .xml
grep -rn --include="*.xml" 'PATRON' ROOT

# Buscar en .csv
grep -rn --include="*.csv" 'PATRON' ROOT

# Buscar en migrations
grep -rn --include="*.py" 'PATRON' ROOT/*/migrations
```

---

## 1. Python — Compute / Related / Depends / Inverse / Search

### Referencia directa como nombre de string
```bash
grep -rn --include="*.py" "'CAMPO'" ROOT
grep -rn --include="*.py" '"CAMPO"' ROOT
```

### `compute=`
```bash
grep -rn --include="*.py" "compute='_compute.*CAMPO\|compute='.*CAMPO" ROOT
```

### `related=`
```bash
grep -rn --include="*.py" "related='.*CAMPO" ROOT
```

### `@api.depends` — el campo en la lista de dependencias
```bash
grep -rn --include="*.py" "api\.depends.*['\"].*CAMPO" ROOT
```

### `inverse=` / `search=`
```bash
grep -rn --include="*.py" "inverse='.*CAMPO\|search='.*CAMPO" ROOT
```

### Acceso directo al campo (`.CAMPO`)
```bash
grep -rn --include="*.py" "\.CAMPO[^_a-zA-Z]" ROOT
```

### `_inherit` — módulos que heredan el MODELO
```bash
grep -rn --include="*.py" "_inherit\s*=\s*['\"]MODELO['\"]" ROOT
grep -rn --include="*.py" "_inherit\s*=\s*\[.*['\"]MODELO['\"]" ROOT
```

### `_inherits`
```bash
grep -rn --include="*.py" "_inherits\s*=.*['\"]MODELO['\"]" ROOT
```

### SQL directo (columna en ALTER TABLE / SELECT)
```bash
grep -rn --include="*.py" "CAMPO" ROOT/*/migrations
```

---

## 2. XML — Vistas

### Referencia directa al campo
```bash
grep -rn --include="*.xml" "name=['\"]CAMPO['\"]" ROOT
```

### Campo en `attrs` (v14-v16)
```bash
grep -rn --include="*.xml" "attrs=.*CAMPO" ROOT
```

### Atributos directos (v17+)
```bash
grep -rn --include="*.xml" "invisible=.*CAMPO\|required=.*CAMPO\|readonly=.*CAMPO" ROOT
```

### Dominio en vista
```bash
grep -rn --include="*.xml" "domain=.*CAMPO" ROOT
```

### `xpath` apuntando al campo
```bash
grep -rn --include="*.xml" "xpath.*CAMPO\|@name=['\"]CAMPO['\"]" ROOT
```

### Herencia de vista — `inherit_id` + búsqueda del campo en mismo fichero
```bash
grep -rn --include="*.xml" "CAMPO" ROOT  # captura todo lo anterior y más
```

---

## 3. XML — Informes QWeb

```bash
grep -rn --include="*.xml" "t-field=.*CAMPO\|t-esc=.*CAMPO\|t-if=.*CAMPO\|t-att.*CAMPO" ROOT/*/report
grep -rn --include="*.xml" "t-field=.*CAMPO\|t-esc=.*CAMPO\|t-if=.*CAMPO\|t-att.*CAMPO" ROOT/*/views
```

---

## 4. XML — Server Actions / Automatizaciones

### Código Python inline en `ir.actions.server`
```bash
grep -rn --include="*.xml" "CAMPO" ROOT/*/data
grep -rn --include="*.xml" "model=\"ir.actions.server\"" ROOT  # localiza ficheros relevantes
grep -rn --include="*.xml" "model=\"base.automation\"" ROOT
```

---

## 5. XML — Record Rules con dominio

```bash
grep -rn --include="*.xml" "ir\.rule.*CAMPO\|domain_force.*CAMPO" ROOT
grep -rn --include="*.xml" "CAMPO" ROOT/*/security
```

---

## 6. CSV — `ir.model.access.csv` (solo para MODELO eliminado)

```bash
grep -rn --include="*.csv" "MODELO\|model_MODELO" ROOT
```

Formato esperado de nombre de modelo en CSV: `model_sale_order` (puntos → guión bajo).

---

## 7. Manifest — Dependencias entre módulos

### Módulos que dependen del módulo que contiene el CAMPO/MODELO
```bash
# Primero identificar el módulo propietario:
grep -rn --include="*.py" "CAMPO\|MODELO" ROOT --include="__manifest__.py"

# Luego buscar quién depende de ese módulo:
grep -rn --include="*.py" "['\"]MODULO_PROPIETARIO['\"]" ROOT/*/\_\_manifest\_\_.py
```

---

## 8. Scans combinados — orden recomendado

Ejecutar en este orden para máxima cobertura sin duplicados:

```bash
# 1. Python completo
grep -rn --include="*.py" "'CAMPO'\|\"CAMPO\"\|\.CAMPO[^_a-zA-Z]" ROOT 2>/dev/null

# 2. XML completo
grep -rn --include="*.xml" "CAMPO" ROOT 2>/dev/null

# 3. CSV completo
grep -rn --include="*.csv" "CAMPO" ROOT 2>/dev/null

# 4. Herencia de modelo
grep -rn --include="*.py" "_inherit.*MODELO\|_inherits.*MODELO" ROOT 2>/dev/null

# 5. Migraciones
find ROOT -path "*/migrations/*.py" | xargs grep -l "CAMPO" 2>/dev/null
```

---

## Tips de filtrado

- Excluir carpetas de Odoo base del escaneo si solo interesa código custom:
  ```bash
  grep -rn --include="*.py" 'CAMPO' ROOT --exclude-dir=".git"
  ```
- Excluir falsos positivos por nombre de módulo que contenga el campo:
  ```bash
  grep -rn --include="*.py" 'CAMPO' ROOT | grep -v "__pycache__"
  ```
- Limitar a un módulo específico:
  ```bash
  grep -rn --include="*.py" 'CAMPO' ROOT/nombre_modulo
  ```
