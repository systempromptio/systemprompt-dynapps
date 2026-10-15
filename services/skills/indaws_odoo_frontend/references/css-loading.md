# CSS Loading in Odoo

## Website Pages (Self-Contained)

For **website pages**: the CSS is always inlined in the `<style>` block of the generated HTML (see `references/self-contained-output.md`). No external CSS file installation is needed.

## Blog / Email / Other Content Types

For **blog posts, email templates, and other content types** where a `<style>` block is not practical, the `iw-*` classes must be loaded in Odoo via one of these methods:

### Method 1: Website Customization Panel

Odoo > Website > Customize > scroll to bottom > Custom CSS > paste contents of `assets/css/indaws-frontend.css`

### Method 2: Custom Odoo Module

Add to a module's `assets.xml`:

```xml
<template id="assets_frontend" inherit_id="website.assets_frontend">
  <xpath expr="." position="inside">
    <link rel="stylesheet" href="/your_module/static/src/css/indaws-frontend.css"/>
  </xpath>
</template>
```

### Note on Animations

The `iw-reveal` scroll animations from `assets/js/iw-reveal.js` are only available when loaded via a custom Odoo module asset bundle. They are NOT included in self-contained page output (Odoo strips `<script>` tags).
