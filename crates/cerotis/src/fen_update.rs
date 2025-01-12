use std::time::{Instant, Duration};

#[derive(Debug, Clone)]
struct ChessState {
    board: Vec<Vec<char>>,
    active_color: char,
    castling_rights: String,
    en_passant: String,
    halfmove_clock: u32,
    fullmove_number: u32,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    rank: usize,
    file: usize,
}

#[derive(Debug)]
pub struct TimedResult {
    pub fen: String,
    pub duration: Duration,
}

impl Position {
    fn from_algebraic(notation: &str) -> Option<Position> {
        if notation.len() != 2 {
            return None;
        }
        
        let file = (notation.chars().next()? as u8 - b'a') as usize;
        let rank = (notation.chars().nth(1)? as u8 - b'1') as usize;
        
        if file > 7 || rank > 7 {
            return None;
        }
        
        Some(Position { rank, file })
    }
}

pub fn update_fen_with_timing(fen: &str, piece: char, from: &str, to: &str, promotion: Option<char>) -> Option<TimedResult> {
    let start = Instant::now();
    
    let result = update_fen(fen, piece, from, to, promotion)?;
    let duration = start.elapsed();
    
    Some(TimedResult {
        fen: result,
        duration,
    })
}

fn update_fen(fen: &str, piece: char, from: &str, to: &str, promotion: Option<char>) -> Option<String> {
    let mut state = parse_complete_fen(fen)?;
    let from_pos = Position::from_algebraic(from)?;
    let to_pos = Position::from_algebraic(to)?;
    
    // Get the piece at the source position
    let source_piece = state.board[from_pos.rank][from_pos.file];
    
    // Check if the piece color matches the active color
    let is_white_piece = source_piece.is_uppercase();
    if (state.active_color == 'w' && !is_white_piece) || 
       (state.active_color == 'b' && is_white_piece) {
        println!("WRONG COLOR ERROR");
        return None; // Wrong color piece for current turn
    }
    

    
    // Handle pawn promotion
    let piece_to_place = if let Some(promoted_piece) = promotion {
        if piece.to_ascii_uppercase() != 'P' {
            println!("No Pawn error");
            return None; // Only pawns can be promoted
        }
        if to_pos.rank != 0 && to_pos.rank != 7 {
            println!("Invalid Promo ERrror");
            return None; // Promotion only on first/last rank
        }
        // Set the correct case for the promoted piece based on the moving piece's color
        if is_white_piece {
            promoted_piece.to_ascii_uppercase()
        } else {
            promoted_piece.to_ascii_lowercase()
        }
    } else {
        source_piece
    };
    
    // Reset en passant target
    state.en_passant = "-".to_string();
    
    // Update en passant target for pawn double moves
    if piece.to_ascii_uppercase() == 'P' {
        let rank_diff = (to_pos.rank as i32 - from_pos.rank as i32).abs();
        if rank_diff == 2 {
            let ep_rank = (from_pos.rank + to_pos.rank) / 2;
            let file_char = (b'a' + from_pos.file as u8) as char;
            let rank_char = (b'1' + ep_rank as u8) as char;
            state.en_passant = format!("{}{}", file_char, rank_char);
        }
    }
    
    // Update halfmove clock
    if piece.to_ascii_uppercase() == 'P' || state.board[to_pos.rank][to_pos.file] != ' ' {
        state.halfmove_clock = 0;
    } else {
        state.halfmove_clock += 1;
    }
    
    // Update fullmove number
    if state.active_color == 'b' {
        state.fullmove_number += 1;
    }
    
    // Update castling rights
    if piece.to_ascii_uppercase() == 'K' {
        if state.active_color == 'w' {
            state.castling_rights = state.castling_rights.replace("K", "").replace("Q", "");
        } else {
            state.castling_rights = state.castling_rights.replace("k", "").replace("q", "");
        }
    } else if piece.to_ascii_uppercase() == 'R' {
        if from_pos.rank == 0 && from_pos.file == 0 {
            state.castling_rights = state.castling_rights.replace("q", "");
        } else if from_pos.rank == 0 && from_pos.file == 7 {
            state.castling_rights = state.castling_rights.replace("k", "");
        } else if from_pos.rank == 7 && from_pos.file == 0 {
            state.castling_rights = state.castling_rights.replace("Q", "");
        } else if from_pos.rank == 7 && from_pos.file == 7 {
            state.castling_rights = state.castling_rights.replace("K", "");
        }
    }
    
    if state.castling_rights.is_empty() {
        state.castling_rights = "-".to_string();
    }
    
    // Move the piece
    state.board[from_pos.rank][from_pos.file] = ' ';
    state.board[to_pos.rank][to_pos.file] = piece_to_place;
    
    // Switch active color
    state.active_color = if state.active_color == 'w' { 'b' } else { 'w' };
    
    Some(state_to_fen(&state))
}



// [Previous helper functions remain the same]
fn parse_complete_fen(fen: &str) -> Option<ChessState> {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() != 6 {
        return None;
    }
    
    let board = parse_fen_board(parts[0])?;
    
    Some(ChessState {
        board,
        active_color: parts[1].chars().next()?,
        castling_rights: parts[2].to_string(),
        en_passant: parts[3].to_string(),
        halfmove_clock: parts[4].parse().ok()?,
        fullmove_number: parts[5].parse().ok()?
    })
}

fn parse_fen_board(fen_board: &str) -> Option<Vec<Vec<char>>> {
    let ranks: Vec<&str> = fen_board.split('/').collect();
    if ranks.len() != 8 {
        return None;
    }
    
    let mut board = vec![vec![' '; 8]; 8];
    
    for (rank_idx, rank) in ranks.iter().enumerate() {
        let mut file_idx = 0;
        
        for c in rank.chars() {
            if file_idx >= 8 {
                return None;
            }
            
            if c.is_digit(10) {
                let empty_squares = c.to_digit(10)? as usize;
                file_idx += empty_squares;
            } else {
                board[7 - rank_idx][file_idx] = c;
                file_idx += 1;
            }
        }
        
        if file_idx != 8 {
            return None;
        }
    }
    
    Some(board)
}

fn state_to_fen(state: &ChessState) -> String {
    let board_fen = board_to_fen(&state.board);
    format!("{} {} {} {} {} {}", 
        board_fen,
        state.active_color,
        state.castling_rights,
        state.en_passant,
        state.halfmove_clock,
        state.fullmove_number
    )
}

fn board_to_fen(board: &Vec<Vec<char>>) -> String {
    let mut fen = String::new();
    
    for rank in (0..8).rev() {
        let mut empty_count = 0;
        
        for file in 0..8 {
            let piece = board[rank][file];
            
            if piece == ' ' {
                empty_count += 1;
            } else {
                if empty_count > 0 {
                    fen.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                fen.push(piece);
            }
        }
        
        if empty_count > 0 {
            fen.push_str(&empty_count.to_string());
        }
        
        if rank > 0 {
            fen.push('/');
        }
    }
    
    fen
}