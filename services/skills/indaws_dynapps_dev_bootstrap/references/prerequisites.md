# Prerequisites — Dynapps Dev Bootstrap

Comandos exactos para preparar la máquina de desarrollo (§6 proceso Dynapps).

---

## Directorios

```bash
sudo mkdir $ODOO_SRC -p
sudo chown ${USER}: $ODOO_SRC
mkdir $ODOO_SRC/buildouts/ -p
sudo mkdir $ODOO_SRC/tools -p && sudo chown ${USER} $ODOO_SRC/tools
```

Verificar:
```bash
ls -la $ODOO_SRC/
# Esperado: buildouts/ tools/  con owner = usuario actual
```

---

## SSH Key (ed25519)

Sustituir `first.last@dynapps.be` por el email real del desarrollador.

```bash
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519 -C "first.last@dynapps.be"
```

Mostrar la clave pública para añadirla a GitLab:
```bash
cat ~/.ssh/id_ed25519.pub
```

**Acción manual del usuario:**
1. Ir a `https://gitlab.dynapps.be → Preferences → SSH Keys`
2. Pegar el contenido de la clave pública
3. Guardar

Añadir GitLab a known_hosts y probar la conexión:
```bash
ssh-keyscan gitlab.dynapps.be >> ~/.ssh/known_hosts
ssh -T git@gitlab.dynapps.be
# Esperado: "Welcome to GitLab, @first.last!"
```

Probar clone real:
```bash
cd /tmp && git clone git@gitlab.dynapps.be:tools/dyn-tools.git
# Si funciona: la clave está bien configurada
```

---

## pre-commit global + hook prepare-commit-msg

### Instalar pre-commit

```bash
sudo apt install pre-commit
```

### Configurar templateDir global

```bash
git config --global init.templateDir ~/.git-templates
pre-commit init-templatedir ~/.git-templates
```

### Crear el hook prepare-commit-msg

```bash
cat > ~/.git-templates/hooks/prepare-commit-msg << 'EOF'
#!/bin/bash
if [ -z "$BRANCHES_TO_SKIP" ]; then
  BRANCHES_TO_SKIP=(master develop test)
fi
BRANCH_NAME=$(git symbolic-ref --short HEAD)
BRANCH_NAME="${BRANCH_NAME##*/}"
BRANCH_NAME="${BRANCH_NAME%%_*}"
BRANCH_EXCLUDED=$(printf "%s\n" "${BRANCHES_TO_SKIP[@]}" | grep -c "^$BRANCH_NAME$")
BRANCH_IN_COMMIT=$(grep -c "\[$BRANCH_NAME\]" $1)
if [ -n "$BRANCH_NAME" ] && ! [[ $BRANCH_EXCLUDED -eq 1 ]] && ! [[ $BRANCH_IN_COMMIT -ge 1 ]]; then
  sed -i.bak -e "1s/^/[$BRANCH_NAME] /" $1
fi
EOF
chmod +x ~/.git-templates/hooks/prepare-commit-msg
```

Verificar:
```bash
cat ~/.git-templates/hooks/prepare-commit-msg
ls -la ~/.git-templates/hooks/prepare-commit-msg
# Debe tener permiso +x
```

### Re-inicializar repos existentes (si la máquina ya tenía repos)

Ejecutar desde el directorio raíz que contiene todos los repos (normalmente `$ODOO_SRC/buildouts/`):

```bash
for i in `find . -name .git -type d`; do pushd "${i}/.."; rm -f .git/hooks/prepare-commit-msg; git init; popd; done
```

Esto elimina cualquier hook antiguo y aplica el nuevo template en cada repo.

---

## copier

```bash
pipx install copier
copier --version  # verificar instalación
```

---

## SSH config (symlink)

Clonar el repo de configuración SSH compartido y enlazarlo:

```bash
cd $ODOO_SRC/tools
git clone git@gitlab.dynapps.be:tools/ssh-config.git
```

Crear el symlink (cuidado: si ya existe `~/.ssh/config` hacer backup primero):
```bash
# Si existe config previo:
# mv ~/.ssh/config ~/.ssh/config.bak

cd ~/.ssh && ln -s $ODOO_SRC/tools/ssh-config/config config
```

Verificar:
```bash
ls -la ~/.ssh/config
# Debe mostrar: config -> $ODOO_SRC/tools/ssh-config/config
```

---

## python-clone

Clonar el repo:
```bash
cd $ODOO_SRC/tools
git clone git@gitlab.dynapps.be:tools/python-clone.git
cd python-clone/
mkdir local_config
```

Crear y activar el virtualenv con pyenv:
```bash
pyenv virtualenv 3.12.0 python-clone
pyenv activate python-clone
```

Instalar dependencias:
```bash
pip3 install -r requirements.txt
```

Verificar:
```bash
python-clone --help
```

### Versiones Python por versión Odoo

| Versión Odoo | Versión Python |
|-------------|---------------|
| v13 | 3.8.0 |
| v14 | 3.10.0 |
| v15 | 3.10.0 |
| v16 | 3.10.0 |
| v17 | 3.10.0 |
| v18+ | 3.12.0 |

Instalar las versiones necesarias con pyenv antes de crear entornos de cliente:
```bash
pyenv install 3.8.0
pyenv install 3.10.0
pyenv install 3.12.0
```
