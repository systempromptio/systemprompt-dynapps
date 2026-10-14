# Dynbrain — bucle de aprendizaje entre migraciones

Sistema de **dos vías** contra el catálogo vivo de errores comunes en Dynbrain. La idea:
que cada migración deje escrito lo que rompió el mecanismo, y que la siguiente lo lea
**antes** de tropezar con ello. Con varias migraciones, el catálogo evita más de lo que
cuesta mantener.

**Dónde vive:** Dynbrain → espacio `Desarrollo` (16) → `Migraciones › Errores comunes`
(`migraciones/errores-comunes.md`). Las fichas son sus hijos.

El catálogo está organizado **por la fase en que el fallo aparece**, siguiendo el viaje del
dump. Un error nuevo se añade como **sección `##` dentro del documento de su fase**, NUNCA
como documento nuevo: así el catálogo crece sin convertirse en un índice inmanejable.

| Documento | Frontera — qué cae ahí |
|---|---|
| `errores-comunes/pre-migracion.md` | Antes de enviar el dump al Upgrade Service |
| `errores-comunes/upgrade-service.md` | Lo que el servicio le hace a los datos |
| `errores-comunes/post-migracion.md` | Desde que vuelve el dump hasta el entorno levantado |
| `errores-comunes/errores-de-version.md` | **Puntero, no destino** — ver abajo |

Metadatos de esos documentos: `doc_type: gotcha`, `territory: development`,
`categories: ["development","upgrade"]`, `truth_source: observed`, `ai_writable: true`.

### Dos destinos, una pregunta para decidir

> **¿Esto pasaría igual migrando entre otro par de versiones?**

- **Sí** → es **del mecanismo** → sección en el documento de su fase, aquí.
- **No, depende de la versión** (API eliminada, `attrs`, `tree`→`list`, un campo que Odoo
  quitó) → **no va a Dynbrain**: va a `odoo-module-upgrader/patterns/odooXX_to_YY.md`, que es
  lo que lee la sub-skill de port. Duplicarlo aquí crearía dos fuentes de verdad.

---

## VÍA DE LECTURA — antes de tropezar

### Cuándo

1. **BLOQUE 0, en el pre-flight** (junto a `requirements.md`): una sola consulta. Barata y
   preventiva; es la que evita el grueso.
2. **En el punto de convergencia** (antes del `-u all`, ver `workflow.md §Paralelización`):
   segunda consulta, porque ahí es donde aparece la mayoría.

### Cómo

```
docs_search(
  query = "errores comunes al migrar Odoo entre versiones mayores: upgrade service,
           vistas, orden de carga, neutralización",
  space = [16],
  where = {"doc_type": "gotcha", "categories": ["upgrade"]},
  limit = 30
)
```

Leer las fichas relevantes con `docs_get` y **llevarlas como checklist preventivo** durante
toda la corrida: cada una dice *cómo evitarlo*, no solo qué es.

### Qué hacer con ellas

- Las que traen **detección previa** (una consulta SQL, un `grep`) → ejecutarlas en el
  BLOQUE 1/2 y actuar **antes** de que el fallo ocurra.
- Las que traen **medida preventiva** (un script `end-`, un doble paso de neutralización,
  un cálculo de dependencias) → incorporarlas al plan del BLOQUE 2/3 desde el principio.
- Si Dynbrain no está disponible, **la migración no se detiene**: se registra que se corrió
  sin catálogo y se sigue. Es una ayuda, no un gate.

---

## VÍA DE ESCRITURA — dejar escrito lo que rompió

### Cuándo

Al detectar un fallo durante la migración, en cualquier bloque. Los puntos donde de verdad
aparecen: iterate/`-u all` (BLOQUE 2), port de módulos (BLOQUE 3) y paridad (BLOQUE 4) —
incluida la **Capa 2b de navegador**, que destapa una familia que las otras capas no ven
(assets que no cargan, herencias de plantilla que no casan, vistas reactivadas que se pisan).
Esos hallazgos van a `errores-comunes/post-migracion.md` **si pasan el filtro**: el mecanismo
sí entra ("reactivar vistas de una generación anterior duplica columnas"), la regresión de un
módulo propietario concreto no.

### Filtro de admisión — las TRES, o no entra

Como cada ficha entra **confirmada desde la primera detección**, este filtro es la única
salvaguarda. Ante la duda, **no se escribe**.

1. **Es del mecanismo**, no del negocio de un cliente: Upgrade Service, orden de carga,
   vistas, neutralización, estado de módulos, scripts de migración.
2. **Es reproducible en cualquier proyecto**: no depende de un módulo propietario, de un
   dato concreto ni de una decisión de negocio de ese cliente.
3. **Se puede genericar**: el patrón sigue siendo verdad y útil sin los datos del cliente.

**Ejemplos que SÍ entran:** el upgrade archiva las vistas heredadas del custom; `-u all` no
instala dependencias nuevas; los conectores que nacen en el `-u all` quedan sin neutralizar.

**Ejemplos que NO entran:** "el plazo de pago X del cliente Y se migró mal", "faltaba la
columna Z en la lista de productos", cualquier decisión de negocio, cualquier regresión de
un módulo propietario concreto. Eso va a la documentación del proyecto.

### Genericación obligatoria

Antes de escribir, sustituir: nombres de cliente → fuera; prefijos de módulo reales →
`<prefijo>_`; campos concretos → `<campo_custom>`; ids y cantidades del cliente → fuera o
"una fracción significativa". **Una ficha con el nombre de un cliente es una ficha mal
escrita.**

### Procedimiento (deduplicar SIEMPRE antes de crear)

```
1. Clasificar: ¿mecanismo o versión? (la pregunta de arriba).
   Si es de versión → patterns/, NO Dynbrain. Fin.
2. Elegir la fase → uno de los 3 documentos (pre / upgrade-service / post).
3. docs_search(query=<el síntoma en lenguaje natural>, space=[16],
               where={"doc_type": "gotcha"})
4. ¿Existe ya una sección del mismo patrón?
   SÍ  → docs_get del documento, añadir el par de versiones a su línea
         "Visto en", docs_update con el cuerpo completo. No duplicar la sección.
   NO  → docs_get del documento de la fase, APPEND de una sección nueva al final,
         docs_update con el cuerpo completo.
5. Informar al usuario de lo registrado (una línea), sin pedir permiso: es automático.
```

> `docs_update` reemplaza el cuerpo entero: hay que reenviar el documento completo, así que
> **siempre `docs_get` inmediatamente antes**, para no pisar lo que otra migración escribió.

### Forma de una sección

Encabezado `##`, línea de aplicabilidad, y cuatro apartados `###`. Sin relleno:

```markdown
## <El fallo, enunciado como lo que se observa>

**Aplica a:** todos los pares de versión | solo vXX+ · **Visto en:** v16→v19, v14→v17

### Síntoma
Qué se ve. Lo que haría que alguien reconociera su caso.

### Por qué pasa
El mecanismo. Es la parte que hace la sección reutilizable.

### Cómo evitarlo
La medida preventiva concreta. Es la razón de ser del catálogo.

### Detección
La consulta SQL o el comando que lo caza ANTES de que ocurra. Omitir si no existe.
```

La línea **«Aplica a / Visto en»** es obligatoria: es lo que permite saber si algo es
universal o está atado a un par de versiones, sin fragmentar el catálogo por versiones.

---

## Límites honestos

- Dynbrain es **git-synced**: lo que se escribe queda vivo en Odoo y **pendiente de
  publicación** al repo. No anunciarlo como publicado.
- El catálogo **no sustituye** a los gates duros (neutralización, pipeline). Es prevención,
  no verificación.
- Si un hallazgo de este catálogo resulta con el tiempo ser puramente de versión, **se mueve
  a `patterns/`**, no se le crea un documento aquí.
- Una sección nace de **una** observación. Si una migración posterior la contradice, se
  corrige la ficha: `truth_source: observed` significa que describe lo visto, no una ley.
