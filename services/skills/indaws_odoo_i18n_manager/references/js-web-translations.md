# Traducciones JavaScript / OWL en Odoo

## Mecanismo general

El cliente web Odoo descarga las traducciones como JSON al cargar la sesión. Los strings se extraen de los archivos JS/OWL del módulo y se incluyen en el `.po` estándar del módulo. No existe un sistema de traducción separado para JS.

## Importar `_t` en JavaScript / OWL

```js
// OWL (v16+) — import desde @web/core/l10n/translation
import { _t } from "@web/core/l10n/translation";

// Uso directo
const label = _t("Confirmar pedido");

// Con placeholder (template literal NO permitido dentro de _t)
const msg = _t("El pedido %s ha sido confirmado");
// Luego: msg.replace("%s", orderName)  — no hay sprintf nativo
```

## `_lt` lazy en JS

```js
import { _lt } from "@web/core/l10n/translation";

// Lazy — para constantes de módulo evaluadas antes de que el contexto esté listo
const LABELS = {
    confirm: _lt("Confirmar"),
    cancel: _lt("Cancelar"),
};
```

## Dónde viven los strings JS en el `.po`

Los strings extraídos de JS aparecen en el `.po` con referencia a su archivo fuente:

```po
#: static/src/components/my_widget.js:42
msgid "Confirmar pedido"
msgstr "Confirm order"
```

El extractor de Odoo escanea `static/src/**/*.js` y `static/src/**/*.xml` (templates OWL).

## Templates OWL (XML) — QUÉ se extrae exactamente

Verificado contra `odoo/tools/translate.py` (`_extract_translatable_qweb_terms` + `OWL_TRANSLATED_ATTRS`).
De un template OWL salen **dos cosas y solo dos**:

1. **Nodos de texto** (y el `tail` de cada nodo).
2. **Nueve atributos**, y ninguno más:
   `title` · `aria-label` · `placeholder` · `alt` · `label` · `data-tooltip` ·
   `aria-placeholder` · `aria-roledescription` · `aria-valuetext`

```xml
<!-- SE EXTRAE: nodo de texto + atributo de la lista -->
<button title="Guardar">Confirmar pedido</button>

<!-- NO SE EXTRAE: el valor de un t-att-* es JAVASCRIPT, no texto de template -->
<button t-att-title="state.on ? 'Desactivar' : 'Activar'">…</button>
```

### La trampa: `t-att-*` no se extrae y no avisa

Un literal dentro de `t-att-title`, `t-att-placeholder`, `t-attf-*`… nunca llega al `.pot`. No hay
error, no hay warning: simplemente ese texto se queda en el idioma en el que lo escribiste, para
siempre. Es el fallo más caro de este apartado porque se detecta en producción, en otro idioma.

**Patrón correcto** — el literal vive en el componente, donde `_t` sí se extrae:

```js
// bubble.js
mode_title() {
    return this.state.useTools
        ? _t("Modo acciones: el asistente puede modificar tus datos.")
        : _t("Solo conversar.");
}
```
```xml
<button t-att-title="mode_title()"/>
```

Regla práctica: **si el atributo lleva `t-att-`, su texto va en el componente**. Y al revés: si es un
atributo estático de la lista de nueve, déjalo en el XML.

### Frases partidas por un `<t t-esc/>`

```xml
<!-- MAL: dos msgid sueltos que nadie puede recomponer -->
<div>Primera fila · <t t-esc="n"/> más iguales</div>
```
El extractor produce `"Primera fila ·"` y `"más iguales"` por separado. Además del problema de
traducción, el orden de las palabras cambia según el idioma y el hueco queda donde no toca.

```js
// BIEN: un solo término, con su placeholder
rows_note() {
    return _t("Primera fila · %s más iguales", this.count);
}
```

### Literales partidos por longitud: se UNEN

El extractor JS (`odoo/tools/babel/javascript_extractor.py`) acumula los trozos en `message_buffer`,
así que esto es **un** `msgid`:

```js
_t("una frase larga que no cabe en 100 columnas " +
   "y continúa aquí")
```
Lo mismo con literales adyacentes en Python. Consecuencia práctica: cualquier verificación de
"este `msgid` sigue en el código" tiene que normalizar espacios **y** deshacer ese empalme antes de
comparar, o dará falsos positivos.

### Comprobar qué recibe REALMENTE el navegador

```js
// en la consola del backend
const r = await fetch('/web/webclient/translations?mods=mi_modulo').then(x => x.json());
r.modules.mi_modulo.messages.filter(m => m.id.includes("Confirmar"));
```
Si el `msgid` no está en esa respuesta, el fallo es de **extracción** (mira si vive en un `t-att-*`),
no de traducción. Ahorra media hora de mirar el `.po`.

## Bundle i18n — requisitos

Para que las traducciones JS se carguen en el cliente:
1. El módulo debe tener `web` (o `base_setup`) en `depends` del `__manifest__.py`
2. Los archivos JS deben estar declarados en los assets del manifest:
   ```python
   'assets': {
       'web.assets_backend': [
           'my_module/static/src/components/*.js',
           'my_module/static/src/components/*.xml',
       ],
   }
   ```
3. El archivo `.po` del idioma activo debe existir en `i18n/`

Si las traducciones JS no aparecen tras importar el `.po`, verificar estos 3 puntos.

## Exportar strings JS

El CLI de exportación recoge automáticamente los strings JS si el módulo está instalado y sus assets registrados:

```bash
# v19: odoo-bin i18n export -c <conf> -d <db> my_module -l es_ES   (escribe i18n/es.po)
# v17/v18:
python odoo-bin --i18n-export=my_module/i18n/es.po \
  --language=es_ES --modules=my_module -d <db> --stop-after-init 2>&1 | tail -20
```

Si los strings JS no aparecen en el `.pot` exportado, verificar que los assets están declarados y el módulo está instalado en la BD usada para exportar.

## Diferencias v17 vs v18

| Aspecto | v17 | v18 |
|---------|-----|-----|
| Import `_t` | `@web/core/l10n/translation` | igual |
| Lazy `_lt` | `@web/core/l10n/translation` | igual |
| Template OWL strings | automático | automático |
| `env._t` en componente | no disponible | disponible como `this.env._t` |

En v18, dentro de un componente OWL se puede usar `this.env._t("string")` que lleva el contexto de idioma del usuario. Fuera del componente, seguir usando el import estático de `_t`.
