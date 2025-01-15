
#[derive(Debug, PartialEq)]
pub enum GameBetStatus {
    InProgress,
    ToSettle,
    Settled
}


impl GameBetStatus {
    // Convert enum to String
    fn to_string(&self) -> String {
        match self {
            GameBetStatus::InProgress => "in-progress".to_string(),
            GameBetStatus::ToSettle => "to-settle".to_string(),
            GameBetStatus::Settled => "settled".to_string(),
        }
    }

    // Convert String to enum
    fn from_string(s: &str) -> Result<GameBetStatus, String> {
        match s.to_lowercase().as_str() {
            "in-progress" => Ok(GameBetStatus::InProgress),
            "to-settle" => Ok(GameBetStatus::ToSettle),
            "settled" => Ok(GameBetStatus::Settled),
            _ => Err(format!("Invalid game bet status: {}", s)),
        }
    }
}

// Alternatively, you can implement the standard ToString and FromStr traits:
impl std::fmt::Display for GameBetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for GameBetStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GameBetStatus::from_string(s)
    }
}