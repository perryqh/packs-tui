use crate::pks_tree_node::{
    pack_name_to_node_names, prepend_dot_to_path, PksTreeBuilder, PksTreeNode,
};
use packs::packs::configuration::Configuration;
use packs::packs::pack::Pack;
use std::path::PathBuf;
use std::rc::Rc;
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

/// Pks is the primary data structure for the packs client.
/// It contains all the data needed to render the UI.
/// It is populated by the packs crate.
///
/// Reasoning for collections of the form <Arc<Vec<Arc<T>>>:
/// - Arc<Vec<Arc<T>>> is used instead of Vec<Arc<T>> because the latter
///  would require cloning the entire vector when passing it to a component.
/// - The inner Arc is used to allow for multiple components to share the same
/// data without cloning it.
/// - Arc is used in instead of Rc because the data is shared across threads.
///
pub struct Pks {
    configuration: Configuration,
    packs: Option<Arc<Vec<Arc<Pack>>>>,
    num_packs: Option<usize>,
    pack_dependents: Option<HashMap<String, Arc<BTreeSet<String>>>>,
    pack_dependent_violations: Option<Arc<Vec<Arc<PackDependentViolation>>>>,
    constant_violations: Option<Arc<Vec<Arc<ConstantViolation>>>>,
    pks_tree_data: Option<Rc<Vec<PksTreeNode>>>,
    path_violations: Option<Arc<HashMap<String, Arc<PathViolations>>>>,
}

#[derive(Debug, PartialEq)]
pub struct PackDependentViolation {
    pub defining_pack_name: String,
    pub referencing_pack_name: String,
    pub violation_type_counts: HashMap<String, usize>,
    pub constant_counts: HashMap<String, usize>,
}

#[derive(Debug, PartialEq)]
pub struct ConstantViolation {
    pub defining_pack_name: String,
    pub constant: String,
    pub count: usize,
    pub violation_type_counts: HashMap<String, usize>,
    pub referencing_pack_counts: HashMap<String, usize>,
}

#[derive(Debug, PartialEq, Default)]
pub struct PathViolations {
    pub path: String,
    pub uncontained_in_violations: Vec<Arc<PackDependentViolation>>,
    pub uncontained_out_violations: Vec<Arc<PackDependentViolation>>,
    pub contained_in_violations: Vec<Arc<PackDependentViolation>>,
    pub contained_out_violations: Vec<Arc<PackDependentViolation>>,

    pub uncontained_in_violations_count: usize,
    pub uncontained_out_violations_count: usize,
    pub contained_in_violations_count: usize,
    pub contained_out_violations_count: usize,
}

impl PathViolations {
    pub fn set_violations_count(&mut self) {
        self.uncontained_in_violations_count =
            PathViolations::sum_counts(&self.uncontained_in_violations);
        self.uncontained_out_violations_count =
            PathViolations::sum_counts(&self.uncontained_out_violations);
        self.contained_in_violations_count =
            PathViolations::sum_counts(&self.contained_in_violations);
        self.contained_out_violations_count =
            PathViolations::sum_counts(&self.contained_out_violations);
    }
    fn sum_counts(violations: &[Arc<PackDependentViolation>]) -> usize {
        violations
            .iter()
            .map(|violation| violation.all_violation_counts())
            .sum()
    }
}

impl Pks {
    pub fn new(path: Option<PathBuf>) -> Self {
        let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
        let configuration = packs::packs::configuration::get(path.as_path());

        Self {
            configuration,
            packs: None,
            pack_dependents: None,
            pack_dependent_violations: None,
            constant_violations: None,
            pks_tree_data: None,
            path_violations: None,
            num_packs: None,
        }
    }

    pub fn get_packs(&mut self) -> Arc<Vec<Arc<Pack>>> {
        if self.packs.is_none() {
            let mut packs = self
                .configuration
                .pack_set
                .packs
                .clone()
                .into_iter()
                .map(Arc::new)
                .collect::<Vec<Arc<Pack>>>();

            packs.sort_by(|a, b| a.name.cmp(&b.name));
            self.packs = Some(Arc::new(packs));
        }
        self.packs.as_ref().unwrap().clone()
    }

    pub fn get_num_packs(&mut self) -> usize {
        if self.num_packs.is_none() {
            self.num_packs = Some(self.get_packs().len());
        }
        self.num_packs.unwrap()
    }

    ///
    /// Key is pack name
    /// Value is a set of packs names that have the key pack name as a dependency
    pub fn get_pack_dependents(&mut self) -> HashMap<String, Arc<BTreeSet<String>>> {
        if self.pack_dependents.is_none() {
            let pack_dependents: HashMap<String, BTreeSet<String>> = self.get_packs().iter().fold(
                HashMap::new(),
                |mut map: HashMap<String, BTreeSet<String>>, pack| {
                    map.entry(pack.name.clone()).or_default(); // handles pack without dependents
                    for dependency_pack_name in pack.dependencies.iter() {
                        let entry: &mut BTreeSet<String> =
                            map.entry(dependency_pack_name.clone()).or_default();
                        entry.insert(pack.name.clone());
                    }
                    map
                },
            );
            let mut pack_arc_dependents: HashMap<String, Arc<BTreeSet<String>>> = HashMap::new();
            for (pack_name, dependents) in pack_dependents {
                pack_arc_dependents.insert(pack_name, Arc::new(dependents));
            }

            self.pack_dependents = Some(pack_arc_dependents);
        }
        self.pack_dependents.as_ref().unwrap().clone()
    }

    pub fn get_pack_dependent_violations(&mut self) -> Arc<Vec<Arc<PackDependentViolation>>> {
        if self.pack_dependent_violations.is_none() {
            let mut dependent_map: HashMap<(String, String), PackDependentViolation> =
                self.configuration.pack_set.all_violations.iter().fold(
                    HashMap::new(),
                    |mut map, violation| {
                        let defining_pack_name = violation.defining_pack_name.clone();
                        let referencing_pack_name = violation.referencing_pack_name.clone();
                        let key = (defining_pack_name.clone(), referencing_pack_name.clone());
                        let entry = map.entry(key).or_insert(PackDependentViolation {
                            defining_pack_name,
                            referencing_pack_name,
                            violation_type_counts: HashMap::new(),
                            constant_counts: HashMap::new(),
                        });
                        entry
                            .violation_type_counts
                            .entry(violation.violation_type.clone())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                        // TODO: constant counts are not accurate because there is a separate
                        // pks violation identifier for each type of violation. So each violation type
                        // is counted
                        entry
                            .constant_counts
                            .entry(violation.constant_name.clone())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                        map
                    },
                );
            let mut pack_dependent_violations: Vec<Arc<PackDependentViolation>> =
                dependent_map.drain().map(|(_, v)| Arc::new(v)).collect();
            pack_dependent_violations.sort_by(|a, b| {
                a.referencing_pack_name
                    .cmp(&b.referencing_pack_name)
                    .then(a.defining_pack_name.cmp(&b.defining_pack_name))
            });

            self.pack_dependent_violations = Some(Arc::new(pack_dependent_violations));
        }

        self.pack_dependent_violations.as_ref().unwrap().clone()
    }

    pub fn get_pack_dependent_violations_by_defining_pack_name(
        &mut self,
        defining_pack_name: &str,
    ) -> Vec<Arc<PackDependentViolation>> {
        let pack_dependent_violations = self.get_pack_dependent_violations();
        pack_dependent_violations
            .iter()
            .filter(|pack_dependent_violation| {
                pack_dependent_violation.defining_pack_name == defining_pack_name
            })
            .cloned()
            .collect()
    }

    pub fn get_constant_violations(&mut self) -> Arc<Vec<Arc<ConstantViolation>>> {
        if self.constant_violations.is_none() {
            let mut constant_map: HashMap<(String, String), ConstantViolation> =
                self.configuration.pack_set.all_violations.iter().fold(
                    HashMap::new(),
                    |mut map, violation| {
                        let defining_pack_name = violation.defining_pack_name.clone();
                        let constant = violation.constant_name.clone();
                        let key = (defining_pack_name.clone(), constant.clone());
                        let entry = map.entry(key).or_insert(ConstantViolation {
                            defining_pack_name,
                            constant,
                            count: 0,
                            violation_type_counts: HashMap::new(),
                            referencing_pack_counts: HashMap::new(),
                        });
                        entry.count += 1;
                        entry
                            .violation_type_counts
                            .entry(violation.violation_type.clone())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                        entry
                            .referencing_pack_counts
                            .entry(violation.referencing_pack_name.clone())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                        map
                    },
                );
            let mut constant_violations: Vec<Arc<ConstantViolation>> =
                constant_map.drain().map(|(_, v)| Arc::new(v)).collect();
            constant_violations.sort_by(|a, b| {
                a.constant
                    .cmp(&b.constant)
                    .then(a.defining_pack_name.cmp(&b.defining_pack_name))
            });

            self.constant_violations = Some(Arc::new(constant_violations));
        }

        self.constant_violations.as_ref().unwrap().clone()
    }

    pub fn get_pks_tree_data(&mut self) -> Rc<Vec<PksTreeNode>> {
        if self.pks_tree_data.is_none() {
            let pack_names = self.get_pack_names();
            let pks_tree_data =
                PksTreeBuilder::new(pack_names, self.get_path_violations()).children;
            self.pks_tree_data = Some(Rc::new(pks_tree_data));
        }
        self.pks_tree_data.as_ref().unwrap().clone()
    }

    fn get_pack_names(&mut self) -> Vec<String> {
        self.get_packs()
            .iter()
            .map(|pack| pack.name.clone())
            .collect()
    }

    pub fn get_path_violations_for_path(&mut self, path: &str) -> Option<Arc<PathViolations>> {
        self.get_path_violations().get(path).cloned()
    }
    pub fn get_path_violations(&mut self) -> Arc<HashMap<String, Arc<PathViolations>>> {
        if self.path_violations.is_none() {
            let mut map: HashMap<String, PathViolations> = HashMap::new();
            let all_violations = self.get_pack_dependent_violations();
            for violation in all_violations.iter() {
                let defining_pack_name = prepend_dot_to_path(&violation.defining_pack_name);
                let referencing_pack_name = prepend_dot_to_path(&violation.referencing_pack_name);
                for (part, _) in pack_name_to_node_names(&defining_pack_name) {
                    if part_contained_in_other_path(&part, &referencing_pack_name) {
                        map.entry(part.to_string())
                            .or_insert_with(|| PathViolations {
                                path: part.to_string(),
                                ..Default::default()
                            })
                            .contained_in_violations
                            .push(violation.clone());
                    } else {
                        map.entry(part.to_string())
                            .or_insert_with(|| PathViolations {
                                path: part.to_string(),
                                ..Default::default()
                            })
                            .uncontained_in_violations
                            .push(violation.clone());
                    }
                }

                for (part, _) in pack_name_to_node_names(&referencing_pack_name) {
                    if part_contained_in_other_path(&part, &defining_pack_name) {
                        map.entry(part.to_string())
                            .or_insert_with(|| PathViolations {
                                path: part.to_string(),
                                ..Default::default()
                            })
                            .contained_out_violations
                            .push(violation.clone());
                    } else {
                        map.entry(part.to_string())
                            .or_insert_with(|| PathViolations {
                                path: part.to_string(),
                                ..Default::default()
                            })
                            .uncontained_out_violations
                            .push(violation.clone());
                    }
                }
            }
            self.path_violations = Some(Arc::new(map.drain().fold(
                HashMap::new(),
                |mut collect_map, (name, mut violation)| {
                    violation.set_violations_count();
                    collect_map.insert(name, Arc::new(violation));
                    collect_map
                },
            )));
        }

        self.path_violations.as_ref().unwrap().clone()
    }
}

fn part_contained_in_other_path(part: &str, path: &String) -> bool {
    if part == path {
        return true;
    }
    let part = format!("{}/", part);
    path.starts_with(&part)
}

impl PackDependentViolation {
    pub fn count_for_violation_type(&self, violation_type: &str) -> usize {
        match self.violation_type_counts.get(violation_type) {
            Some(count) => *count,
            None => 0,
        }
    }

    pub fn all_violation_counts(&self) -> usize {
        self.violation_type_counts.values().sum()
    }

    pub fn num_constants(&self) -> usize {
        self.constant_counts.len()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn new_pks() -> Pks {
        Pks::new(Some(
            PathBuf::from("../tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path"),
        ))
    }

    fn new_pks_with_violations() -> Pks {
        Pks::new(Some(
            PathBuf::from("../tests/fixtures/contains_stale_violations")
                .canonicalize()
                .expect("Could not canonicalize path"),
        ))
    }

    #[test]
    fn test_new_pks_values() {
        let pks = new_pks();
        assert_eq!(pks.configuration.pack_set.packs.len(), 4);
        assert_eq!(pks.packs, None);
        assert_eq!(pks.pack_dependents, None);
        assert_eq!(pks.pack_dependent_violations, None);
        assert_eq!(pks.constant_violations, None);
    }

    #[test]
    fn test_get_packs() {
        let mut pks = new_pks();
        let packs = pks.get_packs();
        let pack_names: Vec<String> = packs.iter().map(|pack| pack.name.clone()).collect();
        assert_eq!(pack_names, vec![".", "packs/bar", "packs/baz", "packs/foo"]);
    }

    #[test]
    fn test_get_pack_dependents() {
        let mut pks = new_pks();
        let pack_dependents = pks.get_pack_dependents();
        assert_eq!(pack_dependents.len(), 4);
        assert_eq!(pack_dependents.get(".").unwrap().len(), 0);
        assert_eq!(pack_dependents.get("packs/baz").unwrap().len(), 1);
        assert!(pack_dependents
            .get("packs/baz")
            .unwrap()
            .contains("packs/foo"));
    }

    #[test]
    fn test_get_pack_dependent_violations() {
        let mut pks = new_pks_with_violations();
        let pack_dependent_violations = pks.get_pack_dependent_violations();
        assert_eq!(pack_dependent_violations.len(), 2);
        assert_eq!(pack_dependent_violations[0].defining_pack_name, "packs/foo");
        let bar_violation = pack_dependent_violations[1].clone();
        assert_eq!(bar_violation.defining_pack_name, "packs/bar");
        assert_eq!(bar_violation.referencing_pack_name, "packs/foo");
        assert_eq!(bar_violation.violation_type_counts.len(), 2);
        assert_eq!(bar_violation.count_for_violation_type("privacy"), 1);
        assert_eq!(bar_violation.constant_counts.len(), 1);
        assert_eq!(bar_violation.constant_counts.get("::Bar").unwrap(), &2);
        assert_eq!(bar_violation.num_constants(), 1);
    }

    #[test]
    fn test_part_contained_in_other_path() {
        assert!(part_contained_in_other_path(
            "./foo",
            &"./foo/bar".to_string()
        ));
        assert!(part_contained_in_other_path(
            "./foo",
            &"./foo/bar/baz".to_string()
        ));
        assert!(!part_contained_in_other_path("./foo", &"./bar".to_string()));
        assert!(!part_contained_in_other_path(
            "./foo",
            &"./bar/foo".to_string()
        ));
        assert!(part_contained_in_other_path("./foo", &"./foo".to_string()));
    }

    #[test]
    fn test_get_path_violations() {
        let mut pks = new_pks_with_violations();
        let path_violations = pks.get_path_violations();
        assert_eq!(path_violations.len(), 4);
        let foo_violation = path_violations.get("./packs/foo").unwrap();
        assert_eq!(foo_violation.path, "./packs/foo");
        assert_eq!(foo_violation.uncontained_in_violations.len(), 1);
        assert_eq!(
            foo_violation.uncontained_in_violations[0].defining_pack_name,
            String::from("packs/foo")
        );
        assert_eq!(
            foo_violation.uncontained_in_violations[0].referencing_pack_name,
            String::from("packs/bar")
        );
        assert_eq!(foo_violation.uncontained_out_violations.len(), 1);
        assert_eq!(foo_violation.contained_in_violations.len(), 0);
        assert_eq!(foo_violation.contained_out_violations.len(), 0);

        let packs_violation = path_violations.get("./packs").unwrap();
        assert_eq!(packs_violation.contained_in_violations_count, 4);
        assert_eq!(packs_violation.contained_out_violations_count, 4);
        assert_eq!(packs_violation.uncontained_out_violations_count, 0);
        assert_eq!(packs_violation.uncontained_in_violations_count, 0);
    }

    #[test]
    fn test_get_pack_dependent_violations_by_defining_pack_name() {
        let mut pks = new_pks_with_violations();
        let pack_dependent_violations =
            pks.get_pack_dependent_violations_by_defining_pack_name("packs/foo");
        assert_eq!(pack_dependent_violations.len(), 1);
        let violation = pack_dependent_violations[0].clone();
        assert_eq!(violation.defining_pack_name, "packs/foo");
        assert_eq!(violation.referencing_pack_name, "packs/bar");
        assert_eq!(violation.constant_counts.len(), 1);
        assert_eq!(violation.constant_counts.get("::Foo").unwrap(), &2);
        assert_eq!(violation.violation_type_counts.len(), 2);
        assert_eq!(violation.violation_type_counts.get("privacy").unwrap(), &1);
    }

    #[test]
    fn test_get_constant_violations() {
        let mut pks = new_pks_with_violations();
        let constant_violations = pks.get_constant_violations();
        assert_eq!(constant_violations.len(), 2);
        assert_eq!(constant_violations[0].defining_pack_name, "packs/bar");
        let bar_violation = constant_violations[0].clone();
        assert_eq!(bar_violation.constant, "::Bar");
        assert_eq!(bar_violation.count, 2);
        assert_eq!(bar_violation.violation_type_counts.len(), 2);
        assert_eq!(
            bar_violation.violation_type_counts.get("privacy").unwrap(),
            &1
        );
        assert_eq!(bar_violation.referencing_pack_counts.len(), 1);
        assert_eq!(
            bar_violation
                .referencing_pack_counts
                .get("packs/foo")
                .unwrap(),
            &2
        );
    }

    #[test]
    fn test_get_pks_tree_data() {
        let mut pks = new_pks();
        let pks_tree_data = pks.get_pks_tree_data();
        assert_eq!(pks_tree_data.len(), 1);
        let root = &pks_tree_data.iter().find(|node| node.path == ".").unwrap();
        assert!(root.children.is_some());
        assert!(root.has_package_definition);
        let packs = root
            .children
            .as_ref()
            .unwrap()
            .iter()
            .find(|node| node.path == "./packs")
            .unwrap();
        assert!(packs.children.is_some());
        assert!(!packs.has_package_definition);
        let bar = packs
            .children
            .as_ref()
            .unwrap()
            .iter()
            .find(|node| node.path == "./packs/bar")
            .unwrap();
        assert!(bar.children.is_none());
        assert!(bar.has_package_definition);
    }

    // fn new_pks_with_service_violations() -> Pks {
    //     Pks::new(Some(
    //         PathBuf::from("../tests/fixtures/services_contains_stale_violations")
    //             .canonicalize()
    //             .expect("Could not canonicalize path"),
    //     ))
    // }
    // #[test]
    // fn test_get_pks_tree_data_with_violations() {
    //     let mut pks = new_pks_with_service_violations();
    //     let pks_tree_data = pks.get_pks_tree_data();
    //     dbg!(&pks_tree_data);
    //     assert!(false);
    // }
    //
    // #[test]
    // fn test_get_pack_dependent_service_violations() {
    //     let mut pks = new_pks_with_service_violations();
    //     let pack_dependent_violations = pks.get_pack_dependent_violations();
    //     dbg!(&pack_dependent_violations);
    //     assert!(false);
    // }
}
