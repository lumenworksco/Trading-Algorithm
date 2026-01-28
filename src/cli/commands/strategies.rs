//! List strategies command.

use anyhow::Result;
use trading_strategies::StrategyRegistry;

pub async fn run() -> Result<()> {
    let registry = StrategyRegistry::new();

    println!("Available Strategies");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    for info in registry.list() {
        println!("  {} ", info.name);
        println!("  ───────────────────────────────────────────────────────");
        println!("  {}", info.description);
        println!();
    }

    println!("Use --strategy <name> to select a strategy.");
    println!();
    println!("Strategy names: ma_crossover, mean_reversion, momentum, rsi");

    Ok(())
}
