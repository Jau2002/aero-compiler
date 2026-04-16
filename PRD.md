# PRD: AeroSchedule DSL

**Version**: 0.1.0  
**Tipo**: Lenguaje de Dominio Específico — validador de restricciones operacionales aeronáuticas  
**Lenguaje de implementación**: Rust  
**Estilo**: Académico — arquitectura LL(1): Lexer → Parser → AST → Validator  
**Status**: Ready for implementation

---

## 1. Problem Statement

Las operaciones de scheduling aeronáutico están gobernadas por un conjunto de restricciones regulatorias y operacionales (EASA, FAA, IATA) que hoy se implementan como validaciones dispersas en código de aplicación. Esto genera tres problemas concretos:

1. Las reglas no son legibles ni auditables por personal operacional (no programadores).
2. Cuando una regulación cambia, no hay un lugar único donde encontrar y modificar esa regla.
3. No existe una forma estándar de expresar una restricción nueva sin tocar el código de producción.

**Este DSL resuelve exactamente eso**: proveer un lenguaje declarativo donde las restricciones operacionales se expresan como reglas formales, separadas completamente del código de ejecución. El engine carga datos externos (CSV), lee el archivo `.aero`, evalúa cada regla contra los datos, y reporta violaciones con precisión.

---

## 2. Arquitectura del sistema

La arquitectura sigue el mismo pipeline del ejemplo de referencia `ArithmeticParser_LL1` (Java), trasladado a Rust:

```
Archivo .aero (texto plano)
        │
        ▼
   ┌─────────┐
   │  Lexer  │  ← Tokenizer: convierte texto en stream de tokens
   └─────────┘
        │  Vec<Token>
        ▼
   ┌─────────┐
   │ Parser  │  ← LL(1): consume tokens, valida gramática, construye AST
   └─────────┘
        │  AST (árbol de nodos)
        ▼
   ┌───────────┐
   │ Validator │  ← Recorre el AST, carga CSV externos, evalúa cada regla
   └───────────┘
        │
        ▼
   ValidationReport { valid: bool, violations: Vec<Violation> }
```

### Módulos Rust

```
src/
├── main.rs              ← Entry point: args, orquesta pipeline
├── lexer/
│   ├── mod.rs
│   ├── token.rs         ← Enum Token + TokenKind
│   └── tokenizer.rs     ← Tokenizer struct (equivalente a Tokenizer.java)
├── parser/
│   ├── mod.rs
│   ├── parser.rs        ← Parser LL(1) (equivalente a Parser.java)
│   └── error.rs         ← ParseError
├── ast/
│   └── mod.rs           ← Nodos del AST: RuleSet, Rule, Condition, Constraint
├── validator/
│   ├── mod.rs
│   ├── validator.rs     ← Recorre AST y evalúa contra datos
│   ├── data_loader.rs   ← Carga CSVs externos
│   └── report.rs        ← ValidationReport, Violation
└── grammar.txt          ← Gramática formal del DSL (ver sección 5)
```

---

## 3. Modelo del DSL (Modelo B — puro)

**El DSL solo contiene reglas.** Los datos a validar son 100% externos (CSV). El archivo `.aero` no describe vuelos concretos — describe las condiciones bajo las cuales cualquier vuelo es válido o inválido.

### Principio fundamental

```
DSL file (.aero)  →  "qué debe cumplirse"
CSV files         →  "los datos reales a validar"
Validator         →  "verifica que los datos cumplan las reglas"
```

### Ejemplo de archivo `.aero` completo

```aero
-- AeroSchedule DSL v0.1
-- Restricciones operacionales aeronáuticas

RESTRICCION descanso_minimo_piloto:
  CONTEXTO piloto
  CUANDO vuelo.duracion > 6
  ENTONCES descanso_siguiente >= 10
  UNIDAD horas
  SEVERIDAD critica
  NORMA "EASA ORO.FTL.235"

RESTRICCION horas_mensuales_piloto:
  CONTEXTO piloto
  SIEMPRE horas_mes <= 100
  UNIDAD horas
  SEVERIDAD critica
  NORMA "EASA ORO.FTL.210"

RESTRICCION turnaround_minimo:
  CONTEXTO aeronave
  SIEMPRE turnaround >= 45
  UNIDAD minutos
  SEVERIDAD operacional
  NORMA "IATA AHM"

RESTRICCION curfew_aeropuerto:
  CONTEXTO vuelo
  SIEMPRE hora_llegada FUERA_DE 23:00 06:00
  SEVERIDAD regulatoria
  NORMA "LOCAL"
```

---

## 4. Tokens del lenguaje

### Tabla de tokens

| Token                 | Patrón regex                     | Ejemplo                    |
| --------------------- | -------------------------------- | -------------------------- |
| `KEYWORD_RESTRICCION` | `RESTRICCION`                    | `RESTRICCION`              |
| `KEYWORD_CONTEXTO`    | `CONTEXTO`                       | `CONTEXTO`                 |
| `KEYWORD_CUANDO`      | `CUANDO`                         | `CUANDO`                   |
| `KEYWORD_ENTONCES`    | `ENTONCES`                       | `ENTONCES`                 |
| `KEYWORD_SIEMPRE`     | `SIEMPRE`                        | `SIEMPRE`                  |
| `KEYWORD_UNIDAD`      | `UNIDAD`                         | `UNIDAD`                   |
| `KEYWORD_SEVERIDAD`   | `SEVERIDAD`                      | `SEVERIDAD`                |
| `KEYWORD_NORMA`       | `NORMA`                          | `NORMA`                    |
| `KEYWORD_FUERA_DE`    | `FUERA_DE`                       | `FUERA_DE`                 |
| `IDENTIFIER`          | `[a-zA-Z][a-zA-Z0-9_.]*`         | `piloto`, `vuelo.duracion` |
| `NUMBER`              | `[0-9]+(\.[0-9]+)?`              | `6`, `45`, `100`           |
| `TIME`                | `[0-9]{2}:[0-9]{2}`              | `23:00`, `06:00`           |
| `STRING`              | `"[^"]*"`                        | `"EASA ORO.FTL.235"`       |
| `COMPARATOR`          | `>=`, `<=`, `>`, `<`, `==`, `!=` | `>=`                       |
| `COLON`               | `:`                              | `:`                        |
| `COMMENT`             | `--[^\n]*`                       | `-- esto es comentario`    |
| `WHITESPACE`          | `[ \t\n\r]+`                     | (ignorado)                 |
| `EOF`                 | fin de archivo                   |                            |

### Contextos válidos (valores semánticos de IDENTIFIER tras CONTEXTO)

- `piloto`
- `aeronave`
- `vuelo`
- `aeropuerto`
- `tripulacion`

### Severidades válidas

- `critica` — violación bloquea la operación
- `regulatoria` — violación reportable a autoridad
- `operacional` — advertencia interna

---

## 5. Gramática formal (LL(1))

```
G = (
  NT = { <RuleSet>, <Rule>, <RuleBody>, <Statement>,
         <ConditionalStmt>, <AlwaysStmt>, <Condition>,
         <Constraint>, <Operand>, <Comparator>,
         <Metadata>, <MetadataList> }

  T = { KEYWORD_RESTRICCION, KEYWORD_CONTEXTO, KEYWORD_CUANDO,
        KEYWORD_ENTONCES, KEYWORD_SIEMPRE, KEYWORD_UNIDAD,
        KEYWORD_SEVERIDAD, KEYWORD_NORMA, KEYWORD_FUERA_DE,
        IDENTIFIER, NUMBER, TIME, STRING, COMPARATOR, COLON, EOF }

  S = <RuleSet>

  P = {
    <RuleSet>         ::= <Rule> <RuleSet> | EOF

    <Rule>            ::= KEYWORD_RESTRICCION IDENTIFIER COLON <RuleBody>

    <RuleBody>        ::= KEYWORD_CONTEXTO IDENTIFIER <Statement> <MetadataList>

    <Statement>       ::= <ConditionalStmt> | <AlwaysStmt>

    <ConditionalStmt> ::= KEYWORD_CUANDO <Condition>
                          KEYWORD_ENTONCES <Constraint>

    <AlwaysStmt>      ::= KEYWORD_SIEMPRE <Constraint>

    <Condition>       ::= IDENTIFIER COMPARATOR <Operand>

    <Constraint>      ::= IDENTIFIER COMPARATOR <Operand>
                        | IDENTIFIER KEYWORD_FUERA_DE TIME TIME

    <Operand>         ::= NUMBER | IDENTIFIER

    <MetadataList>    ::= <Metadata> <MetadataList> | <Epsilon>

    <Metadata>        ::= KEYWORD_UNIDAD IDENTIFIER
                        | KEYWORD_SEVERIDAD IDENTIFIER
                        | KEYWORD_NORMA STRING
  }
)
```

---

## 6. AST — Nodos

```rust
// ast/mod.rs

pub struct RuleSet {
    pub rules: Vec<Rule>,
}

pub struct Rule {
    pub name: String,           // identificador de la restricción
    pub context: ContextKind,   // piloto | aeronave | vuelo | ...
    pub statement: Statement,   // condicional o absoluta
    pub metadata: Metadata,
}

pub enum ContextKind {
    Piloto,
    Aeronave,
    Vuelo,
    Aeropuerto,
    Tripulacion,
}

pub enum Statement {
    Conditional {
        condition: Condition,
        constraint: Constraint,
    },
    Always {
        constraint: Constraint,
    },
}

pub struct Condition {
    pub field: String,          // ej: "vuelo.duracion"
    pub comparator: Comparator,
    pub value: Operand,
}

pub enum Constraint {
    Comparison {
        field: String,
        comparator: Comparator,
        value: Operand,
    },
    OutsideRange {
        field: String,
        from: String,           // ej: "23:00"
        to: String,             // ej: "06:00"
    },
}

pub enum Operand {
    Number(f64),
    Field(String),
}

pub enum Comparator { Gte, Lte, Gt, Lt, Eq, Neq }

pub struct Metadata {
    pub unit: Option<String>,       // "horas", "minutos"
    pub severity: Severity,
    pub norm: Option<String>,       // "EASA ORO.FTL.235"
}

pub enum Severity { Critica, Regulatoria, Operacional }
```

---

## 7. Datos externos (CSV)

El Validator carga estos archivos desde el directorio que se le pase como argumento. Los nombres de columna son fijos y el DSL los referencia por campo con notación `contexto.campo`.

### `pilotos.csv`

```csv
id,nombre,horas_mes,horas_90d,horas_12m,ultimo_vuelo_duracion,descanso_siguiente,certificaciones
P001,RAMIREZ,87,240,950,7.5,9,A320;B737
P002,TORRES,42,120,600,4.0,12,ATR72
```

### `aeronaves.csv`

```csv
id,matricula,tipo,horas_desde_mantenimiento,turnaround,ciclos
A001,HK-4567,A320,180,55,1240
A002,HK-3891,ATR72,320,40,890
```

### `vuelos.csv`

```csv
id,piloto_id,aeronave_id,origen,destino,hora_llegada,hora_salida,duracion,aeropuerto_curfew_inicio,aeropuerto_curfew_fin
V001,P001,A001,BOG,MDE,22:45,21:00,1.75,23:00,06:00
V002,P002,A002,BOG,CTG,23:15,22:30,0.75,23:00,06:00
```

### Resolución de campos en el DSL

Cuando una regla dice `CONTEXTO piloto` y la condición es `vuelo.duracion > 6`, el validator itera sobre todos los pilotos, busca sus vuelos asociados, y evalúa la condición. La notación con punto (`contexto.campo`) permite cruzar entidades sin necesidad de joins explícitos en el DSL.

---

## 8. Validator — comportamiento

```
Para cada Rule en el AST:
  Cargar entidades del contexto (ej: todos los pilotos del CSV)
  Para cada entidad:
    Si Statement es Conditional:
      Evaluar Condition contra los datos de la entidad
      Si la condición se cumple:
        Evaluar Constraint
        Si Constraint falla → agregar Violation al reporte
    Si Statement es Always:
      Evaluar Constraint directamente
      Si falla → agregar Violation al reporte

Retornar ValidationReport
```

### Estructura del reporte

```rust
pub struct ValidationReport {
    pub valid: bool,
    pub violations: Vec<Violation>,
}

pub struct Violation {
    pub rule_name: String,
    pub entity_id: String,      // ej: "P001"
    pub entity_name: String,    // ej: "RAMIREZ"
    pub message: String,        // descripción legible
    pub severity: Severity,
    pub norm: Option<String>,
}
```

### Ejemplo de output en consola

```
AeroSchedule DSL Validator v0.1
================================
Archivo: restricciones.aero
Datos:   ./data/

Evaluando 4 reglas contra 2 pilotos, 2 aeronaves, 2 vuelos...

[CRITICA]     descanso_minimo_piloto
              Piloto RAMIREZ (P001)
              vuelo.duracion=7.5h > 6h → descanso_siguiente debe ser >= 10h
              Valor actual: 9h | Norma: EASA ORO.FTL.235

[REGULATORIA] curfew_aeropuerto
              Vuelo V002
              hora_llegada=23:15 está dentro del curfew 23:00–06:00
              Norma: LOCAL

================================
Resultado: INVÁLIDO
Violaciones: 2 (1 crítica, 1 regulatoria, 0 operacionales)
```

---

## 9. Restricciones implementadas en v1

Las 18 restricciones del dominio se expresan como reglas `.aero`. El engine las evalúa todas. A continuación la tabla de correspondencia:

| #   | Nombre regla DSL                 | Contexto    | Tipo statement | Norma            |
| --- | -------------------------------- | ----------- | -------------- | ---------------- |
| 01  | `horas_mensuales_piloto`         | piloto      | Always         | EASA ORO.FTL.210 |
| 02  | `descanso_minimo_piloto`         | piloto      | Conditional    | EASA ORO.FTL.235 |
| 03  | `certificacion_tipo_aeronave`    | piloto      | Always         | EASA FCL.060     |
| 04  | `licencia_vigente`               | piloto      | Always         | EASA FCL.030     |
| 05  | `composicion_minima_tripulacion` | tripulacion | Always         | EASA OPS 1.990   |
| 06  | `limite_mantenimiento_aeronave`  | aeronave    | Always         | EASA Part-M      |
| 07  | `turnaround_minimo`              | aeronave    | Always         | IATA AHM         |
| 08  | `performance_aeropuerto`         | vuelo       | Always         | LOCAL            |
| 09  | `mel_equipo_minimo`              | aeronave    | Conditional    | LOCAL            |
| 10  | `slot_aeropuerto_coordinado`     | vuelo       | Always         | IATA WSG         |
| 11  | `curfew_aeropuerto`              | vuelo       | Always         | LOCAL            |
| 12  | `espacio_aereo_restringido`      | vuelo       | Always         | AIP              |
| 13  | `derechos_trafico_bilateral`     | vuelo       | Always         | BILATERAL        |
| 14  | `disponibilidad_gate`            | vuelo       | Always         | LOCAL            |
| 15  | `continuidad_tripulacion`        | piloto      | Conditional    | EASA ORO.FTL.235 |
| 16  | `etops_bimotor`                  | vuelo       | Conditional    | EASA OPS 1.245   |
| 17  | `combustible_reserva`            | vuelo       | Always         | EASA OPS 1.255   |
| 18  | `peso_maximo_despegue`           | vuelo       | Always         | EASA CS-25       |

---

## 10. Manejo de errores

### Errores léxicos (LexerError)

- Carácter no reconocido → `LexerError { msg, position }`
- String sin cerrar → `LexerError`

### Errores sintácticos (ParseError)

- Token inesperado → `ParseError { expected, found, line }`
- Regla sin cuerpo
- Contexto inválido (no es ninguno de los 5 permitidos)
- Severidad inválida

### Errores semánticos (ValidatorError)

- CSV no encontrado → advertencia, la regla se salta
- Campo referenciado en el DSL no existe en el CSV → `ValidatorError`
- Tipo incompatible (comparar un TIME con un NUMBER) → `ValidatorError`

---

## 11. CLI — interfaz de uso

```bash
# Uso básico
aerodsl validate --rules restricciones.aero --data ./data/

# Solo una regla específica
aerodsl validate --rules restricciones.aero --data ./data/ --only descanso_minimo_piloto

# Output JSON (para integraciones)
aerodsl validate --rules restricciones.aero --data ./data/ --output json

# Solo mostrar críticas
aerodsl validate --rules restricciones.aero --data ./data/ --severity critica
```

### Argumentos

| Flag         | Descripción                      | Requerido |
| ------------ | -------------------------------- | --------- |
| `--rules`    | Ruta al archivo `.aero`          | Sí        |
| `--data`     | Directorio con los CSV           | Sí        |
| `--output`   | `text` (default) o `json`        | No        |
| `--only`     | Nombre de restricción específica | No        |
| `--severity` | Filtrar por severidad            | No        |

---

## 12. Dependencias Rust (Cargo.toml)

```toml
[package]
name = "aerodsl"
version = "0.1.0"
edition = "2021"

[dependencies]
csv     = "1.3"        # lectura de archivos CSV
serde   = { version = "1", features = ["derive"] }
clap    = { version = "4", features = ["derive"] }  # CLI args
```

> **Nota**: El lexer y parser se implementan a mano siguiendo la arquitectura del ejemplo Java. No se usa ANTLR, logos, chumsky ni ningún generador de parsers — el objetivo académico es implementar el pipeline completo desde cero.

---

## 13. Ejemplo de archivo `.aero` completo para pruebas

```aero
-- restricciones.aero
-- Archivo de prueba completo para AeroSchedule DSL v0.1

RESTRICCION descanso_minimo_piloto:
  CONTEXTO piloto
  CUANDO vuelo.duracion > 6
  ENTONCES descanso_siguiente >= 10
  UNIDAD horas
  SEVERIDAD critica
  NORMA "EASA ORO.FTL.235"

RESTRICCION horas_mensuales_piloto:
  CONTEXTO piloto
  SIEMPRE horas_mes <= 100
  UNIDAD horas
  SEVERIDAD critica
  NORMA "EASA ORO.FTL.210"

RESTRICCION turnaround_minimo:
  CONTEXTO aeronave
  SIEMPRE turnaround >= 45
  UNIDAD minutos
  SEVERIDAD operacional
  NORMA "IATA AHM"

RESTRICCION curfew_aeropuerto:
  CONTEXTO vuelo
  SIEMPRE hora_llegada FUERA_DE 23:00 06:00
  SEVERIDAD regulatoria
  NORMA "LOCAL"
```

---

## 14. Criterios de aceptación

El agente implementador debe verificar que:

- [ ] El lexer tokeniza correctamente el archivo `.aero` de prueba (sección 13)
- [ ] El parser construye el AST sin errores para el archivo de prueba
- [ ] El parser lanza `ParseError` con mensaje claro si la gramática es incorrecta
- [ ] El validator detecta las 2 violaciones del ejemplo de datos (sección 7)
- [ ] El reporte imprime severidad, entidad, campo, valor actual y norma violada
- [ ] El CLI acepta `--rules` y `--data` como mínimo
- [ ] Re-ejecutar el validator con datos válidos produce `Resultado: VÁLIDO`
- [ ] El código compila sin warnings con `cargo build --release`

---

## 15. Lo que está fuera de alcance (v1)

- Generación de schedules (el DSL no propone, solo valida)
- Persistencia de reportes (solo stdout / JSON en stdout)
- Editor con syntax highlighting
- Imports o herencia entre archivos `.aero`
- Evaluación de expresiones aritméticas en condiciones (solo comparaciones simples)
- Llamadas a servicios externos en tiempo real (todo viene de CSV)
