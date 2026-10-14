# Field Contract Preflight — fallar ruidoso, no en silencio

Patrón compartido para skills que leen/escriben datos Odoo en vivo y dependen de
**campos custom/Studio** (`x_...`, `x_studio_...`) o **modelos Studio** completos
(`x_client_request`, `x_project_indaws_check`). Estos campos/modelos **no son parte
del core**: se renombran, se desinstalan o no existen en la instancia destino. Sin
preflight, la skill produce salida basura o un error críptico **delante del cliente**.

## Principio
Antes de ejecutar el workflow, **verifica que existen los modelos/campos que necesitas**.
Si falta alguno → **aborta con un mensaje accionable** (qué falta, en qué modelo, dónde
se configura), nunca sigas con datos parciales. El preflight convierte un fallo silencioso
en uno contenido y diagnosticable.

## Receta (vía `odoo-pilot`)
Autentica primero (`eval $(./scripts/auth.sh)`), luego comprueba campos con `fields_get`
(devuelve SOLO los campos que existen → los que falten son los que hay que reportar):

```bash
# ¿Existen estos campos en el modelo?
./scripts/execute_method.sh project.project fields_get \
  '[["x_studio_knowledge","x_studio_go_live_date"]]' '{}'
# La respuesta es un dict {campo: {...}} solo con los EXISTENTES.
# requeridos − claves_devueltas = campos que FALTAN.
```

Para un **modelo Studio completo**, comprueba que el modelo existe antes de tocarlo:

```bash
./scripts/search_records.sh ir.model '[["model","=","x_client_request"]]' '["model"]'
# 0 resultados → el modelo no existe en esta instancia → aborta con mensaje.
```

> Regla de oro contra typos: **nunca hardcodees el nombre exacto** de un campo Studio
> propenso a errores (p.ej. `x_studio_fecha_prevista_salida_produccin`, sin "ó") en
> varios sitios. Decláralo UNA vez en el bloque de contrato y reúsalo; si Odoo lo
> renombra, lo cambias en un punto y el preflight lo detecta.

## Bloque a incluir en el SKILL.md
Añade una sección **`## Field contract`** que declare lo que la skill necesita y remita
a este patrón. Ejemplo:

```markdown
## Field contract (preflight OBLIGATORIO)
Antes del workflow, verifica con `odoo-pilot` (ver
`common:references/shared/field-contract-preflight.md`):
- modelo `project.project`: campos `x_studio_knowledge`, `x_studio_go_live_date`
- modelo Studio `x_project_indaws_check` (debe existir)
Si falta algo → **STOP**: "El campo `<campo>` no existe en `<modelo>` de esta instancia.
Revisa la configuración Studio o el mapeo en `references/<fichero>.md`." NO continúes.
```

## Mensaje de fallo (formato)
`✗ Field contract: falta '<campo>' en '<modelo>'. Esta skill lo necesita para <para qué>.
Revísalo en Studio o ajusta el mapeo en references/<fichero>.md. Abortando.`

## Por qué aquí (SSOT)
La receta de preflight vive en este shared para no reimplementarla en 13 skills. Cada
skill solo declara SU contrato (modelos + campos) y delega la mecánica aquí.
