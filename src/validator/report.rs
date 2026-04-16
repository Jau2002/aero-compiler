use crate::ast::Severity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub valid: bool,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Violation {
    pub rule_name: String,
    pub entity_id: String,
    pub entity_name: String,
    pub message: String,
    pub severity: Severity,
    pub norm: Option<String>,
}
