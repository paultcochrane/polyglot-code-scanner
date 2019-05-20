#![warn(clippy::all)]

extern crate ignore;
extern crate tokei;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate failure_tools;

use failure::Error;
use std::io;
use std::path::PathBuf;

mod file_walker;
mod flare;
mod loc;

use file_walker::ToxicityIndicatorCalculator;
use loc::LocCalculator;

// TODO: would love to somehow calculate this from the types (via macro?) but for now this is manual:
#[allow(dead_code)]
const TOXICITY_INDICATOR_CALCULATOR_NAMES: &[&str] = &["loc"];

pub fn named_toxicity_indicator_calculator(
    name: &str,
) -> Option<Box<dyn ToxicityIndicatorCalculator>> {
    match name {
        "loc" => Some(Box::new(LocCalculator {})),
        _ => None,
    }
}

pub fn run<W>(
    root: PathBuf,
    toxicity_indicator_calculator_names: Vec<String>,
    out: W,
) -> Result<(), Error>
where
    W: io::Write,
{
    let maybe_tics: Option<Vec<_>> = toxicity_indicator_calculator_names
        .iter()
        .map(|name| named_toxicity_indicator_calculator(name))
        .collect();

    let mut tics = maybe_tics.expect("Some toxicity indicator calculator names don't exist!");

    let tree = file_walker::walk_directory(&root, &mut tics)?;

    serde_json::to_writer_pretty(out, &tree)?;
    Ok(())
}