//! Validated workflow transitions and review gates.

use crate::error::{RequirementsError, Result};
use crate::model::{
    Artifact, Evidence, Gate, Phase, Receipt, Review, VALIDATOR_VERSION, Workflow, WorkflowId,
    digest, valid_slug,
};
use std::collections::BTreeMap;
use uuid::Uuid;

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(RequirementsError::Validation(message.to_owned()))
    }
}

impl Workflow {
    pub fn new(name: String, delivery_scope: String, mut evidence: Vec<Evidence>) -> Result<Self> {
        require(valid_slug(&name), "Use a lowercase change slug")?;
        require(
            !delivery_scope.trim().is_empty() && delivery_scope.len() <= 4000,
            "Describe the delivery scope (1–4000 bytes)",
        )?;
        require(!evidence.is_empty(), "Complete evidence is required")?;
        evidence
            .sort_by(|a, b| (&a.source_type, &a.source_ref).cmp(&(&b.source_type, &b.source_ref)));
        for doc in &evidence {
            require(
                !doc.body.is_empty() && !doc.uri.is_empty(),
                "Source text and citation are required",
            )?;
        }
        require(
            evidence.windows(2).all(|w| {
                (w[0].source_type.as_str(), w[0].source_ref.as_str())
                    != (w[1].source_type.as_str(), w[1].source_ref.as_str())
            }),
            "Duplicate evidence",
        )?;
        Ok(Self {
            id: WorkflowId(Uuid::new_v4()),
            name,
            evidence_hash: digest(&evidence)?,
            evidence,
            delivery_scope,
            destination: None,
            validator_version: VALIDATOR_VERSION.to_owned(),
            artifacts: BTreeMap::new(),
            context_review: None,
            publication_review: None,
            publications: BTreeMap::new(),
        })
    }

    pub fn set_destination(&mut self, destination: crate::model::Destination) -> Result<()> {
        destination.validate()?;
        if self.destination.as_ref() != Some(&destination) {
            self.destination = Some(destination);
            self.publication_review = None;
        }
        Ok(())
    }

    pub fn dependency_hash(&self, phase: Phase) -> Result<String> {
        let hashes: Vec<_> = self
            .artifacts
            .iter()
            .filter(|(p, _)| **p < phase)
            .map(|(p, a)| (p, &a.hash))
            .collect();
        digest(&(
            &self.evidence_hash,
            &self.delivery_scope,
            &self.validator_version,
            hashes,
        ))
    }

    pub fn bundle_hash(&self, gate: Gate) -> Result<String> {
        let boundary = if gate == Gate::Context {
            Phase::Context
        } else {
            Phase::Specification
        };
        let artifacts: Vec<_> = self
            .artifacts
            .iter()
            .filter(|(p, _)| **p <= boundary)
            .collect();
        let destination = if gate == Gate::Publication {
            self.destination.as_ref()
        } else {
            None
        };
        digest(&(
            self.id,
            &self.evidence_hash,
            &self.delivery_scope,
            &self.validator_version,
            artifacts,
            destination,
        ))
    }

    pub fn require_predecessors(&self, phase: Phase) -> Result<()> {
        for earlier in Phase::ALL.into_iter().filter(|p| *p < phase) {
            self.require_validated(earlier)?;
        }
        if phase >= Phase::Specification {
            self.require_approved(Gate::Context)?;
        }
        Ok(())
    }

    pub fn require_validated(&self, phase: Phase) -> Result<()> {
        let artifact =
            self.artifacts
                .get(&phase)
                .ok_or_else(|| RequirementsError::MissingPhase {
                    phase: phase.name(),
                })?;
        let validation =
            artifact
                .validation
                .as_ref()
                .ok_or_else(|| RequirementsError::NotValidated {
                    phase: phase.name(),
                })?;
        let current = validation.passed
            && validation.artifact_hash == artifact.hash
            && validation.dependency_hash == self.dependency_hash(phase)?
            && validation.validator_version == self.validator_version;
        if current {
            Ok(())
        } else {
            Err(RequirementsError::StaleValidation {
                phase: phase.name(),
            })
        }
    }

    pub fn submit(&mut self, phase: Phase, files: BTreeMap<String, String>) -> Result<()> {
        self.require_predecessors(phase)?;
        require(
            !files.is_empty() && files.len() <= 64,
            "Supply 1–64 artifact files",
        )?;
        require(files.len() == 1, "This phase owns one document")?;
        require(
            files
                .iter()
                .all(|(p, b)| phase.accepts_path(p) && !b.trim().is_empty() && b.len() <= 512_000),
            "Invalid artifact path, empty content or size limit exceeded",
        )?;
        let hash = digest(&files)?;
        if self.artifacts.get(&phase).is_some_and(|a| a.hash == hash) {
            return Ok(());
        }
        let dependency_hash = self.dependency_hash(phase)?;
        self.artifacts.retain(|p, _| *p < phase);
        self.artifacts.insert(
            phase,
            Artifact {
                files,
                hash,
                dependency_hash,
                validation: None,
            },
        );
        if phase <= Phase::Context {
            self.context_review = None;
        }
        self.publication_review = None;
        Ok(())
    }

    pub fn request_review(&mut self, gate: Gate) -> Result<()> {
        let last = if gate == Gate::Context {
            Phase::Context
        } else {
            Phase::Specification
        };
        for phase in Phase::ALL.into_iter().filter(|p| *p <= last) {
            self.require_validated(phase)?;
        }
        if gate == Gate::Publication {
            self.destination
                .as_ref()
                .ok_or(RequirementsError::DestinationRequired)?
                .validate()?;
            self.require_approved(Gate::Context)?;
        }
        let hash = self.bundle_hash(gate)?;
        let slot = match gate {
            Gate::Context => &mut self.context_review,
            Gate::Publication => &mut self.publication_review,
        };
        if slot
            .as_ref()
            .is_some_and(|r| r.bundle_hash == hash && r.rejection.is_none())
        {
            return Ok(());
        }
        *slot = Some(Review {
            gate,
            bundle_hash: hash,
            receipt: None,
            rejection: None,
        });
        Ok(())
    }

    pub fn require_approved(&self, gate: Gate) -> Result<&Receipt> {
        let review = match gate {
            Gate::Context => &self.context_review,
            Gate::Publication => &self.publication_review,
        };
        let review = review.as_ref().ok_or(RequirementsError::ReviewRequired)?;
        if !(review.bundle_hash == self.bundle_hash(gate)? && review.rejection.is_none()) {
            return Err(RequirementsError::ApprovalStale);
        }
        review
            .receipt
            .as_ref()
            .ok_or(RequirementsError::AwaitingApproval)
    }

    pub fn review_mut(&mut self, gate: Gate) -> Result<&mut Review> {
        let hash = self.bundle_hash(gate)?;
        let review = match gate {
            Gate::Context => &mut self.context_review,
            Gate::Publication => &mut self.publication_review,
        };
        let review = review
            .as_mut()
            .ok_or(RequirementsError::ReviewNotRequested)?;
        if review.bundle_hash != hash || review.receipt.is_some() || review.rejection.is_some() {
            return Err(RequirementsError::ReviewDecided);
        }
        Ok(review)
    }
}
