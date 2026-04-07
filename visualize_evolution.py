import pandas as pd
import matplotlib.pyplot as plt
import sys
import os

def visualize(csv_path):
    # Load the data
    df = pd.read_csv(csv_path)
    
    # Set Generation as index
    df.set_index('Generation', inplace=True)
    
    # Filter: only keep strategies that had at least 5% of population at some point
    # to avoid cluttering the graph with hundreds of lines
    pop_threshold = df.max().max() * 0.05
    significant_strats = [col for col in df.columns if df[col].max() > pop_threshold]
    
    df_filtered = df[significant_strats]
    
    # Create the plot
    plt.figure(figsize=(15, 8))
    
    # Stacked Area Chart for population share
    df_filtered.plot.area(ax=plt.gca(), alpha=0.7)
    
    plt.title('Evolution of Strategy Populations Over Generations', fontsize=16)
    plt.xlabel('Generation', fontsize=12)
    plt.ylabel('Number of Individuals', fontsize=12)
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    plt.grid(alpha=0.3)
    
    # Save the plot
    output_path = csv_path.replace('.csv', '.png')
    plt.tight_layout()
    plt.savefig(output_path, dpi=300)
    print(f"Visualization saved to {output_path}")

    # Top 5 final strategies
    print("\n--- Top 5 Survivors ---")
    final_gen = df.iloc[-1].sort_values(ascending=False).head(5)
    for name, count in final_gen.items():
        if count > 0:
            print(f"{name}: {count} individuals")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python visualize_evolution.py <path_to_evolution_csv>")
    else:
        visualize(sys.argv[1])
