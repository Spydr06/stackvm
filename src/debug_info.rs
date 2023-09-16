use std::collections::{HashSet, HashMap};


#[derive(Default, Clone, Debug)]
pub struct DebugInfo {
    breakpoints: HashSet<i64>,
    labels: HashMap<i64, String>,
    verbose: bool
}

impl DebugInfo {
    pub fn add_breakpoint(&mut self, addr: i64) {
        self.breakpoints.insert(addr);
    }

    pub fn breakpoint_at(&self, addr: i64) -> bool {
        self.breakpoints.contains(&addr)
    }

    pub fn add_label(&mut self, addr: i64, label: String) {
        self.labels.insert(addr, label);
    }

    pub fn label_at(&self, addr: i64) -> Option<&String> {
        self.labels.get(&addr)
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }
}
