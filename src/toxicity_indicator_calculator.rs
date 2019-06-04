#![warn(clippy::all)]

use failure::Error;
use serde_json::Value;
use std::path::Path;

/// Wrapper for the logic that calculates toxicity indicators
pub trait ToxicityIndicatorCalculator: Sync + std::fmt::Debug {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn calculate(&mut self, path: &Path) -> Result<Option<Value>, Error>;
}