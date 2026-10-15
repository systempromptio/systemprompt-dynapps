# Workflow detallado: hr-leave-attendance

**CONFIDENTIAL - HR TEAM ONLY** · **Document ID:** HR-SKILL-005

Comandos y fórmulas verbosos extraídos del SKILL.md. El SKILL.md mantiene los pasos
resumidos; aquí está el detalle ejecutable.

---

## Prerequisitos

- **HiBob API** (primaria): `HIBOB_API_URL`, `HIBOB_API_TOKEN`, `HIBOB_SERVICE_USER`
- **Odoo XML-RPC** (secundaria): `ODOO_URL`, `ODOO_DB`, `ODOO_USERNAME`, `ODOO_PASSWORD`
- **Python 3** para extracción, cálculo y detección de anomalías
- **Librería compartida**: `lib/` (hibob_client, odoo_hr_client, config, report_utils)

Los scripts resuelven la raíz de la suite con `REPO_ROOT = Path(__file__).resolve().parents[3]`
y hacen `sys.path.insert(0, str(REPO_ROOT))`. Son portables: ejecútalos desde la raíz de la
suite o desde cualquier ruta sin path absoluto de máquina.

---

## Paso 0 — Recopilación de información (DETENTE)

Preguntar al usuario:

1. **Tipo de consulta** (obligatorio): resumen mensual de ausencias · saldos de vacaciones ·
   who-is-off (hoy / semana / rango) · anomalías de asistencia · Bradford Factor ·
   cumplimiento de políticas · informe completo.
2. **Periodo** (obligatorio): mes/año (mensual y anomalías) · fecha o rango (who-is-off) ·
   últimos 12 meses (Bradford Factor, automático).
3. **Alcance** (opcional, default toda la empresa): departamento · empleado concreto
   (solo saldos, si el solicitante es su manager o RRHH) · ubicación.
4. **Credenciales** si no están configuradas.

Esperar respuesta antes de continuar.

---

## Paso 1 — Extracción de datos

Ejecutar desde la raíz de la suite (`cd` a la raíz del repo, no a una ruta de máquina).

**Ausencias y saldos:**
```bash
python3 skills/hr-leave-attendance/scripts/fetch_leave_data.py --output leave_data.json --days 90
```
Extrae: HiBob TimeOff requests, balances, policies, Who's Out; Odoo hr.leave (complementario).

**Asistencia y anomalías:**
```bash
python3 skills/hr-leave-attendance/scripts/fetch_attendance_data.py --output attendance_data.json --days 30
```
Extrae: HiBob Attendance; Odoo hr.attendance.

**Verificar:** HiBob devuelve datos (0 solicitudes puede ser periodo sin ausencias);
saldos contienen ≥1 política; asistencia tiene registros si el módulo está activo;
si Odoo no responde, continuar solo con HiBob y avisar.

---

## Paso 2 — Cálculo de métricas (OBLIGATORIO, nunca manual)

```bash
python3 skills/hr-leave-attendance/scripts/anomaly_detector.py \
  --leave leave_data.json --attendance attendance_data.json --output analysis.json
```

**NUNCA** calcular métricas a mano. El script calcula:
- Tasa de absentismo global y por departamento
- Bradford Factor por empleado y departamento (últimos 12 meses)
- Anomalías: llegadas tarde (>15 min), salidas anticipadas (>30 min), fichajes omitidos
  (entrada sin salida o viceversa), horas extras excesivas (>10h semanales)
- Cumplimiento de políticas (asuntos propios consumidos vs disponibles)
- Alertas según umbrales de `lib/config.py`

Leer `analysis.json` para continuar.

### Fórmulas y umbrales

- **Bradford Factor**: `BF = S² × D` — S = número de episodios de ausencia, D = días totales
  de ausencia en los últimos 12 meses. Penaliza ausencias frecuentes de corta duración.
- **Umbrales BF** (en `lib/config.py`): `< 200` Normal · `200–500` Warning · `> 500` Crítico.
- **Tasa de absentismo**: días de ausencia / días laborables del periodo × 100, global y por
  departamento.
- **Saldos de vacaciones**: HiBob los devuelve por política y devengo (accrual); no se
  recalculan, se leen tal cual de HiBob.
- **Horas extras**: límite legal España 80 h/mes (Art. 35 ET). Alertar si se supera.

---

## Paso 2.5 — Validación (DETENTE)

Presentar resumen según el tipo de consulta y esperar aprobación antes del informe completo.

**Resumen mensual:**
```
Resumen de Ausencias - {mes} {año}
Total días de ausencia: {total_days} · Tasa de absentismo: {absence_rate}%
Por tipo: Vacaciones {vac} · Asuntos propios {personal} · Otros {other}
Departamentos con mayor absentismo: {dept}: {rate}% ({estado})
Alertas: {alert_count}
```

**Who-is-off:**
```
Ausentes {fecha/rango}:
  {nombre} - {departamento} - {tipo_ausencia} ({inicio} - {fin})
Total: {count} empleados ausentes
```

**Bradford Factor:**
```
Bradford Factor - Top alertas:
  {empleado}: BF = {score} ({estado})
Umbrales: < 200 Normal | 200-500 Warning | > 500 Crítico
```

---

## Paso 3 — Generación del informe

SOLO tras aprobación del Paso 2.5. Plantilla según consulta:

**Informe mensual de ausencias** → `templates/leave_report.md`:
1. Resumen ejecutivo · 2. Ausencias por tipo · 3. Por departamento ·
4. Saldos de vacaciones (agregado por departamento) · 5. Bradford Factor (top 10 y alertas) ·
6. Cumplimiento de políticas · 7. Tendencia (3 meses si hay datos).

**Informe de anomalías de asistencia** → `templates/attendance_report.md`:
1. Resumen de anomalías · 2. Llegadas tarde por departamento · 3. Fichajes omitidos ·
4. Horas extras · 5. Patrones detectados · 6. Recomendaciones.

**Reglas de generación:**
- Saldos individuales solo si el solicitante es manager directo o RRHH.
- Bradford Factor: nombre del empleado solo a RRHH o manager directo.
- Agrupar departamentos con <3 empleados como "Otros" (RGPD).
- Tipo de ausencia: "Vacaciones", "Asuntos propios", "Otros" — nunca "Baja médica" como detalle.
- Horas extras: alertar si superan 80 h/mes.
- Asistencia: mostrar horas trabajadas, nunca motivos de ausencia.
- Fechas en formato DD/MM/YYYY. Datos reales del JSON, nunca inventados.

Guardar como `informe_ausencias_{periodo}.md` o `informe_asistencia_{periodo}.md`.
Los JSON intermedios son temporales: no persistir tras generar el informe.

---

## Estructura de archivos

```
hr-leave-attendance/
├── SKILL.md
├── scripts/
│   ├── fetch_leave_data.py        # Extracción de ausencias (HiBob + Odoo)
│   ├── fetch_attendance_data.py   # Extracción de asistencia (HiBob + Odoo)
│   └── anomaly_detector.py        # Bradford Factor, absentismo, anomalías
├── references/
│   ├── workflow.md                # Este archivo
│   └── leave_policies.md          # Legislación laboral y convenio colectivo
└── templates/
    ├── leave_report.md
    └── attendance_report.md
```

Días de asuntos propios y demás permisos: ver `references/leave_policies.md` (convenio aplicable).
