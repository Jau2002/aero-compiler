use aero_compiler::lexer::Tokenizer;
use aero_compiler::parser::Parser;
use aero_compiler::validator::Validator;
use std::path::Path;

#[test]
fn test_full_pipeline_reference_rules() {
    let rules = include_str!("../restricciones.aero");
    let tokens = Tokenizer::new(rules).tokenize().expect("Should tokenize");
    let mut parser = Parser::new(tokens);
    let rule_set = parser.parse_rule_set().expect("Should parse");

    let validator = Validator::new(Path::new("data")).expect("Should create validator");
    let report = validator.validate(&rule_set, None, None);

    // We expect exactly 2 violations based on current data
    assert!(!report.valid, "Report should be invalid");
    assert_eq!(
        report.violations.len(),
        2,
        "Should have exactly 2 violations"
    );

    // Check specific violations
    let pilot_violation = report
        .violations
        .iter()
        .any(|v| v.entity_id == "P001" && v.rule_name == "descanso_minimo_piloto");
    let curfew_violation = report
        .violations
        .iter()
        .any(|v| v.entity_id == "V002" && v.rule_name == "curfew_aeropuerto");

    assert!(
        pilot_violation,
        "Should detect P001 descanso_minimo_piloto violation"
    );
    assert!(
        curfew_violation,
        "Should detect V002 curfew_aeropuerto violation"
    );
}

#[test]
fn test_lexer_edge_cases() {
    let input = "-- only comments\n   \n-- more comments";
    let tokens = Tokenizer::new(input).tokenize().unwrap();
    // Should only have EOF
    assert_eq!(tokens.len(), 1);
    assert!(matches!(
        tokens[0].kind,
        aero_compiler::lexer::TokenKind::Eof
    ));
}

#[test]
fn test_parser_empty_input() {
    let tokens = Tokenizer::new("").tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let rule_set = parser.parse_rule_set().unwrap();
    assert!(rule_set.rules.is_empty());
}
