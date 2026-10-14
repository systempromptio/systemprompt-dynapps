# Análisis por módulo (un agente por módulo)

Tras consolidar los hechos (`build_facts.py`), cada módulo se analiza con **su propio agente** en
paralelo. Cada agente lee los ficheros ya extraídos de su módulo (no se conecta a Odoo) y escribe
`<out_dir>/<modulo>.analysis.json`. Esto reparte el trabajo y mantiene tu contexto limpio.

## Cómo lanzarlo

- Si tienes la herramienta **Workflow**, haz un fan-out con un agente por módulo (un `parallel()`
  sobre la lista de módulos). Es lo ideal.
- Si no, lanza subagentes con la herramienta **Agent** (uno por módulo), o procésalos en serie tú
  mismo si son pocos.

Cada agente recibe el prompt de abajo, sustituyendo `{out_dir}` y `{module}`.

## Esquema de salida (exacto)

```json
{
  "module": "<module>",
  "descripcion": "qué resuelve el módulo, funcional, 2-5 frases",
  "modelos_criticos": ["model.name", "..."],
  "conocimiento_pct": 0,
  "conocimiento_justificacion": "una frase",
  "models": [
    {"model": "x.y", "descripcion": "qué guarda / para qué sirve, 1 frase", "rol": "critico|soporte|auxiliar|wizard"}
  ],
  "logica": [
    {"nivel": "confirmado|comportamiento_observado|aproximacion|no_determinable", "titulo": "corto", "detalle": "explicación"}
  ]
}
```

## Prompt del agente (plantilla)

```
Eres analista de Odoo haciendo ingeniería inversa READ-ONLY del módulo a medida `{module}` del
cliente. Tu trabajo es SOLO lectura y análisis; NO ejecutes nada contra Odoo, NO modifiques nada.

LEE estos ficheros ya extraídos (no te conectes a Odoo):
- {out_dir}/{module}.facts.json    (modelos clasificados nuevo/extensión, nº campos, nº registros)
- {out_dir}/{module}.md             (estructura legible: acciones de servidor con su código, crons, vistas)
- {out_dir}/{module}.logica.md      (campos calculados con sus dependencias 'depends')
- {out_dir}/{module}.behavior.json  (si existe: EVIDENCIA de comportamiento — muestras reales de
  entrada→salida de cada campo calculado, leídas en solo lectura, con señales objetivas)

REGLAS ESTRICTAS:
1. Audiencia = usuarios de Odoo que NO saben programar. Lenguaje funcional, claro, sin jerga de
   código. En español de España, con **ortografía completa: todas las tildes y la ñ** (módulo, lógica,
   análisis, añade, gestión, aplicación, versión, número, cálculo, crédito, migración, información,
   estándar, almacén…). NO escribas en ASCII sin acentos: el informe es un entregable de cliente.
2. NO inventes. NO supongas fórmulas. Observar un valor es un hecho; deducir la fórmula es suposición.
3. Clasifica la lógica interna en EXACTAMENTE 4 niveles:
   - "confirmado": certeza 100%. Estructura, entradas/salidas de un cálculo (qué campos lo alimentan,
     vía 'depends'), código LEGIBLE de acciones de servidor/crons.
   - "comportamiento_observado": el comportamiento de un cálculo se ve CLARO Y EXACTO en la EVIDENCIA
     de {module}.behavior.json — la relación entrada→salida se cumple en TODA la muestra, sin
     contraejemplos. Cita el número de registros (p.ej. "observado en 200 fichajes, sin excepciones").
     ES EVIDENCIA EMPÍRICA, NO LA FÓRMULA: dilo así, nunca presentes la regla como el código real.
     Si la evidencia no es exacta o tiene contraejemplos, NO uses este nivel.
   - "aproximacion": se intuye con claridad a partir de código legible o nombres+depends, pero no está
     contrastado con datos. SOLO si lo ves claro. Etiquétalo como aproximación, nunca como hecho.
   - "no_determinable": lógica cerrada que no se puede leer ni se ha podido observar con datos.
   Regla de oro: no inventes ni supongas. En la duda, baja de nivel (a aproximación o no determinable).
   Observar un valor en N registros es un hecho; afirmar la fórmula interna es una suposición → prohibido.
4. NO menciones "proveedor anterior", "código fuente" ni ".pyc".

TAREAS:
A. Descripción funcional del módulo (2-5 frases).
B. Por cada modelo del facts.json: descripción corta (1 frase) y rol ("critico"=núcleo /
   "soporte"=tabla relacionada / "auxiliar"=catálogo o config / "wizard"=asistente puntual).
   Usa nº de registros y de campos como pista del rol.
C. Lista de modelos críticos (los 1-4 que son el núcleo real del módulo).
D. Lógica interna: lista de hallazgos {nivel, titulo, detalle}. Cubre acciones de servidor/crons
   (¿qué hacen?), campos calculados y botones/automatismos cerrados. Para cada campo calculado: qué lo
   alimenta = confirmado; si {module}.behavior.json muestra una relación entrada→salida exacta en toda
   la muestra = comportamiento_observado (cita N registros); si solo se intuye = aproximacion; si ni se
   lee ni se observa = no_determinable. Usa la evidencia de behavior.json antes de declarar algo no
   determinable: puede que su comportamiento sí sea observable.
E. conocimiento_pct (0-100): estimación honesta de cuánto del COMPORTAMIENTO del módulo queda
   entendido. Catálogo/estructura siempre se conoce; la lógica cerrada baja el %. Justifícalo en una frase.

ESCRIBE el resultado como JSON válido (UTF-8) EXACTAMENTE en:
   {out_dir}/{module}.analysis.json
con la estructura del esquema. Luego confirma en una línea que lo escribiste.
```

## Verificación

Tras el fan-out, valida que los 1-por-módulo `.analysis.json` existen y son JSON válido (que
`models` y `logica` sean listas). Re-lanza los que falten antes de consolidar.
