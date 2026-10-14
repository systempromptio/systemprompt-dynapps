# Plan de Rollback — {Cliente} v{origen} → v{destino}

## Datos del proyecto

- **Proyecto:** {nombre}
- **Fecha Go-Live prevista:** {fecha}
- **Ventana de mantenimiento:** {horas}h ({hora_inicio} — {hora_fin})

---

## Equipo de guardia

| Rol | Nombre | Teléfono | Disponibilidad confirmada |
|-----|--------|----------|--------------------------|
| TL | | | [ ] |
| Dev | | | [ ] |
| BC | | | [ ] |
| PM | | | [ ] |

---

## Tiempos medidos (del rehearsal)

| Paso | Duración |
|------|----------|
| Backup completo | |
| Restauración BD | |
| Restauración filestore | |
| Verificación | |
| Reapertura accesos | |
| **TOTAL** | |

---

## Punto de no retorno

- **Hora límite para decidir rollback:** {hora} (= hora_fin_ventana − tiempo_total_rollback)
- Si a las {hora} no se ha completado el go-live exitosamente → **ROLLBACK automático**

---

## Precondiciones verificadas

- [ ] Backup validado y accesible
- [ ] Restauración probada en rehearsal
- [ ] Acceso SSH/admin confirmado
- [ ] Canal de emergencia establecido
- [ ] Equipo de guardia confirmado
- [ ] Cliente informado de la ventana

---

## Canales de comunicación

- **Equipo interno:** {canal_slack/teams}
- **Cliente:** {contacto_BC}
- **Escalación COO/CTO:** {contacto}

---

## Firmado

- **TL:** _______________ Fecha: ___/___
- **Dev:** _______________ Fecha: ___/___
