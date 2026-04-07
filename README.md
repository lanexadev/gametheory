# 🦀 Axelrod's Game Theory Engine (Rust)

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An advanced, high-performance simulation engine for Robert Axelrod's **Iterated Prisoner's Dilemma**. This project implements complex game theory dynamics, including evolutionary selection, spatial cellular automata, and multi-layered noise models.

---

## 🚀 Key Features

*   **Advanced Noise Models**: Action noise (execution errors) and Perception noise (misunderstandings).
*   **Evolutionary Dynamics**: Darwinian selection over thousands of generations.
*   **Spatial Grid**: Territorial wars on a 2D grid (Cellular Automata).
*   **Parallel Execution**: Multi-threaded engine using `Rayon` for massive speed.
*   **Automated Analytics**: Integrated Python scripts for population trend visualization.

---

## 📊 Automated Analysis Workflow

The project includes an automated pipeline for deep evolutionary analysis. It handles compilation, simulation, and data visualization in one go.

### Running a Complete Simulation
Use the provided shell script to run and visualize an evolutionary tournament:

```bash
# Basic usage
./run_complete_analysis.sh -g 100 -i 200

# Massive analytical simulation (The "Nuclear" option)
./run_complete_analysis.sh -g 2000 -i 500 -r 0.05 -a 0.04 -p 0.02 -s 999 -d 0.001
```

### Script Flags (`run_complete_analysis.sh`)
*   `-g, --generations` : Number of generations.
*   `-i, --iterations` : Turns per match.
*   `-r, --repro-rate` : Population percentage replaced each generation (e.g., 0.05 for 5%).
*   `-a, --action-noise` : Probability of move flip.
*   `-p, --perception-noise` : Probability of misinterpreting the opponent.
*   `-s, --seed` : RNG seed for reproducibility.
*   `-d, --discount` : Probability of match end per turn.

### Results & Visualization
All results are stored in the `/results` directory (excluded from Git):
*   `evo_TIMESTAMP.csv` : Final scores.
*   `evo_TIMESTAMP_evolution.csv` : Full population history for every generation.
*   `evo_TIMESTAMP_evolution.png` : **Stacked Area Chart** showing population dynamics over time.

---

## 🧪 Adding a New Strategy

The project uses a modular architecture. Each strategy has its own file in `src/strategies/`.

1.  **Create** `src/strategies/my_new_strategy.rs`.
2.  **Implement** the `Strategy` trait.
3.  **Register** it in `src/strategies/mod.rs`.

---

## 💻 Installation & Requirements

### Rust Engine
```bash
cargo build --release
```

### Visualization (Python)
The visualization script requires `pandas` and `matplotlib`:
```bash
pip install pandas matplotlib
```

---

## 📜 License
This project is licensed under the MIT License - see the LICENSE file for details.
