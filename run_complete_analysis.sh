#!/bin/bash

# Valeurs par défaut
GEN=100
ITER=200
NOISE_A=0.03
NOISE_P=0.01
REPRO=0.1
SEED=123
DISCOUNT=0.0
T=5
R=3
P=1
S=0

# Aide
usage() {
  echo "Usage: $0 [options]"
  echo "Options:"
  echo "  -g, --generations INT    Nombre de générations (default: $GEN)"
  echo "  -i, --iterations INT     Nombre d'itérations par match (default: $ITER)"
  echo "  -a, --action-noise FLOAT Bruit d'action (default: $NOISE_A)"
  echo "  -p, --perception-noise FLOAT Bruit de perception (default: $NOISE_P)"
  echo "  -r, --repro-rate FLOAT   Taux de reproduction (default: $REPRO)"
  echo "  -s, --seed INT           Graine aléatoire (default: $SEED)"
  echo "  -d, --discount FLOAT     Discount factor (default: $DISCOUNT)"
  echo "  --payoff-t INT           Payoff Temptation (default: $T)"
  echo "  --payoff-r INT           Payoff Reward (default: $R)"
  echo "  --payoff-p INT           Payoff Punishment (default: $P)"
  echo "  --payoff-s INT           Payoff Sucker (default: $S)"
  exit 1
}

# Parsing des arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    -g|--generations) GEN="$2"; shift 2 ;;
    -i|--iterations)  ITER="$2"; shift 2 ;;
    -a|--action-noise) NOISE_A="$2"; shift 2 ;;
    -p|--perception-noise) NOISE_P="$2"; shift 2 ;;
    -r|--repro-rate)  REPRO="$2"; shift 2 ;;
    -s|--seed)        SEED="$2"; shift 2 ;;
    -d|--discount)    DISCOUNT="$2"; shift 2 ;;
    --payoff-t)       T="$2"; shift 2 ;;
    --payoff-r)       R="$2"; shift 2 ;;
    --payoff-p)       P="$2"; shift 2 ;;
    --payoff-s)       S="$2"; shift 2 ;;
    -h|--help)        usage ;;
    *) echo "Unknown parameter: $1"; usage ;;
  esac
done

NAME="evo_$(date +%Y%m%d_%H%M%S)"

echo "🚀 Starting Analytical Simulation..."
echo "Configuration: Gen=$GEN, Iter=$ITER, NoiseA=$NOISE_A, NoiseP=$NOISE_P, Repro=$REPRO, Seed=$SEED"

# 1. Run the Rust Engine
time cargo run --release -- \
  --evolution \
  --generations "$GEN" \
  --iterations "$ITER" \
  --action-noise "$NOISE_A" \
  --perception-noise "$NOISE_P" \
  --reproduction-rate "$REPRO" \
  --discount-factor "$DISCOUNT" \
  --payoff-t "$T" --payoff-r "$R" --payoff-p "$P" --payoff-s "$S" \
  --seed "$SEED" \
  --export-csv "results/${NAME}.csv"

# 2. Run the Visualization
if [ -f "results/${NAME}_evolution.csv" ]; then
    echo "📊 Generating Visualizations..."
    python3 visualize_evolution.py "results/${NAME}_evolution.csv"
else
    echo "❌ Error: Evolution history file not found."
fi

echo "✅ Done. Results: results/${NAME}.csv | Visual: results/${NAME}_evolution.png"
