# Scaffolding por Nivel del Dev

## Modelo mental

| Aspecto | Junior (<1 ano) | Mid (1-3 anos) | Senior (3+ anos) |
|---------|-----------------|----------------|-------------------|
| **Rol** | Ejecutor de tarea | Dueno de feature | Dueno del problema |
| **Bloom** | Recordar / Aplicar | Aplicar / Analizar | Evaluar / Crear |
| **Scaffolding** | Alto (modelo + coaching) | Medio (estructura) | Minimo (autonomo) |
| **Detalle tarea** | Paso a paso con archivos | Resultado + constraints | Problema + contexto negocio |

---

## Junior (<1 ano Odoo)

### Que incluir
- Archivos exactos a modificar (rutas completas)
- Modelo/vista/controlador concreto
- Referencia a ejemplo en el codigo: "mira como `sale_margin` lo hace en `sale_order.py`"
- Pasos ordenados como checklist
- Input/output esperado con valores concretos
- Timer de ayuda: "estimado 4h, pide ayuda si llevas 1.5h atascado"

### Que NO incluir
- Codigo de la solucion (tiene que escribirlo el)
- Arquitectura completa del sistema (solo el slice relevante)
- No asignar tareas en ruta critica (bloquea al equipo si se atrasa)

### Verbos a usar
"Sigue el patron de...", "Mira como se hace en...", "Paso 1: ..., Paso 2: ..."

### Ejemplo de guia Junior

```
Paso 1: Crea modulo sale_margin_line (depende de sale, sale_margin)
        Mira $ODOO_SRC/odoo18/addons/sale_margin como referencia de estructura
Paso 2: Hereda sale.order.line con campo computed margin_percent
        Cuidado: standard_price es company-dependent
Paso 3: Hereda vista tree con xpath despues de price_subtotal
        Ref: como sale_margin hereda la vista en sale_margin_view.xml
Paso 4: Tests — crea SO con linea, verifica calculo con assertEqual
        Ref: test_sale_margin.py para patron de test
```

### Preguntas de verificacion

1. "Describeme con tus palabras que hace esta tarea"
2. "Que archivos vas a tocar?"
3. "Puedes mostrarme el ejemplo de referencia que menciono?"
4. "Que vas a hacer primero?"
5. "Que buscarias si te atascas?"

---

## Mid (1-3 anos Odoo)

### Que incluir
- Contexto de negocio y POR QUE importa
- Criterios de aceptacion (que, no como)
- Constraints y edge cases
- Puntos de integracion con otros modulos
- Quien consultar si hay dependencias

### Que NO incluir
- Archivos a editar (debe saberlo)
- Pasos paso a paso (es condescendiente)
- Codigo de referencia (salvo patron realmente no obvio)
- Enfoque tecnico pre-decidido (deja que lo proponga)

### Verbos a usar
"Implementa...", "Considera...", "Ten en cuenta el edge case de...", "Propon tu enfoque antes de empezar"

### Ejemplo de guia Mid

```
Extiende sale.order.line con computed para margen porcentual.
Ref: sale_margin (margen absoluto) — adapta a porcentual.
Considera: standard_price en multi-company, que pasa con qty=0.
Propon tu enfoque en 15 min de sync antes de codificar.
```

### Preguntas de verificacion

1. "Cuentame tu enfoque propuesto"
2. "Que edge cases has identificado?"
3. "Que modelos vas a crear o modificar?"
4. "Como vas a testearlo?"
5. "Cual es la parte mas arriesgada?"

---

## Senior (3+ anos Odoo)

### Que incluir
- El problema de negocio (no la solucion tecnica)
- Impacto de negocio y prioridad
- Constraints (presupuesto, timeline, compatibilidad)
- Stakeholders para coordinar
- Entregables esperados (no pasos)

### Que NO incluir
- Enfoque tecnico (el lo decide)
- Subtareas (el las descompone)
- Micro-deadlines (solo deadline final)
- Tecnologia o patrones pre-seleccionados

### Verbos a usar
"Disena...", "Resuelve...", "Evalua las opciones...", "Presenta tu propuesta tecnica antes de..."

### Ejemplo de guia Senior

```
El director comercial necesita margen % por linea para aprobar descuentos.
Hoy lo calcula en Excel. 3 comerciales lo usan diario.
Atencion: multi-company, coexistencia con pricelist.
Entrega: modulo con tests + PR. Propuesta tecnica antes de empezar.
```

### Preguntas de verificacion

1. "Cual es tu enfoque de alto nivel?"
2. "Cuales son los riesgos principales y como los mitigas?"
3. "Como descompondrias esto para el equipo?"
4. "Que necesitas de mi para estar desbloqueado?"
5. "Cuando puedo ver el doc tecnico?"

---

## Protocolo de Escalacion (cuando pedir ayuda)

Reglas para todo el equipo:

1. **Lee el error/docs** (10 min)
2. **Busca** en codigo, Stack Overflow, wiki interna (15 min)
3. **Reproduce y aisla** el problema (15 min)
4. **Formula la pregunta** por escrito: que intente, que paso, que esperaba
5. **Pregunta** — pero trae las notas del paso 4

### Tiempo maximo atascado antes de pedir ayuda

| Nivel | Timer | Motivo |
|-------|-------|--------|
| Junior | 30-45 min | Si lleva 45 min sin progreso, esta perdido. Intervenir antes de la frustracion |
| Mid | 2-4 horas | Necesita espacio para explorar, pero medio dia perdido es demasiado |
| Senior | 1 dia | Rara vez deberia estar completamente bloqueado sin un camino por probar |

### Cuando dejar que luche vs intervenir

| Dejar que luche | Intervenir |
|----------------|-----------|
| Hace progreso (aunque lento) | Timer superado sin progreso |
| Problema resoluble con docs disponibles | Va en direccion completamente incorrecta |
| La lucha construye una habilidad transferible | Muestra frustracion/bloqueo emocional |
| No esta en ruta critica | La tarea bloquea un deadline |
| | Necesita conocimiento de dominio que no puede encontrar solo |
