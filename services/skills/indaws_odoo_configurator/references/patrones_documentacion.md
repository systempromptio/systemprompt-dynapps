# Patrones de Documentacion

> Reglas para determinar donde y como documentar cada tipo de configuracion.

---

## Regla Principal

| Tipo de cambio | Destino | Formato |
|---------------|---------|---------|
| CONFIG simple (activar setting, instalar modulo, cambiar parametro) | **Nota interna** en chatter de la tarea | HTML breve, agrupado |
| STUDIO/AUTO (campo nuevo, server action, automated action, pagina) | **Knowledge > Adaptaciones** | Articulo HTML detallado |
| Configuracion compleja (multi-paso, con dependencias) | **Knowledge > Adaptaciones** | Articulo HTML detallado |

---

## Criterios de Decision

### Va a Nota Interna cuando:
- Es un cambio de un solo paso (activar/desactivar)
- No requiere explicacion para el usuario final
- Es reversible facilmente
- No crea objetos nuevos (campos, acciones, modelos)
- Ejemplos: activar multi-moneda, instalar idioma, cambiar setting

### Va a Knowledge > Adaptaciones cuando:
- Crea objetos nuevos en el sistema (campos, actions, vistas)
- Requiere explicacion de como funciona para el usuario
- Tiene logica de negocio (triggers, condiciones, automatizaciones)
- Afecta al comportamiento visible del sistema
- Necesita documentacion para futuras referencias (soporte, formacion)
- Ejemplos: campo custom, automated action, server action, pagina/tab nueva

---

## Reglas de Agrupacion

### Notas internas
- TODAS las configuraciones CONFIG de una ejecucion se agrupan en UNA SOLA nota
- Formato: lista HTML con cada item configurado
- Incluir flag de entorno (PRODUCCION/TEST)
- Template: `templates/nota_interna_config.md`

### Articulos Knowledge
- UN articulo por cambio Studio/AUTO o grupo de cambios relacionados
- Si multiples campos van al mismo modelo y estan relacionados → un solo articulo
- Si son independientes → articulos separados
- Padre: articulo "Adaptaciones" bajo x_studio_knowledge del proyecto
- Template: `templates/adaptacion_knowledge.md`

---

## Contenido Obligatorio

### En Nota Interna:
- Nombre del item del checklist
- Descripcion breve de lo configurado
- Ruta en Odoo (Settings > Seccion > Opcion)
- Flag de entorno

### En Articulo Knowledge:
- Que se ha hecho (descripcion funcional)
- Por que (referencia al analisis funcional si existe)
- Detalles tecnicos (modelo, campo, tipo, opciones)
- Como funciona para el usuario (perspectiva de uso)
- Placeholders para capturas de pantalla

---

## Anti-patrones (NO hacer)

- NO crear articulo Knowledge para una activacion simple de setting
- NO poner detalles tecnicos en notas internas (el consultor no necesita ver campos y modelos en el chatter)
- NO duplicar informacion entre nota interna y Knowledge
- NO crear multiples notas internas por ejecucion (siempre una sola, agrupada)
- NO documentar items DEV/MIGR que no se han ejecutado
