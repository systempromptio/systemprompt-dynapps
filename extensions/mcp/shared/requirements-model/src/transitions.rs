//! Validated workflow transitions and review gates.

use crate::model::{
    Artifact, Evidence, Gate, Phase, RUBRIC_COMMIT, Receipt, Review, Workflow, WorkflowId, digest,
    valid_slug,
};
use anyhow::{Result, bail, ensure};
use std::collections::BTreeMap;
use uuid::Uuid;

impl Workflow {
    pub fn new(name: String, mut evidence: Vec<Evidence>) -> Result<Self> {
        ensure!(valid_slug(&name), "Use a lowercase change slug");
        ensure!(!evidence.is_empty(), "Complete evidence is required");
        evidence
            .sort_by(|a, b| (&a.source_type, &a.source_ref).cmp(&(&b.source_type, &b.source_ref)));
        for doc in &evidence {
            ensure!(
                !doc.body.is_empty() && !doc.uri.is_empty(),
                "Source text and citation are required"
            );
        }
        ensure!(
            evidence
                .windows(2)
                .all(|w| (w[0].source_type.as_str(), w[0].source_ref.as_str())
                    != (w[1].source_type.as_str(), w[1].source_ref.as_str())),
            "Duplicate evidence"
        );
        Ok(Self {
            id: WorkflowId(Uuid::new_v4()),
            name,
            evidence_hash: digest(&evidence)?,
            evidence,
            rubric_commit: RUBRIC_COMMIT.to_owned(),
            artifacts: BTreeMap::new(),
            grooming_review: None,
            publication_review: None,
            publications: BTreeMap::new(),
        })
    }

    pub fn dependency_hash(&self, phase: Phase) -> Result<String> {
        let hashes: Vec<_> = self
            .artifacts
            .iter()
            .filter(|(p, _)| **p < phase)
            .map(|(p, a)| (p, &a.hash))
            .collect();
        digest(&(&self.evidence_hash, &self.rubric_commit, hashes))
    }

    pub fn bundle_hash(&self, gate: Gate) -> Result<String> {
        let boundary = if gate == Gate::Grooming {
            Phase::Grooming
        } else {
            Phase::Fsd
        };
        let artifacts: Vec<_> = self
            .artifacts
            .iter()
            .filter(|(p, _)| **p <= boundary)
            .collect();
        digest(&(self.id, &self.evidence_hash, &self.rubric_commit, artifacts))
    }

    pub fn require_predecessors(&self, phase: Phase) -> Result<()> {
        for earlier in Phase::ALL.into_iter().filter(|p| *p < phase) {
            self.require_validated(earlier)?;
        }
        if phase >= Phase::Specs {
            self.require_approved(Gate::Grooming)?;
        }
        Ok(())
    }

    pub fn require_validated(&self, phase: Phase) -> Result<()> {
        let artifact = self
            .artifacts
            .get(&phase)
            .ok_or_else(|| anyhow::anyhow!("Missing {}", phase.name()))?;
        let validation = artifact
            .validation
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Validate {} first", phase.name()))?;
        ensure!(
            validation.passed
                && validation.artifact_hash == artifact.hash
                && validation.dependency_hash == self.dependency_hash(phase)?
                && validation.rubric_commit == self.rubric_commit,
            "{} has failed or stale validation",
            phase.name()
        );
        Ok(())
    }

    pub fn submit(&mut self, phase: Phase, files: BTreeMap<String, String>) -> Result<()> {
        self.require_predecessors(phase)?;
        ensure!(
            !files.is_empty() && files.len() <= 64,
            "Supply 1–64 artifact files"
        );
        ensure!(
            matches!(phase, Phase::Specs | Phase::Fsd) || files.len() == 1,
            "This phase owns one document"
        );
        ensure!(
            files
                .iter()
                .all(|(p, b)| phase.accepts_path(p) && !b.trim().is_empty() && b.len() <= 512_000),
            "Invalid artifact path, empty content or size limit exceeded"
        );
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
        if phase <= Phase::Grooming {
            self.grooming_review = None;
        }
        self.publication_review = None;
        Ok(())
    }

    pub fn request_review(&mut self, gate: Gate) -> Result<()> {
        let last = if gate == Gate::Grooming {
            Phase::Grooming
        } else {
            Phase::Fsd
        };
        for phase in Phase::ALL.into_iter().filter(|p| *p <= last) {
            self.require_validated(phase)?;
        }
        if gate == Gate::Publication {
            self.require_approved(Gate::Grooming)?;
        }
        let hash = self.bundle_hash(gate)?;
        let slot = match gate {
            Gate::Grooming => &mut self.grooming_review,
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
            Gate::Grooming => &self.grooming_review,
            Gate::Publication => &self.publication_review,
        };
        let review = review
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Human review is required"))?;
        ensure!(
            review.bundle_hash == self.bundle_hash(gate)? && review.rejection.is_none(),
            "Approval is stale or rejected"
        );
        review
            .receipt
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Awaiting human approval"))
    }

    pub fn review_mut(&mut self, gate: Gate) -> Result<&mut Review> {
        let hash = self.bundle_hash(gate)?;
        let review = match gate {
            Gate::Grooming => &mut self.grooming_review,
            Gate::Publication => &mut self.publication_review,
        };
        let review = review
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Request review first"))?;
        if review.bundle_hash != hash || review.receipt.is_some() || review.rejection.is_some() {
            bail!("Review is stale or already decided");
        }
        Ok(review)
    }
}
