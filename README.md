# 🦀 Axelrod's Game Theory Engine (Rust)

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An advanced, high-performance simulation engine for Robert Axelrod's **Iterated Prisoner's Dilemma**. This project implements complex game theory dynamics, including evolutionary selection, spatial cellular automata, and multi-layered noise models.

---

## 📖 Theoretical Background

The **Prisoner's Dilemma** is a standard example of a game analyzed in game theory that shows why two completely rational individuals might not cooperate, even if it appears that it is in their best interests to do so.

### The Payoff Matrix
| Player A / Player B | Cooperate (C) | Defect (D) |
| :--- | :---: | :---: |
| **Cooperate (C)** | 3 / 3 (R) | 0 / 5 (S/T) |
| **Defect (D)** | 5 / 0 (T/S) | 1 / 1 (P) |

*   **R (Reward)**: Mutual cooperation.
*   **T (Temptation)**: One defects while the other cooperates.
*   **S (Sucker)**: One cooperates while the other defects.
*   **P (Punishment)**: Mutual defection.

---

## 🚀 Key Features

### 🔬 Advanced Simulation Engine
*   **Action Noise**: The probability that a strategy's intended move is flipped (e.g., "mis-click").
*   **Perception Noise**: The probability that a move is misinterpreted by the opponent (e.g., "miscommunication").
*   **Discount Factor**: A probability (0.0 to 1.0) that the game ends after any given turn, simulating an infinite horizon with an unknown end.
*   **Custom Payoffs**: Fully adjustable T, R, P, S values via CLI.
*   **Deterministic Seeds**: Use `--seed <u64>` to reproduce exact simulation results.

### 🏆 Tournament Modes
1.  **Round Robin**: Every strategy plays against everyone else (including itself). Optimized with **Rayon** for massive parallelism.
2.  **Swiss System**: Used for large populations. Strategies are paired against others with similar scores to find the elite quickly.
3.  **Evolutionary Dynamics**: A Darwinian simulation where unsuccessful strategies go extinct and winners reproduce over generations.
4.  **Spatial Grid (Cellular Automata)**: Strategies live on a 2D grid and interact only with neighbors. Successful strategies "conquer" adjacent cells.

### 🛠 Modular Architecture
*   **File-per-Strategy**: Every strategy is isolated in its own file under `src/strategies/`.
*   **Functional Strategy Factory**: Create hundreds of variants (e.g., Forgiving-Tit-for-Tat with 5% to 95% forgiveness) using closures.

---

## 💻 Installation

Ensure you have [Rust](https://rustup.rs/) installed.

```bash
git clone https://github.com/lanexadev/gametheory.git
cd gametheory
cargo build --release
```

---

## 🕹 Usage Examples

### 1. Standard Round Robin
Run a tournament with 200 iterations and 2% action noise:
```bash
cargo run -- --iterations 200 --action-noise 0.02
```

### 2. Evolutionary Survival
Simulate 50 generations of evolution. In each generation, the bottom 20% of the population is replaced by the top 20%:
```bash
cargo run -- --evolution --generations 50 --reproduction-rate 0.2
```

### 3. Spatial Territorial War
Run a 30x30 grid for 100 steps. Watch how "cooperation bubbles" form and resist "defector invasions":
```bash
cargo run -- --spatial --grid-size 30 --generations 100
```

### 4. High-Stakes Finale
Identify the top 3 strategies from a round-robin and make them play a long-duration final (5x iterations):
```bash
cargo run -- --finale --iterations 1000
```

---

## 🧪 Adding a New Strategy

The project is designed for easy extension. To add a strategy named `MyStrategy`:

1.  Create `src/strategies/my_strategy.rs`:
    ```rust
    use crate::{Action, Strategy};

    #[derive(Clone, Default)]
    pub struct MyStrategy;

    impl Strategy for MyStrategy {
        fn name(&self) -> &str { "My Strategy Name" }
        fn next_move(&self, my_history: &[Action], opponent_history: &[Action]) -> Action {
            // Your logic here
            Action::Cooperate
        }
        fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
    }
    ```
2.  In `src/strategies/mod.rs`:
    *   Add `pub mod my_strategy;`
    *   Add `Box::new(my_strategy::MyStrategy::default()),` to `get_all_strategies()`.

---

## 📊 CLI Reference

| Flag | Description | Default |
| :--- | :--- | :---: |
| `-i, --iterations` | Number of turns per match | 200 |
| `--action-noise` | Probability of move flip (0.0 - 1.0) | 0.0 |
| `--perception-noise` | Probability of misinterpretation | 0.0 |
| `--discount-factor` | Probability of match end per turn | 0.0 |
| `--evolution` | Enable evolutionary mode | false |
| `--spatial` | Enable spatial grid mode | false |
| `--grid-size` | Width/Height of spatial grid | 20 |
| `--seed` | RNG seed for reproducibility | None |
| `--export-csv` | Path to export final results | None |

---

## 📜 License
This project is licensed under the MIT License - see the LICENSE file for details.
