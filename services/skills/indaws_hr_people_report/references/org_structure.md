# Estructura Organizativa - Referencia

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-001-REF-002

---

## Organigrama General de Indaws

```
CEO / Director General
├── Direccion Tecnica
│   ├── Desarrollo (Odoo)
│   │   ├── Backend Developers
│   │   ├── Frontend Developers
│   │   └── QA / Testing
│   ├── Sistemas / DevOps
│   └── Soporte Tecnico
├── Direccion Comercial
│   ├── Ventas (Account Executives)
│   ├── Preventa / Consultoria
│   └── Marketing
├── Direccion de Operaciones
│   ├── Project Management
│   ├── Consultoria Funcional
│   └── Formacion
├── Direccion Financiera
│   ├── Contabilidad
│   ├── Administracion
│   └── Control de Gestion
└── Direccion de Personas (RRHH)
    ├── People Operations
    ├── Talent Acquisition
    └── Learning & Development
```

---

## Mapeo de Departamentos: HiBob <-> Odoo

| HiBob Department | Odoo hr.department | Codigo | Area |
|------------------|--------------------|--------|------|
| Engineering | Desarrollo | DEV | Tecnica |
| Engineering - Backend | Desarrollo Backend | DEV-BE | Tecnica |
| Engineering - Frontend | Desarrollo Frontend | DEV-FE | Tecnica |
| QA | Quality Assurance | QA | Tecnica |
| DevOps | Sistemas | SYS | Tecnica |
| IT Support | Soporte Tecnico | SUP | Tecnica |
| Sales | Ventas | VEN | Comercial |
| Pre-Sales | Preventa | PRE | Comercial |
| Marketing | Marketing | MKT | Comercial |
| Operations | Operaciones | OPS | Operaciones |
| Project Management | Gestion de Proyectos | PM | Operaciones |
| Consulting | Consultoria Funcional | CON | Operaciones |
| Training | Formacion | FOR | Operaciones |
| Finance | Finanzas | FIN | Financiera |
| Accounting | Contabilidad | ACC | Financiera |
| Administration | Administracion | ADM | Financiera |
| HR / People | Recursos Humanos | HR | Personas |
| Talent Acquisition | Seleccion | TA | Personas |
| L&D | Formacion y Desarrollo | LD | Personas |

---

## Ubicaciones / Sedes

| Sede | Ciudad | Pais | Codigo HiBob | Tipo |
|------|--------|------|--------------|------|
| HQ Valencia | Valencia | Espana | VAL | Oficina principal |
| Madrid Office | Madrid | Espana | MAD | Oficina secundaria |
| Remote Spain | Remoto | Espana | REM-ES | Teletrabajo |
| Remote LATAM | Remoto | LATAM | REM-LA | Teletrabajo |

---

## Reglas de Agrupacion para Informes

### Por Area (nivel 1)
Agrupar departamentos en 5 areas para el informe ejecutivo:
- **Tecnica**: Engineering, QA, DevOps, IT Support
- **Comercial**: Sales, Pre-Sales, Marketing
- **Operaciones**: Operations, PM, Consulting, Training
- **Financiera**: Finance, Accounting, Administration
- **Personas**: HR, Talent Acquisition, L&D

### Departamentos Pequenos (RGPD)
- Si un departamento tiene **menos de 3 empleados**, agrupar como "Otros" en tablas publicas
- En el detalle por departamento (seccion 7 del informe), marcar como "Equipo reducido - metricas omitidas por privacidad"
- Excepcion: el responsable de RRHH puede solicitar detalle si tiene justificacion

### Jerarquia de Managers
- **Nivel 0**: CEO / Director General
- **Nivel 1**: Directores de area (reportan al CEO)
- **Nivel 2**: Responsables de departamento (reportan a directores)
- **Nivel 3**: Team leads / coordinadores (reportan a responsables)
- **Nivel 4+**: Empleados individuales

Para el ratio manager:empleado, considerar como "manager" a cualquier persona con al menos 1 report directo en HiBob.

---

## Tipos de Contrato

| Tipo HiBob | Tipo Odoo | FTE Default | Notas |
|-------------|-----------|-------------|-------|
| Full-time permanent | Indefinido | 1.0 | Contrato indefinido a jornada completa |
| Part-time permanent | Indefinido parcial | 0.5-0.8 | Segun horas semanales |
| Fixed-term | Temporal | 1.0 | Contrato temporal con fecha fin |
| Intern | Becario | 0.5 | Contrato de practicas/formacion |
| Contractor | Freelance | N/A | No computa en headcount ni FTE |
| Apprentice | Formacion dual | 0.5 | Contrato de formacion en alternancia |

**Nota**: Los contractors/freelancers NO se incluyen en el headcount ni en las metricas de rotacion.

---

## Periodos de Referencia

| Concepto | Periodo | Notas |
|----------|---------|-------|
| Periodo de prueba | 6 meses | Estandar en Espana (puede variar por convenio) |
| Evaluacion anual | Enero-Diciembre | Ciclo de evaluacion de desempeno |
| Cierre fiscal | Enero-Diciembre | Ano natural |
| Convenio colectivo | Consultorias TIC | XVII Convenio Colectivo estatal de empresas de consultorias, tecnologias de la informacion y estudios de mercado |
