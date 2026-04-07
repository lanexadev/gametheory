use clap::Parser;
use game_theory::{Tournament, strategies, Game, SpatialTournament};
use std::collections::HashMap;

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

    let strategies = strategies::get_all_strategies();
    let mut results: HashMap<String, i32> = HashMap::new();

    if args.spatial {
        println!("Running Spatial Tournament ({}x{} grid) for {} generations...", args.grid_size, args.grid_size, args.generations);
        let mut spatial_tournament = SpatialTournament::new(args.grid_size, args.grid_size, strategies, game.clone());
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

    let mut tournament = Tournament::new(strategies.clone(), game.clone());

    if args.evolution {
        println!("Running Evolutionary Tournament ({} generations, {:.0}% reproduction)...", args.generations, args.reproduction_rate * 100.0);
        results = tournament.run_evolution(args.generations, args.reproduction_rate);
        
        // Results map strategy names to scores in the final generation
        display_results(&results);
    } else if args.swiss {
        println!("Running Swiss System ({} rounds)...", args.swiss_rounds);
        results = tournament.run_swiss(args.swiss_rounds);
        display_results(&results);
    } else {
        println!("Running Round Robin...");
        for _ in 0..args.repetitions {
            let round_results = tournament.run_round_robin();
            for (name, score) in round_results {
                *results.entry(name).or_insert(0) += score;
            }
        }
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
