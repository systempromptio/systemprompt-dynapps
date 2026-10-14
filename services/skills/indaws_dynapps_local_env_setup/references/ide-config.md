# IDE Config — SmartGit y PyCharm

Configuración completa de SmartGit y PyCharm para un proyecto de cliente Dynapps.

Variables usadas:
- `$CUSTOMER` — slug del cliente (ej: `acme`)
- `$VERSION` — versión Odoo (ej: `17`, `18`)
- `$BUILDOUT_PATH` — `$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}`

---

## SmartGit

### Añadir el repositorio

1. Abrir SmartGit
2. Menú: `Repository → Add or Create` (atajo: `Ctrl+O`)
3. Navegar a: `$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}`
4. Confirmar

SmartGit detectará el repo git (o el submodulo principal). Si hay submódulos, aceptar añadirlos también.

### Configurar Git-Flow

1. Con el repo seleccionado, ir a: `Git-Flow → Configure`
2. Seleccionar: **Full** (habilita todos los tipos de rama)
3. Configurar los prefijos de rama (valores por defecto de Dynapps):

| Tipo | Prefijo |
|------|---------|
| Feature branches | `feature/` |
| Release branches | `release/` |
| Hotfix branches | `hotfix/` |
| Support branches | `support/` |
| Version tag prefix | (vacío o `v`) |

4. Confirmar con OK

### Verificar

En la barra lateral de SmartGit deben aparecer las secciones:
- `BRANCHES` con `master`, `develop`
- `GIT-FLOW` con las opciones de iniciar feature/release/hotfix

---

## PyCharm — Interpreter

### Configurar el intérprete Python del proyecto

1. Abrir PyCharm
2. Ir a: `File → Settings → Project: [nombre] → Python Interpreter`
3. Hacer clic en el engranaje (⚙) → `Add Interpreter → Add Local Interpreter`
4. Seleccionar: `Existing environment`
5. Introducir el path según el tipo de virtualenv:

**pyenv:**
```
~/.pyenv/versions/$CUSTOMER/bin/python
```
Path absoluto expandido:
```
/home/$USER/.pyenv/versions/$CUSTOMER/bin/python
```

**virtualenv:**
```
$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}/venv/bin/python
```

6. Confirmar con OK

### Verificar

En la barra inferior de PyCharm debe aparecer el nombre del intérprete seleccionado.
Abrir `Tools → Python Console` y verificar la versión:
```python
import sys; print(sys.version)
```

---

## PyCharm — Run/Debug Configuration

### Crear la configuración principal de ejecución

1. Ir a: `Run → Edit Configurations` (o `Shift+Alt+F10 → Edit Configurations`)
2. Hacer clic en `+` → `Python`
3. Rellenar los campos:

| Campo | Valor |
|-------|-------|
| **Name** | `Odoo` |
| **Script path** | `$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}/parts/odoo/odoo/odoo-bin` |
| **Parameters** | `-c odoo.cfg -d ${CUSTOMER}${VERSION} --dev=xml -u x` |
| **Working directory** | `$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}` |
| **Python interpreter** | El configurado en el paso anterior |

Ejemplo con cliente `acme` v18:

| Campo | Valor |
|-------|-------|
| Script path | `$ODOO_SRC/buildouts/acme18/parts/odoo/odoo/odoo-bin` |
| Parameters | `-c odoo.cfg -d acme18 --dev=xml -u x` |
| Working directory | `$ODOO_SRC/buildouts/acme18` |

4. Confirmar con OK

**Nota sobre `-u x`:** Sustituir `x` por el nombre del módulo a actualizar al lanzar.
Normalmente se deja `x` como placeholder y se modifica según la sesión de trabajo.

---

## PyCharm — Test Configuration

### Crear la configuración de test

1. En `Run → Edit Configurations`, hacer clic en `+` → `Python`
2. Rellenar los campos:

| Campo | Valor |
|-------|-------|
| **Name** | `Odoo Test` |
| **Script path** | `$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}/parts/odoo/odoo/odoo-bin` |
| **Parameters** | `-c odoo.cfg -d ${CUSTOMER}${VERSION} --dev=xml -i x --test-tags /x` |
| **Working directory** | `$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}` |
| **Python interpreter** | El configurado anteriormente |

Ejemplo con cliente `acme` v18:

| Campo | Valor |
|-------|-------|
| Script path | `$ODOO_SRC/buildouts/acme18/parts/odoo/odoo/odoo-bin` |
| Parameters | `-c odoo.cfg -d acme18 --dev=xml -i x --test-tags /x` |
| Working directory | `$ODOO_SRC/buildouts/acme18` |

**Nota sobre `-i x` y `--test-tags /x`:** Sustituir `x` por el módulo o clase de test a ejecutar.
El formato de `--test-tags` es `/ModuleClass` o `/ModuleClass.method_name`.

3. Confirmar con OK

---

## PyCharm — Mark Directories

Marcar directorios correctamente evita que PyCharm indexe código irrelevante y mejora el autocompletado.

### Pasos

1. En el panel `Project` (izquierda), hacer clic derecho sobre cada directorio
2. Seleccionar: `Mark Directory as`

### Directorios a configurar

| Directorio (relativo a `$BUILDOUT_PATH`) | Tipo |
|-----------------------------------------|------|
| `data` | **Excluded** (no indexar datos) |
| `parts/odoo/odoo` | **Sources Root** (raíz del código fuente de Odoo) |

**Excluded:** El directorio `data/` contiene attachments, filestore y logs — no tiene Python.
Excluirlo acelera la indexación.

**Sources Root:** Marcar `parts/odoo/odoo` como Sources Root permite que PyCharm resuelva
correctamente los imports de Odoo (`from odoo import models, fields, api`).

### Verificar

Después de marcar, los directorios aparecen con iconos distintos en el panel Project:
- `data` → icono de carpeta naranja/excluida
- `parts/odoo/odoo` → icono de carpeta azul (sources)

Los imports de Odoo deben resolverse sin subrayado rojo en los archivos `.py` del proyecto.

---

## Troubleshooting IDE

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| PyCharm no resuelve `from odoo import ...` | `parts/odoo/odoo` no marcado como Sources Root | Marcar el directorio |
| Run config falla con `No module named odoo` | Intérprete incorrecto | Verificar que el venv del cliente está seleccionado |
| SmartGit no muestra Git-Flow | Git-Flow no configurado | `Git-Flow → Configure → Full` |
| Tests no se encuentran | `--test-tags` incorrecto | Usar formato `/ClassName` o `/ClassName.method` |
| Odoo arranca pero no carga módulos custom | `addons_path` en `odoo.cfg` incompleto | Verificar que incluye `parts/custom` |
