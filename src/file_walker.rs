#![warn(clippy::all)]

use super::flare;
use super::flare::FlareTreeNode;
use failure::Error;
use ignore::{Walk, WalkBuilder};
use std::path::Path;

/// Wrapper for the logic that calculates toxicity indicators
pub trait ToxicityIndicatorCalculator: Sync + std::fmt::Debug {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn calculate(&mut self, path: &Path) -> Result<serde_json::Value, Error>;
}

fn walk_tree_walker(
    walker: Walk,
    prefix: &Path,
    toxicity_indicator_calculators: &mut Vec<Box<ToxicityIndicatorCalculator>>,
) -> Result<flare::FlareTreeNode, Error> {
    let mut tree = FlareTreeNode::from_dir("flare");

    for result in walker.map(|r| r.expect("File error!")).skip(1) {
        // note we skip the root directory!
        let p = result.path();
        let relative = p.strip_prefix(prefix)?;
        let new_child = if p.is_file() {
            let mut f = FlareTreeNode::from_file(p.file_name().unwrap());
            toxicity_indicator_calculators.iter_mut().for_each(|tic| {
                let indicators = tic.calculate(p);
                match indicators {
                    Ok(indicators) => f.add_data(tic.name().to_string(), indicators),
                    Err(error) => {
                        warn!(
                            "Can't find {} indicators for {:?} - cause: {}",
                            tic.name(),
                            p,
                            error
                        );
                    }
                }
            });
            Some(f)
        } else if p.is_dir() {
            Some(FlareTreeNode::from_dir(p.file_name().unwrap()))
        } else {
            warn!("Not a file or dir: {:?} - skipping", p);
            None
        };

        if let Some(new_child) = new_child {
            match relative.parent() {
                Some(new_parent) => {
                    let parent = tree
                        .get_in_mut(&mut new_parent.components())
                        .expect("no parent found!");
                    parent.append_child(new_child);
                }
                None => {
                    tree.append_child(new_child);
                }
            }
        }
    }
    Ok(tree)
}

pub fn walk_directory(
    root: &Path,
    toxicity_indicator_calculators: &mut Vec<Box<ToxicityIndicatorCalculator>>,
) -> Result<flare::FlareTreeNode, Error> {
    walk_tree_walker(
        WalkBuilder::new(root).build(),
        root,
        toxicity_indicator_calculators,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use serde_json::Value;

    #[test]
    fn scanning_a_filesystem_builds_a_tree() {
        let root = Path::new("./tests/data/simple/");
        let tree = walk_directory(root, &mut Vec::new()).unwrap();
        let json = serde_json::to_string_pretty(&tree).unwrap();
        let parsed_result: Value = serde_json::from_str(&json).unwrap();

        let expected =
            std::fs::read_to_string(Path::new("./tests/expected/simple_files.json")).unwrap();
        let parsed_expected: Value = serde_json::from_str(&expected).unwrap();

        assert_eq!(parsed_result, parsed_expected);
    }

    #[derive(Debug)]
    struct SimpleTIC {}

    impl ToxicityIndicatorCalculator for SimpleTIC {
        fn name(&self) -> String {
            "foo".to_string()
        }
        fn description(&self) -> String {
            "Foo".to_string()
        }
        fn calculate(&mut self, _path: &Path) -> Result<serde_json::Value, Error> {
            Ok(json!("bar"))
        }
    }

    #[derive(Debug)]
    struct SelfNamingTIC {}

    impl ToxicityIndicatorCalculator for SelfNamingTIC {
        fn name(&self) -> String {
            "filename".to_string()
        }
        fn description(&self) -> String {
            "Filename".to_string()
        }
        fn calculate(&mut self, path: &Path) -> Result<serde_json::Value, Error> {
            Ok(json!(path.to_str()))
        }
    }

    #[test]
    fn scanning_merges_data_from_mutators() {
        let root = Path::new("./tests/data/simple/");
        let simple_tic = SimpleTIC {};
        let self_naming_tic = SelfNamingTIC {};
        let calculators: &mut Vec<Box<ToxicityIndicatorCalculator>> =
            &mut vec![Box::new(simple_tic), Box::new(self_naming_tic)];

        let tree = walk_directory(root, calculators).unwrap();
        let json = serde_json::to_string_pretty(&tree).unwrap();
        let parsed_result: Value = serde_json::from_str(&json).unwrap();

        let expected =
            std::fs::read_to_string(Path::new("./tests/expected/simple_files_with_data.json"))
                .unwrap();
        let parsed_expected: Value = serde_json::from_str(&expected).unwrap();

        assert_eq!(parsed_result, parsed_expected);
    }

    #[derive(Debug)]
    struct MutableTIC {
        count: i64,
    }

    impl ToxicityIndicatorCalculator for MutableTIC {
        fn name(&self) -> String {
            "file count".to_string()
        }
        fn description(&self) -> String {
            "Mutable TIC".to_string()
        }
        fn calculate(&mut self, _path: &Path) -> Result<serde_json::Value, Error> {
            let result = json!(self.count);
            self.count += 1;
            Ok(result)
        }
    }

    #[test]
    fn can_mutate_state_of_calculator() {
        let root = Path::new("./tests/data/simple/");
        let tic = MutableTIC { count: 0 };
        let calculators: &mut Vec<Box<ToxicityIndicatorCalculator>> = &mut vec![Box::new(tic)];

        let tree = walk_directory(root, calculators).unwrap();
        let json = serde_json::to_string_pretty(&tree).unwrap();
        let parsed_result: Value = serde_json::from_str(&json).unwrap();

        let expected =
            std::fs::read_to_string(Path::new("./tests/expected/simple_files_with_counts.json"))
                .unwrap();
        let parsed_expected: Value = serde_json::from_str(&expected).unwrap();

        assert_eq!(parsed_result, parsed_expected);
    }
}
