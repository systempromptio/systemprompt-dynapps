# Gotchas — Errores frecuentes al auditar codigo Odoo

Errores reales que Claude comete al analizar modulos. Actualizar cada vez que se detecte uno nuevo.

---

## Falsos Positivos en Seguridad

### 1. SQL injection en queries ORM con `%s`
**Trigger**: Veo `cr.execute("... %s", (var,))` en el codigo.
**Error**: Claude reporta SQL injection.
**Realidad**: `%s` con tupla es el patron CORRECTO en psycopg2. Solo es vulnerable con concatenacion (`+`), f-strings, o `%` sin tupla.
**Regla**: Solo reportar si hay `"SELECT ..." + variable`, `f"..."`, o `% variable` sin tupla.

### 2. `sudo()` clasificado siempre como problema
**Trigger**: Veo cualquier `.sudo()` en Python.
**Error**: Claude marca como "excessive sudo usage".
**Realidad**: Necesario en crons, automated actions, cross-company, config. Solo problema si evade seguridad intencionada.
**Regla**: <5 usos/modulo = normal. >10 = revisar. Verificar contexto.

### 3. `t-raw` reportado sin mirar el contexto
**Trigger**: Veo `t-raw` en templates QWeb XML.
**Error**: Claude reporta XSS.
**Realidad**: Correcto para campos `Html` con sanitize=True (default). Solo XSS con input no sanitizado.
**Regla**: Verificar origen del dato. Campo `fields.Html` → no es XSS.

---

## Falsos Positivos en Performance

### 4. N+1 por `mapped()` confundido con loop explicito
**Trigger**: Veo `records.mapped('partner_id.name')`.
**Error**: Claude reporta N+1.
**Realidad**: `mapped()` hace prefetch. Solo N+1 en loop explicito `for rec: rec.partner_id.name`.
**Regla**: Solo reportar con loop explicito. `mapped()`, `filtered()`, `sorted()` optimizados.

### 5. `search([])` sin limite en contextos validos
**Trigger**: Veo `self.search([])` sin limit, dentro de un cron o batch.
**Error**: Claude reporta "unlimited search".
**Realidad**: En crons/batch, buscar todo es intencionado. Problema solo en HTTP controllers o vistas.
**Regla**: Contexto importa. Cron + filtro estado = OK. Controller HTTP sin limit = problema.

### 6. `store=False` reportado como problema de rendimiento
**Trigger**: Veo campo computado sin `store=True`.
**Error**: Claude reporta "recalculated on every access".
**Realidad**: Muchos computes son ligeros (`field_a + field_b`). Solo problema si hace queries.
**Regla**: Solo reportar si `_compute_*` contiene `search()`, `browse()` masivo, o logica compleja.

---

## Falsos Positivos en Convenciones

### 7. Metodos largos que son overrides necesarios
**Trigger**: Veo metodo >50 lineas que hereda un metodo core (`_prepare_invoice_line`, etc.).
**Error**: Claude reporta "method too long".
**Realidad**: Overrides core a veces copian logica base + modifican. Medir logica ANADIDA, no total.
**Regla**: Override que anade 10 lineas a un metodo core de 50 → no es code smell.

### 8. "Missing docstrings" en modelos obvios
**Trigger**: Veo modelo/metodo sin docstring.
**Error**: Claude reporta en CADA caso.
**Realidad**: `sale.order.discount._compute_discount_amount` no necesita docstring.
**Regla**: Solo reportar si la funcion NO es evidente por el nombre.

---

## Errores de Clasificacion

### 9. Codigo OCA clasificado como "custom de mala calidad"
**Trigger**: Veo modulo con author="OCA" o website="github.com/OCA".
**Error**: Claude lo clasifica como custom del partner.
**Realidad**: OCA tiene su propio estandar. No auditar como custom.
**Regla**: Verificar `__manifest__.py` author/website. Si OCA → reportar aparte.

### 10. Funcionalidad estandar reportada como "reimplementada"
**Trigger**: Veo metodo que hace algo similar a metodo core.
**Error**: Claude reporta reimplementacion.
**Realidad**: Si llama a `super()` + anade logica → es extension valida.
**Regla**: Solo reimplementacion si NO llama a `super()` y hace lo mismo.

### 10b. Un override sin `super()` solo es seguro si nadie mas extiende ese metodo
**Trigger**: Un metodo estandar reemplazado entero, sin `super()`, con una justificacion razonable.
**Error**: Claude juzga el override contra el estandar y lo da por bueno.
**Realidad**: el estandar no es el unico que puede estar ahi. Si OTRO modulo de Odoo sobreescribe
el mismo metodo, gana el que cargue el ultimo y la aportacion del otro **desaparece sin error**.
**Como se comprueba** (30 s, y da la respuesta exacta):
```bash
grep -rn "def _el_metodo" --include=*.py /opt/odoo/odoo<N>/addons | grep -v "/tests/"
```
**Caso medido (PR real, 2026-08-12)**: un modulo redefinia
`_compute_amount_total_without_delivery` para tarificar sobre base neta.
`sale_loyalty_delivery/models/sale_order.py:12`, `SaleOrder` tambien lo sobreescribe, restando tarjetas regalo
y monederos. Con los dos instalados se pierde uno de los dos ajustes, y nada avisa.
**Regla**: ante un override sin `super()`, grep de otros overrides ANTES de opinar. Si hay mas de
uno, es hallazgo aunque hoy ese modulo no este instalado — porque el dia que se instale no salta.

### 10c. El apano sobrevive a su motivo
**Trigger**: Una copia de tripas del estandar justificada por «en la version X el estandar hacia otra cosa».
**Error**: Claude lee la justificacion, la da por buena y sigue.
**Realidad**: la justificacion pudo ser cierta y haber caducado. Una copia de codigo interno de
Odoo **se desincroniza en silencio** en cada point release.
**Como se comprueba**: leer el metodo en la fuente de la build objetivo, no en la que motivo el apano.
**Caso medido (mismo PR)**: se copiaba `_get_price_available` porque una build antigua metia
`amount_total` en linea. En la build del repo el estandar **ya llamaba al hook**, asi que la copia
sobraba y solo aportaba deriva.
**Regla**: toda copia del estandar justificada por version se verifica contra la version objetivo.
Si ya no hace falta, el hallazgo es «retirar la copia», no «esta justificada».

### 11. Severidad inflada en modulos pequenos
**Trigger**: Modulo <100 lineas con multiple findings.
**Error**: Claude reporta 5 CRITICAL en un modulo de 50 lineas.
**Realidad**: Modulo pequeno raramente tiene criticalities reales.
**Regla**: Escalar severidad con tamano. `eval()` produccion = CRITICAL. Campo sin label en 20 lineas = LOW.

---

## Errores de Analisis

### 12. No seguir la cadena de herencia
**Trigger**: Veo `_inherit = 'sale.order'` en un modelo.
**Error**: Claude analiza sin leer el modelo padre → reporta campos/seguridad faltantes.
**Realidad**: Hay que leer el modelo base para saber que ya tiene.
**Regla**: Siempre verificar `_inherit` y consultar modelo base ANTES de reportar.

### 13. Ignorar la version de Odoo del modulo
**Trigger**: Veo `name_get()` o `<tree>` o `group_operator` en el codigo.
**Error**: Claude reporta "deprecated API" sin mirar la version.
**Realidad**: `name_get()` es correcto en v14. Solo deprecated en v16+.
**Regla**: Leer version en `__manifest__.py`. Consultar `odoo-version-api-reference` ANTES de reportar deprecated.

### 14. Un dominio no siempre es una búsqueda — MEDIDO
**Trigger**: veo el nombre de un campo dentro de un `domain=`.
**Error**: darlo por un `search` contra la base de datos.
**Realidad**: hay tres cosas que se escriben igual y no lo son:
- **el campo a la derecha**: `domain="[('id', 'in', available_currency_ids)]"` acota un m2m en un
  formulario — usa el *valor* del campo, no busca por él. Un compute no almacenado vale ahí;
- **modificadores de vista** (`attrs`, `invisible`, `readonly`, `column_invisible`): los evalúa el
  **cliente** contra el registro que tiene delante, no llegan al servidor;
- **listas de nombres de campo**: `@api.depends("a", "b")`, `read([...])`, `mapped()`. Parecen
  condiciones y no lo son.

Solo es una búsqueda si el campo está a la **izquierda** y el segundo elemento es un **operador**
(ojo: en XML van escapados — `&gt;`, `&lt;`). Los tres casos salieron midiendo el pre-localizador
sobre 174 módulos reales; el fixture sintético los daba por buenos.
**Regla**: antes de reportar, mira la posición del campo y quién evalúa ese dominio.

### 15. El pre-localizador cruza por NOMBRE, no sabe de modelos
**Trigger**: un candidato de `compute_no_store`.
**Realidad**: el motor busca el nombre del campo por todo el módulo. Un `("user_id", "=", …)` de
**otro modelo** del mismo módulo cuenta igual.
**Regla**: comprobar que el dominio es del mismo modelo que el campo antes de reportarlo.

---

## Regla general

Antes de reportar un finding:
1. Verificar la version de Odoo del modulo
2. Verificar el contexto (cron, controller, wizard, vista)
3. Verificar si es un override con `super()` o reimplementacion
4. Preferir NO reportar a reportar un falso positivo — la credibilidad del informe depende de su precision
