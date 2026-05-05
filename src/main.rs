use clap::{Parser, ValueEnum};
use game_theory::{Tournament, strategies, Game, SpatialTournament, Neighborhood};
use std::collections::HashMap;

#[derive(ValueEnum, Clone, Copy, Debug)]
enum TopologyArg { Moore, Vonneumann, Hex }

impl From<TopologyArg> for Neighborhood {
    fn from(t: TopologyArg) -> Self {
        match t {
            TopologyArg::Moore => Neighborhood::Moore,
            TopologyArg::Vonneumann => Neighborhood::VonNeumann,
            TopologyArg::Hex => Neighborhood::Hex,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 200)]
    iterations: usize,

    #[arg(long, default_value_t = 0.0)]
    action_noise: f64,
    
    #[arg(long, default_value_t = 0.0)]
    perception_noise: f64,
    
    #[arg(long, default_value_t = 0.0)]
    discount_factor: f64,

    #[arg(short, long, default_value_t = 1)]
    repetitions: usize,

    #[arg(short, long, default_value_t = 10)]
    swiss_rounds: usize,

    #[arg(long)]
    swiss: bool,

    #[arg(long)]
    finale: bool,
    
    #[arg(long)]
    evolution: bool,
    
    #[arg(long, default_value_t = 50)]
    generations: usize,
    
    #[arg(long, default_value_t = 0.2)]
    reproduction_rate: f64,
    
    #[arg(long)]
    spatial: bool,
    
    #[arg(long, default_value_t = 20)]
    grid_size: usize,
    
    #[arg(long)]
    export_csv: Option<String>,
    
    #[arg(long)]
    seed: Option<u64>,
    
    #[arg(long, default_value_t = 5)]
    payoff_t: i32,
    #[arg(long, default_value_t = 3)]
    payoff_r: i32,
    #[arg(long, default_value_t = 1)]
    payoff_p: i32,
    #[arg(long, default_value_t = 0)]
    payoff_s: i32,

    /// Disable self-play in round-robin (Axelrod's original setup includes it).
    #[arg(long)]
    no_self_play: bool,

    /// Spatial neighborhood topology: moore (default, 8), vonneumann (4), hex (6).
    #[arg(long, value_enum, default_value_t = TopologyArg::Moore)]
    topology: TopologyArg,

    /// Export the full N×N pair-score matrix (mean per-turn) as CSV.
    #[arg(long)]
    export_matrix: Option<String>,

    /// Evolution mutation rate in [0,1]: probability that a child slot is replaced
    /// by a fresh draw from the global strategy pool (exploration). 0 = legacy
    /// behaviour (no mutation, population bounded forever by the initial set).
    #[arg(long, default_value_t = 0.0)]
    mutation_rate: f64,

    /// Evolution selection temperature. 0 = top-N truncation (legacy, deterministic).
    /// >0 = softmax-weighted roulette over fitness — higher T preserves diversity,
    /// lower T converges toward truncation.
    #[arg(long, default_value_t = 0.0)]
    selection_temperature: f64,
}

fn main() {
    let args = Args::parse();

    println!("Starting Advanced Axelrod Tournament...");
    println!("Iterations: {}, Action Noise: {}, Perception Noise: {}, Discount Factor: {}", 
             args.iterations, args.action_noise, args.perception_noise, args.discount_factor);
    println!("Payoffs - T:{}, R:{}, P:{}, S:{}", args.payoff_t, args.payoff_r, args.payoff_p, args.payoff_s);
    if let Some(seed) = args.seed {
        println!("Seed: {}", seed);
    }

    let game = Game {
        iterations: args.iterations,
        action_noise: args.action_noise,
        perception_noise: args.perception_noise,
        discount_factor: args.discount_factor,
        payoffs: (args.payoff_t, args.payoff_r, args.payoff_p, args.payoff_s),
        seed: args.seed,
    };

    if let Err(e) = game.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let strategies = strategies::get_all_strategies();

    // ZD strategies are derived from canonical Axelrod payoffs (5,3,1,0). Under
    // any other valid IPD payoff vector the Press-Dyson invariant no longer
    // holds — the strategies still play coherently but lose their theoretical
    // guarantees. Warn the user once at startup so the result interpretation
    // stays honest.
    let canonical_payoffs = (5, 3, 1, 0);
    let user_payoffs = (args.payoff_t, args.payoff_r, args.payoff_p, args.payoff_s);
    if user_payoffs != canonical_payoffs && strategies.iter().any(|s| s.name().starts_with("ZD-")) {
        eprintln!(
            "Warning: ZD strategies present but payoffs are non-canonical (expected {:?}, got {:?}). The Press-Dyson invariant no longer holds; ZD agents will run but their extortion/generosity guarantees are void.",
            canonical_payoffs, user_payoffs
        );
    }

    let results: HashMap<String, i32>;

    if args.spatial {
        println!(
            "Running Spatial Tournament ({}x{} grid, topology={:?}) for {} generations...",
            args.grid_size, args.grid_size, args.topology, args.generations
        );
        let mut spatial_tournament = SpatialTournament::new_with_topology(
            args.grid_size,
            args.grid_size,
            strategies,
            game.clone(),
            args.topology.into(),
        );
        for _ in 0..args.generations {
            spatial_tournament.step();
        }
        let counts = spatial_tournament.get_population_counts();
        println!("\nFinal Spatial Population:");
        let mut sorted_counts: Vec<_> = counts.into_iter().collect();
        sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));
        for (name, count) in sorted_counts.iter().take(20) {
            println!("{:<30} | {} cells", name, count);
        }
        return; // Spatial has different metric (population count, not score)
    }

    let mut tournament = Tournament::new(strategies.clone(), game.clone())
        .with_match_repetitions(args.repetitions)
        .with_include_self_play(!args.no_self_play);

    if args.evolution {
        println!(
            "Running Evolutionary Tournament ({} generations, {:.0}% reproduction, mutation={}, selection_T={})...",
            args.generations,
            args.reproduction_rate * 100.0,
            args.mutation_rate,
            args.selection_temperature,
        );
        // Mutation pool: full diverse set of strategies, NOT just the current
        // tournament population — this is what gives evolution real exploration
        // power instead of mere recombination.
        let mutation_pool = if args.mutation_rate > 0.0 {
            Some(strategies::get_all_strategies())
        } else {
            None
        };
        let (final_scores, evolution_history) = tournament.run_evolution_with_options(
            args.generations,
            args.reproduction_rate,
            args.mutation_rate,
            args.selection_temperature,
            mutation_pool,
        );
        results = final_scores;
        
        if let Some(path) = &args.export_csv {
            let history_path = path.replace(".csv", "_evolution.csv");
            if let Err(e) = export_evolution_history(&history_path, &evolution_history) {
                eprintln!("Failed to export evolution history: {}", e);
            } else {
                println!("Evolution history exported to {}", history_path);
            }
        }
        display_results(&results);
    } else if args.swiss {
        println!("Running Swiss System ({} rounds)...", args.swiss_rounds);
        results = tournament.run_swiss(args.swiss_rounds);
        display_results(&results);
    } else {
        println!(
            "Running Round Robin (match_repetitions={}, self_play={})...",
            args.repetitions,
            !args.no_self_play
        );
        results = tournament.run_round_robin();
        display_results(&results);
    }

    if args.finale {
        println!("\nRunning Grand Finale for top 3...");
        let winner = tournament.run_grand_finale(3);
        println!("The Grand Winner is: {}", winner);
    }

    if let Some(path) = args.export_csv {
        if let Err(e) = export_to_csv(&path, &results) {
            eprintln!("Failed to export CSV: {}", e);
        } else {
            println!("Results exported to {}", path);
        }
    }

    if let Some(path) = args.export_matrix {
        // Re-run a clean round-robin report to capture the matrix. Cheaper
        // than threading it through every code path; the report itself is
        // cached implicitly because the Game seed makes the run deterministic.
        let report = tournament.run_round_robin_report();
        match report.export_matrix_csv(&path) {
            Ok(()) => println!("Pair-score matrix exported to {}", path),
            Err(e) => eprintln!("Failed to export matrix: {}", e),
        }
    }
}

fn display_results(scores: &HashMap<String, i32>) {
    let mut final_results: Vec<_> = scores.iter().collect();
    final_results.sort_by(|a, b| b.1.cmp(a.1));

    println!("\nFinal Results (Top 20):");
    println!("{:<30} | {:<10}", "Strategy", "Total Score");
    println!("{:-<30}-|-{:-<10}", "", "");
    for (name, score) in final_results.iter().take(20) {
        println!("{:<30} | {:<10}", name, score);
    }
}

fn export_evolution_history(path: &str, history: &[HashMap<String, usize>]) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    
    // Get all unique strategy names
    let mut all_names: Vec<_> = history.iter()
        .flat_map(|h| h.keys())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    all_names.sort();

    // Header: Generation, Strategy1, Strategy2, ...
    let mut header = vec!["Generation".to_string()];
    for name in &all_names {
        header.push(name.to_string());
    }
    wtr.write_record(&header)?;

    for (generation, counts) in history.iter().enumerate() {
        let mut row = vec![generation.to_string()];
        for name in &all_names {
            let count = counts.get(*name).unwrap_or(&0);
            row.push(count.to_string());
        }
        wtr.write_record(&row)?;
    }
    
    wtr.flush()?;
    Ok(())
}

fn export_to_csv(path: &str, scores: &HashMap<String, i32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(&["Strategy", "Score"])?;
    
    let mut final_results: Vec<_> = scores.iter().collect();
    final_results.sort_by(|a, b| b.1.cmp(a.1));
    
    for (name, score) in final_results {
        wtr.write_record(&[name, &score.to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}
