# Factores IA de Esfuerzo — FUENTE DE VERDAD

Unico sitio donde viven los factores IA. Cualquier estimacion de horas los aplica desde aqui.
Una tabla de factores IA en cualquier otro sitio es un duplicado y se borra.

**Alcance:** desarrollo, migracion y estimacion general.

**Excepcion:** los skills de `migration-lab` (Upgrade Lab / Dynapps) no aplican estos factores.
Trabajan sobre migraciones 1:1 acotadas con sus propias reglas ya calibradas.

---

## 1. Regla de una sola cifra

**Una tarea tiene UNA cifra de esfuerzo: la real, con IA.**

Si comercialmente se cotiza otra cosa, es un **multiplicador comercial explicito** aplicado
encima. Nunca se produce una cifra "sin IA" como sustituto de la cifra real.

---

## 2. Principio rector

**La IA reduce el trabajo mecanico. No reduce el criterio, ni el trato con el cliente, ni el
coste de que una tarea exista.**

El factor se aplica **por componente de trabajo**, nunca al total de una tarea de golpe.

---

## 3. Factores por componente

Contrastados con horas reales de proyectos cerrados, salvo donde se indica.

| Componente | Factor | Por que |
|---|---:|---|
| Escribir codigo nuevo (modelos, vistas, wizards) | **×0.70** | Es lo que la IA hace mejor, pero la tarea es mas que teclear |
| Portar codigo de modulo entre versiones | **×0.45** | El trabajo mas mecanico que hacemos. Medido: el coste por modulo baja de 9.0h a 3.9h |
| Migracion de base de datos | **×0.50** | Medido: de 30-42h a ~15h por migracion |
| Migracion de datos / importacion | **×0.80** | Mecanico, pero depende de la calidad del origen |
| Analisis de migracion (1:1 vs Core vs Studio) | **×0.90** | Es una decision de arquitectura, no tecleo |
| Logica de negocio compleja (multi-paso, criterio) | **×0.85** | La IA ayuda; el diseno lo guia el dev |
| Analisis de verificacion (leer fuente, sondear una instancia, probar hipotesis) | **×0.75** | Busqueda y comprobacion, terreno de la IA. Derivado, sin contraste amplio |
| Integraciones con API externa | **×0.85** | La IA parsea docs; sandbox y pruebas siguen siendo manuales |
| Frontend OWL / JS | **×0.85** | Menos predecible, mas debugging manual |
| Tests unitarios | **×0.35** | La IA genera scaffolding, data setup y assertions; el dev revisa |
| Tests de tour / UI, con navegador automatizado disponible | **×0.70** | Requieren navegador real, pero se conducen con IA |
| Tests de tour / UI, sin navegador automatizado | **×1.00** | Interaccion manual |
| Analisis de decision (acordar alcance, elegir entre disenos) | **×1.00** | Reuniones y criterio humano |
| Configuracion funcional, Studio, formacion | **×1.00** | Ya era trabajo humano |
| Overhead tecnico: analisis tecnico, code review, deploy | **×1.00** | Proceso humano. Es el suelo de toda tarea |
| Gestion de proyecto | **×1.00** | Proceso humano |

---

## 4. Estimacion en bloque

Solo cuando la tarea se estima con una cifra unica, sin desglose por componentes:

| Tarea estimada en | Factor efectivo |
|---|---:|
| ≤ 10h | **×0.85** |
| 10 - 25h | **×0.70** |
| > 25h | **×0.75** |

El tramo >25h no es un error: las tareas grandes ya se estimaban con mas holgura, asi que su
margen adicional es menor.

**Desglosar da numeros mas bajos y mas defendibles que el tramo**, porque el overhead queda
explicito en vez de promediado.

---

## 5. Suelo por overhead

No hay tope por tamano de tarea. Una tarea de 4h puede quedarse en 2h si el desglose lo
justifica. Lo que no desaparece es el coste de existir.

| Regla | Valor |
|---|---|
| Suelo por tarea de cliente | **1h** |
| Overhead sin factor, siempre | Analisis tecnico, code review, deploy |

En una tarea pequena el codigo es una fraccion del coste: leer el ticket, entender el contexto
del cliente, decidir, desplegar, avisar e imputar no se comprimen.

---

## 6. Contingencia

**No se toca al aplicar factores IA.** Ni subirla ni bajarla.

Contrastado con el historico de desviaciones: lo que la IA ha cambiado no es tanto la media del
esfuerzo como la **frecuencia de los desbordes graves**, que casi han desaparecido. Antes la
contingencia se quedaba muy corta ante ellos; hoy cubre la dispersion real. Por eso baja el
esfuerzo y no hace falta tocar el porcentaje.

---

## 7. Reglas de aplicacion

1. **Multiplicativo, no aditivo.** ×0.70 sobre 10h = 7h, no 10h − 3h.
2. **Por componente.** Nunca un factor plano al total de la tarea.
3. **Despues de los factores de riesgo, antes de la contingencia.**
4. **Una sola cifra.** El margen comercial va aparte y se ve.
5. **Nunca dos veces sobre la misma cifra.** Una cifra que ya lleva factor se hereda tal cual:
   quien la consume despues no vuelve a factorizarla.
6. **Umbral de revision:** si el conjunto de un presupuesto baja mas del **30%**, no se bloquea
   — se justifica por escrito que partidas lo producen.
7. **Recalibrar solo con tareas >10h.** En tareas pequenas la imputacion tiende a anclarse a la
   estimacion —se imputa lo previsto, no lo gastado— y el ratio deja de medir esfuerzo.

---

## 8. Formula de cobertura de tests

```
gap_LOC       = LOC_code × (gap_percentage / 100)
manual_hours  = gap_LOC × RATE_PER_LOC
ai_hours      = max(manual_hours × 0.35, MINIMUM_HOURS)
final_hours   = ceil(ai_hours × (1 + risk_percentage))
```

| Complejidad | Rate | Minimo |
|---|---|---|
| Low (<500 LOC) | 0.05 h/LOC | 1h |
| Medium (500-2000 LOC) | 0.04 h/LOC | 2h |
| High (>2000 LOC) | 0.03 h/LOC | 4h |

**Scope:** solo modulos custom. Cobertura objetivo 90% (la cobertura del codigo, no confundir con
el 20-30% de *esfuerzo* que cuestan los tests sobre las horas de desarrollo). Coverage existente ≥80% → solo
adaptacion; <80% → adaptacion + gap; 0% → completo.
`coverage_estimate = LOC_tests / (LOC_code × 0.6)`

**No aplica** a tests de integracion con APIs externas: requieren setup manual.
