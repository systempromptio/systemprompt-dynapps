# Catalogo de anti-patrones de filosofia Core

Cada anti-patron: que es, por que rompe la filosofia, como detectarlo
(patron de busqueda) y ejemplo mal vs bien. Filtra siempre con `gotchas.md`.

## 1. Hardcoding de datos de cliente  [CRITICO]

**Por que rompe Core:** un valor de un cliente concreto en codigo impide
reutilizar el modulo en otro cliente. Es la antitesis de "multi-cliente".

### 1.1 Identificadores fiscales / empresa
Buscar CIF/NIF, razones sociales, IBAN, codigos de empresa en `.py`/`.xml`.
- Regex CIF/NIF ES: `[ABCDEFGHJNPQRSUVW]?\d{7,8}[A-Z0-9]`
- Nombres de empresa cliente conocidos, "S.L.", "S.A." en literales de codigo.

Mal:
```python
if self.company_id.vat == 'B12345678':
    account = self.env['account.account'].browse(471)
```
Bien: el comportamiento se decide por un campo de configuracion
(`res.config.settings` / campo en `res.company`), no por el VAT.

### 1.2 IDs de base de datos hardcodeados
IDs numericos en `browse(...)`, en domains o en `ref` que no apunten a XML data
propio del modulo.
- Buscar: `browse(\d+)`, `\[\d+\]` en domains, `default=\d+` en Many2one.

Mal: `partner = self.env['res.partner'].browse(4521)`
Bien: `ref('module.xmlid')`, parametro de configuracion, o busqueda por criterio
estable.

### 1.3 Emails, URLs, IPs, rutas absolutas
Endpoints o credenciales de un cliente embebidos.
- Buscar: `https?://`, `@\w+\.\w+`, `\b\d{1,3}(\.\d{1,3}){3}\b`,
  `/home/`, `/opt/`, `C:\\`.

Mal: `requests.post('https://api.clientex.com/v1', ...)`
Bien: URL en `ir.config_parameter` o campo de ajustes.

> Excepciones legitimas en `gotchas.md` (URLs de esquema/namespace XML, AEAT,
> schemas SEPA, docs de Odoo, etc.).

## 2. Logica condicional por cliente  [CRITICO]

Ramas que existen solo para un cliente.
- Buscar: `if .*company.*name ==`, `if .*\.id ==`, nombres de cliente en
  comentarios junto a `if/else`.

Mal:
```python
if self.env.company.name == 'Clientex':
    self._do_clientex_flow()
```
Bien: el comportamiento se activa por una opcion de configuracion neutra
(`group_*`, campo booleano de ajustes), nombrada por la funcionalidad, no por el
cliente.

## 3. Datos de cliente en `data/` con noupdate  [CRITICO/ALTO]

`data/*.xml` con `noupdate="1"` que carga registros propios de un cliente
(cuentas, diarios, partners, secuencias especificas) en vez de configuracion
generica o demo.
- Revisar `data/` vs `demo/`. Datos de cliente en `data/` = fallo. Datos de
  ejemplo en `demo/` = correcto.

## 4. Falta de parametrizacion  [ALTO]

Valores que varian por cliente escritos como constantes/`default=` fijos en vez
de configuracion: plazos, porcentajes, textos legales, cuentas, plantillas.
- Sintoma: constantes de negocio en el cuerpo del modelo; ausencia de
  `res.config.settings` cuando claramente hay parametros.

Bien: `res.config.settings` + `ir.config_parameter`, o campos en `res.company`
para multi-compania.

## 5. Reimplementacion de estandar / violacion Golden Rules  [CRITICO/ALTO]

Rompe el principio anti-codigo. La deteccion fina se **delega a
odoo-code-auditor (Standard Features)** y a **odoo-classification-guide**; este
skill consolida el veredicto:
- Import/export/conectores hechos a mano → debio usarse `indaws_data_import` /
  `indaws_data_export` (GR1).
- Dashboards/KPIs en codigo → Spreadsheets nativo (GR2).
- Funcionalidad ya cubierta por otro modulo Core → Core Supremacy.
- Reescritura de algo nativo de Odoo (numeracion, mail, actividades, etc.).

## 6. Acoplamiento a un cliente  [ALTO]

`depends` en el manifest hacia modulos custom de un cliente concreto, o
imports de modulos no Core/no estandar.
- Revisar `__manifest__.py` `depends` y `import` cruzados.

## 7. Naming / empaquetado incorrecto  [MEDIO]

- Nombre tecnico no empieza por `indaws_`.
- Modulo huerfano: no encaja en un paquete por area (sale, website_sale,
  purchase, account, stock, project, hr, ...).
- Manifest: `author` no Indaws, `license` ausente, `version` sin formato Odoo
  (p.ej. `19.0.1.0.0`), `category` incoherente, `website` ausente.
- Uso de `x_`/Studio dentro de un modulo Core (Studio no se empaqueta como
  Core; si aparece, es senal de origen Studio sin migrar).

## 8. Documentacion ausente  [MEDIO]

- Sin `README`/`README.rst`/`README.md` ni `static/description/index.html`, o
  manifest con `summary`/`description` vacios o no informativos.
- Alcance: **solo documentacion dentro del modulo**. La ficha en Conocimiento
  es proceso interno del equipo y NO la audita este skill.

## 9. Override de comportamiento estandar SIN gate opt-in  [CRITICO]

El anti-patron mas caro y mas frecuente en modulos Core reales: el modulo
**cambia un flujo nativo de Odoo para TODOS los clientes por defecto** porque
los defaults se pensaron para el cliente donde nacio.

**Por que rompe Core:** un modulo Core se instala en muchos clientes; si al
instalarlo altera create/write, checkout, constraints, reglas o `post_init`
sin que el cliente lo haya activado, rompe instalaciones que no querian ese
comportamiento. "El cliente se adapta al sistema" no significa imponer el flujo
del cliente origen a todos.

**Senales (interpretar, no marcar a ciegas):**
- `create`/`write`/`constrains`/override de metodo nativo cuyo efecto se dispara
  por una condicion generica (`website_id`, `parent_id`, `commercial_partner_id
  != self`, `type='contact'`) en vez de por un flag explicito.
- `default=False`/`True` en un Boolean que **bloquea** un flujo estandar via
  `@api.constrains` (p.ej. exigir aprobadores a cualquier contacto).
- Hook "overridable" pero **hardcoded** a `True`/`False` (`_should_apply_x()`
  que siempre devuelve True) — falso punto de extension.
- `post_init_hook`/`_post_init` que escribe flags en registros existentes
  (carriers, partners) de forma incondicional.
- Controlador que reescribe direcciones/listas nativas para usuarios sin la
  feature configurada.

**Bien:** el comportamiento nuevo va detras de un **flag opt-in
(`<feature>_active` en `res.company`/`res.config.settings`/commercial_partner),
default OFF**, con un **hook real `_should_apply_<feature>(record)`** que el
modulo de cliente pueda extender. Si el flag esta OFF, el modulo no toca nada
nativo. Config funcional neutra ya existente (un enum como
`auto_timesheet_type`, un `groups=` de campo) **SI es un gate valido** — no es
"default de cliente".

## 10. `ir.rule` global no atada a grupo  [ALTO]

`ir.rule` sin `groups` (aplica a TODOS los usuarios en cada lectura del modelo),
a menudo con fallback permisivo `[(1,'=',1)]` y `safe_eval` evaluado en cada
query. Rompe rendimiento y es fragil de seguridad.

**Bien:** regla atada a un grupo concreto (`groups` informado); el caso portal
canonico filtra por `commercial_partner_id` + grupo portal (patron correcto).

## 11. Inconsistencia de empaquetado en modulo nuevo  [MEDIO]

- Modulo **nuevo** que arranca en version avanzada (`18.0.1.7.11`) en vez de
  `<serie>.1.0.0`, arrastrando carpetas `migrations/<serie>.1.x.x/` de saltos
  de desarrollo que ningun cliente tendra: **ruido, eliminar**.
- `i18n/` con `es_ES.po` o `*.pot` cuando la convencion del repo es **solo
  `es.po`**: inconsistencia sin valor.
- `auto_install: True` en un modulo que es **feature opt-in** (solo es valido si
  es infraestructura tipo `indaws_base_*`).
- `icon.png` o URL de logo divergentes del canonico del repo. Excepcion: si hay
  un workstream global de rebranding/logo en marcha, **no** marcarlo aqui (se
  unifica en su propio PR) — ver `gotchas.md`.

## Mapa anti-patron → principio

| # | Anti-patron | Principio violado |
|---|-------------|-------------------|
| 1 | Hardcoding datos cliente | Multi-cliente |
| 2 | Logica por cliente | Multi-cliente / cliente se adapta |
| 3 | Datos cliente en data/ | Multi-cliente |
| 4 | Falta parametrizacion | Multi-cliente |
| 5 | Reimplementar estandar | Anti-codigo / Golden Rules |
| 6 | Acoplamiento a cliente | Multi-cliente |
| 7 | Naming/empaquetado | Empaquetado |
| 8 | Sin documentacion (README/manifest) | Documentacion en el modulo |
| 9 | Override sin gate opt-in | Multi-cliente / cliente se adapta / anti-codigo |
| 10 | ir.rule global no atada a grupo | Multi-cliente / seguridad |
| 11 | Empaquetado modulo nuevo (version/migrations/i18n/auto_install/icon) | Empaquetado |
