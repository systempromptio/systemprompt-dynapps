# Mejores Practicas para Demos de Odoo

## Estructura Temporal

Las demos de Odoo se adaptan a tres formatos segun la duracion disponible. La estructura sigue un patron de apertura-desarrollo-cierre con tiempos proporcionales.

### Demo Corta (30 min)

| Fase | Duracion | Contenido |
|------|----------|-----------|
| Apertura | 5 min | Contexto del cliente, agenda, preguntas de encuadre |
| Demo | 20 min | 1-2 flujos principales, momentos WOW seleccionados |
| Cierre | 5 min | Resumen de valor, proximos pasos, Q&A rapida |

**Regla**: Maximo 2 flujos. Sin navegacion por menus. Ir directo al valor.

### Demo Estandar (60 min)

| Fase | Duracion | Contenido |
|------|----------|-----------|
| Apertura | 10 min | Contexto, agenda, discovery rapido (2-3 preguntas) |
| Demo | 40 min | 3-4 flujos con transiciones, momentos WOW, framing before/after |
| Q&A | 5 min | Preguntas del cliente sobre lo mostrado |
| Cierre | 5 min | Resumen de beneficios, proximos pasos concretos |

**Regla**: 10 minutos por flujo maximo. Transiciones narrativas entre flujos.

### Demo Extendida (90 min)

| Fase | Duracion | Contenido |
|------|----------|-----------|
| Apertura | 10 min | Contexto, agenda detallada, discovery (3-5 preguntas) |
| Demo Parte 1 | 35 min | 2-3 flujos core con profundidad |
| Pausa interactiva | 5 min | Q&A parcial, feedback del cliente |
| Demo Parte 2 | 25 min | 2-3 flujos complementarios, integraciones |
| Q&A | 10 min | Preguntas, objeciones, escenarios "y si..." |
| Cierre | 5 min | Resumen, roadmap propuesto, call-to-action |

**Regla**: Incluir pausa. Maximo 5 flujos. La segunda parte responde a lo detectado en la primera.

---

## Micro-framework F.A.V.O.R.I.T.E.

Cada funcionalidad o flujo se presenta siguiendo esta estructura:

| Paso | Significado | Que hacer |
|------|-------------|-----------|
| **F**rame | Encuadrar | "Actualmente ustedes hacen X de forma manual..." |
| **A**sk | Preguntar | "¿Cuanto tiempo les lleva este proceso?" |
| **V**alue | Valor | "Con Odoo, esto se reduce a 3 clics..." |
| **O**perate | Operar | Ejecutar el flujo en la demo en vivo |
| **R**eact | Reaccionar | Observar la reaccion del cliente, pausar si hay interes |
| **I**terate | Iterar | Si hay pregunta, profundizar; si no, avanzar |
| **T**ransition | Transicionar | "Y esto conecta directamente con..." |
| **E**ngage | Comprometer | "¿Esto resolveria su necesidad de...?" |

**Uso**: No es necesario ejecutar los 8 pasos en cada funcionalidad. Para funcionalidades menores, Frame + Operate + Transition es suficiente. El framework completo se reserva para los momentos WOW.

---

## Framing Before/After

Antes de cada flujo principal, establecer el contraste:

```
ANTES (sin Odoo):
- "{pain_point_concreto}"
- Proceso: {pasos_manuales}
- Resultado: errores, retrasos, falta de visibilidad

DESPUES (con Odoo):
- [Ejecutar flujo en vivo]
- Resultado: {beneficio_medible}
```

**Template de frase**: "Hoy ustedes {proceso_actual}. Con Odoo, van a ver como {resultado_deseado} en {tiempo/clics}."

---

## Momentos WOW Generales de Odoo

Estos momentos generan impacto visual independientemente del modulo:

| Momento WOW | Descripcion | Cuando usarlo |
|-------------|-------------|---------------|
| Pipeline Kanban | Drag-and-drop de oportunidades entre etapas | CRM, Proyecto |
| Creacion rapida | Presupuesto en 3 clics desde oportunidad | Ventas + CRM |
| Portal del cliente | El cliente ve sus pedidos, facturas, tickets | Cualquier demo con portal |
| Automatizaciones | Reglas que ejecutan acciones sin intervencion | Cualquier modulo |
| Dashboard en tiempo real | Graficos actualizados al instante | Inventario, Contabilidad |
| Conciliacion bancaria | Match automatico de movimientos | Contabilidad |
| Trazabilidad completa | Click en factura → pedido → albaran → presupuesto | Flujos combinados |
| App movil | Aprobar gastos, fichar, ver stock desde el movil | RRHH, Inventario |
| Chatter / Historial | Todo el historico de comunicacion en un lugar | Cualquier registro |
| Configuracion Studio | Personalizar sin codigo (campos, vistas, automatizaciones) | Cierre, diferenciacion |

---

## Anti-patrones (EVITAR)

### Feature Overload
- **Error**: Mostrar 15 funcionalidades en 60 minutos
- **Consecuencia**: El cliente no recuerda nada, se siente abrumado
- **Regla**: Maximo 1 flujo cada 10 minutos. Menos es mas.

### Demo Generica
- **Error**: Usar datos por defecto de Odoo (My Company, Product A, Customer 1)
- **Consecuencia**: El cliente no se identifica con la solucion
- **Regla**: Personalizar datos con nombre del cliente, productos de su sector, clientes tipicos

### Lenguaje "Yo"
- **Error**: "Ahora yo voy a crear un presupuesto, yo configuro..."
- **Consecuencia**: El cliente se siente espectador, no protagonista
- **Regla**: "Ustedes van a poder crear presupuestos...", "Su equipo de ventas vera..."

### Navegacion por Menus
- **Error**: Explicar la estructura de menus del ERP
- **Consecuencia**: Aburrimiento, sensacion de complejidad
- **Regla**: Navegar con proposito. "Vamos a ver como se gestiona un pedido" → click directo.

### Demo sin Datos
- **Error**: Mostrar listas vacias, formularios en blanco
- **Consecuencia**: El cliente no visualiza el uso real
- **Regla**: Tener datos demo realistas preparados SIEMPRE

### Ignorar Senales del Cliente
- **Error**: Seguir el guion rigidamente cuando el cliente quiere profundizar en algo
- **Consecuencia**: Oportunidad perdida, desconexion
- **Regla**: El guion es una guia, no un script rigido. Si el cliente se interesa, profundizar.

### Hablar de Tecnologia
- **Error**: Mencionar Python, PostgreSQL, ORM, API, modulos tecnicos
- **Consecuencia**: Desconexion del cliente no tecnico
- **Regla**: Solo hablar de resultados de negocio. "El sistema automaticamente..." en vez de "El workflow ORM ejecuta..."

---

## Preguntas de Engagement por Fase

### Apertura
- "¿Como gestionan actualmente {proceso}?"
- "¿Cual es el mayor reto que enfrentan en {area}?"
- "¿Cuantas personas participan en {proceso}?"

### Durante Demo
- "¿Esto se parece a como trabajan ustedes?"
- "¿Ven como esto les ahorraria tiempo en {tarea}?"
- "¿Quien de su equipo usaria esta funcionalidad?"

### Cierre
- "¿Que les ha parecido mas util de lo que hemos visto?"
- "¿Hay algun proceso que no hayamos cubierto y les gustaria ver?"
- "¿Cual seria el siguiente paso ideal para ustedes?"

---

## Reglas de Oro

1. **Personalizar**: Datos del sector del cliente, no datos genericos
2. **Menos es mas**: 3 flujos bien mostrados > 10 funcionalidades rapidas
3. **Narrativa**: Contar una historia, no listar funcionalidades
4. **Interactuar**: Preguntar, no solo mostrar
5. **Valor primero**: Empezar por lo que mas duele al cliente
6. **Preparacion**: 1 hora de preparacion por cada 30 min de demo
7. **Datos listos**: Verificar datos demo ANTES de la llamada
8. **Plan B**: Tener capturas de pantalla por si la conexion falla
