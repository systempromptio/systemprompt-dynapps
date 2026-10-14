# Calibración de la estimación — modelo de 4 buckets

La estimación se estructura SIEMPRE en los **4 buckets** del Executive Summary
(los "puntos 1 2 3 4"). El motor (`odoo-database-analyzer` / Claude) **sobreestima**
(da 300-800h donde la realidad con IA son 100-150h): esta capa corrige.

## Los 4 buckets y sus reglas

| # | Bucket | Tipo | Regla |
|---|---|---|---|
| 1 | **Code Migration** | variable | Ajustar a horas reales con IA, por módulo. Solo módulos 🔴 Custom |
| 2 | **Database Upgrade** | variable | El **más recortable**, pero **nunca < 20h**. ~K iteraciones × tamaño |
| 3 | **Unit Testing** | fijo | **10 h** (excepción: módulo único pequeño → menos, ~2h) |
| 4 | **Project Management** | fijo | **10 h** |

**TOTAL = 1 + 2 + 3 + 4.** Objetivo **100-150h**; **>200h = bandera roja**.

## Bucket 1 — Code Migration (calibrar por módulo)
1. **OCA y módulos de terceros/vendor — árbol de decisión** (aplica IGUAL a ambos).
   Por cada módulo OCA o de terceros, comprobar en este orden contra la versión
   destino (v19):
   1. **¿Existe candidato/port en la versión superior?** (port OCA v19, o release v19
      del vendor) → **usar el port. Fuera de presupuesto** (~1h solo para verificar
      comportamiento si hace falta).
   2. **¿Odoo absorbió la funcionalidad al core y OCA la descartó?** (la feature ya es
      nativa en v19 y el módulo OCA dejó de mantenerse) → **usar lo nativo. Fuera de
      presupuesto** (~1h para verificar el comportamiento nativo / pequeño reajuste).
   3. **¿No existe port, no está en core, y hay que portarlo?** → **SÍ se estima su
      migración en el presupuesto** (bucket 1, horas según complejidad como un módulo
      más) **Y se deja anotado en el PDF (sección 04) como decisión**: Dynapps/cliente
      decide si se porta o no. Es trabajo real que, de hacerse, cuesta esas horas.
   - **Regla de oro:** *todo lo que NO existe en destino y habría que portar se estima
     en el presupuesto y se anota en el PDF como punto de decisión.* No se asume ni que
     se hace ni que se descarta: se cuantifica y se deja decidir.
   - **Drift:** si un OCA/vendor fue **parcheado localmente**, el parche vuelve a scope
     aunque exista port v19 (hay que re-aplicar/re-portar la personalización).
   - **Módulo que existe en OCA y en Odoo a la vez** (caso típico **comisiones** /
     `sale_commission`): **se mantiene el que ya está instalado**, no se migra como
     custom ni se fuerza el cambio. Fuera del desglose.
   - Entran al desglose: 🔴 Custom + OCA/terceros sin alternativa (caso 3, marcados como
     decisión pendiente).
2. **Excluir módulos ya nativos** en la versión destino (ej. funcionalidad nativa en
   v17+): se eliminan; ~1h para verificar comportamiento nativo.
3. **Eliminar trabajo humano que Indaws no hace**: testing/validación funcional,
   "confirmación humana", QA funcional → fuera (lo hace Dynapps).
4. **Asignar horas por módulo por complejidad** (todo asistido 100% IA). Referencia:
   - Mechanical (cambios mecánicos: tree→list, attrs/states, groups_id→group_ids,
     decoradores/imports) → **1h**.
   - Standard (attrs×N, SQL×pocos, vistas) → **2-3h**.
   - Complex (>500 LOC, OWL rewrite, raw SQL ×muchos, API externa) → **4-9h**.
   - **Cap por módulo ~20h** salvo complejidad excepcional justificada (configurador
     web full OWL rewrite, app externa). Con IA un módulo rara vez pasa de 20h.
5. Subtotal = bucket 1.

## Bucket 2 — Database Upgrade (el más recortable)
- Horas ≈ **nº iteraciones × tamaño** (iteraciones típicas: Test 1 · Test 2 ·
  Rehearsal · Production = 4). En los ejemplos: 4 iter ≈ **40h**.
- Es buffer de cobertura; es el bucket que **más se puede recortar**.
- **Floor duro: 20h.** Nunca bajar de ahí.
- Saltos grandes de versión / BD muy grande pueden subirlo (hasta ~60h).

## Buckets 3 y 4 — fijos por defecto
- **Unit Testing = 10 h** por defecto (cobertura objetivo 80-90%). Excepción: módulo
  único pequeño → ~2h.
- **Project Management = 10 h** por defecto (coordinación, seguimiento, hand-offs, release).
- **Recálculo (solo proyectos muy grandes):** si el bucket 1 es alto, hay muchos
  módulos/conectores o el total se acerca al techo de 200h, **pregunta al usuario**
  si quiere subir 3 y/o 4 (más cobertura de tests / más coordinación) antes de
  hacerlo. Nunca los infles por defecto: 10h+10h es el caso normal.

## Validación final
- Sumar 1+2+3+4. **100-150h es la media histórica** de estos proyectos, no un techo:
  sirve para detectar inflado del analyzer, no para forzar el total a ese rango.
- **>200h → parar y avisar al usuario.** Si el exceso viene de **sobreestimación**,
  recorta. Si viene de **volumen real** (muchos módulos/conectores, saltos grandes
  tipo 12→19), es un **caso excepcional legítimo**: mantenlo y **justifícalo** en la
  sección 04 del informe. No recortes horas reales solo para caber en 150h.
- "Vamos bien de horas, **no añadas horas**". Ante la duda, recortar. Todo recorte
  —y toda hora por encima del rango— debe ser **defendible**.
