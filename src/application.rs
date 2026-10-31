//! Selection remains revision-bound; source and result geometry are distinct targets.
use crate::{
    document::Revision,
    engine::{EvaluationResult, Instance},
};
use anyhow::{Result, bail};

#[derive(Clone, Copy, Debug, PartialEq)]
/// Whether selection addresses final results or inspectable generator geometry.
pub enum SelectionMode {
    Result,
    Origin,
}

/// M0 one-result coordinator. Late results cannot overwrite a newer document state.
#[derive(Default)]
pub struct ModelState {
    pub result: Option<EvaluationResult>,
}
impl ModelState {
    pub fn accept(&mut self, current: Revision, result: EvaluationResult) -> bool {
        if current != result.revision {
            return false;
        }
        self.result = Some(result);
        true
    }
    pub fn select(&self, current: Revision, id: u64, mode: SelectionMode) -> Result<&Instance> {
        let result = self
            .result
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no evaluation"))?;
        if result.revision != current {
            bail!("stale_target");
        }
        let node = result
            .manifest
            .nodes
            .iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("unknown instance"))?;
        if mode == SelectionMode::Result && node.subtracted {
            bail!("operand requires origin view");
        }
        Ok(node)
    }
}
