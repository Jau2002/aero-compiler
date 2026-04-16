#[derive(Debug, Clone, PartialEq)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub context: ContextKind,
    pub statement: Statement,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextKind {
    Piloto,
    Aeronave,
    Vuelo,
    Aeropuerto,
    Tripulacion,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Conditional {
        condition: Condition,
        constraint: Constraint,
    },
    Always {
        constraint: Constraint,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub field: String,
    pub comparator: Comparator,
    pub value: Operand,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    Comparison {
        field: String,
        comparator: Comparator,
        value: Operand,
    },
    OutsideRange {
        field: String,
        from: String,
        to: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Field(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparator {
    Gte,
    Lte,
    Gt,
    Lt,
    Eq,
    Neq,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub unit: Option<String>,
    pub severity: Severity,
    pub norm: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Critica,
    Regulatoria,
    Operacional,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Critica => "critica",
            Severity::Regulatoria => "regulatoria",
            Severity::Operacional => "operacional",
        }
    }
}
