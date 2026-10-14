# Paradas y preguntas al usuario

Contrato de interacción del review. La regla, en una línea:

> **Para** cuando la acción sale fuera o es irreversible · **Pregunta** cuando la respuesta cambia el
> trabajo · **En lo demás, decide y sigue.**

Sin esto la skill se va a uno de los dos extremos: un interrogatorio que nadie usa, o un agente que
mergea solo porque su propio checklist salió verde.

---

## 1. Confirmaciones duras (2) — sin respuesta explícita NO se hace

Las dos son acciones **hacia fuera**, con nombre y apellidos del revisor, y una de ellas irreversible.

### 1.1 Publicar el review en el PR

Antes de escribir un solo comentario en el PR de otra persona: **enseñar el texto completo** que se va
a publicar (bloqueantes, mayores, menores, revisión manual, lo que está bien) y pedir el OK.

Se muestra tal cual va a quedar, no un resumen: el dev va a leer eso, y el tono importa tanto como el
contenido.

### 1.2 Mergear

**Nunca como consecuencia automática de tener los gates en verde.** El veredicto lo calcula la skill;
la decisión de mergear la toma una persona.

Antes de preguntar, poner delante:

- las 10 casillas de la puerta de merge (`decision-and-feedback.md`) con su estado,
- el **radio de explosión** (`impact-checklist.md`): en Core puede ser 136 módulos y **20 proyectos**
  (remedido 2026-08-17; **mídelo, no lo cites** — el comando está en esa reference),
- las referencias exactas revisadas (`Core 18.0@c72eafe`, `PR pr-42@a1b2c3d`),
- los disparadores manuales y **quién** los firmó.

Un merge de Core no es "aprobar un PR": es meter código en **una veintena de instalaciones** —
el número exacto se mide, no se recuerda.

## 2. Parada con registro (1) — firmas de revisión manual

Si el Step 6 dispara, **parada explícita**, no un campo de texto en la plantilla. Hay que listar:

| Disparador | Por qué el test no basta | Quién firma | Estado |
|---|---|---|---|

Y para cada uno, respuesta real: **firmado por quién**, o **pendiente**. Mientras quede uno pendiente,
la confirmación de merge (1.2) no se ofrece siquiera.

El resultado se anota en el **ledger de calibración** de `manual-review-triggers.md` (útil / no útil),
que es lo que con el tiempo sube la precisión de los disparadores. Sin ese apunte, el bucle no existe.

> **El modelo no firma.** Puede constatar que un disparador aplica y a quién le toca; no puede darlo
> por revisado. Rellenar "firmado" sin humano detrás vacía de sentido el Step 6 entero.

## 3. Preguntas (3) — cambian el trabajo, no lo bloquean

### 3.1 Estrategia de datos (Step 0)

BD limpia · dump del cliente neutralizado · clonar una instancia viva. No hay regla automática todavía:
se pregunta con el contexto delante (qué módulos toca el PR, si el comportamiento depende de datos
reales) y **se registra la elección y el porqué** en el veredicto. Ese registro es lo que permitirá
automatizarlo cuando haya casos suficientes.

### 3.2 Modo, solo si es ambiguo (Step 0)

Sin MD se va a **modo bug-fix sin preguntar** — eso está resuelto y no debe parar. La pregunta es solo
para el caso dudoso: el PR parece una tarea grande (muchos módulos, modelos nuevos, migraciones) y no
hay contrato. Se pregunta **con los números delante** (N ficheros, N módulos, ¿hay `migrations/`?), no
en abstracto.

### 3.3 Destruir o mantener la instancia (Step 7)

Por defecto se destruye. Pero si el veredicto es **correcciones**, es probable que quieras conservarla
para re-verificar cuando el dev suba el arreglo. Se pregunta, y si se mantiene, se dice **el comando
exacto** para matarla después:

```bash
destroy.sh <proyecto> --yes
git -C <repo> worktree remove .worktrees/pr-<n> && git -C <repo> branch -D pr-<n>
```

## 4. Lo que NO para nunca

Para que esto no se convierta en un interrogatorio:

- **Falta el MD** → modo bug-fix, sin preguntar (`GOTCHA 1`).
- **Findings intermedios** → se acumulan, se deciden al final (Step 7).
- **Montar el docker, restaurar, neutralizar, ejecutar tests** → son mecánica, no decisiones.
- **Que un gate salga en rojo** → no se pregunta si seguir: se sigue recogiendo evidencia y se decide
  al final. Un review a medias no sirve de feedback.

## 5. Sin terminal interactiva (CI, otra skill llamando)

Mismo criterio que en `docker-deployment`: **jamás dar por contestada una pregunta**.

- No se publica en el PR, no se mergea, no se firma nada.
- El review se completa igual y **el veredicto se deja en fichero**, con las preguntas pendientes
  marcadas explícitamente.
- La instancia se queda **viva** (no se destruye a ciegas) y se imprime el comando de teardown.
