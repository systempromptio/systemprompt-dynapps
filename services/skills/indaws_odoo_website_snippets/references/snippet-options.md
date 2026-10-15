# Snippet Options Panel

Shared standards: `references/shared/coding-standards.md`

The options panel is what the user sees in the right sidebar when a snippet is selected.
v17/v18 and v19 use **completely different** systems — pick by target version (Step 1).

## v17 / v18 — declarative `we-*` widgets + `options.registry`

### Declaring the panel (XML)

Options are declared by inheriting `website.snippet_options` and adding a `<div>` with a
selector. Verified `website/views/snippets/s_alert.xml:13-39`, `s_alert_options`:

```xml
<template id="s_alert_options" inherit_id="website.snippet_options">
  <xpath expr="//div[@id='so_width']" position="after">
    <div data-selector=".s_alert">
      <we-select string="Size">
        <we-button data-select-class="s_alert_sm">Small</we-button>
        <we-button data-select-class="s_alert_md">Medium</we-button>
        <we-button data-select-class="s_alert_lg">Large</we-button>
      </we-select>
      <we-colorpicker string="Color" data-name="alert_colorpicker_opt"
        data-select-style="true" data-css-property="background-color"
        data-color-prefix="alert-"/>
    </div>
  </xpath>
</template>
```

### Container `<div>` data-attributes
- `data-selector` — CSS selector the panel applies to (the snippet root `.s_<name>`).
  **If it matches nothing, the panel silently does not mount.**
- `data-exclude` — selector to exclude (e.g. nested snippets). See `website/views/snippets/snippets.xml:873`, `snippet_options`.
- `data-target` — apply the option to a sub-element instead of the root
  (e.g. `data-target=".s_dynamic_snippet_title"`, `website/views/snippets/s_dynamic_snippet.xml:46`, `website.s_dynamic_snippet_options_template`).
- `data-js` — name of a `options.registry.<Name>` JS class implementing custom methods.
  Without it, the generic `options.Class` is used. Mapping at `web_editor/static/src/js/editor/snippets.editor.js:829`, `_initializeOptions`, `_initializeOptions`:
  `new (options.registry[optionName] || options.Class)(...)`.
- `data-drop-in` / `data-drop-near` — restrict where the snippet can be dropped (parsed
  `web_editor/static/src/js/editor/snippets.editor.js:815-816`, `_initializeOptions`).
- `data-no-preview="true"` — disable live hover preview (mandatory for dynamic snippets,
  `website/views/snippets/s_dynamic_snippet.xml:36`, `website.s_dynamic_snippet_options_template`).

### `we-*` widgets (verified across `s_alert.xml`, `s_countdown.xml`)
- `we-select` + `we-button` — dropdown; buttons are mutually exclusive within the select.
- `we-button-group` — segmented toggle.
- `we-colorpicker` — color (with `data-select-style`/`data-css-property`/`data-color-prefix`).
- `we-input`, `we-checkbox`, `we-range`, `we-toggler`, `we-row` (group widgets in a row,
  `website/views/snippets/s_countdown.xml:34`, `s_countdown_options`), `we-many2one`, `we-list`.

### Button action data-attributes (what the button DOES)
- `data-select-class="x"` — toggle class `x` on the target. Mutually exclusive within the
  same select/group (`website/views/snippets/s_alert.xml:29-33`, `s_alert_options`).
- `data-select-style` + `data-css-property` — set an inline/computed style → method
  `selectStyle`.
- `data-select-data-attribute` / `data-attribute-name` — set a `data-*` attribute on the
  target → method `selectDataAttribute` (`website/views/snippets/s_dynamic_snippet.xml:42`, `website.s_dynamic_snippet_options_template`).
- `data-name` — id used by `data-trigger`/dependencies between widgets
  (`website/views/snippets/s_alert.xml:16`, `s_alert_options`, y `website/views/snippets/s_alert.xml:34`).

### JS class (`options.registry`) — only when you need logic

Verified pattern (`website/static/src/snippets/s_map/options.js:11-21`, `onBuilt`):
```js
/** @odoo-module **/
import options from '@web_editor/js/editor/snippets.options';

options.registry.Map = options.Class.extend({
    onBuilt() {                       // runs WHEN the snippet is dropped
        if (!this.$target[0].querySelector('.s_map_embedded')) {
            // ... initialize content, e.g. insert an iframe
        }
    },
    async selectDataAttribute(previewMode, widgetValue, params) {
        await this._super(...arguments);
        // react to a data-attribute option change
    },
});
```

Key methods (base in `web_editor/.../snippets.options.js`):
- `start()` — widget init (rarely needed for simple snippets).
- `onBuilt(options)` — **runs once when dropped** (`:3445`). Put drop-time init here, NOT
  in `start`.
- `cleanForSave()` — strip editor-only DOM before persisting (`:3502`, `:9345`).
- `selectClass(previewMode, widgetValue, params)` — base toggle-class handler (`:3540`).
- A custom button method is named after the attribute used in XML (`data-select-style` →
  `selectStyle`, custom `data-foo` handled in a method you name and reference).

`data-js="Foo"` with no `options.registry.Foo` → falls back to `options.Class`; custom
methods never fire.

## v19 — OWL components + `html_builder` Plugin

`options.registry`, `we-*` widgets and `inherit_id="website.snippet_options"` are GONE.
Options are OWL components registered through a Plugin.

Verified component (`website/static/src/builder/plugins/options/process_steps_option.js:3-17`, `connectorOptionParams`, v19):
```js
import { BaseOptionComponent } from "@html_builder/core/utils";

export class ProcessStepsOption extends BaseOptionComponent {
    static template = "website.ProcessStepsOption";   // OWL template (XML)
    static selector = ".s_process_steps";             // replaces data-selector
    setup() { super.setup(); /* ... */ }
}
```

Verified registration via Plugin (`odoo19/addons/website/static/src/builder/plugins/options/process_steps_option_plugin.js:1-44`):
```js
import { Plugin } from "@html_editor/plugin";
import { withSequence } from "@html_editor/utils/resource";
import { registry } from "@web/core/registry";

class ProcessStepsOptionPlugin extends Plugin {
    static id = "processStepsOption";
    resources = {
        builder_options: [ withSequence(after(CONTAINER_WIDTH), ProcessStepsOption) ],
        builder_actions: { ChangeConnectorAction, ChangeArrowColorAction },
        dropzone_selector: { selector: ".s_process_step", dropLockWithin: ".s_process_steps" },
    };
}
registry.category("builder-plugins").add(ProcessStepsOptionPlugin.id, ProcessStepsOptionPlugin);
```

v19 mapping:
- `data-selector` → `static selector` on the component.
- `we-*` widgets → markup in the component's OWL `static template` using `html_builder`
  building blocks.
- `selectClass`/`selectStyle` → `builder_actions` (e.g. `ClassAction`) declared in the
  Plugin `resources`.
- `data-drop-in`/`drop-near` → `dropzone_selector` in `resources`.

Never mix the two systems in one module/version.
