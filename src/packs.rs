use packs::packs::configuration::Configuration;
use packs::packs::pack::Pack;
use ratatui::widgets::ListState;
use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

pub struct Packs {
    pub configuration: Configuration,
    // todo: use https://github.com/notify-rs/notify
    // pub timestamp: DateTime<Utc>,
    pub packs: Option<Vec<Rc<Pack>>>,
    pub pack_list: Option<PackList>,
    pub pack_dependents: Option<HashMap<String, BTreeSet<String>>>,
    pub pack_dependent_violations: Option<Vec<PackDependentViolation>>,
}

pub struct PackDependentViolation {
    pub defining_pack_name: String,
    pub referencing_pack_name: String,
    pub violation_type_counts: HashMap<String, usize>,
    pub constant_counts: HashMap<String, usize>,
}

impl PackDependentViolation {
    pub fn count_for_violation_type(&self, violation_type: &str) -> usize {
        match self.violation_type_counts.get(violation_type) {
            Some(count) => *count,
            None => 0,
        }
    }

    pub fn num_constants(&self) -> usize {
        self.constant_counts.len()
    }
}

impl Default for Packs {
    fn default() -> Self {
        let configuration = packs::packs::configuration::get(&std::env::current_dir().unwrap());
        // let timestamp = configuration.pack_set.timestamp();
        Packs {
            configuration,
            // timestamp,
            packs: None,
            pack_list: None,
            pack_dependents: None,
            pack_dependent_violations: None,
        }
    }
}

impl Packs {
    pub fn get_pack_list(&mut self) -> &mut PackList {
        self.check_stale();
        if self.pack_list.is_none() {
            self.pack_list = Some(PackList::with_items(self.get_packs()));
        }
        self.pack_list.as_mut().unwrap()
    }

    pub fn get_packs(&mut self) -> Vec<Rc<Pack>> {
        self.check_stale();
        if self.packs.is_none() {
            let mut packs: Vec<Rc<Pack>> = self
                .configuration
                .pack_set
                .packs
                .iter()
                .map(|p| Rc::new(p.clone()))
                .collect();
            packs.sort_by(|a, b| a.name.cmp(&b.name));
            self.packs = Some(packs);
        }
        self.packs.as_ref().unwrap().clone()
    }

    pub fn get_pack_dependents(&mut self) -> HashMap<String, BTreeSet<String>> {
        self.check_stale();
        if self.pack_dependents.is_none() {
            let pack_dependents =
                self.configuration
                    .pack_set
                    .packs
                    .iter()
                    .fold(HashMap::new(), |mut map, pack| {
                        map.entry(pack.name.clone()).or_default(); // handles standalone packs
                        for dep_pack in pack.dependencies.iter() {
                            let entry: &mut BTreeSet<String> =
                                map.entry(dep_pack.clone()).or_default();
                            entry.insert(pack.name.clone());
                        }
                        map
                    });
            self.pack_dependents = Some(pack_dependents);
        }
        self.pack_dependents.as_ref().unwrap().clone()
    }

    pub fn get_pack_dependent_violations_by_selected_defining_pack_name(
        &mut self,
    ) -> Vec<&PackDependentViolation> {
        let defining_pack_name = match self.get_pack_list().selected_pack() {
            Some(pack) => pack.name.clone(),
            None => return vec![],
        };
        let pack_dependent_violations = self.get_pack_dependent_violations();
        let mut dependents: Vec<&PackDependentViolation> = pack_dependent_violations
            .iter()
            .filter(|violation| violation.defining_pack_name == defining_pack_name)
            .collect();
        dependents.sort_by(|a, b| a.referencing_pack_name.cmp(&b.referencing_pack_name));
        dependents
    }

    pub fn get_pack_dependent_violations(&mut self) -> &mut Vec<PackDependentViolation> {
        self.check_stale();
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
                        entry
                            .constant_counts
                            .entry(violation.constant_name.clone())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                        map
                    },
                );
            let pack_dependent_violations: Vec<PackDependentViolation> =
                dependent_map.drain().map(|(_, v)| v).collect();
            self.pack_dependent_violations = Some(pack_dependent_violations);
        }
        self.pack_dependent_violations.as_mut().unwrap()
    }

    pub fn check_stale(&mut self) {
        // let timestamp = self.configuration.pack_set.timestamp();
        // if timestamp > self.timestamp {
        //     self.timestamp = timestamp;
        //     self.packs = None;
        //     self.pack_list = None;
        // }
    }

    pub fn pack_info(&self, pack: &Pack) -> Vec<String> {
        let serialized = packs::packs::pack::serialize_pack(pack);
        serialized.split('\n').map(|s| s.to_string()).collect()
    }

    pub fn pack_dependents(&mut self, pack: &Pack) -> BTreeSet<String> {
        match self.get_pack_dependents().get(&pack.name) {
            Some(dependents) => dependents.clone(),
            None => BTreeSet::new(),
        }
    }

    pub fn next_pack_list(&mut self) {
        if let Some(ref mut pack_list) = self.pack_list {
            pack_list.next()
        }
    }

    pub fn previous_pack_list(&mut self) {
        if let Some(ref mut pack_list) = self.pack_list {
            pack_list.previous()
        }
    }

    pub fn unselect_pack_list(&mut self) {
        if let Some(ref mut pack_list) = self.pack_list {
            pack_list.unselect()
        }
    }
}

pub struct PackList {
    pub state: ListState,
    pub items: Vec<Rc<Pack>>,
    pub filter: String,
    filtered_items: HashMap<String, Vec<Rc<Pack>>>,
}

impl PackList {
    fn with_items(items: Vec<Rc<Pack>>) -> PackList {
        let mut pack_list = PackList {
            state: ListState::default(),
            items,
            filter: String::default(),
            filtered_items: HashMap::new(),
        };
        if !pack_list.items.is_empty() {
            pack_list.state.select(Some(0));
        }
        pack_list
    }

    pub fn filtered_items(&mut self) -> Vec<Rc<Pack>> {
        let filter = self.filter.clone();
        let filter_regex = Regex::new(&filter.to_string()).unwrap();

        match self.filtered_items.get(&filter) {
            Some(items) => items.clone(),
            None => {
                let filtered_items: Vec<Rc<Pack>> = self
                    .items
                    .iter()
                    .filter(|item| filter_regex.is_match(&item.name))
                    .cloned()
                    .collect();
                self.filtered_items.insert(filter, filtered_items.clone());
                filtered_items
            }
        }
    }

    pub fn selected_pack(&mut self) -> Option<Rc<Pack>> {
        match self.state.selected() {
            Some(i) => {
                if self.filtered_items().is_empty() {
                    None
                } else if i >= self.filtered_items().len() {
                    Some(self.filtered_items()[0].clone())
                } else {
                    Some(self.filtered_items()[i].clone())
                }
            }
            None => None,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
