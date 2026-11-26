//! One of the eighteen operations from a route: the arguments the route read
//! and checked, with the path's id already in them, go to [`Brain::run`] as
//! JSON, and the reply goes back as it came, from the native store or from
//! the source of a proxy instance.

use axum::Json;
use serde::Serialize;
// JSON: the reply of an operation, as the Brain answered it.
use serde_json::Value;

use super::error::HttpFault;
use super::state::BrainState;
use crate::auth::Caller;
use crate::brain::Brain;
use crate::brain::operation::Operation;

// JSON: the operation's reply.
pub async fn relay<A: Serialize + Sync>(
    state: &BrainState,
    caller: &Caller,
    operation: Operation,
    args: &A,
) -> Result<Json<Value>, HttpFault> {
    let args = serde_json::to_value(args)?;
    let brain: &dyn Brain = state.brain.as_ref();
    let reply = brain
        .run(&caller.0, &caller.context(), operation, args)
        .await?;
    Ok(Json(reply))
}
