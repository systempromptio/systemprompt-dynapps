# Template: Email de Seguimiento / Recordatorio

## Estructura del email

### Asunto
```
[{nombre_proyecto}] Recordatorio: {titulo_peticion}
```

### Cuerpo

```html
<p>Hola {nombre_contacto},</p>

<p>Os escribo para hacer seguimiento de una peticion que os enviamos el {fecha_envio}:</p>

<p><strong>{titulo_peticion}</strong></p>

<p>{resumen_breve_peticion}</p>

<p>{parrafo_contexto}</p>

<p>Si ya lo teneis en marcha o necesitais mas tiempo, no hay problema, simplemente hacednos saber el estado para que podamos planificar en consecuencia.</p>

<p>Gracias,</p>
<p>{nombre_consultor}<br/>
Equipo de Implantacion Indaws</p>
```

## Parrafos de contexto segun situacion

### Primer recordatorio (1-5 dias de retraso)
```html
<p>La fecha que habiamos estimado para este punto era el {deadline_formateado}.
Entendemos que puede haber habido otras prioridades, pero nos seria de gran ayuda
recibirlo lo antes posible para mantener el ritmo del proyecto.</p>
```

### Segundo recordatorio (6-14 dias de retraso)
```html
<p>Han pasado {dias_retraso} dias desde la fecha prevista para este punto.
Este tema es importante para poder avanzar con {area_afectada}, y el retraso
podria afectar al calendario general del proyecto.</p>

<p>Si hay algun impedimento o necesitais ayuda para preparar esta informacion,
estamos a vuestra disposicion para buscar una solucion juntos.</p>
```

### Recordatorio critico (>14 dias de retraso)
```html
<p>Llevamos {dias_retraso} dias esperando este punto, que es clave para el avance
del proyecto. En este momento, esta pendencia esta afectando directamente a nuestra
capacidad de avanzar con {area_afectada}.</p>

<p>Os proponemos una de estas opciones:</p>
<ul>
  <li>Una llamada rapida ({duracion}) para resolver dudas y desbloquear</li>
  <li>Si el formato solicitado es complicado, podemos simplificarlo</li>
  <li>Si necesitais mas tiempo, acordar una nueva fecha realista</li>
</ul>
```

### Peticion bloqueante
```html
<p>Queremos recordaros que este punto es critico para continuar con la implantacion.
Actualmente no podemos avanzar con {area_afectada} hasta que lo recibamos.</p>
```

## Reglas de tono

- **Primer recordatorio**: Amable y comprensivo, sin presion
- **Segundo recordatorio**: Firme pero constructivo, ofrece ayuda
- **Critico**: Directo sobre el impacto, siempre ofreciendo soluciones
- **NUNCA** culpar al cliente ni usar tono acusatorio
- **NUNCA** amenazar con consecuencias (retrasos, sobrecostes)
- **SIEMPRE** ofrecer alternativas o ayuda para desbloquear
- **SIEMPRE** dejar la puerta abierta a que el cliente explique la situacion

## Reglas estrictas

- **NUNCA** mencionar horas, costes o informacion interna
- **NUNCA** enviar mas de un recordatorio por semana para la misma peticion
- **SIEMPRE** adaptar la intensidad del mensaje a los dias de retraso
- **SIEMPRE** referenciar la peticion original (fecha de envio, contenido)
