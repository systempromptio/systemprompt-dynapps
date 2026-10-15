# Horas de Migracion — volumen, iteraciones y salto de version

Dimensionamiento de una migracion de version por **volumen de datos y numero de iteraciones**, no
por numero de componentes. Fuente unica: no duplicar estas formulas en ningun otro sitio.

**Son horas base, sin asistencia IA.** El factor de `ai-factors.md` se aplica despues y **antes**
de la contingencia.

Lo que se estima por componente —modulos a portar, vistas, informes— va en
`hours-per-component.md`. Esto es la otra mitad.

---

## 1. Base de datos por volumen

```
BD <= 10 GB  →  30h
BD >  10 GB  →  30 + (ceil((GB - 10) / 5) × 8) h
```

| Tamano BD | Horas base | Calculo |
|---|---|---|
| 5 GB | 30h | ≤ 10 GB |
| 10 GB | 30h | ≤ 10 GB |
| 15 GB | 38h | 30 + (ceil(5/5) × 8) |
| 25 GB | 54h | 30 + (ceil(15/5) × 8) |
| 50 GB | 94h | 30 + (ceil(40/5) × 8) |

---

## 2. Iteraciones por tamano de proyecto

| Tamano | Iteraciones | Horas extra |
|---|---|---|
| Pequeno | 2 (Test, Produccion) | +0h |
| Mediano | 3 (Test1, Ensayo, Produccion) | +10h |
| Grande | 4 (Test1, Test2, Ensayo, Produccion) | +20h |

Las iteraciones de prueba **no llevan factor IA**: son pasos de verificacion que se anaden, no
trabajo mecanico que se acelere. Reducirlas es lo que dispara los errores post-migracion.

---

## 3. Salto de version

Multiplicador sobre las horas base, **antes** de anadir iteraciones.

| Salto | Multiplicador |
|---|---|
| 1 version (v17 → v18) | ×1.0 |
| 2 versiones (v16 → v18) | ×1.1 |
| 3+ versiones (v14 → v18) | ×1.2 |

---

## 4. Formula completa

```
horas_bd = ceil( (base × salto_version + iteraciones) × factor_IA × (1 + contingencia) )
```

Orden, y no es intercambiable:

1. **base** segun volumen (§1)
2. **× salto de version** (§3)
3. **+ iteraciones** (§2)
4. **× factor IA** de `ai-factors.md` — migracion de base de datos
5. **× (1 + contingencia)** de `risk-contingency.md`, redondeo `ceil`

### Ejemplo — BD de 50 GB, proyecto mediano, v16 → v18

| Concepto | Calculo | Resultado |
|---|---|---|
| Base | 30 + (ceil(40/5) × 8) | 94h |
| Salto de version | 94 × 1.1 | 103.4h |
| Iteraciones (mediano) | 103.4 + 10 | 113.4h |
| Factor IA (BD) | 113.4 × 0.50 | 56.7h |
| Contingencia (+18%) | `ceil(56.7 × 1.18)` | **67h** |

A ese subtotal se le suma la **estabilizacion post-go-live** (§5) calculada sobre el total
tecnico del proyecto, no solo sobre la BD.

---

## 5. Estabilizacion post-migracion

Los errores que aparecen despues del go-live son **el 11% del esfuerzo real de una migracion**
(era el 16% antes del skill de migracion). Es mas que toda la base de datos junta, y hasta ahora
no existia como partida: aparecia despues, con el presupuesto ya cerrado.

| Partida | Horas |
|---|---|
| Estabilizacion post-go-live | **10-15% del total tecnico** (codigo + BD) |

**No lleva factor IA**: es diagnostico y correccion sobre un sistema en produccion, trabajo
humano. Lo que la reduce no es la IA sino las iteraciones de prueba (§2): con ellas los errores
pasan de bolsas indefinidas a incidencias acotadas.

Se presupuesta como linea visible, no como contingencia: la contingencia cubre lo imprevisto y
esto es previsible.

---

## 6. Que NO se estima aqui

- **Modulos a portar** → `hours-per-component.md`, seccion de migracion de version.
- **Migracion de datos** (mapeos, transformaciones, binarios) → es trabajo por componente, no por
  volumen de BD.
- **Testing funcional y formacion** → no llevan factor y se estiman como el resto del proyecto.
