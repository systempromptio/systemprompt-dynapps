# Template HTML para Presentacion de Kick-Off

## Instrucciones de Uso

Este template define la estructura HTML de la presentacion interactiva de kick-off.
Es un archivo HTML autocontenido con CSS y JS embebidos, navegable por teclado y scroll.

**Marca Indaws**:
- Blue Space: `#1C265D` (fondos oscuros, texto en slides claras)
- Blue Lilac: `#6B68FA` (titulos, acentos, dividers)
- Warm Yellow: `#E5B92B` (CTAs, highlights, numeros clave)
- Light Sky: `#8AC2DB` (fondos claros, textos secundarios)
- Blanco: `#FFFFFF` (fondos de slides de contenido)
- Tipografia: Montserrat (400/600/700) via Google Fonts

**IMPORTANTE**: En slides con fondo claro, usar el logo OSCURO (logotipo-indaws-color-blue-gradient_blue.svg). En slides con fondo oscuro/gradiente, usar el logo BLANCO (logotipo-indaws-monocolor-white.svg).

**Navegacion**: Scroll-snap vertical + flechas de teclado + dots laterales

---

## Estructura Base

```html
<!DOCTYPE html>
<html lang="es">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>Kick-Off — {nombre_cliente} — Indaws</title>
  <link href="https://fonts.googleapis.com/css2?family=Montserrat:wght@400;600;700&display=swap" rel="stylesheet"/>
  <style>
    /* === RESET & BASE === */
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
    html { scroll-snap-type: y mandatory; scroll-behavior: smooth; overflow-y: scroll; }
    body {
      font-family: 'Montserrat', Verdana, sans-serif;
      color: #1C265D;
      background: #FFFFFF;
      line-height: 1.6;
    }

    /* === SLIDES === */
    .slides-container { width: 100%; }
    .slide {
      width: 100%;
      min-height: 100vh;
      scroll-snap-align: start;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 2.5rem;
      position: relative;
      overflow: hidden;
    }
    .slide-content {
      max-width: 1400px;
      width: 100%;
      z-index: 1;
    }

    /* === NAVIGATION DOTS === */
    .slide-nav {
      position: fixed;
      right: 1.5rem;
      top: 50%;
      transform: translateY(-50%);
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      z-index: 100;
    }
    .slide-nav a {
      width: 10px;
      height: 10px;
      border-radius: 50%;
      background: rgba(28, 38, 93, 0.2);
      transition: background 0.3s, transform 0.3s;
      display: block;
    }
    .slide-nav a.active {
      background: #6B68FA;
      transform: scale(1.3);
    }

    /* === TYPOGRAPHY === */
    /* Sizes optimized for projection (readability at distance) */
    .heading-xl { font-size: 4rem; font-weight: 700; line-height: 1.1; text-transform: uppercase; }
    .heading-lg { font-size: 3.2rem; font-weight: 700; line-height: 1.2; }
    .heading-md { font-size: 2.6rem; font-weight: 700; line-height: 1.3; }
    .heading-sm { font-size: 2rem; font-weight: 600; line-height: 1.3; }
    .body-lg { font-size: 1.6rem; font-weight: 400; }
    .body-md { font-size: 1.4rem; font-weight: 400; }
    .body-sm { font-size: 1.2rem; font-weight: 400; }
    .overline {
      font-size: 1rem;
      font-weight: 600;
      letter-spacing: 0.15em;
      text-transform: uppercase;
      color: #E5B92B;
    }
    .text-center { text-align: center; }
    .text-white { color: #FFFFFF; }
    .text-lilac { color: #6B68FA; }
    .text-yellow { color: #E5B92B; }
    .text-sky { color: #8AC2DB; }
    .text-muted { color: rgba(28, 38, 93, 0.6); }

    /* === BACKGROUNDS === */
    .bg-dark { background: #1C265D; color: #FFFFFF; }
    .bg-gradient { background: linear-gradient(160deg, #1C265D 0%, #6B68FA 50%, #8AC2DB 100%); color: #FFFFFF; }
    .bg-lilac { background: #6B68FA; color: #FFFFFF; }
    .bg-light { background: #F5F7FA; }
    .bg-white { background: #FFFFFF; }

    /* === LAYOUT === */
    .row { display: flex; flex-wrap: wrap; gap: 2rem; }
    .col-2 { flex: 1 1 calc(50% - 1rem); min-width: 280px; }
    .col-3 { flex: 1 1 calc(33.33% - 1.5rem); min-width: 250px; }
    .col-4 { flex: 1 1 calc(25% - 1.5rem); min-width: 200px; }
    .gap-1 { gap: 1rem; }
    .gap-2 { gap: 2rem; }
    .gap-3 { gap: 3rem; }
    .mb-1 { margin-bottom: 0.5rem; }
    .mb-2 { margin-bottom: 1rem; }
    .mb-3 { margin-bottom: 1.5rem; }
    .mb-4 { margin-bottom: 2rem; }
    .mb-5 { margin-bottom: 3rem; }
    .mt-2 { margin-top: 1rem; }
    .mt-3 { margin-top: 1.5rem; }

    /* === COMPONENTS === */

    /* KPI Cards */
    .kpi-grid { display: flex; flex-wrap: wrap; gap: 2rem; justify-content: center; }
    .kpi-card {
      text-align: center;
      padding: 2rem;
      border-radius: 12px;
      min-width: 180px;
      flex: 1;
    }
    .kpi-card.outline {
      border: 2px solid rgba(107, 104, 250, 0.3);
      background: rgba(107, 104, 250, 0.05);
    }
    .kpi-value {
      font-size: 3.5rem;
      font-weight: 700;
      color: #6B68FA;
      display: block;
      margin-bottom: 0.5rem;
    }
    .kpi-label {
      font-size: 1.3rem;
      color: rgba(28, 38, 93, 0.7);
    }

    /* Team Cards */
    .team-grid { display: flex; flex-wrap: wrap; gap: 2rem; justify-content: center; }
    .team-card {
      text-align: center;
      padding: 1.5rem;
      min-width: 180px;
      max-width: 220px;
      flex: 0 1 auto;
    }
    .team-photo {
      width: 120px;
      height: 120px;
      border-radius: 50%;
      object-fit: cover;
      border: 3px solid #6B68FA;
      margin-bottom: 1rem;
    }
    .team-name {
      font-size: 1.5rem;
      font-weight: 700;
      color: #1C265D;
      margin-bottom: 0.25rem;
    }
    .team-role {
      font-size: 1.15rem;
      font-weight: 600;
      color: #6B68FA;
      margin-bottom: 0.25rem;
    }
    .team-title {
      font-size: 1.05rem;
      color: rgba(28, 38, 93, 0.6);
      margin-bottom: 0.25rem;
    }
    .team-email {
      font-size: 1rem;
      color: #8AC2DB;
    }
    .team-contact-badge {
      display: inline-block;
      background: #E5B92B;
      color: #1C265D;
      font-size: 0.7rem;
      font-weight: 700;
      padding: 0.2rem 0.6rem;
      border-radius: 20px;
      margin-top: 0.5rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }

    /* App Grid */
    .app-grid { display: flex; flex-wrap: wrap; gap: 1.5rem; justify-content: center; }
    .app-card {
      background: #FFFFFF;
      border-radius: 12px;
      padding: 1.5rem;
      text-align: center;
      min-width: 150px;
      flex: 1 1 calc(25% - 1.5rem);
      max-width: 250px;
      border: 2px solid rgba(107, 104, 250, 0.15);
      box-shadow: 0 2px 8px rgba(0,0,0,0.1);
      transition: transform 0.2s, box-shadow 0.2s;
    }
    .app-icon {
      width: 64px;
      height: 64px;
      background: #6B68FA;
      border-radius: 10px;
      display: flex;
      align-items: center;
      justify-content: center;
      margin: 0 auto 1rem;
      color: #fff;
      font-size: 1.5rem;
      font-weight: 700;
    }
    .app-name { font-weight: 600; font-size: 1.15rem; color: #1C265D; margin-bottom: 0.5rem; }
    .app-desc { font-size: 1.05rem; color: rgba(28, 38, 93, 0.6); }

    /* Tables */
    .data-table { width: 100%; border-collapse: collapse; margin: 1rem 0; }
    .data-table th {
      background: #6B68FA;
      color: #FFFFFF;
      padding: 0.85rem 1.2rem;
      text-align: left;
      font-size: 1.15rem;
      font-weight: 600;
    }
    .data-table td {
      padding: 0.85rem 1.2rem;
      border-bottom: 1px solid rgba(28, 38, 93, 0.1);
      font-size: 1.15rem;
    }
    .data-table tr:nth-child(even) { background: #F5F7FA; }

    /* Two-Column Comparison */
    .comparison { display: flex; gap: 2rem; flex-wrap: wrap; }
    .comparison-col {
      flex: 1;
      min-width: 280px;
      padding: 2rem;
      border-radius: 12px;
    }
    .comparison-col.included {
      background: rgba(39, 174, 96, 0.08);
      border: 1px solid rgba(39, 174, 96, 0.2);
    }
    .comparison-col.excluded {
      background: rgba(231, 76, 60, 0.08);
      border: 1px solid rgba(231, 76, 60, 0.2);
    }
    .comparison-col h3 {
      margin-bottom: 1rem;
      font-size: 1.2rem;
    }
    .comparison-col.included h3 { color: #27ae60; }
    .comparison-col.excluded h3 { color: #e74c3c; }
    .comparison-col ul { list-style: none; padding: 0; }
    .comparison-col li {
      padding: 0.5rem 0 0.5rem 1.5rem;
      position: relative;
      font-size: 1.1rem;
    }
    .comparison-col.included li::before {
      content: '\2713';
      position: absolute;
      left: 0;
      color: #27ae60;
      font-weight: 700;
    }
    .comparison-col.excluded li::before {
      content: '\2717';
      position: absolute;
      left: 0;
      color: #e74c3c;
      font-weight: 700;
    }

    /* Phase Diagram */
    .phases { display: flex; gap: 0; flex-wrap: wrap; justify-content: center; }
    .phase {
      flex: 1;
      min-width: 140px;
      padding: 1.5rem 1rem;
      text-align: center;
      position: relative;
    }
    .phase-number {
      width: 40px;
      height: 40px;
      border-radius: 50%;
      background: #6B68FA;
      color: #FFFFFF;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 700;
      margin: 0 auto 0.75rem;
    }
    .phase.active .phase-number {
      background: #E5B92B;
      color: #1C265D;
      transform: scale(1.2);
    }
    .phase-name {
      font-weight: 600;
      font-size: 1.05rem;
      color: #1C265D;
      margin-bottom: 0.25rem;
    }
    .phase-desc {
      font-size: 0.9rem;
      color: rgba(28, 38, 93, 0.5);
    }
    .phase-connector {
      position: absolute;
      top: 2.25rem;
      right: -1rem;
      width: 2rem;
      height: 2px;
      background: rgba(107, 104, 250, 0.3);
    }
    .phase:last-child .phase-connector { display: none; }

    /* Timeline Bar */
    .timeline-bar {
      display: flex;
      gap: 0;
      margin: 2rem 0;
      border-radius: 8px;
      overflow: hidden;
    }
    .timeline-segment {
      padding: 1rem;
      text-align: center;
      color: #FFFFFF;
      font-size: 1rem;
      font-weight: 600;
      position: relative;
    }
    .timeline-segment .dates {
      font-size: 0.85rem;
      font-weight: 400;
      opacity: 0.8;
      display: block;
      margin-top: 0.25rem;
    }

    /* Parallel Timeline Tracks (for Formacion + UAT alongside Implantacion) */
    .timeline-tracks { margin: 2rem 0; }
    .timeline-track { display: flex; gap: 0; margin-bottom: 0.5rem; border-radius: 8px; overflow: hidden; }
    .timeline-track-label {
      font-size: 0.9rem;
      font-weight: 600;
      color: rgba(28, 38, 93, 0.5);
      margin-bottom: 0.25rem;
    }
    .timeline-spacer { flex: none; background: transparent; }
    .timeline-track.secondary .timeline-segment { opacity: 0.85; }

    /* Impact Chain */
    .impact-chain { text-align: center; }
    .impact-step {
      display: inline-block;
      background: #F5F7FA;
      border: 1px solid rgba(28, 38, 93, 0.1);
      border-radius: 8px;
      padding: 1.2rem 2rem;
      margin: 0.5rem auto;
      font-size: 1.15rem;
      max-width: 550px;
      width: 100%;
    }
    .impact-arrow {
      display: block;
      text-align: center;
      color: #e74c3c;
      font-size: 1.5rem;
      margin: 0.25rem 0;
    }
    .impact-step.critical {
      background: rgba(231, 76, 60, 0.1);
      border-color: rgba(231, 76, 60, 0.3);
      font-weight: 600;
    }

    /* Checklist */
    .request-list { list-style: none; padding: 0; }
    .request-item {
      display: flex;
      align-items: flex-start;
      gap: 1rem;
      padding: 0.75rem 0;
      border-bottom: 1px solid rgba(28, 38, 93, 0.08);
    }
    .request-priority {
      font-size: 0.7rem;
      font-weight: 700;
      padding: 0.2rem 0.5rem;
      border-radius: 4px;
      text-transform: uppercase;
      white-space: nowrap;
    }
    .priority-critical { background: #e74c3c; color: #fff; }
    .priority-high { background: #E5B92B; color: #1C265D; }
    .priority-medium { background: #8AC2DB; color: #1C265D; }
    .request-content { flex: 1; }
    .request-deadline { font-size: 0.85rem; color: #6B68FA; font-weight: 600; white-space: nowrap; }

    /* Dividers */
    .divider-accent {
      width: 60px;
      height: 3px;
      background: #E5B92B;
      border-radius: 2px;
    }
    .divider-accent.center { margin-left: auto; margin-right: auto; }

    /* Logo */
    .logo-small { height: 40px; opacity: 0.7; }
    .logo-corner {
      position: absolute;
      top: 2rem;
      left: 2rem;
      height: 30px;
      opacity: 0.5;
    }

    /* Slide Number */
    .slide-number {
      position: absolute;
      bottom: 1.5rem;
      right: 2rem;
      font-size: 0.75rem;
      color: rgba(28, 38, 93, 0.3);
      font-weight: 600;
    }
    .bg-dark .slide-number,
    .bg-gradient .slide-number,
    .bg-lilac .slide-number {
      color: rgba(255, 255, 255, 0.3);
    }

    /* Animations */
    .reveal {
      opacity: 0;
      transform: translateY(20px);
      transition: opacity 0.6s ease, transform 0.6s ease;
    }
    .reveal.visible {
      opacity: 1;
      transform: translateY(0);
    }

    /* Responsive */
    @media (max-width: 768px) {
      .slide { padding: 2rem; }
      .heading-xl { font-size: 2rem; }
      .heading-lg { font-size: 1.75rem; }
      .heading-md { font-size: 1.5rem; }
      .slide-nav { right: 0.75rem; }
      .slide-nav a { width: 8px; height: 8px; }
      .team-grid { gap: 1rem; }
      .team-card { min-width: 140px; }
      .team-photo { width: 80px; height: 80px; }
      .phases { flex-direction: column; }
      .phase-connector { display: none !important; }
    }
  </style>
</head>
<body>
  <div class="slides-container">

    <!-- SLIDE 1: PORTADA -->
    <section class="slide bg-gradient" id="slide-1">
      <div class="slide-content text-center">
        <img src="{logo_indaws_white_base64}" alt="Indaws" style="height: 70px; margin-bottom: 2rem;"/>
        <p class="overline mb-3">INDAWS BUSINESS SOLUTIONS</p>
        <h1 class="heading-xl mb-3" style="color: #fff;">Indaws - {nombre_cliente}</h1>
        <div class="divider-accent center mb-3"></div>
        <p class="heading-sm" style="color: rgba(255,255,255,0.9);">KICK-OFF</p>
        <p class="body-lg" style="color: #8AC2DB;">Implantacion de Odoo {version_odoo}</p>
        <p class="body-sm mt-3" style="color: rgba(255,255,255,0.5);">{fecha_kickoff}</p>
      </div>
    </section>

    <!-- SLIDE 2: AGENDA -->
    <section class="slide bg-white" id="slide-2">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">PLAN DE LA REUNION</p>
        <h2 class="heading-md text-lilac mb-4">Agenda de hoy</h2>
        {agenda_content}
        <!-- Tabla o lista con los 5 bloques y tiempos -->
        <p class="body-sm text-muted mt-3">Tiempo total estimado: ~65 minutos</p>
      </div>
      <span class="slide-number">2 / 20</span>
    </section>

    <!-- SLIDE 3: INDAWS EN CIFRAS -->
    <section class="slide bg-light" id="slide-3">
      {logo_corner}
      <div class="slide-content text-center">
        <p class="overline mb-2">QUIENES SOMOS</p>
        <h2 class="heading-md text-lilac mb-5">Indaws Business Solutions</h2>
        <div class="kpi-grid">
          {kpi_cards}
          <!-- Ejemplo:
          <div class="kpi-card outline">
            <span class="kpi-value">10+</span>
            <span class="kpi-label">Anos de experiencia</span>
          </div>
          -->
        </div>
        <p class="body-md mt-3">{indaws_description}</p>
      </div>
      <span class="slide-number">3 / 20</span>
    </section>

    <!-- SLIDE 4: EQUIPO INDAWS -->
    <section class="slide bg-white" id="slide-4">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">PRESENTACION DEL EQUIPO</p>
        <h2 class="heading-md text-lilac mb-4">Vuestro equipo de proyecto</h2>
        <div class="team-grid">
          {team_cards_indaws}
          <!-- Ejemplo por miembro:
          <div class="team-card">
            <img src="data:image/png;base64,{image_128}" class="team-photo" alt="{name}"/>
            <div class="team-name">{name}</div>
            <div class="team-role">{rol_en_proyecto}</div>
            <div class="team-title">{job_title}</div>
            <div class="team-email">{work_email}</div>
            <span class="team-contact-badge">Contacto principal</span> (solo para el PM)
          </div>
          -->
        </div>
      </div>
      <span class="slide-number">4 / 20</span>
    </section>

    <!-- SLIDE 5: EQUIPO DEL CLIENTE -->
    <section class="slide bg-white" id="slide-5">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">PRESENTACION DEL EQUIPO</p>
        <h2 class="heading-md text-lilac mb-3">Vuestro equipo de proyecto</h2>
        <p class="body-md mb-4">Necesitamos definir juntos los roles clave para el exito del proyecto.</p>
        {client_roles_explanation}
        <!-- Tabla con roles por definir -->
        {client_roles_table}
      </div>
      <span class="slide-number">5 / 20</span>
    </section>

    <!-- SLIDE 6: VISION DEL PROYECTO -->
    <section class="slide bg-white" id="slide-6">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">ALINEACION</p>
        <h2 class="heading-md text-lilac mb-4">Vision del proyecto</h2>
        {vision_content}
        <!-- Resumen ejecutivo personalizado: antes vs despues -->
      </div>
      <span class="slide-number">6 / 20</span>
    </section>

    <!-- SLIDE 7: APLICACIONES ODOO -->
    <section class="slide bg-light" id="slide-7">
      {logo_corner}
      <div class="slide-content text-center">
        <p class="overline mb-2">ALCANCE</p>
        <h2 class="heading-md text-lilac mb-4">Aplicaciones Odoo de tu proyecto</h2>
        <div class="app-grid">
          {app_cards}
          <!-- Ejemplo:
          <div class="app-card">
            <div class="app-icon">TPV</div>
            <div class="app-name">Punto de Venta</div>
            <div class="app-desc">Gestion de ventas en tus 3 terminales</div>
          </div>
          -->
        </div>
      </div>
      <span class="slide-number">7 / 20</span>
    </section>

    <!-- SLIDE 8: MODULOS INDAWS CORE (OPCIONAL) -->
    <section class="slide bg-white" id="slide-8">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">ALCANCE</p>
        <h2 class="heading-md text-lilac mb-3">Modulos Indaws Core incluidos</h2>
        <p class="body-md mb-4">Desarrollos propios que potencian tu Odoo.</p>
        {core_modules_content}
      </div>
      <span class="slide-number">8 / 20</span>
    </section>

    <!-- SLIDE 9: INCLUIDO / NO INCLUIDO -->
    <section class="slide bg-white" id="slide-9">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">ALCANCE</p>
        <h2 class="heading-md text-lilac mb-4">Alcance del proyecto</h2>
        <div class="comparison">
          <div class="comparison-col included">
            <h3>Incluido</h3>
            <ul>{included_items}</ul>
          </div>
          <div class="comparison-col excluded">
            <h3>No incluido</h3>
            <ul>{excluded_items}</ul>
          </div>
        </div>
        <p class="body-md mt-3" style="background: rgba(107,104,250,0.08); border-left: 4px solid #6B68FA; padding: 1rem 1.5rem; border-radius: 4px; font-weight: 600;">El alcance se confirma al finalizar el analisis.</p>
        <p class="body-sm text-muted mt-2">Cualquier funcionalidad fuera de este alcance sera evaluada y presupuestada por separado.</p>
      </div>
      <span class="slide-number">9 / 20</span>
    </section>

    <!-- SLIDE 10: METODOLOGIA (Section Divider + Content) -->
    <section class="slide bg-lilac" id="slide-10">
      <div class="slide-content text-center">
        <p class="overline mb-3" style="color: #E5B92B;">SECCION 03</p>
        <h2 class="heading-lg" style="color: #fff;">Nuestra metodologia</h2>
        <div class="divider-accent center mt-3 mb-5"></div>
        <div class="phases">
          {methodology_phases}
          <!-- 6 fases con .phase y .phase.active para kick-off -->
        </div>
      </div>
    </section>

    <!-- SLIDE 11: FASE DE ANALISIS -->
    <section class="slide bg-white" id="slide-11">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">METODOLOGIA</p>
        <h2 class="heading-md text-lilac mb-4">Proxima fase: El Analisis</h2>
        {analysis_phase_content}
      </div>
      <span class="slide-number">11 / 20</span>
    </section>

    <!-- SLIDE 12: PLAN DE COMUNICACION -->
    <section class="slide bg-white" id="slide-12">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">METODOLOGIA</p>
        <h2 class="heading-md text-lilac mb-4">Plan de comunicacion</h2>
        <p class="body-md mb-3">Como vamos a trabajar juntos.</p>
        <table class="data-table">
          {communication_table}
        </table>
        <p class="body-sm text-muted mt-3">Toda decision relevante queda documentada por escrito.</p>
      </div>
      <span class="slide-number">12 / 20</span>
    </section>

    <!-- SLIDE 13: GESTION DE CAMBIOS -->
    <section class="slide bg-white" id="slide-13">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">METODOLOGIA</p>
        <h2 class="heading-md text-lilac mb-4">Gestion de cambios</h2>
        <p class="body-md mb-4">Como gestionamos las peticiones fuera de alcance.</p>
        {change_management_flow}
        <p class="body-sm text-muted mt-3">Las nuevas solicitudes no bloquean el proyecto. Se evaluan y planifican sin afectar los compromisos existentes.</p>
      </div>
      <span class="slide-number">13 / 20</span>
    </section>

    <!-- SLIDE 14: TIMELINE (Section Divider + Content) -->
    <section class="slide bg-white" id="slide-14">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">COMPROMISOS Y FECHAS</p>
        <h2 class="heading-md text-lilac mb-4">Timeline del proyecto</h2>
        {timeline_content}
        <!-- Timeline con pistas paralelas:
          Fila 1 (principal): Kick-Off → Analisis → Implantacion → UAT Final → Go-Live → Soporte
          Fila 2 (paralela):  [spacer]                [Formacion -----------→] Go-Live
          Fila 3 (paralela):  [spacer]                    [UAT parciales ----→] Go-Live
          Las 3 filas convergen en Go-Live. Usar CSS class="timeline-tracks" con
          multiples .timeline-track (principal + .secondary para las paralelas).
          Los spacers alinean el inicio de Formacion/UAT con la fase de Implantacion.
        -->
      </div>
      <span class="slide-number">14 / 20</span>
    </section>

    <!-- SLIDE 15: CALENDARIO SESIONES ANALISIS -->
    <section class="slide bg-white" id="slide-15">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">PLANIFICACION</p>
        <h2 class="heading-md text-lilac mb-4">Propuesta calendario de analisis</h2>
        <table class="data-table">
          {analysis_calendar_table}
        </table>
        <p class="body-sm text-muted mt-2">Este calendario es una propuesta. Lo confirmaremos hoy mismo.</p>
      </div>
      <span class="slide-number">15 / 20</span>
    </section>

    <!-- SLIDE 16: COMPROMISOS MUTUOS -->
    <section class="slide bg-white" id="slide-16">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">REGLAS DEL JUEGO</p>
        <h2 class="heading-md text-lilac mb-3">Compromisos mutuos</h2>
        <p class="body-md mb-4">El exito del proyecto depende de ambos.</p>
        <div class="row gap-2">
          <div class="col-2">
            <h3 class="heading-sm text-lilac mb-2">Indaws se compromete a</h3>
            <ul style="list-style: none; padding: 0;">
              {indaws_commitments}
            </ul>
          </div>
          <div class="col-2">
            <h3 class="heading-sm mb-2" style="color: #E5B92B;">{nombre_cliente} se compromete a</h3>
            <ul style="list-style: none; padding: 0;">
              {client_commitments}
            </ul>
          </div>
        </div>
      </div>
      <span class="slide-number">16 / 20</span>
    </section>

    <!-- SLIDE 17: IMPACTO DE RETRASOS -->
    <section class="slide bg-white" id="slide-17">
      {logo_corner}
      <div class="slide-content text-center">
        <p class="overline mb-2">TRANSPARENCIA</p>
        <h2 class="heading-md text-lilac mb-4">El impacto de los retrasos</h2>
        <div class="impact-chain">
          {impact_chain_steps}
        </div>
        <p class="body-md mt-3" style="max-width: 600px; margin-left: auto; margin-right: auto;">
          Los plazos del proyecto dependen de la velocidad de respuesta de ambas partes. Si hay un bloqueo, hablemos rapido para resolverlo juntos.
        </p>
      </div>
      <span class="slide-number">17 / 20</span>
    </section>

    <!-- SLIDE 18: CLIENT REQUESTS -->
    <section class="slide bg-white" id="slide-18">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">ACCION</p>
        <h2 class="heading-md text-lilac mb-4">Lo que necesitamos de vosotros</h2>
        <ul class="request-list">
          {client_requests}
        </ul>
      </div>
      <span class="slide-number">18 / 20</span>
    </section>

    <!-- SLIDE 19: PROXIMOS PASOS -->
    <section class="slide bg-white" id="slide-19">
      {logo_corner}
      <div class="slide-content">
        <p class="overline mb-2">PROXIMOS PASOS</p>
        <h2 class="heading-md text-lilac mb-4">Proximos pasos</h2>
        {next_steps_content}
        <p class="heading-sm text-lilac mt-3">A partir de hoy, el proyecto esta en marcha.</p>
      </div>
      <span class="slide-number">19 / 20</span>
    </section>

    <!-- SLIDE 20: CIERRE -->
    <section class="slide bg-gradient" id="slide-20">
      <div class="slide-content text-center">
        <img src="{logo_indaws_white_base64}" alt="Indaws" style="height: 70px; margin-bottom: 2rem;"/>
        <h2 class="heading-lg mb-3" style="color: #fff;">Preguntas?</h2>
        <p class="body-lg mb-4" style="color: rgba(255,255,255,0.85);">Estamos aqui para construir juntos.</p>
        <div class="divider-accent center mb-4"></div>
        <p class="body-md" style="color: #8AC2DB;">{pm_name} — {pm_email}</p>
        <p class="body-sm mt-3" style="color: rgba(255,255,255,0.5);">Gracias por vuestra confianza.</p>
      </div>
    </section>

  </div>

  <!-- NAVIGATION DOTS -->
  <nav class="slide-nav"></nav>

  <!-- NAVIGATION JS -->
  <script>
    (function() {
      var slides = document.querySelectorAll('.slide');
      var nav = document.querySelector('.slide-nav');
      slides.forEach(function(slide, i) {
        var dot = document.createElement('a');
        dot.href = '#slide-' + (i + 1);
        if (i === 0) dot.classList.add('active');
        nav.appendChild(dot);
      });
      var observer = new IntersectionObserver(function(entries) {
        entries.forEach(function(entry) {
          if (entry.isIntersecting) {
            var index = Array.from(slides).indexOf(entry.target);
            nav.querySelectorAll('a').forEach(function(dot, i) {
              dot.classList.toggle('active', i === index);
            });
            entry.target.querySelectorAll('.reveal').forEach(function(el) {
              el.classList.add('visible');
            });
          }
        });
      }, { threshold: 0.5 });
      slides.forEach(function(slide) { observer.observe(slide); });
      document.addEventListener('keydown', function(e) {
        var current = Math.round(window.scrollY / window.innerHeight);
        if (e.key === 'ArrowDown' || e.key === 'ArrowRight' || e.key === ' ') {
          e.preventDefault();
          var next = Math.min(current + 1, slides.length - 1);
          slides[next].scrollIntoView({ behavior: 'smooth' });
        }
        if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
          e.preventDefault();
          var prev = Math.max(current - 1, 0);
          slides[prev].scrollIntoView({ behavior: 'smooth' });
        }
      });
    })();
  </script>
</body>
</html>
```

---

## Placeholders

Todos los valores entre `{...}` deben ser reemplazados con datos reales del proyecto:

### Datos del proyecto
- `{nombre_cliente}` — Nombre del cliente (de res.partner)
- `{version_odoo}` — Version de Odoo (ej: "v18")
- `{fecha_kickoff}` — Fecha del kick-off (DD de Mes de YYYY)
- `{logo_indaws_white_base64}` — Logo Indaws en base64 (variante blanca)
- `{logo_corner}` — `<img src="..." class="logo-corner" alt="Indaws"/>` (isotipo)

### Datos del equipo
- `{team_cards_indaws}` — HTML de tarjetas del equipo Indaws (desde hr.employee)
- `{pm_name}`, `{pm_email}` — Datos del PM para la slide de cierre

### Datos de contenido
- `{agenda_content}` — Lista/tabla con los 5 bloques y tiempos
- `{kpi_cards}` — KPIs de Indaws (hardcoded, actualizar periodicamente)
- `{indaws_description}` — Breve descripcion de Indaws
- `{vision_content}` — Resumen ejecutivo antes/despues
- `{app_cards}` — Grid de aplicaciones Odoo
- `{core_modules_content}` — Lista de modulos Indaws Core
- `{included_items}`, `{excluded_items}` — Listas de alcance
- `{methodology_phases}` — Diagrama de fases
- `{analysis_phase_content}` — Subfases del analisis
- `{communication_table}` — Tabla de plan de comunicacion
- `{change_management_flow}` — Flujo de gestion de cambios
- `{timeline_content}` — Barra temporal del proyecto
- `{analysis_calendar_table}` — Tabla de sesiones de analisis
- `{indaws_commitments}`, `{client_commitments}` — Listas de compromisos
- `{impact_chain_steps}` — Cadena de impacto de retrasos
- `{client_requests}` — Solicitudes al cliente con prioridad y fecha
- `{next_steps_content}` — Acciones inmediatas

### Regla de numeracion
Si se omite la slide 8 (sin modulos Core), renumerar todas las slides y los IDs.
El numero total en los indicadores "X / 20" se ajusta automaticamente.
