use std::collections::{BTreeSet, HashMap};
use ratatui::widgets::ListState;
use packs::packs::pack::Pack;
use packs::packs::configuration::Configuration;

pub struct Packs {
    pub configuration: Configuration,
    // todo: use https://github.com/notify-rs/notify
    // pub timestamp: DateTime<Utc>,
    pub packs: Option<Vec<Pack>>,
    pub pack_list: Option<PackList>,
    pub pack_dependents: Option<HashMap<String, BTreeSet<String>>>,
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

    pub fn get_packs(&mut self) -> Vec<Pack> {
        self.check_stale();
        if self.packs.is_none() {
            let mut packs = self.configuration.pack_set.packs.clone();
            packs.sort_by(|a, b| a.name.cmp(&b.name));
            self.packs = Some(packs);
        }
        self.packs.as_ref().unwrap().to_vec()
    }

    pub fn get_pack_dependents(&mut self) -> HashMap<String, BTreeSet<String>> {
        self.check_stale();
        if self.pack_dependents.is_none() {
            let pack_dependents = self.configuration.pack_set.packs.iter().fold(
                HashMap::new(),
                |mut map, pack| {
                    map.entry(pack.name.clone()).or_default(); // handles standalone packs
                    for dep_pack in pack.dependencies.iter() {
                        let entry: &mut BTreeSet<String> = map.entry(dep_pack.clone()).or_default();
                        entry.insert(pack.name.clone());
                    }
                    map
                },
            );
            self.pack_dependents = Some(pack_dependents);
        }
        self.pack_dependents.as_ref().unwrap().clone()
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
        serialized.split("\n").map(|s| s.to_string()).collect()
    }

    pub fn pack_dependents(&mut self, pack: &Pack) -> BTreeSet<String> {
        match self.get_pack_dependents()
            .get(&pack.name) {
            Some(dependents) => dependents.clone(),
            None => BTreeSet::new(),
        }
    }

    pub fn next_pack_list(&mut self) {
        match self.pack_list {
            Some(ref mut pack_list) => pack_list.next(),
            None => (),
        }
    }

    pub fn previous_pack_list(&mut self) {
        match self.pack_list {
            Some(ref mut pack_list) => pack_list.previous(),
            None => (),
        }
    }

    pub fn unselect_pack_list(&mut self) {
        match self.pack_list {
            Some(ref mut pack_list) => pack_list.unselect(),
            None => (),
        }
    }
}

pub struct PackList {
    pub state: ListState,
    pub items: Vec<Pack>,
}

impl PackList {
    fn with_items(items: Vec<Pack>) -> PackList {
        PackList {
            state: ListState::default(),
            items,
        }
    }

    pub fn selected_pack(&self) -> Option<Pack> {
        match self.state.selected() {
            Some(i) => Some(self.items[i].clone()),
            None => None,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
                    self.items.len() - 1
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
