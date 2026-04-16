# AGENTS.md — AeroSchedule DSL

## Build & run

```bash
cargo build --release
cargo run -- validate --rules restricciones.aero --data ./data/
```

## Comandos de desarrollo

```bash
cargo check                    # verificación rápida de tipos
cargo clippy -- -D warnings    # linting; el build de CI falla con cualquier warning
cargo fmt                      # formateo obligatorio antes de cada commit
cargo test                     # corre todos los tests unitarios e integración
cargo test lexer               # filtrar por módulo
```

> El proyecto compila sin warnings en `cargo build --release`. Si introduces uno, corrígelo antes de continuar.

## Testing

Los tests viven junto al código que prueban (`#[cfg(test)]` al final de cada módulo). No hay directorio `tests/` separado salvo para tests de integración CLI.

Casos mínimos esperados por módulo:

- **lexer**: tokeniza el archivo `.aero` de prueba completo sin errores; lanza `LexerError` ante input inválido.
- **parser**: construye el AST correcto para el archivo de prueba; lanza `ParseError` con línea ante gramática incorrecta.
- **validator**: detecta exactamente las 2 violaciones del dataset de ejemplo (`descanso_minimo_piloto` → P001, `curfew_aeropuerto` → V002); retorna `valid: true` con datos limpios.

## Estilo de código

- `snake_case` para variables, funciones, módulos y campos de struct.
- `PascalCase` para tipos, structs, enums y variantes.
- `SCREAMING_SNAKE_CASE` solo para constantes (`const`).
- Máximo 100 caracteres por línea (`rustfmt` lo controla).
- Sin `unwrap()` ni `expect()` en código de producción; usa `?` y propaga con `Result<T, E>`.
- Sin `unsafe`.
- Define errores con `impl std::fmt::Display` a mano; no agregues `thiserror` ni `anyhow` como dependencia.
- `main.rs` es solo orquestador: lee args → ejecuta pipeline → imprime → devuelve exit code. Sin lógica de negocio.

## Dependencias

Las únicas dependencias permitidas son las declaradas en `Cargo.toml`:

```
csv, serde (derive), clap (derive)
```

No agregues ninguna otra sin aprobación explícita. En particular: **ningún generador de parsers ni lexers** (`logos`, `chumsky`, `pest`, `nom`, `lalrpop`, etc.). El lexer y el parser se implementan a mano; es el objetivo académico del proyecto.

## Estructura del repo

```
src/
├── main.rs
├── lexer/        mod.rs · token.rs · tokenizer.rs
├── parser/       mod.rs · parser.rs · error.rs
├── ast/          mod.rs
└── validator/    mod.rs · validator.rs · data_loader.rs · report.rs
grammar.txt
data/             ← CSVs de prueba (pilotos.csv, aeronaves.csv, vuelos.csv)
```

No crees archivos fuera de esta estructura sin una razón concreta.

## Spec de referencia

Toda la lógica del dominio (tokens, gramática, AST, comportamiento del validator, formato de output, criterios de aceptación) está en **`PRD.md`**. Léelo antes de implementar cualquier módulo.
