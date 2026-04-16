use std::path::Path;

use crate::ast::{
    Comparator, Constraint, ContextKind, Operand, Rule, RuleSet, Severity, Statement,
};

use super::data_loader::{DataSet, Row, load_data};
use super::report::{ValidationReport, Violation};

pub struct Validator {
    data: DataSet,
}

struct Entity<'a> {
    id: String,
    name: String,
    row: &'a Row,
    flights: Vec<&'a Row>,
}

impl Validator {
    pub fn new(data_dir: &Path) -> Result<Self, String> {
        Ok(Self {
            data: load_data(data_dir)?,
        })
    }

    pub fn validate(
        &self,
        rules: &RuleSet,
        only: Option<&str>,
        severity: Option<Severity>,
    ) -> ValidationReport {
        let mut violations = Vec::new();
        for rule in &rules.rules {
            if only.map(|name| name != rule.name).unwrap_or(false) {
                continue;
            }
            if severity
                .map(|s| s != rule.metadata.severity)
                .unwrap_or(false)
            {
                continue;
            }
            let entities = self.entities_for_rule(rule);
            for entity in entities {
                if self.evaluate_rule(rule, &entity) {
                    continue;
                }
                let entity_id = entity.id.clone();
                let entity_name = entity.name.clone();
                let message = build_message(rule, &entity);
                violations.push(Violation {
                    rule_name: rule.name.clone(),
                    entity_id,
                    entity_name,
                    message,
                    severity: rule.metadata.severity,
                    norm: rule.metadata.norm.clone(),
                });
            }
        }
        ValidationReport {
            valid: violations.is_empty(),
            violations,
        }
    }

    fn entities_for_rule(&self, rule: &Rule) -> Vec<Entity<'_>> {
        match rule.context {
            ContextKind::Piloto => self
                .data
                .pilotos
                .iter()
                .map(|row| Entity {
                    id: row.get("id").cloned().unwrap_or_default(),
                    name: row.get("nombre").cloned().unwrap_or_default(),
                    row,
                    flights: self
                        .data
                        .vuelos
                        .iter()
                        .filter(|flight| flight.get("piloto_id") == row.get("id"))
                        .collect(),
                })
                .collect(),
            ContextKind::Aeronave => self
                .data
                .aeronaves
                .iter()
                .map(|row| Entity {
                    id: row.get("id").cloned().unwrap_or_default(),
                    name: row.get("matricula").cloned().unwrap_or_default(),
                    row,
                    flights: Vec::new(),
                })
                .collect(),
            ContextKind::Vuelo | ContextKind::Aeropuerto => self
                .data
                .vuelos
                .iter()
                .map(|row| Entity {
                    id: row.get("id").cloned().unwrap_or_default(),
                    name: row.get("id").cloned().unwrap_or_default(),
                    row,
                    flights: vec![row],
                })
                .collect(),
            ContextKind::Tripulacion => self
                .data
                .pilotos
                .iter()
                .map(|row| Entity {
                    id: row.get("id").cloned().unwrap_or_default(),
                    name: row.get("nombre").cloned().unwrap_or_default(),
                    row,
                    flights: Vec::new(),
                })
                .collect(),
        }
    }

    fn evaluate_rule(&self, rule: &Rule, entity: &Entity<'_>) -> bool {
        match &rule.statement {
            Statement::Conditional {
                condition,
                constraint,
            } => {
                if !self.eval_condition(condition, entity) {
                    true
                } else {
                    self.eval_constraint(constraint, entity)
                }
            }
            Statement::Always { constraint } => self.eval_constraint(constraint, entity),
        }
    }

    fn eval_condition(&self, condition: &crate::ast::Condition, entity: &Entity<'_>) -> bool {
        if condition.field.starts_with("vuelo.") {
            return entity.flights.iter().any(|flight| {
                self.compare_field(
                    flight,
                    &condition.field[6..],
                    condition.comparator,
                    &condition.value,
                )
            });
        }
        self.compare_field(
            entity.row,
            &condition.field,
            condition.comparator,
            &condition.value,
        )
    }

    fn eval_constraint(&self, constraint: &Constraint, entity: &Entity<'_>) -> bool {
        match constraint {
            Constraint::Comparison {
                field,
                comparator,
                value,
            } => self.compare_field(entity.row, field, *comparator, value),
            Constraint::OutsideRange { field, from, to } => {
                let value = entity.row.get(field).and_then(|v| parse_time_minutes(v));
                let from = parse_time_minutes(from);
                let to = parse_time_minutes(to);
                matches!((value, from, to), (Some(value), Some(from), Some(to)) if !is_in_range(value, from, to))
            }
        }
    }

    fn compare_field(
        &self,
        row: &Row,
        field: &str,
        comparator: Comparator,
        value: &Operand,
    ) -> bool {
        let lhs = match row.get(field) {
            Some(value) => value,
            None => return false,
        };
        match value {
            Operand::Number(rhs) => match lhs.parse::<f64>() {
                Ok(left) => compare_numbers(left, comparator, *rhs),
                Err(_) => false,
            },
            Operand::Field(rhs_field) => match row.get(rhs_field) {
                Some(rhs_value) => compare_fields(lhs, rhs_value, comparator),
                None => false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Validator;
    use crate::lexer::Tokenizer;
    use crate::parser::Parser;
    use std::path::Path;

    #[test]
    fn detects_two_reference_violations() {
        let rules = include_str!("../../restricciones.aero");
        let tokens = Tokenizer::new(rules).tokenize().expect("tokenize");
        let mut parser = Parser::new(tokens);
        let rule_set = parser.parse_rule_set().expect("parse");
        let validator = Validator::new(Path::new("data")).expect("validator");
        let report = validator.validate(&rule_set, None, None);
        assert!(!report.valid);
        assert_eq!(report.violations.len(), 2);
    }
}

fn compare_numbers(left: f64, comparator: Comparator, right: f64) -> bool {
    match comparator {
        Comparator::Gte => left >= right,
        Comparator::Lte => left <= right,
        Comparator::Gt => left > right,
        Comparator::Lt => left < right,
        Comparator::Eq => (left - right).abs() < f64::EPSILON,
        Comparator::Neq => (left - right).abs() >= f64::EPSILON,
    }
}

fn compare_fields(left: &str, right: &str, comparator: Comparator) -> bool {
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(left), Ok(right)) => compare_numbers(left, comparator, right),
        _ => match comparator {
            Comparator::Eq => left == right,
            Comparator::Neq => left != right,
            _ => false,
        },
    }
}

fn parse_time_minutes(value: &str) -> Option<i32> {
    let (hours, minutes) = value.split_once(':')?;
    Some(hours.parse::<i32>().ok()? * 60 + minutes.parse::<i32>().ok()?)
}

fn is_in_range(value: i32, from: i32, to: i32) -> bool {
    if from <= to {
        value >= from && value <= to
    } else {
        value >= from || value <= to
    }
}

fn build_message(rule: &Rule, entity: &Entity<'_>) -> String {
    match &rule.statement {
        Statement::Conditional {
            condition,
            constraint,
        } => {
            let field_value = if condition.field.starts_with("vuelo.") {
                entity
                    .flights
                    .iter()
                    .find_map(|flight| flight.get(&condition.field[6..]).cloned())
                    .unwrap_or_default()
            } else {
                entity
                    .row
                    .get(&condition.field)
                    .cloned()
                    .unwrap_or_default()
            };
            format!("{}={field_value} → {:?}", condition.field, constraint)
        }
        Statement::Always { constraint } => format!("{:?}", constraint),
    }
}
