//! Which model answers a request, and which models a chat can pick.
//!
//! [`ModelSource`] is the seam the chats state holds: production is
//! [`Gateway`] over the instance's AI service and [`Admission`], a suite brings
//! its own.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::ai::AiService;

use crate::admission::Admission;
use crate::auth::Caller;
use crate::catalog::Listing;
use crate::error::ChatsError;
use crate::reply::model::ReplyModel;
use crate::store::ModelRef;

/// Which model answers a request, and which models a chat can pick.
///
/// Core hands each request the instance's AI service (wrapped as a
/// [`GatewayModel`] by the handler, `None` when the instance has none);
/// production answers with it, a suite with its fake. `list` is `None` when
/// there is no AI service to list from. `#[async_trait]` because the chats
/// state holds it as `Arc<dyn ModelSource>`.
#[async_trait]
pub trait ModelSource: Send + Sync + fmt::Debug {
    fn pick(&self, gateway: Option<Arc<dyn ReplyModel>>) -> Option<Arc<dyn ReplyModel>>;

    async fn list(
        &self,
        caller: &Caller,
        ai: Option<&AiService>,
    ) -> Result<Option<Listing>, ChatsError>;

    async fn admits(
        &self,
        caller: &Caller,
        ai: Option<&AiService>,
        model: &ModelRef,
    ) -> Result<bool, ChatsError> {
        let listing = self
            .list(caller, ai)
            .await?
            .ok_or(ChatsError::AiUnavailable)?;
        Ok(listing.contains(model))
    }
}

/// [`ModelSource`] of a real instance: the AI service of the request and its
/// catalog, and the instance's rules for who may use what ([`Admission`]).
#[derive(Debug)]
pub struct Gateway {
    admission: Admission,
}

impl Gateway {
    pub const fn new(admission: Admission) -> Self {
        Self { admission }
    }
}

#[async_trait]
impl ModelSource for Gateway {
    fn pick(&self, gateway: Option<Arc<dyn ReplyModel>>) -> Option<Arc<dyn ReplyModel>> {
        gateway
    }

    async fn list(
        &self,
        caller: &Caller,
        ai: Option<&AiService>,
    ) -> Result<Option<Listing>, ChatsError> {
        match ai {
            Some(ai) => self.admission.listing(ai.catalog(), caller).await.map(Some),
            None => Ok(None),
        }
    }
}
