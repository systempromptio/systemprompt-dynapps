# Escribir en DatoCMS sin romper otro mercado

Esta referencia solo aplica **cuando la sesión tiene el MCP de DatoCMS**. Si no lo
tiene, la skill entrega texto pegable y aquí no hay nada que hacer.

Nació el 18-sep-2026 haciendo `/es/cases/nobi`, que estaba entero en inglés por debajo
del hero. Todo lo de abajo está verificado contra ese registro, no deducido del manual.

## Lo primero: esto no es «una página»

Un caso de éxito **no es un registro por idioma**. Es **un solo registro
`page_case_study` con once locales** —`en-BE, en-CH, en-ES, en-FR, en-NL, es-ES, fr-BE,
fr-CH, fr-FR, nl-BE, nl-NL`— y los campos de contenido (`seo`, `card_kicker`,
`card_heading`, `card_description`, `header`, `sections`) son **localizados**.

Dos consecuencias que mandan sobre todo lo demás:

1. **Que `/es/` salga en inglés no significa que el español esté vacío heredando.** En
   Nobi el `es-ES` tenía sus propios 34 bloques, con el kicker y dos `alt` ya en
   español: alguien montó la estructura y dejó el cuerpo en inglés dentro. Hay que
   mirarlo, no suponerlo.
2. **Tocar `es-ES` puede o no tocar otros mercados.** En Nobi no: 0 bloques compartidos.
   Pero eso **se comprueba por registro**, porque un bloque reutilizado entre locales
   haría que una edición «solo española» saliera publicada en belga.

## El freno que sustituye al viejo «no hay API»

La regla vieja decía «no publica nunca porque no hay API». Ya la hay, y con permiso de
publicar. El freno sigue en pie **por otro motivo**, y es mejor motivo:

- **La cuenta del MCP no es la de quien pide el cambio.** En esta instalación firma
  `elena.soler@dynapps.es` (nivel `content_only`: crear, editar, borrar y **publicar**).
  Todo lo que se escriba queda en el historial a su nombre. Hay que decirlo antes.
- **El registro es compartido y puede tener trabajo ajeno a medias.** Nobi estaba en
  estado `updated` con un borrador sin publicar en `en-BE` (13 bloques frente a 11).
  Un `items.publish(id)` a secas **habría publicado la versión belga sin terminar.**

Por eso: **se guarda, no se publica**, salvo orden explícita y por locale.

## Procedimiento

### 1. Localizar el registro y mirar el estado

```ts
client.items.list<Schema.PageCaseStudy>({
  filter: { type: 'page_case_study', fields: { slug: { eq: 'nobi' } } },
})
```

Interesan `meta.status` y `meta.current_version`. Si el status es `updated`, **hay
cambios sin publicar de alguien**: averiguar de qué locale antes de seguir, comparando
`version: 'current'` contra `version: 'published'` locale a locale.

### 2. Comprobar el aislamiento de locales

Recoger todos los ids de bloque (incluidos los anidados) de `sections` y `header` por
locale, e intersecar el del idioma que se va a tocar con los otros diez. **Si la
intersección no es cero, parar y decirlo**: ya no es «solo esta página».

### 3. Sacar las cadenas exactas

Volcar las hojas de texto con su ruta, **con `JSON.stringify`**, del locale que se toca.
Hace falta `nested: true` o los bloques vienen como ids y no hay texto que leer.

### 4. Pasada en seco en el sandbox de solo lectura

Se escribe el diccionario `[original exacto, propuesto]` y se corre **con
`upsert_and_execute_safe_script`**, que bloquea las escrituras. El script cuenta cuántos
pares casan y **aborta si alguno no casa**. Nunca se escribe con cadenas sin casar: una
que no casa significa que el texto no es el que crees, y el resto tampoco es de fiar.

### 5. Escribir, sin publicar

```ts
await client.items.update<Schema.PageCaseStudy>(ID, {
  header: hdrOut,
  sections: secOut,
  seo: seoOut,
  meta: { current_version: rec.meta.current_version },  // 409 en vez de pisar
});
```

Tres cosas no negociables en ese payload:

- **`meta.current_version`**. El update es *last-write-wins* por defecto: sin esto, si
  alguien guardó entre tu lectura y tu escritura, te lo llevas por delante en silencio.
- **El hash de locales viaja completo**, pero los que no se tocan van como
  **referencias de id** (`arr.map(b => b.id)`), que significa «este bloque, sin
  cambios». Así no se reenvía el contenido del borrador ajeno.
- **Ni una llamada a `publish`.**

### 6. Verificar contra una lectura nueva

No vale con que el update no dé error. Se relee y se comprueba: que el texto nuevo está,
que **los otros locales siguen con sus bloques**, que `published_at` **no ha cambiado**
(prueba de que no se ha publicado nada) y que no queda inglés residual.

### 7. Publicar, si y solo si lo piden

```ts
await client.items.publish(ID, { content_in_locales: ['es-ES'] });
```

**Selectivo siempre.** Y avisar de que en el registro quedan dos borradores: el nuevo y
el ajeno, así que quien publique desde la UI tiene que elegir locale o subirá los dos.

## Lo que queda fuera de «solo esta página»

- **Los `alt` de imagen que viven en el registro del *upload*.** El upload es **global a
  los once mercados**: arreglarlo ahí es tocar otros mercados. Los `alt` que están dentro
  del bloque (y por tanto son del locale) sí se pueden arreglar.
- **`card_heading` y `card_description`.** Son del registro, sí, pero es lo que se ve en
  los carruseles **de otras páginas**. Si ya están en español y decentes, se dejan.
- **Cualquier otro registro.** Los casos «relacionados» del pie son otros registros.

## Auditoría de colección (antes de prometer nada)

Cuatro comprobaciones que se hacen sobre la colección entera, no sobre una página, y que
el 22-sep-2026 sacaron cosas que nadie había reportado. Todas son de solo lectura.

| Qué | Cómo | Qué salió el 22-sep |
| :--- | :--- | :--- |
| **Qué está publicado de verdad** | `meta.published_at` + diff `published` vs `current` por locale | Nobi seguía sin publicar pese a «Spanish is already published» |
| **Registros que no se pueden publicar** | `meta.is_valid`, `meta.is_current_version_valid` | 3: `caresportconcept`, `estudio360`, `friactiv` |
| **Locales vacíos o clonados** | contar bloques por locale; comparar `en-ES` con `en-CH`/`en-BE`/`en-NL` | `firesystem` publicado con 0 bloques; `en-ES` clon en 21 de 24 |
| **CTA sin destino** | listar `cta_block` **paginando**, y marcar los que no tienen `page` ni `external_url` | 202 de 5.488 (3,7 %) |

Dos cautelas al informar de esto:

- **Paginar o no contar.** `items.list` corta en 500 sin decirlo. Sin paginar, el informe
  dice 500 CTA cuando hay 5.488.
- **Un bloque sin destino puede ser huérfano.** El recuento crudo es el techo del
  problema, no el número de botones rotos que ve un visitante. Si hace falta el número
  real, hay que resolver qué página referencia cada bloque antes de dar la cifra.
