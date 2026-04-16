# AeroSchedule DSL

Validador en Rust para reglas operacionales aeronáuticas expresadas en un DSL `.aero`.

## Requisitos

- Rust estable instalado (`cargo` disponible)

## Instalación

```bash
cargo build
```

## Ejecutar tests

```bash
cargo test
```

Qué debería pasar:

- el lexer tokeniza el archivo de ejemplo
- el parser construye el AST sin errores
- el validator detecta las 2 violaciones esperadas del dataset de ejemplo

## Ejecutar la aplicación

```bash
cargo run -- validate --rules restricciones.aero --data ./data/
```

Salida esperada:

- `Resultado: INVÁLIDO`
- `Violaciones: 2`

Eso ocurre porque `restricciones.aero` está escrito para fallar contra los CSV de ejemplo.

## Probar un caso válido

Si querés ver un caso limpio, usá un archivo de reglas sin violaciones. Ejemplo:

```bash
cargo run -- validate --rules /tmp/valid_restricciones.aero --data ./data/
```

Salida esperada:

- `Resultado: VÁLIDO`
- `Violaciones: 0`

## Opciones de CLI

```bash
cargo run -- validate --rules <archivo.aero> --data <directorio_csv>
```

Flags soportados:

- `--rules` ruta al archivo `.aero`
- `--data` directorio con los CSV
- `--output json|text`
- `--only <nombre_regla>`
- `--severity critica|regulatoria|operacional`

## Estructura de ejemplo

- `restricciones.aero` — reglas de referencia
- `data/pilotos.csv`
- `data/aeronaves.csv`
- `data/vuelos.csv`

## Notas

- El lexer y parser están implementados a mano.
- El proyecto sigue una arquitectura: Lexer → Parser → AST → Validator.
