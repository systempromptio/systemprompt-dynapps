//! Whether the judge runs on its own. The profile's `judge.automatic`
//! switches every background judge spend; when it is off (development,
//! local), the Analysis pages draw the buttons that ask for a verdict, and
//! when it is on (production) verdicts simply arrive and the pages show
//! only the circles.

use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

pub(crate) fn automatic(boot: &ExtensionBoot) -> bool {
    boot.profile().judge.automatic
}
