//! Whether the judge runs on its own. The profile's `judge.automatic`
//! switches every background judge spend; when it is off (development,
//! local), the Analysis pages draw the buttons that ask for a verdict, and
//! when it is on (production) verdicts simply arrive and the pages show
//! only the circles.

use systemprompt::config::ProfileBootstrap;

pub(crate) fn automatic() -> bool {
    ProfileBootstrap::get().is_ok_and(|p| p.judge.automatic)
}
