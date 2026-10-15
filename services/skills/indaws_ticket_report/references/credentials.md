# Credenciales de Conexion

El skill necesita credenciales para conectarse al Odoo corporativo.

## SIEMPRE solicitar credenciales al usuario

**OBLIGATORIO**: Al iniciar el skill, SIEMPRE solicitar las credenciales al usuario. NO buscar variables de entorno preconfiguradas.

## Credenciales a solicitar

| Credencial | Obligatorio | Por defecto | Descripcion |
|------------|-------------|-------------|-------------|
| **URL del entorno** | No | `https://www.indaws.es` | URL del servidor Odoo |
| **Base de datos** | No | `master` | Nombre de la base de datos |
| **Usuario** | **Si** | - | Email del usuario (login) |
| **API Key** | **Si** | - | API Key del usuario |
| **Version** | No | `19.0` | Version de Odoo |

## Mensaje para solicitar credenciales

```
Necesito tus credenciales de conexion al Odoo corporativo.

Por favor, proporciona:
1. **Usuario** (tu email): [OBLIGATORIO]
2. **API Key**: [OBLIGATORIO]

Valores por defecto (pulsa Enter para aceptar):
- URL: https://www.indaws.es
- Base de datos: master
- Version: 19.0

La API Key se genera en Odoo: Preferencias > Seguridad > Claves API
```

## Configurar las variables

Una vez obtenidas las credenciales del usuario, exportarlas en la sesion:

```bash
export ODOO_CORPORATE_URL="${URL:-https://www.indaws.es}"
export ODOO_CORPORATE_DB="${DB:-master}"
export ODOO_CORPORATE_USER="<email_usuario>"
export ODOO_CORPORATE_KEY="<api_key>"
export ODOO_CORPORATE_VERSION="${VERSION:-19.0}"
```
