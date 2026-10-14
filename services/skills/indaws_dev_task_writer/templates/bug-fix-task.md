# Bug Fix — {Titulo conciso del bug}

**Modulo**: `{modulo}` | **Version Odoo**: {X.0} | **Asignado**: {dev}
**Nivel**: Junior/Mid/Senior | **Severidad**: Bloqueante/Alta/Media/Baja
**Estimacion**: {h} (max {max}h — si supera, escalar)

## Contexto de Negocio

{Que usuario esta afectado, en que operacion, con que frecuencia.
Si bloquea operacion → flagged Severidad Bloqueante.}

Patron: "El {usuario} no puede {accion} porque {sintoma}. Esto le impide {consecuencia de negocio}."

## Comportamiento Actual (lo que pasa)

- Sintoma observado: {descripcion del bug}
- Mensaje de error (si hay): {error literal}
- Modulos / vista / accion donde ocurre: {ubicacion}

## Comportamiento Esperado (lo que deberia pasar)

{Comportamiento correcto que el usuario espera. Concreto y verificable.}

## Pasos para Reproducir

1. {Login como rol {X}}
2. {Ir a {menu/vista/url}}
3. {Hacer {accion concreta con valores reales}}
4. {Observar: {sintoma del bug}}

**Datos de prueba**: {registros concretos donde se reproduce — IDs, names}

## Donde buscar la causa

{Pistas iniciales — modulo/modelo/metodo sospechoso. NO dar la solucion.}

- Modulo sospechoso: `{modulo}`
- Modelo / vista probable: `{model.name}` o `{view_xml_id}`
- Logs relevantes: {ruta o filtro grep}
- Cambio reciente que pudo introducirlo: {commit/PR/version}

## Criterios de Aceptacion

- [ ] El comportamiento esperado se cumple en los pasos de reproduccion
- [ ] No se introducen regresiones en {flujos relacionados}
- [ ] Test de regresion cubre el caso (evita reaparicion)
- [ ] Code review aprobado

## Impacto Odoo (si el fix lo toca)

> Solo lo que APLICA (`references/odoo-impact-checklist.md`). Un fix "pequeño" que toca un método
> core, una record rule o un campo con datos existentes NO es pequeño.

- {p.ej. **Upgrade-safety**: el override debe mantener `super()`, no copiar el método core}
- {— o "el fix es local, sin impacto transversal" —}

## Restricciones

- {Restricciones tecnicas + POR QUE}
- Si el fix requiere cambio de schema → NO hacerlo: escalar al redactor para evaluar migracion

## Logs / Capturas / Tickets

- Ticket / Issue: {referencia}
- Logs: {**adjunto a la tarea** — nunca una ruta de tu máquina}
- Captura del bug: {adjunto}

## Ayuda

- Timer: pide ayuda si llevas {Junior: 1h | Mid: 3h | Senior: medio dia} sin localizar la causa
- Contacto: {redactor} por {canal}

## Entregables

1. [ ] Rama `fix/{nombre-corto}`
2. [ ] Test de regresion
3. [ ] PR con descripcion del bug, causa raiz y fix
4. [ ] Verificacion en staging con el caso original
