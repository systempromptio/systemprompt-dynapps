# JavaScript Patterns — odoo.com Portal

Snippets probados en producción para interactuar con el portal de partner de odoo.com.
Todos los snippets asumen que la página ya está cargada (no combinar con navigate).

---

## waitForElement — Espera dinámica con MutationObserver

Reemplaza los `setTimeout` fijos. Detecta cuándo un elemento aparece en el DOM sin bloquear innecesariamente.

```javascript
function waitForElement(selector, timeoutMs = 3000) {
  return new Promise((resolve, reject) => {
    const el = document.querySelector(selector);
    if (el) return resolve(el);
    const observer = new MutationObserver(() => {
      const el = document.querySelector(selector);
      if (el) { observer.disconnect(); resolve(el); }
    });
    observer.observe(document.body, { childList: true, subtree: true });
    setTimeout(() => { observer.disconnect(); reject(new Error(`Timeout esperando ${selector}`)); }, timeoutMs);
  });
}

// Uso:
await waitForElement('form [name="contact_name"]', 3000);
// Continúa inmediatamente cuando aparece, o lanza error tras 3s
```

**Por qué usar esto en vez de setTimeout(700):**
- Si el modal abre en 200ms, continúa al instante (ahorra 500ms por operación)
- Si la red va lenta y tarda 2s, lo espera sin fallar
- Timeout claro con mensaje de error descriptivo

---

## setVal — Función helper para campos reactivos Odoo

```javascript
function setVal(el, val) {
  el.focus();
  el.value = val;
  ['input', 'change'].forEach(e => el.dispatchEvent(new Event(e, {bubbles: true})));
  el.blur();
}
```

Necesaria para todos los campos. Sin los eventos `input` y `change`, Odoo no detecta el cambio.

---

## Crear oportunidad (modal)

```javascript
async function crearOportunidad(contactName, title, description) {
  const btn = [...document.querySelectorAll('button')].find(b =>
    b.textContent.includes('Crear oportunidad') ||
    b.textContent.includes('Añadir') ||
    b.textContent.includes('Nueva oportunidad')
  );
  if (!btn) throw new Error('Botón crear no encontrado');
  btn.click();

  // Esperar renderizado del modal — ESPERA DINÁMICA
  try {
    await waitForElement('form [name="contact_name"]', 3000);
  } catch (e) {
    throw new Error('Modal no apareció tras 3s: ' + e.message);
  }

  function setVal(el, val) {
    el.focus(); el.value = val;
    ['input','change'].forEach(e => el.dispatchEvent(new Event(e, {bubbles:true})));
    el.blur();
  }

  const oppForm = [...document.querySelectorAll('form')]
    .find(f => f.querySelector('[name="contact_name"]'));
  if (!oppForm) throw new Error('Modal form no encontrado');

  setVal(oppForm.querySelector('[name="contact_name"]'), contactName);
  setVal(oppForm.querySelector('[name="title"]'), title);
  setVal(oppForm.querySelector('[name="description"]'), description);

  await new Promise(r => setTimeout(r, 200));
  const confirmBtn = oppForm.querySelector('.new_opp_confirm');
  if (!confirmBtn) throw new Error('Botón .new_opp_confirm no encontrado');
  confirmBtn.click();
  return 'creado';
}

// Llamada:
await crearOportunidad('Juan García', 'Implantación Odoo - Empresa XYZ', 'Descripción detallada...');
```

---

## Actualizar datos de contacto (email, teléfono, empresa)

En la página de detalle `/my/opportunity/{slug}`. Formulario con clase `.edit_contact_confirm`.
Campos disponibles (verificados 30/06/2026): `partner_name`, `phone`, `email_from`, `street`, `street2`,
`city`, `state_id`, `zip`, `country_id`. **Este formulario RECARGA la página al enviar**, así que cualquier
otra escritura en el detalle debe hacerse antes (o re-navegar después).

```javascript
async function actualizarContacto(phone, email, partnerName) {
  function setVal(el, val) {
    el.focus(); el.value = val;
    ['input','change'].forEach(e => el.dispatchEvent(new Event(e, {bubbles:true})));
    el.blur();
  }

  // Esperar formulario de contacto disponible
  await waitForElement('.edit_contact_confirm', 3000);
  const contactForm = [...document.querySelectorAll('form')]
    .find(f => f.querySelector('.edit_contact_confirm'));
  if (!contactForm) throw new Error('Formulario contacto no encontrado');

  const phoneEl = contactForm.querySelector('[name="phone"]');
  const emailEl = contactForm.querySelector('[name="email_from"]');
  const partnerEl = contactForm.querySelector('[name="partner_name"]');

  if (phoneEl && phone) setVal(phoneEl, phone);
  if (emailEl && email) setVal(emailEl, email);
  if (partnerEl && partnerName) setVal(partnerEl, partnerName);

  await new Promise(r => setTimeout(r, 200));
  contactForm.querySelector('.edit_contact_confirm').click();
  return 'contacto actualizado';
}

// Llamada:
await actualizarContacto('+34 612 345 678', 'contacto@empresa.com', 'Empresa XYZ S.L.');
```

---

## Actualizar ingresos y probabilidad de la oportunidad

Formulario con clase `.edit_opp_confirm`. **NO contiene campo `description`** (verificado en producción
30/06/2026): solo se pueden refrescar `expected_revenue` y `probability`. La descripción solo se setea al
crear la oportunidad (modal). Ver SKILL.md → GOTCHA 6.

```javascript
async function actualizarOportunidad(expectedRevenue, probability) {
  function setVal(el, val) {
    el.focus(); el.value = val;
    ['input','change'].forEach(e => el.dispatchEvent(new Event(e, {bubbles:true})));
    el.blur();
  }

  const oppForm = [...document.querySelectorAll('form')]
    .find(f => f.querySelector('.edit_opp_confirm'));
  if (!oppForm) throw new Error('Formulario oportunidad no encontrado');

  const revenueEl = oppForm.querySelector('[name="expected_revenue"]');
  const probEl = oppForm.querySelector('[name="probability"]');

  if (revenueEl && expectedRevenue != null) setVal(revenueEl, String(expectedRevenue));
  if (probEl && probability != null) setVal(probEl, String(probability));

  await new Promise(r => setTimeout(r, 300));
  oppForm.querySelector('.edit_opp_confirm').click();
  return 'oportunidad actualizada';
}
```

---

## Verificar valor guardado en un campo input

`get_page_text` no muestra valores de `<input>`. Para verificar:

```javascript
// Verificar email (form de contacto)
document.querySelector('[name="email_from"]').value

// Verificar teléfono (form de contacto)
document.querySelector('[name="phone"]').value

// Verificar ingresos esperados (form de oportunidad .edit_opp_confirm)
document.querySelector('[name="expected_revenue"]').value

// Verificar probabilidad (form de oportunidad .edit_opp_confirm)
document.querySelector('[name="probability"]').value
```

> El detalle del portal NO tiene campo `description` (ver GOTCHA 6): no intentes verificar `[name="description"]`
> ahí. La descripción solo existe en el modal de creación.

---

## Verificar sesión activa

```javascript
// Devuelve true si hay sesión iniciada
document.body.innerHTML.includes('Cerrar sesión') || 
document.body.innerHTML.includes('juliop24')
```

---

## Listar URLs de todas las oportunidades en /my/opportunities

```javascript
[...document.querySelectorAll('a[href*="/my/opportunity/"]')]
  .map(a => ({text: a.textContent.trim(), href: a.href}))
```
