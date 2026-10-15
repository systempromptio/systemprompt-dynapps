---
name: indaws-onboarding
description: >
  Sistema de onboarding para el marketplace Dynapps Spain AI. Gestiona credenciales compartidas
  (Odoo corporativo) y configuracion por plugin. Usa este skill cuando el usuario necesite
  configurar sus credenciales de acceso, verificar su estado de configuracion, o cuando otro
  skill reporte credenciales faltantes. Invocado automaticamente por skills que detectan
  credenciales no configuradas. NO ejecuta el onboarding del consultor nuevo (usar
  claude-onboarding-dynapps); NO hace mentoria continua (usar odoo-mentor).
version: 1.0.4
effort: low
model: sonnet
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion]
---

# Onboarding Manager — Dynapps Spain AI

Este skill gestiona la configuracion de credenciales y preferencias del marketplace.

See `references/shared/odoo-connection.md` for connection pattern and `references/shared/batch-limits.md` for pagination rules.

## Comandos

### `/setup-credentials` — Configurar credenciales

Flujo interactivo para configurar las credenciales del marketplace.

**Pasos:**

1. **Migracion automatica**: Si existe `~/.claude/.env.odoo` sin estado de onboarding,
   ejecutar `scripts/migrate_onboarding.sh` para migrar al nuevo formato.

2. **Verificar estado actual**: Ejecutar `scripts/check_prerequisites.sh dynapps-spain-ai odoo-corporate`.
   - Si exit 0: credenciales ya configuradas. Informar al usuario y preguntar si quiere reconfigurar.
   - Si exit 1: continuar con la configuracion.

3. **Recopilar credenciales**: Usar AskUserQuestion para solicitar:
   - **Email de Odoo** (ej: usuario@indaws.es)
   - **API Key de Odoo** (generada en Preferencias > Seguridad > Claves API)

   Los valores por defecto de URL (`https://portal.dynapps.es`) y DB (`master`) se usan automaticamente.

4. **Guardar y validar**: Ejecutar:
   ```bash
   ./scripts/setup_credentials.sh dynapps-spain-ai odoo-corporate \
     --field ODOO_URL=https://portal.dynapps.es \
     --field ODOO_DB=master \
     --field ODOO_USER=<email> \
     --field ODOO_KEY=<api_key>
   ```

5. **Resultado**: Informar al usuario del resultado de la validacion.
   - Si valido: credenciales guardadas y verificadas.
   - Si invalido: pedir al usuario que revise sus datos.

### `/check-setup` — Verificar estado

Muestra el estado actual de configuracion del onboarding.

**Pasos:**

1. Ejecutar `scripts/check_prerequisites.sh dynapps-spain-ai odoo-corporate`
2. Leer `<onboarding-root>/dynapps-spain-ai/status.json` si existe
3. Mostrar resumen:
   - Estado de credenciales Odoo (configurado/pendiente/invalido)
   - Fecha de ultima validacion
   - Plugins configurados/pendientes/saltados

## Integracion con otros skills

Cuando un skill detecta credenciales faltantes (exit 1 de check_prerequisites.sh),
debe informar al usuario:

```
Las credenciales de Odoo no estan configuradas. Ejecuta /setup-credentials para configurarlas.
```

## Archivos del sistema

```
${DYNAPPS_HOME:-~/.dynapps}/onboarding/dynapps-spain-ai/      # legacy: ~/.claude/onboarding/...
├── status.json              # Estado general del onboarding
├── credentials/
│   └── odoo-corporate.env   # Credenciales Odoo compartidas
└── plugins/
    └── <plugin-name>.json   # Estado por plugin
```

## Compatibilidad

- `~/.claude/.env.odoo` se mantiene como symlink al nuevo archivo de credenciales
- Scripts existentes que hacen `source ~/.claude/.env.odoo` siguen funcionando sin cambios

## GOTCHAS
1. **ODOO_KEY is not the password** — it's an API key generated in Odoo settings. Users confuse the two.
2. **ODOO_PROTOCOL depends on version** — json2 for Odoo >=19.0, jsonrpc for <19.0. Wrong protocol = silent auth failure.
3. **Credentials are stored in env vars** — they persist only in the current shell. Users must add them to .bashrc/.zshrc.
4. **Corporate Odoo URL includes /web** — but API calls go to root URL without /web. Strip it if user provides full URL.
5. **Employee ID != User ID** — Odoo separates hr.employee (ID) from res.users (UID). Skills need both.

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Verify credentials with test API call |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| All 6 HR skills | hr | Credential setup for HR operations |
| indaws-project-setup | common | Credential setup during project bootstrap |
| odoo-pilot | common | Points to onboarding for initial credential/connection setup |

## RULES
### NEVER
- Store credentials anywhere other than `credentials/odoo-corporate.env`, and only through `setup_credentials.sh`
- Assume credentials are configured — always check first
### ALWAYS
- Validate credentials by making a test API call after setup
- Guide user through the full setup if any credential is missing
