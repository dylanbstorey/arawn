//! Index report types for session indexing pipeline results.

/// Report summarizing the results of indexing a session.
#[derive(Debug, Clone, Default)]
pub struct IndexReport {
    /// Number of entities extracted and stored.
    pub entities_stored: usize,
    /// Number of facts stored (new insertions).
    pub facts_inserted: usize,
    /// Number of existing facts reinforced (same content seen again).
    pub facts_reinforced: usize,
    /// Number of old facts superseded by new contradicting facts.
    pub facts_superseded: usize,
    /// Number of relationships stored in the knowledge graph.
    pub relationships_stored: usize,
    /// Whether a session summary was generated and stored.
    pub summary_stored: bool,
    /// Errors encountered during indexing (non-fatal).
    pub errors: Vec<String>,
}

impl IndexReport {
    /// Total number of facts processed (inserted + reinforced + superseded).
    pub fn total_facts(&self) -> usize {
        self.facts_inserted + self.facts_reinforced + self.facts_superseded
    }

    /// Whether any errors occurred during indexing.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl std::fmt::Display for IndexReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IndexReport {{ entities: {}, facts: {} (new: {}, reinforced: {}, superseded: {}), relationships: {}, summary: {}, errors: {} }}",
            self.entities_stored,
            self.total_facts(),
            self.facts_inserted,
            self.facts_reinforced,
            self.facts_superseded,
            self.relationships_stored,
            self.summary_stored,
            self.errors.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_default() {
        let report = IndexReport::default();
        assert_eq!(report.entities_stored, 0);
        assert_eq!(report.total_facts(), 0);
        assert!(!report.summary_stored);
        assert!(!report.has_errors());
    }

    #[test]
    fn test_report_total_facts() {
        let report = IndexReport {
            facts_inserted: 3,
            facts_reinforced: 2,
            facts_superseded: 1,
            ..Default::default()
        };
        assert_eq!(report.total_facts(), 6);
    }

    #[test]
    fn test_report_has_errors() {
        let mut report = IndexReport::default();
        assert!(!report.has_errors());
        report.errors.push("something failed".into());
        assert!(report.has_errors());
    }

    #[test]
    fn test_report_display() {
        let report = IndexReport {
            entities_stored: 3,
            facts_inserted: 2,
            facts_reinforced: 1,
            facts_superseded: 0,
            relationships_stored: 4,
            summary_stored: true,
            errors: vec![],
        };
        let s = format!("{}", report);
        assert!(s.contains("entities: 3"));
        assert!(s.contains("facts: 3"));
        assert!(s.contains("relationships: 4"));
        assert!(s.contains("summary: true"));
    }
}
