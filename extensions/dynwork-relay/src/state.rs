//! What every relay route shares, built once per process.

use std::ops::Deref;
use std::sync::Arc;

use systemprompt::models::auth::TokenAuthority;
use systemprompt::traits::BackgroundTasks;
use tokio::sync::Semaphore;

use crate::audit::{AuditSink, TracingAudit};
use crate::auth::{CredentialVerifier, DeviceVerifier};
use crate::clock::{Clock, SystemClock};
use crate::config::{LimitsError, RelayLimits};
use crate::hub::Hub;
use crate::store::RelayStore;

#[derive(Debug)]
pub struct Shared {
    pub store: Arc<dyn RelayStore>,
    pub hub: Arc<Hub>,
    pub authority: Arc<dyn TokenAuthority>,
    pub device_verifiers: Vec<Arc<dyn DeviceVerifier>>,
    pub limits: RelayLimits,
    pub tasks: BackgroundTasks,
    pub audit: Arc<dyn AuditSink>,
    pub clock: Arc<dyn Clock>,
    pub tunnels: Arc<Semaphore>,
}

#[derive(Debug, Clone)]
pub struct RelayState(Arc<Shared>);

impl Deref for RelayState {
    type Target = Shared;

    fn deref(&self) -> &Shared {
        &self.0
    }
}

impl RelayState {
    pub fn builder(
        store: Arc<dyn RelayStore>,
        authority: Arc<dyn TokenAuthority>,
        tasks: BackgroundTasks,
    ) -> RelayStateBuilder {
        RelayStateBuilder {
            store,
            authority,
            tasks,
            limits: RelayLimits::default(),
            audit: Arc::new(TracingAudit),
            clock: Arc::new(SystemClock),
            extra_verifiers: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct RelayStateBuilder {
    store: Arc<dyn RelayStore>,
    authority: Arc<dyn TokenAuthority>,
    tasks: BackgroundTasks,
    limits: RelayLimits,
    audit: Arc<dyn AuditSink>,
    clock: Arc<dyn Clock>,
    extra_verifiers: Vec<Arc<dyn DeviceVerifier>>,
}

impl RelayStateBuilder {
    pub const fn limits(mut self, limits: RelayLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn audit(mut self, audit: Arc<dyn AuditSink>) -> Self {
        self.audit = audit;
        self
    }

    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    pub fn device_verifier(mut self, verifier: Arc<dyn DeviceVerifier>) -> Self {
        self.extra_verifiers.push(verifier);
        self
    }

    pub fn build(self) -> Result<RelayState, LimitsError> {
        self.limits.validate()?;
        let mut device_verifiers: Vec<Arc<dyn DeviceVerifier>> =
            vec![Arc::new(CredentialVerifier::new(Arc::clone(&self.store)))];
        device_verifiers.extend(self.extra_verifiers);
        Ok(RelayState(Arc::new(Shared {
            hub: Arc::new(Hub::new(self.limits.ring, self.limits.outbound_queue)),
            tunnels: Arc::new(Semaphore::new(self.limits.max_tunnels)),
            store: self.store,
            authority: self.authority,
            device_verifiers,
            limits: self.limits,
            tasks: self.tasks,
            audit: self.audit,
            clock: self.clock,
        })))
    }
}
