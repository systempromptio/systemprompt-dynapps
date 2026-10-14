# Env Creation — Dynapps Local Environment Setup

Comandos exactos para crear el entorno local de un cliente (§7 proceso Dynapps).

Variables usadas en este documento:
- `$CUSTOMER` — slug del cliente (ej: `acme`)
- `$VERSION` — versión Odoo destino (ej: `17`, `18`)
- `$BUILDOUT_PATH` — `$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}`

---

## setup-local-environment

El script `/usr/local/bin/setup-local-environment` crea el buildout completo.
Debe ejecutarse con `.` (source) para que las variables de entorno queden en la sesión.

### Opción 1: pyenv (venv en `~/.pyenv/versions/$CUSTOMER/bin`)

```bash
. /usr/local/bin/setup-local-environment --version $VERSION --virt-env-choice 1
```

### Opción 2: virtualenv (venv en `$ODOO_SRC/buildouts/${CUSTOMER}/venv/bin`)

```bash
. /usr/local/bin/setup-local-environment --version $VERSION --virt-env-choice 2
```

### Parámetros disponibles

| Parámetro | Descripción | Default |
|-----------|-------------|---------|
| `--version` | Versión Odoo destino (ej: `18`) | Obligatorio |
| `--virt-env-choice` | `1` = pyenv, `2` = virtualenv | Obligatorio |
| `--destination-dir` | Directorio base de buildouts | `$ODOO_SRC/buildouts/` |
| `--odoo-sh` | Flag para proyectos en Odoo SH | No activo |

### Repos de clientes

**GitLab Dynapps:**
```
git@gitlab.dynapps.be:customers/${VERSION}/${CUSTOMER}/odoo
```

**GitHub DynAppsNV:**
```
git@github.com:DynAppsNV/${CUSTOMER}.git
```

Para proyectos GitHub/DynAppsNV: responder `N` cuando el script pregunte si el repo está en GitLab.

### Verificar resultado

```bash
ls $ODOO_SRC/buildouts/${CUSTOMER}${VERSION}/
# Esperado: odoo.cfg  parts/  dyn-build.yaml  buildout.cfg  bin/  eggs/  ...
```

---

## dyn-build.yaml

Fichero de configuración del buildout. Se encuentra en `$BUILDOUT_PATH/dyn-build.yaml`.

### Config mínima para proyectos de migración

```yaml
- repo: parts/custom
  source: local
  test_enabled: true
- repo: parts/oca_pull
  source: local
- repo: parts/modules_to_remove
  source: local
- branch: <version_destino>.0
  repo: odoo/odoo.git
  source: dynapps
```

Sustituir `<version_destino>.0` por la rama correspondiente (ej: `17.0`, `18.0`, `19.0`).

### Fuentes disponibles (`source`)

| Valor | Descripción |
|-------|-------------|
| `local` | Repo local ya clonado en `parts/` |
| `dynapps` | Repo interno de Dynapps (requiere acceso GitLab) |
| `oca` | Repositorio OCA oficial |
| `github` | GitHub público |

---

## PostgreSQL setup

### Verificar que PostgreSQL está activo

```bash
pg_isready -h localhost
# Salida esperada: localhost:5432 - accepting connections
```

Si falla:
```bash
# Debian/Ubuntu
sudo systemctl start postgresql
sudo systemctl enable postgresql   # para que arranque al boot

# macOS (Homebrew)
brew services start postgresql@15
```

### Verificar versión y acceso

```bash
psql --version
psql -U postgres -c "SELECT version();"
# Si el usuario 'odoo' no existe:
sudo -u postgres createuser --superuser odoo
sudo -u postgres psql -c "ALTER USER odoo PASSWORD 'odoo';"
```

### Restaurar el dump migrado

```bash
# Crear BD limpia
createdb ${CUSTOMER}${VERSION}_migrated -T template0

# Restaurar dump (formato -Fc de pg_dump)
pg_restore -U odoo -d ${CUSTOMER}${VERSION}_migrated $DUMP_PATH/original.dump

# Verificar restauración
psql -U odoo -d ${CUSTOMER}${VERSION}_migrated -c "SELECT COUNT(*) FROM ir_module_module WHERE state='installed';"
```

Si el dump está en formato SQL plano (`.sql`):
```bash
createdb ${CUSTOMER}${VERSION}_migrated
psql -U odoo -d ${CUSTOMER}${VERSION}_migrated < $DUMP_PATH/original.dump
```

### Neutralizar credenciales para desarrollo local

```bash
psql -U odoo -d ${CUSTOMER}${VERSION}_migrated -c "UPDATE res_users SET password = 'admin' WHERE id = 2;"
psql -U odoo -d ${CUSTOMER}${VERSION}_migrated -c "UPDATE res_users SET login = 'admin' WHERE id = 2;"
# Verificar neutralización
psql -U odoo -d ${CUSTOMER}${VERSION}_migrated -c "SELECT value FROM ir_config_parameter WHERE key = 'database.is_neutralized';"
```

### Copia para desarrollo iterativo

```bash
createdb -T ${CUSTOMER}${VERSION}_migrated ${CUSTOMER}${VERSION}_dev
```

---

## Inicialización DB del cliente

La BD migrada ya está restaurada. Este paso actualiza los módulos custom con odoo-bin.
La DB debe estar accesible en el PostgreSQL local (paso anterior).

### Verificar que la DB existe

```bash
psql -l | grep ${CUSTOMER}
# O bien:
psql -c "\l" | grep ${CUSTOMER}
```

### Inicializar con el módulo base del cliente

```bash
cd $BUILDOUT_PATH
./parts/odoo/odoo/odoo-bin -c odoo.cfg -d ${CUSTOMER}${VERSION}_base -i xx_base --stop-after-init
```

Sustituir `xx_base` por el módulo base real del cliente (normalmente `${CUSTOMER}_base` o
similar — revisar en `parts/custom/`).

Ejemplo con cliente `acme` versión `17`:
```bash
cd $ODOO_SRC/buildouts/acme17
./parts/odoo/odoo/odoo-bin -c odoo.cfg -d acme17_base -i acme_base --stop-after-init
```

### Verificar inicialización correcta

El log no debe contener `ERROR` ni `CRITICAL`. Al terminar debe mostrar:
```
INFO ... Modules loaded.
INFO ... odoo: addons_path=...
```

### odoo.cfg mínimo (referencia)

Si el fichero `odoo.cfg` no existe o está vacío, estructura básica:

```ini
[options]
addons_path = parts/odoo/odoo/addons,parts/odoo/addons,parts/custom,parts/oca_pull
data_dir = $ODOO_SRC/buildouts/${CUSTOMER}${VERSION}/data
db_name = ${CUSTOMER}${VERSION}
```

---

## Activar el virtualenv

**pyenv:**
```bash
pyenv activate $CUSTOMER
# O con path completo:
source ~/.pyenv/versions/$CUSTOMER/bin/activate
```

**virtualenv:**
```bash
source $ODOO_SRC/buildouts/${CUSTOMER}${VERSION}/venv/bin/activate
```

Verificar que el Python correcto está activo:
```bash
which python
python --version
```
