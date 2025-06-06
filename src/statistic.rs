use std::collections::HashMap;

#[derive(Debug)]
pub struct Report {
    _id: usize,
    key: Option<String>,
}

impl Report {
    pub fn new(id: usize, key: Option<String>) -> Self {
        Report { _id: id, key }
    }
}

#[derive(Default, Debug)]
pub struct Statistics {
    hits: HashMap<Option<String>, usize>,
}

impl Statistics {
    pub fn add_report(&mut self, report: Report) {
        let hits = self.hits.entry(report.key).or_default();
        *hits += 1;
    }
}
