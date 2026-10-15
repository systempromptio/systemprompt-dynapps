# Contexto de origen y acceso al codigo

Que exigir cuando la peticion llega desde un registro de Odoo, tipicamente via
`dev-assessment`. Existe porque el sintoma real fue: **el dev recibia la tarea sin contexto,
sin los adjuntos y sin forma de bajarse el codigo.**

## El paquete de traspaso (6 puntos)

`dev-assessment` lee los adjuntos para clasificar. Esa lectura tiene que cruzar hasta aqui:

1. **Clasificacion ya resuelta** — no se re-clasifica (romperia el ciclo con dev-assessment).
2. **Registro origen**: modelo + ID + URL (`{base}/odoo/{model}/{id}`).
3. **Adjuntos**: `ir.attachment` con `id`, `name` y una linea de que aporta cada uno. El que
   cambio el veredicto va marcado **DECISIVO**.
4. **Acceso al codigo**: repo, rama, version del modulo y la via de descarga que funciono.
5. **Zona de campo afectada** y que modulo la posee hoy (si hubo radar de colisiones).
6. **Identificadores exactos** de lo que hay que tocar (xmlid, modelo, campo).

**Si falta el enlace al origen o la lista de adjuntos, el intake esta incompleto:** devolver,
no adivinar. Un dev que tiene que volver a pedir el contexto es una tarea mal redactada.

No aplica al **fast-path trivial** (Step 0.5): si el trabajo es un campo o una vista de ≤1h,
no ha pasado por assessment y no hay paquete que exigir.

## Adjuntos: cuales se re-adjuntan y cuales se enlazan

- **DECISIVOS** → se re-adjuntan al `project.task` creado (`ir.attachment` con
  `res_model='project.task'`, `res_id={id}`). El dev los necesita donde trabaja.
- **El resto** → se enlazan por URL al registro origen desde la seccion "Contexto de origen"
  de la plantilla. No se duplica lo accesorio.
- **⚠ Limite duro:** los adjuntos grandes **no pasan por `odoo-pilot`** (el JSON viaja como
  argumento de shell; Linux corta en 128 KB, o sea ~90 KB en base64). **Un DECISIVO que pase
  de ~90 KB se enlaza, no se re-adjunta** — y se dice explicitamente en la tarea que el dev
  tiene que abrirlo en el registro origen. Ver GOTCHA 5.

## Acceso al codigo: las tres vias

En la tarea van **siempre las tres**, para que el dev use la que tenga disponible:

```
gh repo clone indawsodoo/{repo} -- --depth=1
git clone --depth=1 git@github.com:indawsodoo/{repo}.git
https://github.com/indawsodoo/{repo}/archive/refs/heads/{rama}.zip
```

**`gh repo clone` va por HTTPS con el token de `gh`** y funciona sin clave SSH. La segunda
via (`git clone git@github.com:...`) SI la necesita, y sin ella el error de permisos no
menciona SSH y despista. Por eso las tres vias van siempre: el dev usa la que tenga.

Junto a las vias van **repo, rama, version del modulo** y **la pieza exacta a extender**
(`{modulo}` → `{modelo / campo / xmlid}`), con una linea de por que esa y no otra.
