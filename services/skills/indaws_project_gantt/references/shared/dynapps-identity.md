# Dynapps Identity (SSOT de hechos de empresa)

**Único sitio de la verdad** para los *hechos* de empresa Dynapps: nombre, dominios, emails,
URLs e instancias Odoo. **NO reimplementes ni hardcodees estos valores en tu skill** — léelos de
`dynapps-identity.json` (este fichero es el wrapper humano de ese JSON).

> **Frontera con `dynapps-brand`:** aquí van los *hechos* (qué es, cómo se contacta). El *diseño*
> (colores, tipografía, logos, voz/tono) vive en la skill `dynapps-brand`. No dupliques.

## Estado

🚧 **DRAFT** — los valores están pendientes de confirmar (Toni los irá dando). Hasta que
`_status` del JSON sea `confirmed`, **no se migran consumidores**: el SSOT existe para fijar la
ubicación y el esquema, no aún los valores.

## Campos (ver `dynapps-identity.json`)

| Campo | Para qué | Estado |
|---|---|---|
| `brand_name` | nombre de marca en prosa/marketing | pendiente |
| `legal_name` | razón social en documentos | pendiente |
| `website` | dominio web oficial | pendiente |
| `logo_url` | logo alojado en la web | pendiente |
| `email_domain` | dominio de correo corporativo | **pendiente (clave)** |
| `emails.{info,rrhh,soporte_it,desarrollo}` | contactos por área | pendiente |
| `marketing_sender.{name,email,reply_to}` | remitente de campañas | pendiente |
| `odoo_corporate.{url,db,protocol}` | instancia Odoo corporativa | `portal.dynapps.es` (confirmar) |
| `github_org` | org de repos (NO es marca) | `indawsodoo` |
| `social.{linkedin,x,youtube}` | redes | pendiente |

## Cómo consumirlo

- **Scripts / config.json:** leer los defaults de `dynapps-identity.json`; el `config.json` de la
  skill solo sobreescribe lo específico de despliegue, no reescribe la identidad.
- **Prosa de SKILL.md / plantillas:** citar este fichero en vez de escribir "Indaws"/`indaws.es`
  a fuego.

## Enforcement

`skill-designer/scripts/check_brand_hardcoding.py` falla si aparecen valores de empresa
hardcodeados (p.ej. `www.indaws.es`, nombre de empresa en config) fuera de este SSOT, con
allowlist de identificadores **técnicos legítimos** que NO son marca y se quedan: módulos
`indaws_*`, `indaws_api`, modelos `indaws.*`, la org `indawsodoo`, el dir KB `ecosystem/indaws/`
y las skills `indaws-*`.
