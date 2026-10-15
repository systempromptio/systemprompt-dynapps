# Credentials & Role Detection

## Credentials

**SIEMPRE solicitar credenciales al usuario al iniciar.** NO buscar variables de entorno preconfiguradas.

| Credencial | Obligatorio | Default | Descripción |
|---|---|---|---|
| **Usuario** | **Sí** | - | Email del usuario (login) |
| **API Key** | **Sí** | - | API Key del usuario |
| URL | No | `https://www.indaws.es` | Servidor Odoo |
| Base de datos | No | `master` | Nombre BD |
| Versión | No | `19.0` | Versión Odoo |

### Solicitar credenciales
```
Necesito tus credenciales de conexión al Odoo corporativo.
1. Usuario (tu email): [OBLIGATORIO]
2. API Key: [OBLIGATORIO]
Valores por defecto: URL=https://www.indaws.es, DB=master, Versión=19.0
La API Key se genera en Odoo: Preferencias > Seguridad > Claves API
```

### Configurar variables
```bash
export ODOO_CORPORATE_URL="${URL:-https://www.indaws.es}"
export ODOO_CORPORATE_DB="${DB:-master}"
export ODOO_CORPORATE_USER="<email>"
export ODOO_CORPORATE_KEY="<api_key>"
export ODOO_CORPORATE_VERSION="${VERSION:-19.0}"
```

### Flujo de inicio
1. Solicitar credenciales (usuario + API Key obligatorios)
2. Aplicar defaults para URL, DB, versión
3. Exportar variables
4. Preguntar rol → mostrar bienvenida

---

## Role Detection

Preguntar al iniciar con AskUserQuestion:

| Rol | Qué suele pedir |
|---|---|
| **Developer** | Su informe de horas, planificación propia, info de tareas |
| **Team Leader** | Informe de equipo, partes de miembros, comparativa |
| **CTO** | Informe global departamento, por equipo, por proyecto, cualquier empleado |

### Welcome Messages

**Developer:**
- Tu informe de horas / Tu planificación / Tus tareas

**Team Leader:**
- Planificación tuya o del equipo / Partes equipo / Informe equipo / Proyecto

**CTO:**
- Informe global / Por equipo / Por proyecto / Partes individuales
