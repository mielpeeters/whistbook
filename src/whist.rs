use std::ops::{Add, AddAssign, Index, IndexMut, Sub};

use serde::{Deserialize, Serialize};

/// All bids that are played with one player
const SOLOBIDS: [&str; 12] = [
    "Solo 5",
    "Solo 6",
    "Solo 7",
    "Solo 8",
    "Kleine Miserie",
    "Grote Miserie",
    "Open Miserie",
    "Abondance 9",
    "Abondance 10",
    "Abondance 11",
    "Abondance 12",
    "Solo Slim",
];

/// All bids that are played with two players
const DUOBIDS: [&str; 7] = [
    "Samen 8", "Samen 9", "Samen 10", "Samen 11", "Samen 12", "Samen 13", "Troel",
];

pub fn solo_bids() -> Vec<String> {
    SOLOBIDS.iter().map(|s| s.to_string()).collect()
}

pub fn duo_bids() -> Vec<String> {
    DUOBIDS.iter().map(|s| s.to_string()).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Bid {
    Solo(i16),
    Samen(i16),
    Abondance(i16),
    SmallMisery,
    Trull,
    LargeMisery,
    OpenMisery,
    GrandSlam,
}

impl From<&str> for Bid {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "solo 5" => Bid::Solo(5),
            "solo 6" => Bid::Solo(6),
            "solo 7" => Bid::Solo(7),
            "solo 8" => Bid::Solo(8),
            "samen 8" => Bid::Samen(8),
            "samen 9" => Bid::Samen(9),
            "samen 10" => Bid::Samen(10),
            "samen 11" => Bid::Samen(11),
            "samen 12" => Bid::Samen(12),
            "samen 13" => Bid::Samen(13),
            "abondance 9" => Bid::Abondance(9),
            "abondance 10" => Bid::Abondance(10),
            "abondance 11" => Bid::Abondance(11),
            "abondance 12" => Bid::Abondance(12),
            "abondance 13" => Bid::Abondance(13),
            "solo slim" => Bid::GrandSlam,
            "kleine miserie" => Bid::SmallMisery,
            "grote miserie" => Bid::LargeMisery,
            "open miserie" => Bid::OpenMisery,
            "troel" => Bid::Trull,
            _ => unreachable!(),
        }
    }
}

impl Bid {
    /// the amount of points that the playing team gets
    ///
    /// if it is a solo game, the point amount is to be multiplied by 3
    pub fn points(&self, achieved: i16) -> i16 {
        match self {
            Bid::Solo(5) => {
                if 5 <= achieved {
                    (achieved - 2).min(6)
                } else {
                    achieved - 8
                }
            }
            Bid::Solo(6) => {
                if 6 <= achieved {
                    (achieved - 2).min(6)
                } else {
                    achieved - 10
                }
            }
            Bid::Solo(7) => {
                if 7 <= achieved {
                    (achieved - 2).min(6)
                } else {
                    achieved - 12
                }
            }
            Bid::Solo(8) => {
                if 8 <= achieved {
                    7
                } else {
                    achieved - 15
                }
            }
            Bid::Samen(8) => {
                if achieved == 13 {
                    30
                } else if 8 <= achieved {
                    8 + 3 * (achieved - 8)
                } else {
                    3 * (achieved - 8) - 8
                }
            }
            Bid::Samen(9) => {
                if achieved == 13 {
                    30
                } else if 9 <= achieved {
                    11 + 3 * (achieved - 9)
                } else {
                    3 * (achieved - 9) - 11
                }
            }
            Bid::Samen(10) => {
                if achieved == 13 {
                    30
                } else if 10 <= achieved {
                    14 + 3 * (achieved - 10)
                } else {
                    3 * (achieved - 10) - 14
                }
            }
            Bid::Samen(11) => {
                if achieved == 13 {
                    30
                } else if 11 <= achieved {
                    17 + 3 * (achieved - 11)
                } else {
                    3 * (achieved - 11) - 17
                }
            }
            Bid::Samen(12) => {
                if achieved == 13 {
                    30
                } else if 12 == achieved {
                    20
                } else {
                    3 * (achieved - 12) - 20
                }
            }
            Bid::Samen(13) => {
                if achieved == 13 {
                    30
                } else {
                    3 * (achieved - 13) - 23
                }
            }
            Bid::Abondance(9) => match achieved {
                9 => 10,
                10 => 15,
                11 => 20,
                12 => 30,
                13 => 60,
                _ => -10,
            },
            Bid::Abondance(10) => match achieved {
                10 => 15,
                11 => 20,
                12 => 30,
                13 => 60,
                _ => -15,
            },
            Bid::Abondance(11) => match achieved {
                11 => 20,
                12 => 30,
                13 => 60,
                _ => -20,
            },
            Bid::Abondance(12) => match achieved {
                12 => 30,
                13 => 60,
                _ => -30,
            },
            Bid::SmallMisery => match achieved {
                0 => 6,
                _ => -6,
            },
            Bid::Trull => match achieved {
                x if x >= 8 => 16,
                _ => -16,
            },
            Bid::LargeMisery => match achieved {
                0 => 12,
                _ => -12,
            },
            Bid::OpenMisery => match achieved {
                0 => 24,
                _ => -24,
            },
            Bid::GrandSlam => match achieved {
                13 => 60,
                _ => -60,
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Points(pub Vec<i16>);

impl Points {
    fn new(num_players: usize) -> Self {
        Points(vec![0; num_players])
    }

    pub fn positive(&self, i: &usize) -> bool {
        self[*i] > 0
    }
}

impl Default for Points {
    fn default() -> Self {
        Points::new(4)
    }
}

impl Index<usize> for Points {
    type Output = i16;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Points {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl AddAssign for Points {
    fn add_assign(&mut self, rhs: Self) {
        (0..4).for_each(|i| self.0[i] += rhs.0[i])
    }
}

impl From<&[i16]> for Points {
    fn from(value: &[i16]) -> Self {
        let mut result = Self::new(0);
        result.0.extend(value.iter());
        result
    }
}

impl Add for Points {
    type Output = Points;

    fn add(self, rhs: Self) -> Self::Output {
        self.0
            .iter()
            .enumerate()
            .map(|(i, x)| x + rhs[i])
            .collect::<Vec<_>>()
            .as_slice()
            .into()
    }
}

impl<'b> Sub<&'b Points> for &Points {
    type Output = Points;

    fn sub(self, rhs: &'b Points) -> Self::Output {
        self.0
            .iter()
            .enumerate()
            .map(|(i, x)| x - rhs[i])
            .collect::<Vec<_>>()
            .as_slice()
            .into()
    }
}

impl<'b> Add<&'b Points> for &Points {
    type Output = Points;

    fn add(self, rhs: &'b Points) -> Self::Output {
        self.0
            .iter()
            .enumerate()
            .map(|(i, x)| x + rhs[i])
            .collect::<Vec<_>>()
            .as_slice()
            .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Players(Vec<String>);

impl Players {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn opt_add_player(&mut self, opt_player: &str) {
        if !opt_player.is_empty() {
            self.0.push(opt_player.to_string())
        }
    }
}

impl Index<usize> for Players {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        self.0[index].as_str()
    }
}

impl From<&[String]> for Players {
    fn from(value: &[String]) -> Self {
        Self(value.to_vec())
    }
}

impl From<&[&String]> for Players {
    fn from(value: &[&String]) -> Self {
        Self(value.iter().map(|&p| p.to_owned()).collect::<Vec<_>>())
    }
}

impl From<&[&str]> for Players {
    fn from(value: &[&str]) -> Self {
        Self(value.iter().map(|p| p.to_string()).collect::<Vec<_>>())
    }
}

// Implement IntoIterator for Players, consuming the struct
impl IntoIterator for Players {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Players {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Game {
    /// The Name of the Game
    pub name: String,
    pub players: Players,
    /// cumulative points of the game
    pub scores: Vec<Points>,
    pub deals: Vec<Deal>,
}

/// Team holds indexes into the Player struct that define the team
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Team {
    Solo(usize, (usize, usize, usize)),
    Duo((usize, usize), (usize, usize)),
}

/// Keeps the results of a game
/// team is in relation to the Players struct that is defined elsewhere
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deal {
    pub team: Team,
    pub bid: Bid,
    pub achieved: i16,
}

impl Deal {
    pub fn to_points(self, num_players: usize) -> Points {
        let mut points = Points::new(num_players);

        let team_point = self.bid.points(self.achieved);

        match self.team {
            Team::Solo(player, opps) => {
                points[player] = 3 * team_point;

                points[opps.0] = -team_point;
                points[opps.1] = -team_point;
                points[opps.2] = -team_point;
            }
            Team::Duo(players, opps) => {
                points[players.0] = team_point;
                points[players.1] = team_point;

                points[opps.0] = -team_point;
                points[opps.1] = -team_point;
            }
        }

        points
    }
}

impl Game {
    pub fn new<P: Into<Players>>(name: String, players: P) -> Self {
        let players: Players = players.into();
        Self {
            name,
            scores: vec![Points::new(players.len())],
            players,
            deals: vec![],
        }
    }

    pub fn add_deal(&mut self, deal: Deal) {
        self.deals.push(deal.clone());
        let points = deal.to_points(self.players.len());
        self.add_points(points);
    }

    pub fn add_points(&mut self, points: Points) {
        if let Some(last_scores) = self.scores.last() {
            let new_score = last_scores + &points;
            self.scores.push(new_score);
        } else {
            self.scores.push(points);
        }
    }

    pub fn last_score(&self) -> &Points {
        self.scores.last().unwrap()
    }

    /// Returns the points that were achieved in the round
    /// leading up to the n'th score
    pub fn diff(&self, n: usize) -> Option<Points> {
        match n {
            0 => self.scores.first().cloned(),
            x if x >= self.scores.len() => None,
            _ => Some(self.scores.get(n).unwrap() - self.scores.get(n - 1).unwrap()),
        }
    }

    /// Return the points that were obtained in the last round
    pub fn last_diff(&self) -> Option<Points> {
        self.diff(self.scores.len() - 1)
    }
}
