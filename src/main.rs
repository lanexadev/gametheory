use clap::Parser;
use game_theory::{Tournament, strategies};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 200)]
    iterations: usize,

    #[arg(short, long, default_value_t = 0.0)]
    noise: f64,

    #[arg(short, long, default_value_t = 1)]
    repetitions: usize,

    #[arg(short, long, default_value_t = 10)]
    swiss_rounds: usize,

    #[arg(long)]
    swiss: bool,

    #[arg(long)]
    finale: bool,
}

fn main() {
    let args = Args::parse();

    println!("Starting Axelrod Tournament...");
    println!("Iterations: {}, Noise: {}, Repetitions: {}", args.iterations, args.noise, args.repetitions);

    let strategies = strategies::get_all_strategies();
    let tournament = Tournament::new(strategies, args.iterations, args.noise);

    if args.swiss {
        println!("Running Swiss System ({} rounds)...", args.swiss_rounds);
        let results = tournament.run_swiss(args.swiss_rounds);
        display_results(results);
    } else {
        println!("Running Round Robin...");
        let mut total_scores = std::collections::HashMap::new();

        for _ in 0..args.repetitions {
            let results = tournament.run_round_robin();
            for (name, score) in results {
                *total_scores.entry(name).or_insert(0) += score;
            }
        }
        display_results(total_scores);
    }

    if args.finale {
        println!("\nRunning Grand Finale for top 3...");
        let winner = tournament.run_grand_finale(3);
        println!("The Grand Winner is: {}", winner);
    }
}

fn display_results(scores: std::collections::HashMap<String, i32>) {
    let mut final_results: Vec<_> = scores.into_iter().collect();
    final_results.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\nFinal Results (Top 20):");
    println!("{:<30} | {:<10}", "Strategy", "Total Score");
    println!("{:-<30}-|-{:-<10}", "", "");
    for (name, score) in final_results.iter().take(20) {
        println!("{:<30} | {:<10}", name, score);
    }
}
