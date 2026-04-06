/// Retroactive ELO rating seeder.
///
/// Reads all games from SQLite, computes ELO ratings from scratch, and writes
/// the results to the `rating` table. Run this once after deployment to seed
/// historical data, or after retroactively linking players to old games.
///
/// Usage:
///   cargo run --bin recalculate_ratings
use std::sync::Arc;

use whistbook::Db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = whistbook::db::create_pool().await?;
    let db = Db(Arc::new(pool));

    whistbook::rating::recompute_all(db).await?;

    println!("Ratings recalculated successfully.");
    Ok(())
}
