use std::collections::HashMap;

use crate::whist::Game;
use crate::{db, Db};
use crate::error::Error;

pub const DEFAULT_RATING: i32 = 1000;
const K: f64 = 32.0;

/// Pure computation: takes all games with their plays (ordered by game ID ascending),
/// returns login_id → elo map.
///
/// Uses per-game pairwise ELO: for each game, rank linked players by their current
/// cumulative score and apply pairwise ELO updates (higher score beats lower score).
/// K is divided by (n_linked - 1) so that total ELO impact per game stays constant
/// regardless of how many players are linked.
pub fn compute_ratings(games: &[(Game, Vec<(i64, String, String)>)]) -> HashMap<i64, i32> {
    let mut ratings: HashMap<i64, f64> = HashMap::new();

    for (game, plays) in games {
        let alias_to_id: HashMap<&str, i64> = plays
            .iter()
            .map(|(id, alias, _)| (alias.as_str(), *id))
            .collect();

        // Collect (login_id, final_score) for each linked player position
        let final_scores = game.last_score();
        let linked: Vec<(i64, i16)> = (&game.players)
            .into_iter()
            .enumerate()
            .filter_map(|(pos, name)| {
                alias_to_id
                    .get(name.as_str())
                    .map(|&id| (id, final_scores.0[pos]))
            })
            .collect();

        let n = linked.len();
        if n < 2 {
            continue;
        }

        let k_pair = K / (n - 1) as f64;

        for i in 0..n {
            for j in (i + 1)..n {
                let (id_i, score_i) = linked[i];
                let (id_j, score_j) = linked[j];

                if score_i == score_j {
                    continue; // tie — no update
                }

                let (winner, loser) = if score_i > score_j {
                    (id_i, id_j)
                } else {
                    (id_j, id_i)
                };

                let r_w = *ratings.get(&winner).unwrap_or(&(DEFAULT_RATING as f64));
                let r_l = *ratings.get(&loser).unwrap_or(&(DEFAULT_RATING as f64));
                let e_w = 1.0 / (1.0 + 10_f64.powf((r_l - r_w) / 400.0));
                ratings.insert(winner, r_w + k_pair * (1.0 - e_w));
                ratings.insert(loser, r_l + k_pair * (0.0 - (1.0 - e_w)));
            }
        }
    }

    ratings
        .into_iter()
        .map(|(id, elo)| (id, elo.round() as i32))
        .collect()
}

/// Fetches all game data, computes ELO ratings, and atomically writes them to the DB.
pub async fn recompute_all(db: Db) -> Result<(), Error> {
    let games = db::get_all_games_for_rating(db.clone()).await?;
    let ratings = compute_ratings(&games);
    db::upsert_ratings(db, &ratings).await
}
