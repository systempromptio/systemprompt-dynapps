# Vault Setup — HashiCorp Vault para Dynapps

Instalación y configuración de HashiCorp Vault para acceder a secretos y firmar claves SSH.

---

## Instalación

### Añadir el repositorio de HashiCorp

```bash
wget -O- https://apt.releases.hashicorp.com/gpg | gpg --dearmor | sudo tee /usr/share/keyrings/hashicorp-archive-keyring.gpg
```

Obtener el nombre del codename de Ubuntu/Debian actual:
```bash
lsb_release -cs
# Ejemplo: jammy, noble, bookworm...
```

Añadir el repositorio (sustituir `$(lsb_release -cs)` automáticamente):
```bash
echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/hashicorp.list
```

Instalar:
```bash
sudo apt update && sudo apt install vault
vault version  # verificar
```

---

## Aliases en ~/.bashrc / ~/.zshrc

Añadir al final del fichero RC del shell del usuario (detectar automáticamente si usa bash o zsh):

**Para bash** (`~/.bashrc`):
```bash
cat >> ~/.bashrc << 'EOF'

# Dynapps Vault
alias vault-login="VAULT_ADDR=https://vault.dynapps.be vault login -method=oidc"
alias vault-sign="VAULT_ADDR=https://vault.dynapps.be vault write -field=signed_key ssh/sign/DynApps"
EOF
source ~/.bashrc
```

**Para zsh** (`~/.zshrc`):
```bash
cat >> ~/.zshrc << 'EOF'

# Dynapps Vault
alias vault-login="VAULT_ADDR=https://vault.dynapps.be vault login -method=oidc"
alias vault-sign="VAULT_ADDR=https://vault.dynapps.be vault write -field=signed_key ssh/sign/DynApps"
EOF
source ~/.zshrc
```

---

## OIDC Login Flow

El login en Vault requiere autenticación interactiva via browser. **No se puede automatizar.**

### Pasos manuales para el usuario

1. Abrir una terminal nueva (o hacer `source ~/.bashrc` / `source ~/.zshrc`)
2. Ejecutar:
   ```bash
   vault-login
   ```
3. Se abrirá el browser automáticamente con la página de login de Dynapps (Google/OIDC)
4. Autenticarse con la cuenta `@dynapps.be`
5. El terminal mostrará un token de Vault si el login fue exitoso

### Verificar que el login funcionó

```bash
VAULT_ADDR=https://vault.dynapps.be vault token lookup
# Debe mostrar información del token: display_name, expire_time, policies...
```

### Renovar el token (cuando caduca)

El token de Vault caduca periódicamente. Repetir el proceso de login:
```bash
vault-login
```

---

## Firma de clave SSH con Vault

Una vez logado, para firmar la clave SSH pública y ganar acceso a los servidores:

```bash
vault-sign public_key=@~/.ssh/id_ed25519.pub > ~/.ssh/id_ed25519-cert.pub
```

Verificar la firma:
```bash
ssh-keygen -L -f ~/.ssh/id_ed25519-cert.pub
# Muestra el certificado con su validez y principals
```

Añadir al agente SSH si es necesario:
```bash
ssh-add ~/.ssh/id_ed25519
```

---

## Troubleshooting

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `Error making API request: dial tcp: no route to host` | VPN no conectada | Conectar a la VPN Dynapps primero |
| `permission denied (publickey)` al hacer `vault-login` | Token caducado | Ejecutar `vault-login` de nuevo |
| Browser no se abre | Entorno sin GUI | Copiar la URL que imprime el CLI y abrirla manualmente |
| `vault: command not found` | Instalación incompleta | Verificar que `/usr/bin/vault` existe; repetir instalación |
