# Advanced Axelrod Game Theory Engine in Rust

This project is a highly optimized and feature-rich Rust implementation of Robert Axelrod's Iterated Prisoner's Dilemma tournament. It supports evolutionary dynamics, spatial grid tournaments, complex noise models, and customizable game theory parameters.

## New Advanced Features

- **Evolutionary Dynamics**: Simulate population growth over generations using natural selection (`--evolution`).
- **Spatial Grid Tournaments**: Place strategies on a 2D grid where they play their neighbors and the most successful strategies conquer adjacent cells (`--spatial`).
- **Parallel Execution**: Uses `rayon` to parallelize round-robin and swiss match execution, supporting massive strategy pools.
- **Custom Payoff Matrix**: Adjust the core game theory incentives: Temptation (T), Reward (R), Punishment (P), and Sucker's payoff (S).
- **Advanced Noise Models**: 
  - **Action Noise**: Probablity a player accidentally plays the wrong move.
  - **Perception Noise**: Probability a player *misinterprets* the opponent's move, decoupling action from perceived history.
- **Discount Factor**: Probability that the game ends after any given round, simulating the uncertainty of future interactions.
- **Data Export**: Export final scores to CSV for external analysis (`--export-csv`).
- **Reproducibility**: Set an RNG seed (`--seed`) for deterministic simulations despite noise and random strategies.
- **New Strategy Families**: Added `Handshake` (group recognition), `Statistical` (basic opponent modeling), and `Limited Memory` variants.

## How to Use

### Basic Tournament
```bash
cargo run -- --iterations 200 --action-noise 0.05
```

### Evolutionary Simulation
Simulate 100 generations where the bottom 20% of strategies are replaced by clones of the top 20%:
```bash
cargo run -- --evolution --generations 100 --reproduction-rate 0.2
```

### Spatial Grid Tournament
Run a cellular automaton style simulation on a 50x50 grid for 20 generations:
```bash
cargo run -- --spatial --grid-size 50 --generations 20
```

### Custom Game Parameters
Modify the payoff matrix and add an unknown end-game probability (discount factor):
```bash
cargo run -- --payoff-t 6 --payoff-r 4 --discount-factor 0.01
```

### Exporting Data and Reproducibility
```bash
cargo run -- --export-csv results.csv --seed 42
```

## Adding new strategies

You can quickly add new strategies in `src/strategies.rs` using `FunctionalStrategy`, or by implementing the `Strategy` trait for complex stateful logic.
