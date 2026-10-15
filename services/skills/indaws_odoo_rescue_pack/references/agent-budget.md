# Presupuesto del subagente — cuánto cuesta de verdad esta skill

> Todo lo de aquí está **medido** sobre un rescate completo real (Odoo 17 CE, 13 módulos custom,
> ~8.100 líneas, BD de 2,5 GB), el 2026-09-02. No hay ninguna cifra estimada.

## El reparto sale del revés de lo que uno espera

| | Tokens | % |
|---|--:|--:|
| Chat principal (256 turnos) | 49,7 M | 29,6% |
| **10 subagentes (908 turnos)** | **118,2 M** | **70,4%** |
| **TOTAL de un rescate** | **167,9 M** | |

## Por qué: un agente auditor no son 2 turnos, son 150

| Agente | Turnos | Tool calls | Tokens | tok/turno |
|---|--:|--:|--:|--:|
| Auditoría del módulo grande (3.581 líneas) | 151 | 90 | 22,1 M | 146 K |
| Trazado de flujo (mismo código) | 123 | 73 | 17,3 M | 141 K |
| Auditoría del resto de módulos | 127 | 78 | 16,6 M | 130 K |
| Redacción del anexo técnico | 77 | 47 | 15,8 M | 205 K |
| Construcción del módulo `odoo_studio` | 107 | 64 | 12,6 M | 118 K |
| Render del PDF del anexo | 100 | 62 | 11,5 M | 115 K |
| Auditoría de RRHH/documentos | 98 | 62 | 11,2 M | 114 K |
| Trazado de flujo RRHH | 69 | 36 | 6,3 M | 91 K |
| Render del PDF ejecutivo | 43 | 30 | 3,9 M | 90 K |
| Estimación de horas (lookup de tablas) | 13 | 7 | 0,9 M | 71 K |

**La unidad de coste es la LLAMADA A HERRAMIENTA, no el agente:** 549 tool calls → 118,2 M =
**~215 K tokens por llamada**. Cada `Read` de 40 líneas hace que el agente relea su contexto
acumulado entero. Por eso un agente de 13 llamadas cuesta 0,9 M y uno de 90 cuesta 22 M **haciendo
un trabajo del mismo orden**.

**Corolario incómodo:** el coste de un hijo **no depende del contexto del padre**. `/clear` no lo
baja. Lanzar desde un chat limpio de 50 K no ahorra nada si el agente va a hacer 120 llamadas.

## La amplificación que hay que matar

El corpus custom eran **197 ficheros / 400 KB / ~100 K tokens**. Los 5 agentes que lo leían
gastaron **73,5 M**: una amplificación de **735×**, y con solape — el agente de flujos releía lo que
ya había leído el de auditoría.

## Las cuatro reglas que salen de ahí

1. **Empaquetar antes de repartir** (`scripts/pack_corpus.sh`, Fase 0.5). Un fichero por módulo con
   números de línea reales, para que las citas a fichero y línea sigan siendo exactas. El agente lee su
   paquete de una vez: ~90 llamadas pasan a ~15.
2. **Un agente por ÁREA FUNCIONAL, no uno por módulo y otro por flujo.** Auditar un módulo y trazar
   el flujo que lo atraviesa leen los mismos ficheros: se hacen en la misma pasada.
3. **Presupuesto de llamadas explícito en el prompt.** "Máximo ~20 llamadas; agrupa los greps en un
   solo comando; no abras ficheros sueltos que ya estén en tu paquete."
4. **Lo que es un script, no se delega.** Fases 1 y 3 son `bash` determinista. Un subagente ahí sólo
   añade su propio contexto y no decide nada.

## Lo que NO hay que recortar

- **La profundidad del análisis.** Estas reglas cambian *cómo llega el material* al agente, no
  cuánto mira. Un informe con menos hallazgos no es más barato: es peor.
- **La verificación contra la BD.** Es lo que convirtió proyecciones en hechos y tumbó cinco falsos
  positivos en el rescate medido. Sale por SQL, que es barato.
- **La verificación visual del PDF**, pero hazla en dos pasos: un script con PyMuPDF que compare el
  `bbox` derecho de todas las páginas contra el margen, y PNG **sólo de las que marque**.

## Cómo comprobarlo en tu propia sesión

```bash
python3 ~/.claude/scripts/context_preflight.py --agents <N> --agent-work pesado
```
Desde 2026-09-02 el pre-flight separa lo que paga el padre de lo que gasta el hijo. Antes proyectaba
sólo lo primero y se quedaba corto por **50×**.
