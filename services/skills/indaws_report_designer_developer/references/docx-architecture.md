# DOCX — `indaws_docx_designer` + OnlyOffice — **HISTÓRICO, camino abandonado**

> ## ⛔ Esta página documenta el camino que NO se siguió. Léela como historia.
>
> `indaws_docx_designer` **pivotó a SuperDoc** (editor DOCX 100% navegador, sin server →
> Odoo.sh-native). La arquitectura viva está en **`superdoc.md`**. Lo de abajo —Document Server en
> Docker, conector `onlyoffice_odoo`, JWT, `pyjwt` horneado— **ya no es la implementación**, y se
> conserva porque explica por qué se descartó.
>
> **Comprobado en disco el 2026-08-17, y no queda nada en pie:**
>
> - `indaws_docx_designer/__manifest__.py` → `depends = ["base","web","indaws_report_docx","indaws_report_designer"]`
>   y **0 referencias a onlyoffice** en todo el addon.
> - `EUROLOTES18/desarrollos/onlyoffice_odoo` y `…_templates` son **cascarones**: **0 ficheros
>   `.py`**; lo único que sobrevive son 13 y 12 `.pyc` sueltos dentro de `__pycache__/`. Son
>   **inertes** —con la disposición PEP 3147 un `.pyc` en `__pycache__/` no se importa sin su
>   `.py`, y sin `__manifest__.py` Odoo ni ve el módulo—, así que no hay nada que limpiar por
>   urgencia; pero tampoco hay conector que instalar.
> - El contenedor `onlyoffice-ds`: **`Exited (137)` desde hace 5 semanas**.
>
> O sea: los pasos de «Arranque OnlyOffice (infra — verificado)» **estaban verificados cuando se
> escribieron y hoy no se pueden reproducir sin volver a clonar el conector**. No los sigas
> esperando encontrar el entorno montado.

> **Nota original (de cuando esto era el plan):** módulo nuevo `indaws_docx_designer`: editor DOCX
> tipo Word (OnlyOffice embebido) orientado a Odoo. Render vía `indaws_report_docx` (docxtpl).

## Cuándo DOCX (vs QWeb)
DOCX = tacto **Word real** + humano maqueta libre. **No** sirve para editar el informe
estándar de Odoo (el estándar es QWeb; no hay versión DOCX). La plantilla se **genera**
(desde mockup, python-docx → `report-docx-template`) y se **edita/rellena** en OnlyOffice.
Fidelidad = fiel a lo diseñado en Word (otro motor, no byte-idéntico al QWeb original).

## Piezas
- **`indaws_docx_designer`** (`$PROJECTS_ROOT/indawsodoo/18/`): modelo `indaws.docx.designer`
  (`name`, `model_id`, `docx_attachment_id` = el .docx editable, `docx_template_id`,
  `report_id`, `source_report_id`, `ai_instruction`). Menú *DOCX Designer → Plantillas DOCX*
  (grupo `group_indaws_docx_designer`).
- **Editor** = conector base **`onlyoffice_odoo`** (NO `onlyoffice_odoo_templates`, ver gotchas):
  acción cliente `{type:"ir.actions.client", tag:"onlyoffice_editor", params:{attachment_id}}`
  o URL `/onlyoffice/editor/<attachment_id>`. `action_open_editor()` la devuelve.
- **Campos (F3)** = `get_docx_fields(path)` → árbol de campos del modelo con tags docxtpl;
  panel OWL `ird_docx_fields` (widget en el form) **copia el tag al portapapeles** (flujo que
  recomienda el propio banner "Automation API not activated"). Contexto de render = **`doc`**
  → `{{ doc.<ruta> }}`; loop de líneas `{%tr for line in doc.order_line %}…{{ line.x }}…{%tr endfor %}`.
- **Render/print (F5)** = `action_generate()` sincroniza el `docx.template` y crea el
  `ir.actions.report` (type=docx) + binding al menú Imprimir. `action_print_test()` renderiza
  un registro de muestra. `unlink()` limpia report + template (+ `file_ids`) + attachment.
- **IA (F4)** = `action_seed_from_qweb()` (semilla mecánica; ver gotcha html4docx) y
  `action_ai_generate()` (LLM configurable: `ir.config_parameter` `indaws_docx_designer.anthropic_api_key`
  + `..anthropic_model`; prompt en `_ai_prompt`; salida HTML→docx en `_apply_html_to_docx`).

## Arranque OnlyOffice (infra — verificado)
1. **Document Server** en Docker: `docker run -d --name onlyoffice-ds --network eurolotes18_default
   -p 8085:80 -e JWT_ENABLED=true -e JWT_SECRET=<secreto> -e JWT_HEADER=Authorization
   onlyoffice/documentserver`. Healthcheck: `curl localhost:8085/healthcheck` → `true`.
   Reachability bidireccional por **nombre** en la red (`http://onlyoffice-ds/` ↔
   `http://eurolotes18-odoo-1:8069/`). El DS mantiene viva `eurolotes18_default` → `down/up`
   de EUROLOTES avisa "network in use" (inofensivo).
2. **Conector** `onlyoffice-odoo` en `EUROLOTES18/desarrollos/` (`onlyoffice_odoo` +
   `onlyoffice_odoo_templates`). Config en Ajustes/`res.config.settings`: public
   `http://localhost:8085/` (navegador), inner `http://onlyoffice-ds/` (Odoo→DS), odoo
   `http://eurolotes18-odoo-1:8069/` (DS→callback), jwt secret + header Authorization.

## GOTCHAS (verificados, 2026-07)
1. **Rama del conector**: `master`/`19.0` usa `type="jsonrpc"` (v19) → en v18 **AssertionError**
   al cargar (`http.route` v18 solo acepta `http`/`json`). Usar **rama `18.0`**
   (`git clone -b 18.0 …onlyoffice-odoo.git`, usa `type="json"`).

   Citas (verificadas 2026-08-07, y confirman que el fallo es **al decorar la ruta**, no en runtime):
   `assert routing.get('type', 'http') in _dispatchers.keys()` — `odoo18/odoo/http.py:739`, `route.decorator`. Las claves
   de `_dispatchers` en v18 son `'http'` (`:2264`) y `'json'` (`:2328`); `'jsonrpc'` no está, así que el
   módulo **no llega a cargar**. En v19 son `'http'` (`:2473`) y `'jsonrpc'` (`:2544`).

   **La asimetría importa**: al revés SÍ funciona. En v19 `type='json'` sigue siendo válido como alias
   deprecado — `"Since 19.0, @route(type='json') is a deprecated alias to @route(type='jsonrpc')"` +
   `routing['type'] = 'jsonrpc'` (`odoo19/odoo/http.py:815-819`, `route.decorator`). O sea: la rama `18.0` del conector
   arranca en v19 (con warning); la `master` no arranca en v18. Al elegir rama, la `18.0` es la
   apuesta segura para cubrir las dos.
2. **`pyjwt`**: dependencia del conector. En imágenes sin entrypoint de requirements (p.ej.
   `test18:1.0.0` de EUROLOTES) **el pip no persiste al recrear el contenedor** → EUROLOTES no
   arranca (onlyoffice_odoo sin `jwt`). Fix: hornear en imagen (`docker commit … test18:1.0.1`)
   y apuntar el `image:` del compose ahí. NO confiar en `pip install` en el contenedor vivo.
3. **`onlyoffice_odoo_templates` = solo formularios planos (sin loops/tablas de líneas)** → NO
   vale para informes. Reutilizar solo el **conector base** (editor genérico de adjuntos).
4. **Loop de tabla docxtpl**: `{%tr for%}`/`{%tr endfor%}` en la **misma fila** da "unknown tag
   endfor" **en 0.16.7 (pin de indaws_report_docx) Y en 0.20.0** (verificado en ambas). Forma
   portable: **filas for/endfor SEPARADAS** (docxtpl consume las filas-tag, sin blancos) o `{%p%}`.
   → la VERSIÓN de docxtpl NO es el problema, la FORMA sí. `indaws_docx_designer._docx_make_docxtpl_safe`
   convierte `{%tr%}`→`{% %}` para robustez. La AI-generación debe emitir filas separadas / `{%p%}`.
5. **`html4docx` APLANA la rejilla Bootstrap** a párrafos (`<w:br/>`, columnas colapsadas, texto
   vertical letra a letra) → **la semilla mecánica QWeb→DOCX no es un punto de partida válido**.
   La IA debe **generar estructura docx** (python-docx: tablas reales con anchos, loop V4) →
   delega en `report-docx-template`. La vía HTML→docx (html4docx) sufre el mismo aplanamiento.
6. **`docx.template`**: el cuerpo va en **`file_ids`** (`docx.file.template`, `position="header"`,
   `lang=False` matchea cualquier idioma), **no** en `file` (que es solo cabecera). Síntoma si te
   equivocas: "No hay plantillas configuradas para el idioma". El FK `template_id` es **required**
   → no vale `(5,0,0)` en `file_ids` (NotNull) → borrar (`file_ids.unlink()`) y recrear.
7. **Campos con comprehension de Python** del QWeb (`{{ ', '.join([… for tax in …]) }}`) →
   **jinja no soporta comprehensions** ("expected ',' got 'for'") → anularlos al sanear la semilla.
8. **Automation API de OnlyOffice**: auto-insertar campos en el cursor requiere plugin
   (browser-heavy, quizá gated por edición) → **diferido**; el flujo actual es copiar-tag→pegar.

## Tests / entorno
Tests: `--test-tags /indaws_docx_designer`. Ciclo EUROLOTES igual que QWeb (down→up→`-u`),
Odoo en host **18069**. El editor OnlyOffice **no es testeable por CLI** → validar en navegador;
los tests cubren modelo, `get_docx_fields`, generate/render, seguridad, semilla docxtpl-safe.
