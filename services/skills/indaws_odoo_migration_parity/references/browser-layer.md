# Capa 2b — Validación en navegador (Playwright MCP)

Entre el comportamiento por API (Capa 2a) y el juicio visual humano (Capa 3) hay un hueco:
**lo que solo existe en el cliente web**. Plantillas OWL con `t-inherit`, patches de
componentes, widgets, vistas Studio, wizards y botones que disparan JS.

Un `xpath` de `t-inherit` que no casa **no rompe nada en el servidor**: los tests Python
pasan, el log sale limpio, el pipeline va verde — y la funcionalidad ha desaparecido de la
pantalla. Por API es invisible: el campo sigue en el modelo, la vista sigue en `ir.ui.view`.
Solo se ve usándolo.

Esta capa **no mira píxeles**. Navega el árbol de accesibilidad, que es texto. El aspecto
(maquetación, PDF, tipografía) sigue siendo Capa 3 y sigue siendo humano.

---

## Regla de oro: esto NUNCA es un gate

Una corrida de navegador conducida por un modelo **no es reproducible**. La Capa 2b produce
**hallazgos**, jamás un pass/fail que bloquee una entrega. Los gates duros no cambian:
`-u all` limpio, `checklog-odoo`, y `diff_inventory.py --fail-on-regression`.

Tampoco entra en CI: el gate del repo es `checklog-odoo`, y meter tours ahí los vuelve
flaky. Un hallazgo confirmado se convierte, si acaso, en un test Python barato.

---

## Requisito: el MCP (opcional, degrada solo)

```bash
claude mcp add playwright -- npx @playwright/mcp@latest \
  --headless --isolated \
  --viewport-size 1440x900 \
  --allowed-origins "localhost:8069;127.0.0.1:8069" \
  --output-dir ~/.dynapps/migrations/_browser
```

Requiere Node ≥18. Hay que **reiniciar el proceso `claude`** para que aparezcan las
herramientas `browser_*`; una conversación nueva en el mismo proceso no las ve.

**Si no hay herramientas `browser_*` disponibles, la Capa 2b se salta** con una línea en el
informe (`Capa 2b omitida: Playwright MCP no disponible`). Nunca rompe la validación.

Dos banderas no son cosmética:
- `--allowed-origins` — el entorno local es un **clon de producción**. El gate de
  neutralización cubre el lado servidor (crons, mail, EDI); esto impide que el navegador
  alcance físicamente un dominio del cliente.
- `--isolated` — perfil efímero: no arrastra cookies ni sesiones de nada más.

---

## Qué flujos entran (regla de selección)

**Dirigido por hallazgos, no catálogo fijo.** Si la migración va limpia, esta capa casi no
cuesta; el coste escala con los problemas reales, no con el tamaño del cliente.

Entran, por este orden de prioridad:

1. **DEACTIVATED reactivados** — vistas que el upgrade apagó y se han vuelto a encender de
   forma curada. Es la fuente número uno de daño silencioso: una vista reactivada puede
   duplicar columnas o tumbar el registry.
2. **Vistas Studio** — no tienen código en disco, ningún análisis estático las ve.
3. **Assets portados**: `t-inherit`, patches de componentes, widgets, `*.scss` de tema.
   Búsqueda barata para poblar la lista:
   ```bash
   grep -rln "t-inherit\|patch(\|registry.category(\"fields\")" <addons>/*/static/src
   ```
4. **UNKNOWN de la Capa 1** que sean de interacción, no de metadata.
5. **TODOs del port** (STOP ⑥) que quedaron resueltos "a ojo".
6. **Wizards y botones** de los módulos custom-críticos.

Objetivo de tamaño: **5–15 flujos**. Si la lista supera 15, se recorta por prioridad y el
resto va al checklist humano — no se amplía el presupuesto.

---

## Presupuesto de tokens (obligatorio)

El riesgo real es `browser_snapshot` sobre un formulario de Odoo: vuelca el árbol entero y
se come el contexto en tres pantallas.

| Regla | Por qué |
|---|---|
| **Un** `browser_snapshot` por pantalla, al entrar | Da los refs; después se actúa por ref |
| Aserciones con `browser_verify_text_visible`, `browser_verify_element_visible`, `browser_verify_value`, `browser_verify_list_visible` | Devuelven veredicto, no árbol |
| Evidencia primaria: `browser_console_messages` + `browser_network_requests` | Ahí salen los errores OWL y los `RPC_ERROR`, y son baratos |
| `browser_evaluate` para contar/leer un dato puntual | Más barato que un snapshot entero |
| **Tope duro: ≤ 12 llamadas por flujo** | Excedido → abortar, marcar *no concluyente*, derivar al checklist humano. No se insiste |

Antes de dar por buena la capa en un proyecto nuevo, **medir el consumo real** de un flujo y
ajustar. El tope es una hipótesis hasta que se mide.

---

## Arranque de sesión (una sola vez por corrida)

### 1. Password de admin

En un clon de producción el admin **no** tiene la contraseña `admin`. Antes de la primera
navegación, por `odoo-bin shell` contra la BD de trabajo:

```python
env['res.users'].browse(2).write({'password': 'admin'})
env.cr.commit()
```

Es una BD de trabajo neutralizada; aun así, nunca hacer esto contra nada que no lo sea.

### 2. Login y reutilización del estado

Login una vez → `browser_storage_state` → reutilizar. Así ningún flujo posterior gasta
llamadas en volver a entrar.

```
browser_navigate("http://localhost:8069/odoo")
→ formulario de login → browser_storage_state (guardar)
```

---

## Clasificador de ruido

Un entorno local restaurado ensucia la consola por motivos que **no son el fix**. Estos
patrones se descartan al leer `browser_console_messages` / `browser_network_requests`:

| Patrón | Por qué es ruido |
|---|---|
| `Unexpected token '<'` + `<!doctype` | Fetch interno que devuelve HTML |
| `/web/image/*` con HTTP 500 | **Falta el filestore** — normal en copias locales |
| `Blocked attempt to show a 'beforeunload'` | UX de Chromium |
| 404 de marketplace / themes | Servicio remoto no alcanzable (y `--allowed-origins` lo bloquea a propósito) |

Misma lista que usa el barrido de anchura del migration-kit
(`helpers/validation/playwright_smoke.py`). Se **cita**, no se importa: el kit no se toca.

**Lo que NO es ruido y siempre se reporta:** cualquier `RPC_ERROR` en
`/web/dataset/call_kw`, cualquier `OwlError`, cualquier 5xx que no sea `/web/image`.

---

## Qué se reporta

Por flujo, una entrada corta. Nada de narrativa:

```markdown
### <flujo> — <pantalla o ruta>
**Veredicto:** OK | HALLAZGO | NO CONCLUYENTE
**Esperado:** <lo que la v-origen hacía>
**Observado:** <lo que hace la v-destino>
**Evidencia:** <error de consola / RPC / aserción que falló>
**Llamadas:** <n>/12
```

Un **HALLAZGO** se clasifica con el mismo criterio que el resto de la skill
(`classification.md`): por defecto **regresión a corregir**, no "cambio de versión
aceptado". Desviarse del 1:1 es decisión de negocio explícita.

## Destinos de un hallazgo confirmado — elegir uno

1. **Fix + test Python barato.** Lo que haya que dejar cubierto en el repo se cubre con el
   test más tonto que sirva (p. ej. que los assets estén en el bundle). **Nunca un tour.**
2. **Ítem del checklist humano** (Capa 3) si lo que queda por juzgar es de aspecto.
3. **Ficha en Dynbrain** → `errores-comunes/post-migracion.md`, **solo** si pasa el filtro de
   las tres condiciones (mecanismo + reproducible + genericable). Ver
   `dynapps-migrator/references/dynbrain-feedback-loop.md`. Una regresión de un módulo
   propietario concreto **no entra**: va a la documentación del proyecto.

---

## Límites honestos

- No sustituye al barrido de anchura del kit (que abre todos los menús de forma
  determinista y sin consumir tokens). Son complementarios: aquél da cobertura, éste da
  profundidad sobre lo dudoso.
- No valida maquetación. Si la pregunta es "¿se ve bien?", es Capa 3.
- Un **NO CONCLUYENTE** no es un OK. Se deriva al humano, no se archiva.
