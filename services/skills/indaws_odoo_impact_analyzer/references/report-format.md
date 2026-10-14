# Formato del Informe de Impacto y Checklist de Cambio Seguro

---

## Estructura del Informe de Impacto

### Cabecera

Encabezado con: simbolo objetivo, tipo de cambio, version Odoo, fecha y ruta escaneada.

### 1. Resumen Ejecutivo

Tabla con: simbolo objetivo, tipo de cambio, total dependientes, modulos afectados,
riesgo global (CRITICO/ALTO/MEDIO/BAJO), y si el cambio es seguro sin plan previo.

El riesgo global es el nivel maximo de cualquier dependiente individual.

### 2. Tabla de Dependientes por Tipo

Columnas: Tipo | Fichero/Simbolo | Modulo | Riesgo | Notas

Tipos reconocidos: compute, related, vista XML, attrs/invisible/required/readonly,
dominio, informe QWeb, server action, record rule, _inherit, migracion, test,
controlador, wizard, ir.model.access.csv.

Niveles de riesgo:
- CRITICO: rompe en runtime si no se migra antes de instalar
- ALTO: rompe UI, acceso o logica de negocio
- MEDIO: rompe informes, dominios o filtros guardados
- BAJO: tests, comentarios, strings de cara al usuario

### 3. Modulos Afectados -- Orden de Migracion

Lista numerada con el modulo propietario primero (upstream), seguido de los
modulos que heredan, luego vistas/informes, luego logica de negocio.

Justificacion: las dependencias de tipo compute/related requieren que el campo
ya exista con el nuevo nombre en el modulo base antes de actualizarse.

### 4. Alertas de BD (si aplica)

Advertir que registros creados via UI no aparecen en grep.
Modelos a revisar: ir.actions.server, base.automation, ir.filters, ir.rule.

---

## Checklist de Cambio Seguro

### Antes del cambio
- Crear rama de trabajo en git
- Backup de la BD de desarrollo
- Revisar dependencias en BD (ir.actions.server, base.automation, ir.filters)
- Confirmar version Odoo objetivo y patrones deprecados

### Por cada modulo afectado (en orden de migracion)
- Actualizar la definicion del campo/metodo en .py
- Actualizar @api.depends / related / compute / inverse / search
- Actualizar vistas XML (field name=, attrs/invisible, domain, xpath)
- Actualizar informes QWeb
- Actualizar security/ (ACL, rules)
- Crear script de migracion en migrations/VERSION/ si hay cambio de columna SQL
- Bump de version en __manifest__.py

### Validacion
- py_compile sobre todos los .py modificados
- Instalar con -u MODULOS_AFECTADOS y verificar log sin errores
- Ejecutar tests unitarios afectados
- Revisar vistas en UI y informes con datos reales
- Revisar record rules por grupo

### Post-cambio
- Commit con mensaje descriptivo del cambio de API
- Documentar en changelog del modulo
- Notificar a equipos con modulos dependientes externos

---

## Casos especiales

### Cambio de tipo de campo (Char -> Many2one, etc.)
Anadir alerta de migracion de datos obligatoria con script SQL de conversion.

### Dependientes en modulos de Odoo base u OCA
Marcar como fuera de alcance. Recomendar deprecar en lugar de eliminar,
manteniendo compatibilidad hacia atras al menos una version.

### 0 dependientes encontrados
Emitir igualmente el informe con riesgo BAJO y checklist minimo.
Advertir que el escaneo puede tener alcance limitado.
