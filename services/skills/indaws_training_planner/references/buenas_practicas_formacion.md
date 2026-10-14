# Buenas Practicas de Formacion en Implantaciones Odoo

Reglas y metodologia para diseñar sesiones de formacion efectivas. El skill consulta este documento para estructurar las sesiones correctamente.

---

## Principios Fundamentales

### 1. Regla 50/50
Al menos el 50% del tiempo de la sesion debe ser ejercicios practicos. Si la sesion es de 3h, minimo 1.5h son ejercicios donde el usuario opera Odoo. La teoria sola no genera adopcion.

### 2. Regla de los 20 Minutos
Nunca mas de 20 minutos de teoria seguida sin una practica. El cerebro pierde atencion despues de 15-20 minutos de escucha pasiva. Estructura: 15-20min teoria → 15-20min practica → repetir.

### 3. Aprender Haciendo
El orden siempre es: Concepto breve → Demostracion del consultor → Ejercicio guiado (juntos) → Ejercicio autonomo (solos). Nunca al reves.

### 4. Datos Reales, Siempre
NUNCA usar datos genericos ("Cliente 1", "Producto Test"). Usar los datos reales del entorno del cliente: sus productos, sus clientes, sus almacenes. La diferencia en adopcion es brutal. Si no hay datos, generarlos con `odoo-test-data-generator` antes de la sesion.

### 5. Menos es Mas
Un operario no necesita ver 30 menus. Solo los 3-5 que usara cada dia. Mejor dominar 5 acciones que conocer superficialmente 20. Adaptar siempre al perfil.

---

## Estructura de una Sesion

### Sesion Estandar (2-3h)

```
Bloque 1 (45min)
├── Bienvenida y objetivos (5min)
├── Teoria: conceptos clave (15min)
├── Demo del consultor (10min)
└── Ejercicio guiado: todos juntos (15min)

Descanso (10min)

Bloque 2 (45min)
├── Teoria: flujo principal (15min)
├── Demo del consultor (10min)
└── Ejercicio autonomo: cada uno solo (20min)

Descanso (10min)

Bloque 3 (45min)
├── Teoria: funcionalidades avanzadas/custom (10min)
├── Ejercicio completo: escenario real (25min)
└── Resumen, dudas y cheat sheet (10min)

Cierre (15min)
├── Evaluacion rapida (10min)
└── Preguntas finales y proximos pasos (5min)
```

### Sesion Corta (1-1.5h) — Funcionalidad Concreta

```
Bloque Unico (60-90min)
├── Objetivo y contexto (5min)
├── Demo del flujo completo (15min)
├── Ejercicio guiado (20min)
├── Ejercicio autonomo (20min)
└── Dudas, resumen y cheat sheet (10-20min)
```

### Sesion Ejecutiva (1h) — Direccion

```
Bloque Unico (60min)
├── Vision general del sistema (10min)
├── Demo de dashboards y KPIs (15min)
├── Demo de aprobaciones (10min)
├── Ejercicio guiado: consultar informes (15min)
└── Preguntas y accesos (10min)
```

---

## Errores Comunes a Evitar

1. **Sesiones demasiado largas**: Maximo 3-4h con descansos. Mejor 2 sesiones de 2h que 1 de 4h.
2. **Mezclar perfiles**: No juntar operarios con direccion. Tienen necesidades y nivel completamente distintos.
3. **No tener datos preparados**: Si la instancia de prueba esta vacia, los ejercicios no funcionan. Verificar ANTES de la sesion.
4. **Exceso de teoria**: "Ahora os voy a explicar el modelo de datos de Odoo..." — NO. Nadie necesita eso excepto IT.
5. **No dejar tiempo para dudas**: Las preguntas de los usuarios son ORO. Revelan lo que realmente no entienden.
6. **Asumir conocimiento previo**: Siempre preguntar "¿Alguien ha usado un ERP antes?" al inicio.
7. **Ignorar resistencia al cambio**: Algunos usuarios veran Odoo como una amenaza. Mostrar como les facilita la vida, no como les "controla".

---

## Checklist Pre-Sesion

Antes de cada sesion de formacion, el consultor debe verificar:

- [ ] Instancia de prueba operativa y accesible
- [ ] Datos de prueba cargados (productos, clientes, pedidos reales)
- [ ] Usuarios de prueba creados para los asistentes (1 por persona o compartidos)
- [ ] Proyector / pantalla compartida funcionando
- [ ] Lista de asistentes confirmada
- [ ] Materiales impresos o compartidos (guia + cheat sheet)
- [ ] Ejercicios verificados: el consultor los ha ejecutado al menos 1 vez
- [ ] Cronometro / reloj visible para gestionar el tiempo
- [ ] Plan B si algo falla (capturas de pantalla, demo en local)

---

## Evaluacion Post-Sesion

### Formato Recomendado

3-5 preguntas **PRACTICAS** (no teoricas). Ejemplos:

**CORRECTO** (practico):
- "Crea un presupuesto para el cliente X con estos 3 productos y envialo por email"
- "¿Donde consultas el stock disponible de un producto en todos los almacenes?"

**INCORRECTO** (teorico):
- "¿Que es un campo computed en Odoo?"
- "Describe las diferencias entre sale.order y sale.order.line"

### Criterios de Evaluacion

| Resultado | Significado | Accion |
|-----------|-------------|--------|
| 5/5 correcto | Domina el flujo | Listo para produccion |
| 3-4/5 correcto | Necesita practica | Ofrecer sesion de refuerzo o consulta |
| < 3/5 correcto | No esta preparado | Repetir sesion o sesion individual |

---

## Tips para el Consultor

- **Empezar siempre con "¿Para que?"**: Antes de mostrar un menu, explicar por que existe. "Este informe os dice cuanto habeis vendido este mes y quien ha vendido mas."
- **Usar lenguaje del cliente**: Si ellos dicen "albaranes" en vez de "transferencias de entrega", usar "albaranes".
- **Celebrar los aciertos**: Cuando un usuario completa un ejercicio, reconocerlo. Genera confianza.
- **Tomar nota de las dudas**: Las preguntas que surgen en la formacion son futuros tickets de soporte. Documentarlas.
- **No prometer lo que no esta**: Si algo no esta configurado o es un GAP, decirlo y explicar el plan.
