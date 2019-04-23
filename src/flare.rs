#[derive(PartialEq, PartialOrd, Debug)]
pub enum NodeValue {
    Dir { children: Vec<FlareTree> },
    File {},
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct FlareTree {
    name: String,
    value: NodeValue,
}

impl FlareTree {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn from_file(name: String) -> FlareTree {
        FlareTree {
            name: name,
            value: NodeValue::File {},
        }
    }
    pub fn from_dir(name: String) -> FlareTree {
        FlareTree {
            name: name,
            value: NodeValue::Dir {
                children: Vec::new(),
            },
        }
    }
    pub fn append_child(&mut self, child: FlareTree) {
        match self.value {
            NodeValue::Dir { ref mut children } => {
                children.push(child);
            }
            _ => {}
        }
    }

    pub fn get_in(&self, path: &[&str]) -> Option<&FlareTree> {
        let (first_name, remaining_names) = path.split_first()?;

        if let NodeValue::Dir { ref children } = self.value {
            let first_match = children.iter().filter(|c| &c.name == first_name).next();
            let first_match = first_match?;
            if path.len() == 1 {
                return Some(first_match);
            } else {
                return first_match.get_in(remaining_names);
            }
        };
        return None;
    }

        pub fn get_in_mut(&mut self, path: &[&str]) -> Option<&mut FlareTree> {
        let (first_name, remaining_names) = path.split_first()?;

        if let NodeValue::Dir { ref mut children } = self.value {
            let first_match = children.iter_mut().filter(|c| &c.name == first_name).next();
            let first_match = first_match?;
            if path.len() == 1 {
                return Some(first_match);
            } else {
                return first_match.get_in_mut(remaining_names);
            }
        };
        return None;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn can_build_tree() {
        let mut root = FlareTree::from_dir(String::from("root"));
        root.append_child(FlareTree::from_file(String::from("child")));

        assert_eq!(
            root,
            FlareTree {
                name: String::from("root"),
                value: NodeValue::Dir {
                    children: vec![FlareTree {
                        name: String::from("child"),
                        value: NodeValue::File {},
                    }]
                },
            }
        )
    }

    fn build_test_tree() -> FlareTree {
        let mut root = FlareTree::from_dir(String::from("root"));
        root.append_child(FlareTree::from_file(String::from("root_file_1.txt")));
        root.append_child(FlareTree::from_file(String::from("root_file_2.txt")));
        let mut child1 = FlareTree::from_dir(String::from("child1"));
        child1.append_child(FlareTree::from_file(String::from("child1_file_1.txt")));
        let mut grand_child = FlareTree::from_dir(String::from("grandchild"));
        grand_child.append_child(FlareTree::from_file(String::from("grandchild_file.txt")));
        child1.append_child(grand_child);
        child1.append_child(FlareTree::from_file(String::from("child1_file_2.txt")));
        let mut child2 = FlareTree::from_dir(String::from("child2"));
        child2.append_child(FlareTree::from_file(String::from("child2_file.txt")));
        root.append_child(child1);
        root.append_child(child2);
        root
    }

    #[test]
    fn can_get_elements_from_tree() {
        let tree = build_test_tree();
        let grandchild = tree.get_in(&vec!["child1", "grandchild", "grandchild_file.txt"]);
        assert_eq!(
            grandchild.expect("Grandchild not found!").name(),
            "grandchild_file.txt"
        );
    }

    #[test]
    fn cant_get_missing_elements_from_tree() {
        let tree = build_test_tree();
        let missing = tree.get_in(&vec!["child1", "grandchild", "nonesuch"]);
        assert_eq!(missing, None);
        let missing2 = tree.get_in(&vec!["child1", "grandchild", "grandchild_file.txt", "files have no kids"]);
        assert_eq!(missing2, None);
        let missing3 = tree.get_in(&vec![]);
        assert_eq!(missing3, None);
    }


    #[test]
    fn can_get_mut_elements_from_tree() {
        let mut tree = build_test_tree();
        let grandchild = tree.get_in_mut(&vec!["child1", "grandchild", "grandchild_file.txt"]).expect("Grandchild not found!");
        assert_eq!(
            grandchild.name(),
            "grandchild_file.txt"
        );
        grandchild.name = String::from("fish");
        let grandchild2 = tree.get_in_mut(&vec!["child1", "grandchild", "fish"]);
        assert_eq!(
            grandchild2.expect("fish not found!").name(),
            "fish"
        );

        let grandchild_dir = tree.get_in_mut(&vec!["child1", "grandchild"]).expect("Grandchild dir not found!");
        assert_eq!(
            grandchild_dir.name(),
            "grandchild"
        );
        grandchild_dir.append_child(FlareTree::from_file(String::from("new_kid_on_the_block.txt")));
        let new_kid = tree.get_in_mut(&vec!["child1", "grandchild", "new_kid_on_the_block.txt"]).expect("New kid not found!");
        assert_eq!(
            new_kid.name(),
            "new_kid_on_the_block.txt"
        );

    }
}
