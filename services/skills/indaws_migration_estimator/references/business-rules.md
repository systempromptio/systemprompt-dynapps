# Reglas de negocio fijas — Migración Dynapps

Fuente: reunión 2026-06-19 + PDFs de ejemplo (Geosquare, Luminex, Newstairs). Estas
reglas son el núcleo del skill: codifican lo que hoy se aplica a mano.

## Modelo de 4 buckets (estructura del presupuesto y del informe)
El presupuesto se desglosa SIEMPRE en 4 buckets (los "puntos 1 2 3 4"):

| # | Bucket | Regla resumida |
|---|---|---|
| 1 | **Code Migration** | Variable, por módulo, ajustado a horas reales con IA |
| 2 | **Database Upgrade** | Variable, el más recortable, **nunca < 20h** |
| 3 | **Unit Testing** | **10 h** fijo (excepción: módulo único pequeño → ~2h) |
| 4 | **Project Management** | **10 h** fijo |

Total = 1+2+3+4. Cálculo y topes en `calibration.md`; estructura del PDF en `report.md`.

## Tabla de reglas fijas

| Parámetro | Valor fijo |
|---|---|
| Tipo de migración | **1-a-1 siempre** (Dynapps migra todo tal cual) |
| Testing funcional | **NO** — lo hace Dynapps |
| Alcance Indaws | migración de módulos + migración de BD + correcciones de errores reportados por Dynapps |
| Módulos en scope | **🔴 Custom** (in-house + vendor) + **OCA sin alternativa**. OCA se excluye si existe port nativo v19 **o** equivalente en core; si no existe en ninguno, su migración **se incluye**. Si existe en **OCA y en Odoo** (ej. comisiones) → **se mantiene el instalado**, fuera de scope. Nativos en destino se eliminan (~1h verificar) |
| Cap por módulo | **~20h** salvo complejidad excepcional justificada |
| Unit Tests (bucket 3) | **10 h** (cobertura 80-90%); ~2h si módulo único pequeño |
| Project Management (bucket 4) | **10 h** |
| Database Upgrade (bucket 2) | ≈ nº iteraciones × tamaño (4 iter ≈ 40h); **floor 20h**; hasta ~60h en muchos saltos |
| Versión destino | **19.0** casi siempre |
| Versión origen | deducir de los `__manifest__.py` / repo (rango 14-19) |
| Precio | **45 €/h** (pactado con Dynapps) — ver regla de importes abajo |
| Rango sano total | **100-150 h = media histórica** (referencia, no techo); **> 200 h = caso excepcional por volumen, a justificar en sección 04** (no recortar horas reales) |
| Contingencia | mínima (todo en módulos con git, sin Studio/flujos no-code) |

## Importes — regla dura (solo horas en el informe)
- El **informe PDF lleva SOLO horas** (por bucket, por módulo y total). **Nunca** incluye
  importes en € ni "@ 45 €/h" en ninguna página, card o tabla.
- El **precio (45 €/h) vive únicamente en el `sale.order`** de Odoo (ver `quotation.md`):
  qty = horas estimadas, precio unitario = 45. El € no aparece en el entregable visual.
- Motivo: el informe lo presenta Dynapps a su cliente final; el importe es información
  comercial Indaws↔Dynapps que va en la oferta, no en el análisis técnico.

## Perfil del analyzer (cómo invocar odoo-database-analyzer)
Al delegar en `odoo-database-analyzer`, fijar:
- **Tipo**: `1to1` (sin refactor ni Studio).
- **Testing funcional**: excluido.
- **Asistencia IA**: máxima ("todo 100% con IA").
- **Versión origen/destino**: las de los inputs (destino default 19.0).
- **Salida**: desglose por módulo (se recalibra después, ver `calibration.md`).

> El analyzer es genérico y compartido: **no se modifica**. La especialización
> migration-lab se aplica como capa de calibración encima.

## Excepciones conocidas
- Módulo único pequeño → unit tests pueden bajar a ~2h.
- Desarrollos muy complejos (configurador web full OWL, app aparte, muchos
  conectores, saltos grandes 12→19) → pueden justificar >150h o módulos >20h, pero
  hay que defenderlo en la sección 04 (Critical Points).
