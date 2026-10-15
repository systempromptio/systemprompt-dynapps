# Capa de anotaciones para desarrollo

Lo que separa un mockup bonito de un **entregable**: cada comportamiento no obvio lleva un círculo
numerado que enlaza a un panel al pie con la nota de implementación. El panel se puede ocultar (toggle)
para enseñárselo al cliente sin ruido técnico.

## Patrón (HTML/CSS/JS)

Marcador inline junto al elemento anotado:
```html
Kilómetros<span class="note-badge" title="nota 2">2</span>
```

Toolbar con el toggle + panel de notas al pie:
```html
<div class="toolbar">
  <div class="toggle" onclick="document.body.classList.toggle('hide-notes')">
    <span class="switch"></span><span>Anotaciones para desarrollo</span></div>
  <span class="legend">Los círculos azules marcan comportamientos a implementar.</span></div>

<div class="devpanel">
  <h3>■ ANOTACIONES PARA DESARROLLO — <Pantalla></h3>
  <ol>
    <li><span class="b">Título nota (Bloque X):</span> descripción. Modelo/campo <code>partner_id.credit</code>.
        <b>[ESTÁNDAR|STUDIO|DESARROLLO]</b>. MVP vs Fase 2 si aplica.</li>
  </ol>
</div>
<div class="disc">Mockup de referencia — no funcional. Proyecto NNNN · Cliente · sobre <code>modelo</code> de Odoo VV.</div>
<!-- Made with odoo-mockup-generator · v0.1.0 -->
```
```css
.note-badge{display:inline-flex;align-items:center;justify-content:center;width:18px;height:18px;border-radius:50%;background:#0b5cff;color:#fff;font-size:11px;font-weight:700;margin-left:6px;cursor:help;vertical-align:middle}
body.hide-notes .note-badge,body.hide-notes .devpanel{display:none}
.toolbar{max-width:1000px;margin:14px auto 0;display:flex;gap:12px;align-items:center}
.toggle{display:flex;align-items:center;gap:7px;font-size:12.5px;color:#6b7280;cursor:pointer;user-select:none}
.switch{width:36px;height:20px;background:#0b5cff;border-radius:11px;position:relative}
.switch:after{content:"";position:absolute;width:16px;height:16px;background:#fff;border-radius:50%;top:2px;right:2px;transition:.2s}
body.hide-notes .switch{background:#c3c8d0}body.hide-notes .switch:after{right:18px}
.devpanel{max-width:1000px;margin:0 auto 18px;background:#0b1f3a;color:#dde6f5;border-radius:8px;padding:16px 20px;font-size:12.5px;line-height:1.7}
.devpanel h3{margin:0 0 8px;color:#8fb4ff;font-size:13px}.devpanel code{background:#16305a;padding:1px 6px;border-radius:4px;color:#bcd3ff}.devpanel .b{color:#ffd479;font-weight:700}
.disc{max-width:1000px;margin:10px auto 0;font-size:11px;color:#9aa0a6;text-align:center}
```

## Cómo redactar una nota (obligatorio en cada una)
1. **A qué apunta**: el elemento del mockup (campo, botón, columna, estado).
2. **Constructo real**: modelo/campo/widget de Odoo — `repair.diagnosis`, `partner_id.credit`, `state` computed, `<chatter/>`, QWeb, `TransientModel`. Nombres reales, no inventados (verifícalos contra fuente / `odoo-version-api-reference`).
3. **Clasificación**: **ESTÁNDAR / STUDIO / DESARROLLO** (usa `view-capabilities-and-limits.md`).
4. **Límite o incógnita** si la hay: "Odoo no agrupa o2m nativamente → dev", "pendiente confirmar con cliente si…".
5. **Alcance**: MVP vs Fase 2 cuando el esfuerzo lo justifique.
6. **Reutilización**: si ya existe algo (módulo estándar, Studio previo, campo en staging), dilo — "reutilizar `x_studio_...`, no crear un segundo apunte".

## Buenas notas (ejemplos reales, estilo Autosur)
- *«Estado de la OR = agregado de los diagnósticos: el `state` operativo vive en `repair.diagnosis`; la OR no
  tiene etapa editable, se **deriva** (campo computed) → DESARROLLO. Pendiente con Celia: ¿ajustar `workshop_stage`
  o convertirlo en calculado?»*
- *«Fichajes `readonly` para el operario; solo el Jefe de Taller traspasa entre diagnósticos **sin modificar fechas**
  (crítico para auditoría de garantía) → DESARROLLO.»*
- *«Agrupar Piezas/Fichajes por diagnóstico con subtotales, como secciones de un pedido → **no es nativo** en una
  o2m cualquiera → DESARROLLO.»*

## Anti-patrones
- Nota vaga ("mejorar esto") sin modelo/campo ni clasificación → inútil para el dev.
- Pintar algo como estándar cuando es dev (rompe el presupuesto de esfuerzo y la confianza).
- Inventar nombres de modelo/campo. Si no lo sabes con certeza, márcalo como "a confirmar".
