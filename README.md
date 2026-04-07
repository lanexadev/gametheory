# Axelrod's Game Theory Tournament in Rust

This project is a Rust implementation of Robert Axelrod's famous Iterated Prisoner's Dilemma tournament. It allows for testing numerous strategies, including those with noise (random action flips) and different tournament formats.

## Features

- **Robust Strategy Trait**: Easily implement new strategies by defining their `next_move`.
- **Functional Strategy Factory**: Quickly generate hundreds of strategy variants using closures.
- **Tournament Modes**:
  - **Round Robin**: Every strategy plays against every other strategy (including itself).
  - **Swiss System**: Strategies are paired based on their current scores.
- **Grand Finale**: A high-stakes tournament for the top performers.
- **Configurable Parameters**: Adjust iterations, noise levels, and repetitions via CLI.

## How to use

### Run a basic tournament
```bash
cargo run -- --iterations 200 --noise 0.05 --repetitions 1
```

### Run with a Grand Finale for the top performers
```bash
cargo run -- --iterations 200 --noise 0.02 --finale
```

### Run a Swiss System tournament
```bash
cargo run -- --swiss --swiss-rounds 10
```

## Adding new strategies

You can add new strategies in `src/strategies.rs`. Use the `FunctionalStrategy` for quick parametric generation:

```rust
strategies.push(Box::new(FunctionalStrategy {
    name: "MyCustomStrategy".to_string(),
    next_move_fn: |my_h, opp_h| {
        // Your logic here
        Action::Cooperate
    },
}));
```

## Scoring Table

| Action 1  | Action 2  | Score 1 | Score 2 | Result                |
|-----------|-----------|---------|---------|-----------------------|
| Cooperate | Cooperate | 3       | 3       | Mutual Cooperation    |
| Cooperate | Defect    | 0       | 5       | Sucker's Payoff       |
| Defect    | Cooperate | 5       | 0       | Temptation to Defect  |
| Defect    | Defect    | 1       | 1       | Punishment for Defect |
