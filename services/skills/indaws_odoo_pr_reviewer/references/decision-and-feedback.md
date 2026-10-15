# Decisión y Feedback (Step 7)

Salida binaria: **merge** o **correcciones**. Sin punto medio ("casi", "lo apruebo si luego...").

---

## Puerta de merge (TODO verde o no pasa)

Merge **solo si se cumplen todas**:

- [ ] Módulo **instala/actualiza en Docker limpio** (versión objetivo + dump si aplica).
- [ ] **Modo contrato:** contrato intacto (firmas/campos `# CONTRACT`, fronteras, deps, integración).
      **Modo bug-fix:** fix **source-verified** contra la fuente Odoo + sin regresión en otros caminos.
- [ ] Criterios cubiertos con tests unit + e2e (modo contrato) / **test de regresión con falsación
      superada** — revertir el fix lo hace fallar (modo bug-fix; ver `test-verification.md` §4.1).
- [ ] Tests en **verde real ejecutados en el Docker**, con output capturado — no CI declarado.
- [ ] Impacto Odoo que aplicaba, **resuelto y comprobado en el Docker** (no solo mencionado).
- [ ] **Disparadores de revisión manual** (`manual-review-triggers.md`) que apliquen, **firmados por
      el humano correspondiente** (contable/seguridad/arquitecto). Verde automático NO los cubre.
- [ ] `/review` y `quality-gate` sin bloqueantes.
- [ ] **Referencias anotadas**: SHA del PR y de cada repo montado (`Core 18.0@abc1234`) — sin esto el
      review no es reproducible.
- [ ] Instancia de review **destruida** tras verificar: contenedores, volúmenes, carpeta, slot del
      registro y worktree del PR. Un `down` no basta.

Si falta una sola → **correcciones**. No se negocia el DoD.

## Clasificar el fallo: ¿del dev o del contrato?

- **Del dev**: no cubrió un criterio, tests en rojo, cambió una firma sin motivo, dep de más, no
  resolvió un impacto. → Correcciones al dev.
- **Del contrato**: el diseño era inviable/ambiguo, el mecanismo central no era el correcto, faltaba
  un criterio que ahora se ve necesario. → **Escalar a `odoo-solution-architect`**; NO cargárselo al
  dev como si fuera su error. El arquitecto actualiza el MD y, si toca, re-estima.

## Feedback accionable (si hay correcciones)

Priorizado, con el POR QUÉ y la cláusula que incumple. Nada de "esto está mal" a secas.

- **Bloqueantes** (impiden merge): criterio sin test, test en rojo, contrato roto, dep no acordada.
- **Mayores** (hay que arreglar): impacto no resuelto, e2e débil, integración frágil.
- **Menores** (mejora): estilo, nombres, micro-optimización — **opcionales**, no bloquean.

Cada finding: `qué incumple` → `qué criterio/contrato del MD` (o, en bug-fix, `qué patrón de la fuente
Odoo / qué caso sin cubrir`) → `qué se espera`. Reconocer también lo que está bien (review educativo,
no solo caza de fallos).

## Constancia

- Publicar la decisión **en el PR** (delegar a `git-review-pr`): aprobado+merge, o review con findings.
- Actualizar la `project.task` (delegar a `odoo-pilot`/MCP): estado y una línea de resultado.
- Cerrar el bucle: el dev sabe exactamente qué falta o que ya está mergeado.
