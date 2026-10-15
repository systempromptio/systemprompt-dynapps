---
name: indaws-es-lint
description: >
  Comprueba que un texto en español de España cumple la doctrina de tono de Dynapps antes de
  publicarlo: tratamiento en «tú», ortografía completa, nada de español neutro ni calcos del
  inglés, y sin las muletillas de la prosa generada. Trae el linter mecánico (severidad,
  co-ocurrencia, 17 familias de patrones) y la referencia de criterio para reescribir.
  Es un gate, no un redactor: dice qué falla y por qué, no inventa el texto.
  Usar cuando pidan "revisa este texto en español", "suena a IA", "esto está traducido",
  "pasa el lint de tono", "quita el ai-slop", "¿esto es español de España?", "revisa la copia
  de la web", o antes de subir cualquier cosa que salga con la marca. Para editar prosa en
  inglés o neerlandés, la skill es `utilities:no-ai-slop`, no esta.
version: 1.0.0
effort: medium
---

# Lint de español de España

Dos capas, y hacen cosas distintas:

| Capa | Fichero | Qué resuelve |
| :--- | :--- | :--- |
| **Mecánica** | `lint/es_lint.py` + `lint/brand_lint_es_web.json` | Lo que se puede comprobar sin criterio: tratamiento, tildes, fuga de español neutro, calcos, muletillas, longitud de campo |
| **De criterio** | `references/es.md` | Lo que exige juicio: las trampas estructurales del español, los falsos positivos, y qué registro pide cada tipo de documento |

La mecánica bloquea. La de criterio se lee antes de reescribir. Ninguna de las dos escribe el texto.

## Lo que manda por encima de esta skill

La doctrina, en Odoo Knowledge, space 12. Si esta skill y esos documentos se contradicen, ganan ellos y hay que decirlo:

- `como-trabajamos/que-publicamos-con-nuestro-nombre.md` fija la voz, el «tú» sin excepción, el diccionario de valor y la ortografía completa.
- `como-trabajamos/como-se-lleva-el-seo.md` fija que en la web **se adapta al mercado local, no se traduce**, y quién toca cada cosa.

## Cómo se usa

El bucle son cuatro pasos y el tercero no lo hace el linter:

1. **Pasar el texto** (mecánico: vale inline o, si es un lote, un `Agent(model: sonnet)`
   como subagente por documento).
2. **Leer los hallazgos**, que citan la línea y nombran el patrón.
3. **Reescribir** con `references/es.md` delante. Esto es criterio y se queda en opus.
4. **Volver a pasarlo.** Un texto que no se re-linta no está verificado.

**[STOP]**: un bloqueante no es una sugerencia. Con bloqueantes el texto **no se publica**,
se corrige y se vuelve al paso 1.

```bash
python3 lint/es_lint.py PAGINA.html              # texto visible de un HTML
python3 lint/es_lint.py copia.txt --text         # prosa suelta
python3 lint/es_lint.py campos.json              # {"path.al.campo": "texto", ...}
cat borrador.md | python3 lint/es_lint.py - --text
```

Flags: `--strict` (falla con cualquier hallazgo), `--all` (incluye los informativos), `--allow palabra,otra` (falso positivo aceptado), `--no-mask` (linta también lo citado), `--pack RUTA.json` (otro pack).

Salida: 0 si no hay bloqueantes, 1 si los hay.

## Las tres severidades

**Bloquean**, porque son reglas duras de la doctrina y no juicios de estilo: `USTED`, `MARCA-VIEJA`, `TILDE`, `DASH`, `MARCA`, `LATAM`, `MAQUINA`.

**Avisan**: el resto de patrones de prosa. Un aviso solo se reporta si su párrafo acumula **dos señales o más**. Esa regla es de `utilities:no-ai-slop` y es la que separa un linter útil de uno que la gente aprende a ignorar: un patrón solo no es evidencia.

**Informan**, y solo con `--all`: ritmo, fragmentos, conectores, jerga.

## Lo que no hace

- **No reescribe.** Dice qué falla; el texto lo escribe una persona o un modelo leyendo `references/es.md`.
- **No puntúa.** Ni porcentajes, ni «esto es 70 % IA». Nombra el patrón y cita la línea.
- **No adivina si lo escribió una máquina.** Eso es lo que hacen los detectores, y fallan.
- **No linta lo citado.** Comillas, guillemets, código y cita en bloque se saltan: un testimonio de cliente y un requisito copiado de un pliego se quedan como los escribió su autor, con todo su slop dentro.
- **No toca el pack `es` de `sales-skills:quotation-builder-value-based`.** Ese pack es el oráculo de paridad byte a byte del motor multipaís y tiene cinco goldens congelados. Este es un pack aparte, para prosa. Ver la cabecera del JSON.
- **No sustituye al supervisor humano.** Un texto sin hallazgos puede ser malo; el linter comprueba que no rompe reglas, no que esté bien escrito.

## Calibración

Calibrado el 2026-09-10 contra el texto visible de `www.dynapps.es` (home, casos de éxito, blog). El criterio no es cazar mucho: es que la copia ya aprobada **no** genere bloqueantes y que una traducción automática los genere a montones.

| Texto | Bloqueantes | Avisos |
| :--- | ---: | ---: |
| Copia viva aprobada, tres páginas | 0 | 4 a 6, todos reales |
| Traducción automática típica | 16 | 21 |
| Testimonio de cliente y pliego citados | 0 | 0 |

```bash
bash lint/tests/run.sh
```

Cualquier cambio en el pack se vuelve a pasar por ahí. Si la copia aprobada empieza a dar bloqueantes, el fallo es del pack, no de la web.

## GOTCHAS

1. **`longitud_campos` no es el schema de DatoCMS.** El tip dice «se cortará al renderizar»
   y eso **no está verificado**: el hero de `dynapps.es/es` tiene 348 caracteres con un
   límite declarado de 160 y se renderiza entero. Vale como regla de redacción, no como
   hecho técnico.
2. **La excepción de marca vieja es literal.** `MARCA-VIEJA` deja pasar «pasa a llamarse»,
   «antes era», «antes Indaws» y «ex-Indaws», y nada más. Cualquier otra forma de nombrar
   a Indaws bloquea, aunque sea legítima: se añade al `excepcion_rx` y se recalibra.
3. **Un aviso solo sale si su párrafo acumula dos señales.** Un patrón aislado no se
   reporta: si buscas un hallazgo que sabes que está, pásalo con `--all`.
4. **Lo citado no se linta.** Un testimonio entre comillas con todo el slop dentro sale
   limpio a propósito. Para lintarlo igualmente, `--no-mask`.
5. **El JSON del pack está en formato legible, no compacto, a propósito.** Se edita a mano
   para añadir léxico, y un pack de una sola línea nadie lo toca sin romperlo.

## Cuando toque afinarlo

Los léxicos viven en el JSON y el motor no lleva datos de idioma: añadir una palabra es editar el pack, nunca el script. Antes de añadir nada, dos preguntas que ahorran falsos positivos:

1. **¿Es la frase completa o un trozo?** Se caza la frase. «A medida de tus necesidades» es folleto; «desarrollo a medida» es terminología precisa de Odoo.
2. **¿Aparece en la copia aprobada?** Si sí, o el pack se pasa de estricto o la web tiene un problema real. Averigua cuál de las dos antes de tocar el pack.
