# Fuentes upstream y qué se tomó de ellas

Esta skill **no** es original: instancia el patrón de otro. Aquí queda quién es ese otro, cómo se
detecta que se ha movido, y —lo que ahorra más tiempo— **el veredicto de haberlo comparado ya**, para
que la próxima sesión no repita el análisis.

Vigilancia: `python3 scripts/check_upstream.py` (0 tokens, hash del contenido crudo; los pins viven
en el propio script). Devuelve 1 si algo cambió.

## Las fuentes

| Fuente | Qué es | Copia local | Estado 2026-08-07 |
|---|---|---|---|
| [gist de Karpathy](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) | **El patrón.** Documento de idea, deliberadamente abstracto | `references/source-gist.md` — **byte-idéntica** a la viva | sin cambios |
| [NicholasSpisak/second-brain](https://github.com/NicholasSpisak/second-brain) | Instanciación independiente del mismo gist, como 4 skills instalables | ninguna (a propósito) | sin cambios |

Detalle de anclaje: el API de gists de GitHub devolvió **502 `Server Error`** el 2026-08-07, anónimo y
con `gh` autenticado, mientras la URL raw servía 200. Por eso el ancla es el hash del contenido y no
la revisión del API: un ancla que depende de un servicio caído informa "no sé" justo el día malo.

## Veredicto de comparar `second-brain` con esta skill (2026-08-07)

Es una instanciación **más fina** del mismo gist, no una fuente de la que importar. Su contenido son
4 `SKILL.md` (~17 KB) + 2 referencias + un `onboarding.sh`; esta skill son 148 líneas de SKILL, 8
referencias (~1.500 líneas) y 12 scripts deterministas.

**Lo único que se tomó: `output/`.** Tienen un tercer directorio de primer nivel para artefactos
generados (informes, resultados de query renderizados). Aquí no existía, y la separación es correcta:
una síntesis archivada **es** página de wiki y va a `wiki/syntheses/`, pero un PDF, un deck de Marp o
un PNG de matplotlib **no** son páginas — no se enlazan, no se lintean, y metidos en `wiki/` ensucian
el grafo y los recuentos del lint.

Lo que **ya teníamos, y en forma más fuerte** (por eso no se importó nada más):

| Ellos | Aquí |
|---|---|
| Taxonomía fija `sources/entities/concepts/synthesis` | **Perfiles por dominio** (`research`, `book`, `business`, `code`…), cada uno con su taxonomía — `references/profiles.md` |
| Lint como instrucciones en prosa para que el LLM las siga | `wiki_lint.py`: enlaces roto, huérfanas, callejones, drift de índice, frontmatter, fuentes sin citar — **script, 0 tokens**, y el pase de juicio aparte |
| Frontmatter `tags, sources, created, updated` | los mismos **más `confidence: low\|medium\|high`** — decir lo que no sabes es la mitad del valor |
| — | **Perfil `code`**: wiki sobre un árbol de fuentes, citas ancladas a `@sha`, `code_extract.py`/`code_query.py`, `wiki_cite_check.py` (¿la cita *resuelve*?) |
| — | Disciplina de tokens y reparto opus/sonnet, gotcha de subagentes en paralelo, `open-questions.md`, doctrina de contradicciones fechadas |
| Portabilidad a 4 agentes (Codex/Cursor/Gemini) | irrelevante aquí: suite federada de Claude Code |

**Corroboración, que también vale algo:** sus cuatro cajones (`sources/entities/concepts/synthesis`),
el `[[wikilink]]`, el `index.md` + `log.md` y el "una fuente toca 10-15 páginas" coinciden con lo que
hay aquí. Dos instanciaciones independientes del mismo gist convergieron en la misma estructura, así
que esa parte del diseño no es preferencia nuestra.

**Lo que tienen y aquí falta de verdad, aunque no sea de la skill:** un test de su script de
scaffold (`tests/test_onboarding.sh`). `wiki_scaffold.py` no tiene ninguno. Anotado como deuda, no
resuelto.

## Si `check_upstream.py` avisa de cambio

1. **Gist**: leer el diff y refrescar `references/source-gist.md` — el agente consulta la copia
   local, y si el gist cambió la copia miente. Actualizar el pin.
2. **second-brain**: leer el diff antes de asumir que no aporta. El veredicto de arriba es de una
   fecha; si han añadido algo (p.ej. tests, o un mecanismo de búsqueda), reevaluar **esa parte**, no
   la comparación entera.
3. En los dos casos: actualizar `checked` en `UPSTREAM` y dejar dicho en este fichero qué se decidió.
