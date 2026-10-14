<!-- verified: v18.0 @ 2026-06 -->
# JavaScript — Hooks Reference (Odoo 18.0)

Odoo provides 70+ custom hooks on top of OWL's native hooks. All must be called inside `setup()`.

## Essential hooks

```javascript
import { useService } from "@web/core/utils/hooks";
import { useBus } from "@web/core/utils/hooks";
import { useAutofocus } from "@web/core/utils/hooks";

setup() {
    // Access any registered service
    this.orm = useService("orm");
    this.action = useService("action");
    this.notification = useService("notification");
    this.dialog = useService("dialog");
    this.company = useService("company");
    this.http = useService("http");
    this.hotkey = useService("hotkey");
    // NOTE: `user` is NOT a service in v18 → import { user } from "@web/core/user";
    //       `rpc`  is NOT a service in v18 → import { rpc } from "@web/core/network/rpc";

    // Listen to event bus (auto-cleanup on unmount)
    useBus(this.env.bus, "SOME_EVENT", (ev) => this.onEvent(ev));

    // Auto-focus element on mount
    useAutofocus({ refName: "myInput" });
}
```

## UI hooks

```javascript
import { useHotkey } from "@web/core/hotkeys/hotkey_hook";
import { usePopover } from "@web/core/popover/popover_hook";
import { useTooltip } from "@web/core/tooltip/tooltip_hook";
import { useCommand } from "@web/core/commands/command_hook";
import { useDebounced, useThrottleForAnimation } from "@web/core/utils/timing";
import { useOwnedDialogs } from "@web/core/utils/hooks";
import { useDropdownState } from "@web/core/dropdown/dropdown_hooks";
import { useSortable } from "@web/core/utils/sortable_owl";
import { usePosition } from "@web/core/position/position_hook";
import { useTransition } from "@web/core/transition";

setup() {
    useHotkey("ctrl+s", () => this.save());
    this.popover = usePopover(MyPopoverComponent, { position: "bottom" });
    this.debouncedSearch = useDebounced(this.search, 300);
    useSortable({ ref: this.rootRef, elements: ".item", onDrop: this.onReorder });
}
```

## View/Form hooks

```javascript
import { useSetupAction } from "@web/search/action_hook";
import { usePager } from "@web/search/pager_hook";
import { useModel } from "@web/model/model";
import { useViewButtons } from "@web/views/view_button/view_button_hook";
import { useRecordObserver } from "@web/model/relational_model/utils";

setup() {
    useSetupAction({ rootRef: this.rootRef, beforeLeave: () => this.save() });
    this.model = useModel(RelationalModel, params);
    useRecordObserver((record) => { /* react to record changes */ });
}
```

## Field hooks

```javascript
import { useInputField } from "@web/views/fields/input_field_hook";
import { useNumpadDecimal } from "@web/views/fields/numpad_decimal_hook";
import { useTranslationDialog } from "@web/views/fields/translation_button";
import { useFileViewer } from "@web/core/file_viewer/file_viewer_hook";
import { useFileUploader } from "@web/core/utils/files";
```
