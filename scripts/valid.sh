#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cat > /tmp/valid_restricciones.aero <<'EOF'
RESTRICCION horas_mensuales_piloto:
  CONTEXTO piloto
  SIEMPRE horas_mes <= 100
  UNIDAD horas
  SEVERIDAD critica
  NORMA "EASA ORO.FTL.210"

RESTRICCION turnaround_minimo:
  CONTEXTO aeronave
  SIEMPRE turnaround >= 40
  UNIDAD minutos
  SEVERIDAD operacional
  NORMA "IATA AHM"

RESTRICCION curfew_aeropuerto:
  CONTEXTO vuelo
  SIEMPRE hora_llegada FUERA_DE 23:30 05:00
  SEVERIDAD regulatoria
  NORMA "LOCAL"
EOF

cargo run --manifest-path "$ROOT_DIR/Cargo.toml" -- validate --rules /tmp/valid_restricciones.aero --data "$ROOT_DIR/data" --output text
