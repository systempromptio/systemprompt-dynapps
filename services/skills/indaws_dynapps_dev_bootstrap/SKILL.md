---
name: indaws-dynapps-dev-bootstrap
description: >
  Use this skill when a Dynapps developer needs to set up their machine for the first time
  (or from scratch). Covers directory structure, SSH key generation, GitLab access, pre-commit
  global hooks, copier, HashiCorp Vault, SSH config symlink, and python-clone tool. Run ONCE
  per machine. Activates for: "configurar máquina", "bootstrap dev machine", "setup developer
  environment", "primera vez en esta máquina", "dynapps dev bootstrap".
version: 1.0.3
model: sonnet
effort: medium
allowed-tools: [Bash, Read, Write, Edit, AskUserQuestion]
---

# Dynapps Dev Bootstrap

Prepara la máquina del desarrollador para trabajar en proyectos Dynapps/Indaws. Se ejecuta
**una sola vez por máquina**. Al finalizar, la máquina tendrá directorios, claves SSH, hooks
de pre-commit, Vault y las herramientas base instaladas.

## STOP Points

1. **SSH key pública** — después de generarla, mostrarla al usuario y pedirle que la añada
   manualmente en GitLab (`https://gitlab.dynapps.be → Preferences → SSH Keys`). Esperar confirmación.
2. **Vault OIDC login** — el usuario debe ejecutar `vault-login` en su terminal (abre browser).
   No se puede automatizar. Indicar el comando y esperar confirmación.

## Workflow

### STEP 0 — Verificar estado previo [STOP]

Preguntar con `AskUserQuestion`:
- ¿Es una máquina nueva o ya tenía repos de Odoo clonados?
- ¿Nombre completo y email Dynapps? (para la clave SSH, formato `first.last@dynapps.be`)
- ¿Shell principal: bash o zsh?

### STEP 1 — Directorios base

Ejecutar los comandos de `references/prerequisites.md § Directorios`. Verificar con `ls -la $ODOO_SRC/`.

### STEP 2 — SSH Key ed25519

Ejecutar generación de clave (ver `references/prerequisites.md § SSH Key`).
Mostrar la clave pública. **[STOP]** → Pedir al usuario que la añada en GitLab y confirme.
Tras confirmación: añadir `gitlab.dynapps.be` a `known_hosts` y probar clone de `dyn-tools`.

### STEP 3 — pre-commit global + hook prepare-commit-msg

Instalar `pre-commit`, configurar `init.templateDir`, crear el hook con el contenido exacto
de `references/prerequisites.md § pre-commit`. Si la máquina ya tenía repos, ejecutar el
bucle de re-init para aplicar el hook en todos.

### STEP 4 — copier

```bash
pipx install copier
```

### STEP 5 — HashiCorp Vault

Seguir `references/vault-setup.md § Instalación`. Añadir aliases al fichero RC del shell.
**[STOP]** → Pedir al usuario que ejecute `vault-login` en su terminal y confirme que el
login fue exitoso.

### STEP 6 — SSH config symlink

Clonar `ssh-config` en `$ODOO_SRC/tools/` y crear el symlink en `~/.ssh/config`.
Ver `references/prerequisites.md § SSH config`.

### STEP 7 — python-clone

Clonar repo, crear virtualenv con `pyenv`, instalar dependencias.
Ver `references/prerequisites.md § python-clone`.
Recordar las versiones Python por versión Odoo (tabla en ese mismo fichero).

### STEP 8 — Verificación final

```bash
ssh -T git@gitlab.dynapps.be
copier --version
vault version
python-clone --help  # desde el venv activado
```

Mostrar resumen de lo completado y los STOP points que requirieron acción manual.

## GOTCHAS

1. **La clave SSH debe ser ED25519, no RSA.** GitLab Dynapps espera ed25519; generar RSA puede
   funcionar pero se sale del estándar y algunos hosts internos la rechazan.
2. **`vault-login` no se puede automatizar.** Hace login OIDC vía browser; hay que pedir al usuario
   que lo ejecute en su terminal y confirme. No intentar scriptearlo.
3. **pyenv debe existir ANTES de empezar.** STEP 7 (python-clone) crea el virtualenv con pyenv; si
   pyenv no está instalado, ese paso falla. Verificar `pyenv versions` en STEP 0.
4. **Añadir GitLab a `known_hosts` antes del primer clone**, si no el clone se cuelga esperando
   confirmación interactiva de host key (que no se puede dar en modo no interactivo).
5. **Si la máquina ya tenía repos clonados**, el hook `prepare-commit-msg` global NO se aplica
   retroactivamente — hay que ejecutar el bucle de re-init (STEP 3) en todos los repos existentes.
6. **Las versiones de Python van por versión de Odoo** (tabla en `references/prerequisites.md`);
   usar la versión equivocada en el venv rompe la instalación de dependencias.

## RULES

### NEVER
- Nunca automatizar `vault-login` ni el alta de la clave SSH en GitLab — son pasos manuales del usuario.
- Nunca continuar tras un STOP point sin confirmación explícita del usuario.
- Nunca generar la clave SSH como RSA — siempre ed25519.
- Nunca asumir que pyenv/pipx están instalados — verificarlos en STEP 0.

### ALWAYS
- Ejecutar este skill UNA sola vez por máquina.
- Mostrar la clave pública y esperar a que el usuario la añada en GitLab antes de probar clones.
- Verificar el estado final (STEP 8) antes de dar por completado el bootstrap.
- Aplicar el hook global a los repos preexistentes si la máquina no era nueva.

## Delegates to

None — self-contained (instala herramientas y configura la máquina directamente).

## Consumed by

| Skill | Suite | Context |
|-------|-------|---------|
| `dynapps-migrator` | developer | Prerrequisito de máquina (BLOQUE 1) |
| `dynapps-local-env-setup` | developer | Prerrequisito — la máquina debe estar bootstrapeada |

See `common:references/shared/` for cross-suite shared conventions.

## References

| Fichero | Cuándo leerlo |
|---------|--------------|
| `references/prerequisites.md` | STEPS 1–4, 6–7 — comandos exactos |
| `references/vault-setup.md` | STEP 5 — instalación Vault y aliases |
| `common:references/shared/` | Convenciones compartidas cross-suite |

## Requires

- pyenv instalado en la máquina (`pyenv versions` debe funcionar)
- pipx instalado (`pipx --version`)
- Usuario con sudo
- Acceso a internet para descargar paquetes
