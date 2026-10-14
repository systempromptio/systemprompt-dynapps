# Server Action Eval Context - Odoo 18 / 19

## Available Variables

Source: `ir.actions.server._get_eval_context()` + `IrActions._get_eval_context()`

> **IMPORTANTE**: Este documento tiene tres dimensiones:
> - **Odoo Estandar** - Variables disponibles en cualquier instalacion Odoo 18/19
> - **Con indaws_base** - Variables ADICIONALES cuando `indaws_base` esta instalado
> - **v18 vs v19** - Las variables del eval context son **IDENTICAS** entre v18 y v19.
>   Las diferencias v19 estan en la definicion XML, no en el codigo Python (ver `v19-changes.md`)

### Core ORM

| Variable | Type | Description |
|----------|------|-------------|
| `env` | `Environment` | Entorno ORM completo. Equivalente a `self.env` en codigo fuente |
| `model` | `Recordset` (vacio) | Recordset vacio del modelo objetivo. Equivalente a `self.env['model.name']` |
| `record` | `Recordset` | Registro actual (puede ser vacio). En multi-mode es None o el primero |
| `records` | `Recordset` | Todos los registros en multi-mode. Puede ser vacio |

### Usuarios y Seguridad

| Variable | Type | Description |
|----------|------|-------------|
| `uid` | `int` | ID del usuario actual (`self._uid` en codigo fuente) |
| `user` | `res.users` | Registro del usuario actual (`self.env.user` en codigo fuente) |

### Excepciones

| Variable | Type | Description |
|----------|------|-------------|
| `UserError` | `class` | Clase para lanzar errores visibles al usuario |
| `ValidationError` | **NO EXISTE** | **Medido**: `raise ValidationError(...)` en una server action muere con `NameError` **el día que se ejecute esa rama**, no al guardarla. Para avisar al usuario, `UserError` |

### Librerias de Tiempo

| Variable | Type | Funciones disponibles |
|----------|------|-----------------------|
| `time` | module wrapper | `time()`, `strptime()`, `strftime()`, `sleep()` |
| `datetime` | module wrapper | `date`, `datetime`, `time`, `timedelta`, `timezone`, `tzinfo`, `MAXYEAR`, `MINYEAR` |
| `dateutil` | module wrapper | `tz`, `parser`, `relativedelta`, `rrule` |
| `timezone` | function | `pytz.timezone` - para crear objetos timezone |

### Utilidades

| Variable | Type | Description |
|----------|------|-------------|
| `float_compare` | function | Comparacion de floats con precision (`float_compare(a, b, precision_digits=2)`) |
| `b64encode` | function | Codificar datos a base64 |
| `b64decode` | function | Decodificar datos de base64 |
| `Command` | namespace | Comandos x2many: `Command.create()`, `Command.update()`, `Command.delete()`, `Command.unlink()`, `Command.link()`, `Command.set()`, `Command.clear()` |
| `json` | module wrapper | `json.loads()`, `json.dumps()` (inyectado por modulo `mail`/`base_automation`) |

### Logging

| Variable | Type | Description |
|----------|------|-------------|
| `log(message, level='info')` | function | Escribe en tabla `ir.logging` (visible en Technical > Logging) |
| `_logger` | LoggerProxy | Logger de servidor: `_logger.info()`, `_logger.warning()`, `_logger.error()`, `_logger.exception()` |

### Contexto Webhook (solo si aplica)

| Variable | Type | Description |
|----------|------|-------------|
| `payload` | dict/None | Payload de la peticion webhook (solo cuando state='code' y hay webhook) |

---

## Variables Adicionales con indaws_base

Source: `indaws_base/models/ir_actions_server.py` - extiende `_get_eval_context()`

Cuando `indaws_base` esta instalado (la inmensa mayoria de instalaciones Indaws/Dynapps), el eval context se enriquece con:

| Variable | Type | Description |
|----------|------|-------------|
| `json` | module wrapper | `json.loads()`, `json.dumps()` - **Garantizado** (no depende de `mail`) |
| `open` | builtin function | **Acceso a ficheros del servidor** - `open('/tmp/file.csv', 'r')` |
| `wrap_module(module, methods)` | function | Importar modulo de forma segura, exponiendo solo los metodos indicados |
| `import_module(module, methods)` | function | Importar modulo completo con `fromlist` |

### wrap_module vs import_module

**`wrap_module(module, methods)`** - Importa y envuelve un modulo, exponiendo SOLO los metodos listados:
```python
# Importar re con solo los metodos que necesitas
re = wrap_module('re', ['match', 'search', 'findall', 'sub', 'split', 'compile'])
result = re.findall(r'\d+', record.name)

# Importar csv
csv = wrap_module('csv', ['reader', 'writer', 'DictReader', 'DictWriter'])
```

**`import_module(module, methods)`** - Importa el modulo completo (menos restrictivo):
```python
# Importar requests completo
requests = import_module('requests', ['get', 'post', 'put', 'delete'])
response = requests.get('https://api.example.com/data')
data = json.loads(response.text)

# Importar hashlib
hashlib = import_module('hashlib', ['md5', 'sha256'])
hash_val = hashlib.md5(record.name.encode()).hexdigest()
```

**Cual usar:**
- `wrap_module` -> Mas seguro, solo expone metodos explicitamente listados
- `import_module` -> Mas flexible, da acceso completo al modulo

### Modulos comunmente importados con indaws_base

| Modulo | wrap_module methods | Caso de uso |
|--------|-------------------|-------------|
| `re` | `['match', 'search', 'findall', 'sub', 'split', 'compile']` | Expresiones regulares |
| `requests` | `['get', 'post', 'put', 'delete', 'patch', 'Session']` | Llamadas HTTP/API |
| `csv` | `['reader', 'writer', 'DictReader', 'DictWriter']` | Leer/escribir CSV |
| `hashlib` | `['md5', 'sha256', 'sha1']` | Hashing |
| `io` | `['StringIO', 'BytesIO']` | Streams en memoria |
| `xml.etree.ElementTree` | `['parse', 'fromstring', 'tostring', 'Element', 'SubElement']` | Parsear XML |
| `urllib.parse` | `['urlencode', 'quote', 'unquote', 'urlparse']` | Manipular URLs |
| `collections` | `['OrderedDict', 'defaultdict', 'Counter']` | Estructuras de datos |
| `math` | `['ceil', 'floor', 'sqrt', 'log', 'pow']` | Matematicas |
| `html` | `['escape', 'unescape']` | Escapar HTML |
| `uuid` | `['uuid4', 'uuid1']` | Generar UUIDs |

### open() - Acceso a ficheros

```python
# Leer fichero del servidor
with open('/tmp/data.csv', 'r') as f:
    content = f.read()

# Escribir fichero temporal
with open('/tmp/export.json', 'w') as f:
    f.write(json.dumps({'records': records.mapped('name')}))

# CUIDADO: Solo acceder a rutas seguras (/tmp, directorios de datos)
# Nunca acceder a /etc, /home, rutas de sistema
```

---

## Builtins Permitidos por safe_eval

```python
# Tipos
bool, int, float, str, bytes, list, tuple, dict, set

# Iteracion
enumerate, range, zip, map, filter, sorted, reversed

# Agregacion
len, sum, min, max, all, any, abs, round, divmod

# Conversion
repr, ord, chr, isinstance

# Funcional
reduce  # functools.reduce

# Constantes
True, False, None

# Excepciones
Exception
```

### Builtins NO disponibles (diferencia clave)

```python
# PROHIBIDOS en Odoo estandar (algunos DISPONIBLES con indaws_base - ver abajo)
open()          # No acceso a ficheros [SI con indaws_base]
exec()          # No ejecucion dinamica anidada
eval()          # No evaluacion dinamica anidada
compile()       # No compilacion
globals()       # No acceso a globales
locals()        # No acceso a locales
getattr()       # No disponible como builtin
setattr()       # No disponible como builtin
delattr()       # No disponible como builtin
__import__()    # Restringido - solo modulos pre-importados
type()          # No disponible
dir()           # No disponible
vars()          # No disponible
hasattr()       # No disponible
print()         # No disponible - usar log() o _logger
input()         # No disponible
breakpoint()    # No disponible
```

---

## Opcodes Bloqueados (bytecode level)

| Opcode | Efecto | Implicacion practica |
|--------|--------|---------------------|
| `IMPORT_NAME` | No import | No se pueden importar modulos |
| `IMPORT_FROM` | No from...import | No se pueden importar selectivamente |
| `IMPORT_STAR` | No import * | No se pueden importar todos |
| `STORE_ATTR` | No asignacion de atributos | `record.name = 'x'` FALLA |
| `DELETE_ATTR` | No borrado de atributos | `del record.name` FALLA |
| `STORE_GLOBAL` | No asignacion global | No se puede modificar scope global |
| `DELETE_GLOBAL` | No borrado global | No se puede borrar del scope global |

### Atributos Prohibidos (nombre contiene `__`)

Cualquier acceso a atributos con doble underscore esta bloqueado:
- `record.__class__` -> FALLA
- `env.__dict__` -> FALLA
- `model.__name__` -> FALLA

Ademas, estos atributos internos de Python estan explicitamente bloqueados:
`f_builtins`, `f_code`, `f_globals`, `f_locals`, `func_code`, `func_globals`, `co_code`, `mro`, `tb_frame`, `gi_code`, `gi_frame`, `gi_yieldfrom`, `cr_await`, `cr_code`, `cr_frame`, `ag_await`, `ag_code`, `ag_frame`

---

## Modo de Ejecucion

```
safe_eval(code, eval_context, mode="exec", nocopy=True)
```

- **mode="exec"**: Permite statements (if, for, while, asignaciones, etc.)
- **nocopy=True**: El eval_context se pasa directamente (no se copia), permitiendo que el codigo modifique variables del contexto
- **Retorno**: El valor de la variable `action` en eval_context despues de ejecutar el codigo

## Record vs Records: Cuando es cada uno

| Contexto de ejecucion | `record` | `records` |
|----------------------|----------|-----------|
| Boton en formulario (1 registro) | El registro | El registro (1 elemento) |
| Accion en listado (multi) | None | Todos los seleccionados |
| Accion automatizada (create) | Registro creado | Registro creado |
| Accion automatizada (write) | Registro modificado | Registro modificado |
| Accion planificada (cron) | Vacio | Vacio |
| Onchange | Registro del formulario | Vacio |

## Comprobado ejecutando (v18 y v19)

Sonda: `ai-claude-developer/tools/bench-orm` → `tests/test_server_actions.py`, 5/5 en las dos.

| Se probó | Resultado |
|---|---|
| `record.name = 'x'` | `ValidationError: forbidden opcode(s) … STORE_ATTR`, y el valor **no cambia** |
| `record.write({'name': 'x'})` | funciona — es el camino |
| `import json` | prohibido |
| `env`, `record`, `records`, `Command`, `UserError`, `time`, `datetime`, `dateutil`, `log` | **todos presentes**, comprobados uno a uno |
| `ValidationError` | **ausente** — `NameError` en ejecución |

Cada nombre se probó **por separado**, no en una tupla: en una sola expresión, el primero que falta
tumba la comprobación entera y no se sabe cuál de los demás estaba.
