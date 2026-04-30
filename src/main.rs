use std::fs;
use std::path::PathBuf;

use clap::{Parser as ClapParser, ValueEnum};

use aero_compiler::lexer::Tokenizer;
use aero_compiler::parser::Parser;
use aero_compiler::validator::{ValidationReport, Validator};

#[derive(Debug, ClapParser)]
#[command(name = "aerodsl", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, ClapParser)]
enum Commands {
    Validate {
        #[arg(long)]
        rules: PathBuf,
        #[arg(long)]
        data: PathBuf,
        #[arg(long, default_value = "text")]
        output: OutputFormat,
        #[arg(long)]
        only: Option<String>,
        #[arg(long)]
        severity: Option<SeverityArg>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SeverityArg {
    Critica,
    Regulatoria,
    Operacional,
}

impl From<SeverityArg> for aero_compiler::ast::Severity {
    fn from(value: SeverityArg) -> Self {
        match value {
            SeverityArg::Critica => aero_compiler::ast::Severity::Critica,
            SeverityArg::Regulatoria => aero_compiler::ast::Severity::Regulatoria,
            SeverityArg::Operacional => aero_compiler::ast::Severity::Operacional,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Validate {
            rules,
            data,
            output,
            only,
            severity,
        } => run_validate(
            &rules,
            &data,
            output,
            only.as_deref(),
            severity.map(Into::into),
        ),
    };
    std::process::exit(code);
}

fn run_validate(
    rules_path: &std::path::Path,
    data_path: &std::path::Path,
    output: OutputFormat,
    only: Option<&str>,
    severity: Option<aero_compiler::ast::Severity>,
) -> i32 {
    let rules_text = match fs::read_to_string(rules_path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("failed to read rules: {err}");
            return 1;
        }
    };
    let tokens = match Tokenizer::new(&rules_text).tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("lexer error: {:?}", err);
            return 1;
        }
    };
    let mut parser = Parser::new(tokens);
    let rule_set = match parser.parse_rule_set() {
        Ok(rule_set) => rule_set,
        Err(err) => {
            eprintln!("parser error: {err}");
            return 1;
        }
    };
    let validator = match Validator::new(data_path) {
        Ok(validator) => validator,
        Err(err) => {
            eprintln!("validator error: {err}");
            return 1;
        }
    };
    let report = validator.validate(&rule_set, only, severity);
    render_report(&report, output);
    if report.valid {
        0
    } else {
        2
    }
}

fn render_report(report: &ValidationReport, output: OutputFormat) {
    match output {
        OutputFormat::Text => render_text(report),
        OutputFormat::Json => render_json(report),
    }
}

fn render_text(report: &ValidationReport) {
    println!("AeroSchedule DSL Validator v0.1");
    println!("================================");
    println!();
    println!(
        "Resultado: {}",
        if report.valid { "VÁLIDO" } else { "INVÁLIDO" }
    );
    println!("Violaciones: {}", report.violations.len());
    println!(
        "- Críticas: {}",
        count_by_severity(report, aero_compiler::ast::Severity::Critica)
    );
    println!(
        "- Regulatorias: {}",
        count_by_severity(report, aero_compiler::ast::Severity::Regulatoria)
    );
    println!(
        "- Operacionales: {}",
        count_by_severity(report, aero_compiler::ast::Severity::Operacional)
    );
    println!();
    for violation in &report.violations {
        println!(
            "[{}] {}",
            violation.severity.as_str().to_uppercase(),
            violation.rule_name
        );
        println!(
            "  Entidad: {} ({})",
            violation.entity_name, violation.entity_id
        );
        println!("  {}", violation.message);
        if let Some(norm) = &violation.norm {
            println!("  Norma: {norm}");
        }
        println!();
    }
}

fn render_json(report: &ValidationReport) {
    let mut out = String::new();
    out.push_str("{\"valid\":");
    out.push_str(if report.valid { "true" } else { "false" });
    out.push_str(",\"violations\":[");
    for (idx, v) in report.violations.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"rule_name\":\"{}\",\"entity_id\":\"{}\",\"entity_name\":\"{}\",\"message\":\"{}\",\"severity\":\"{}\",\"norm\":{}}}",
            escape_json(&v.rule_name),
            escape_json(&v.entity_id),
            escape_json(&v.entity_name),
            escape_json(&v.message),
            v.severity.as_str(),
            v.norm.as_ref().map(|s| format!("\"{}\"", escape_json(s))).unwrap_or_else(|| "null".to_string())
        ));
    }
    out.push_str("]}");
    println!("{out}");
}

fn escape_json(value: &str) -> String {
    value.replace('"', "\\\"")
}

fn count_by_severity(report: &ValidationReport, severity: aero_compiler::ast::Severity) -> usize {
    report
        .violations
        .iter()
        .filter(|v| v.severity == severity)
        .count()
}
