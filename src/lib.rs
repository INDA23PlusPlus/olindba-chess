//! # Chess library
//! 
//! ## How to use:
//! The chess game is handled within the [Game] struct.
//! It can be initialized to the starting position with [Game::starting_position] or
//! set to any position from a FEN string with [Game::new]. Squares on the board are indexed from 0-63 
//! and can be accessed with [Game::board].
//! ### Make moves on the board:
//! * The function [Game::make_move_from_to] can be used without first generating legal moves, 
//! but if the move is illegal the game will ignore it. Note that the user has to know if the move is a promotion
//! and then pass the decided promotion to the function. To avoid this, [Game::make_move] can be used.
//! * The function [Game::make_move] takes a move that has already been generated by either [Game::get_all_legal_moves]
//! or [Game::get_legal_moves] and updates the board accordingly. 
//! The user can check if the move is a promotion with [Move::is_promotion]
//! 
//! ### Current game state
//! The function [Game::get_game_state] can be called at any moment and returns the current game state.
//! In the case of draw by 50-move rule or draw by insufficient material, moves can still be generated and made
//! but this funtion will continuously return Draw and what type of draw 
//! 

pub const EMPTY: usize = 0;
pub const PAWN: usize = 1;
pub const KNIGHT: usize = 2;
pub const BISHOP: usize = 3;
pub const ROOK: usize = 4;
pub const QUEEN: usize = 5;
pub const KING: usize = 6;

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;

const HAS_MOVED: usize = 1;

/// The pieces on the board
#[derive(Copy, Clone)]
pub struct Piece {
    piece: usize
}

impl Piece {

    fn new(piece_type: usize, piece_color: usize, piece_flags: usize) -> Piece {
        Piece {
            piece: ((piece_flags & 0x03) << 4) | ((piece_color & 0x01) << 3) | (piece_type & 0x07)
        }
    }

    fn empty() -> Piece {
        Piece {
            piece: 0
        }
    }

    /// Returns a number between 0 and 6 inclusive, matches the constants EMPTY, PAWN, KNIGHT etc.
    pub fn get_type(&self) -> usize { return self.piece & 0x07; }
    /// Returns either 0 or 1, matches the constants WHITE or BLACK
	pub fn get_color(&self) -> usize { return (self.piece >> 3) & 0x01; }
	fn get_flags(&self) -> usize { return (self.piece >> 4) & 0x03; }

	fn set_type(&mut self, piece_type: usize) { self.piece &= !0x07; self.piece |= piece_type & 0x07; }
	fn set_flags(&mut self, piece_flags: usize) { self.piece &= !0x30; self.piece |= (piece_flags & 0x03) << 4; }

	fn has_moved(&self) -> bool { return self.get_flags() & HAS_MOVED != 0; }
}

const QUIET_MOVE: usize	=	        0b0000;
const DOUBLE_PAWN_PUSH: usize =		0b0001;
const KING_CASTLE: usize =			0b0010;
const QUEEN_CASTLE: usize =			0b0011;
const CAPTURE: usize =				0b0100;
const EP_CAPTURE: usize =			0b0101;
pub const KNIGHT_PROMOTION: usize =	0b1000;
pub const BISHOP_PROMOTION: usize =	0b1001;
pub const ROOK_PROMOTION: usize	=	0b1010;
pub const QUEEN_PROMOTION: usize =	0b1011;
const KNIGHT_PROMOTION_CAP: usize =	0b1100;
const BISHOP_PROMOTION_CAP: usize =	0b1101;
const ROOK_PROMOTION_CAP: usize	=	0b1110;
const QUEEN_PROMOTION_CAP: usize =	0b1111;

#[derive(Copy, Clone)]
pub struct Move {
    chess_move: usize
}

impl Move {

    fn new(from: usize, to: usize, flags: usize) -> Move {
        Move {
            chess_move: ((flags & 0xf) << 12) | ((from & 0x3f) << 6) | (to & 0x3f)
        }
    }
    
    pub fn get_to(&self) -> usize { return self.chess_move & 0x3f; }
	pub fn get_from(&self) -> usize { return (self.chess_move >> 6) & 0x3f; }
	fn get_flags(&self) -> usize { return (self.chess_move >> 12) & 0x0f; }

	pub fn is_capture(&self) -> bool { return self.get_flags() & CAPTURE != 0; }
	pub fn is_promotion(&self) -> bool { return self.get_flags() & (1 << 3) != 0; }
	pub fn is_ep_capture(&self) -> bool { return self.get_flags() == EP_CAPTURE; }
	pub fn is_castle(&self) -> bool { return (self.get_flags() | 1) == QUEEN_CASTLE; }
	pub fn is_double_pawn_push(&self) -> bool { return self.get_flags() == DOUBLE_PAWN_PUSH; }
	pub fn is_queen_castle(&self) -> bool { return self.get_flags() == QUEEN_CASTLE; }
	pub fn is_king_castle(&self) -> bool { return self.get_flags() == KING_CASTLE; }
}


struct Mailbox {
    mailbox64: [usize; 64],
    mailbox120: [isize; 120]
}

impl Mailbox {

    fn new() -> Mailbox {
        let mailbox64 = [
            21, 22, 23, 24, 25, 26, 27, 28,
            31, 32, 33, 34, 35, 36, 37, 38,
            41, 42, 43, 44, 45, 46, 47, 48,
            51, 52, 53, 54, 55, 56, 57, 58,
            61, 62, 63, 64, 65, 66, 67, 68,
            71, 72, 73, 74, 75, 76, 77, 78,
            81, 82, 83, 84, 85, 86, 87, 88,
            91, 92, 93, 94, 95, 96, 97, 98
        ];

        let mailbox120 = [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1,  0,  1,  2,  3,  4,  5,  6,  7, -1,
            -1,  8,  9, 10, 11, 12, 13, 14, 15, -1,
            -1, 16, 17, 18, 19, 20, 21, 22, 23, -1,
            -1, 24, 25, 26, 27, 28, 29, 30, 31, -1,
            -1, 32, 33, 34, 35, 36, 37, 38, 39, -1,
            -1, 40, 41, 42, 43, 44, 45, 46, 47, -1,
            -1, 48, 49, 50, 51, 52, 53, 54, 55, -1,
            -1, 56, 57, 58, 59, 60, 61, 62, 63, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
        ];

        return Mailbox {
            mailbox64,
            mailbox120
        }
    }

    fn get_square_with_offset(&self, from: usize, offset: isize) -> isize {
        return self.mailbox120[(self.mailbox64[from] as isize + offset) as usize];
    }
}

struct MoveGenerator {
    piece_offset: [[isize; 8]; 6],
    piece_offsets: [usize; 6],
    sliding_piece: [bool; 6]
}

impl MoveGenerator {
    fn new() -> MoveGenerator {
        let piece_offset = [
            [   0,   0,  0,  0, 0,  0,  0,  0 ], // EMPTY
		    [ -21, -19,-12, -8, 8, 12, 19, 21 ], // KNIGHT
		    [ -11,  -9,  9, 11, 0,  0,  0,  0 ], // BISHOP
		    [ -10,  -1,  1, 10, 0,  0,  0,  0 ], // ROOK
		    [ -11, -10, -9, -1, 1,  9, 10, 11 ], // QUEEN
		    [ -11, -10, -9, -1, 1,  9, 10, 11 ]  // KING
        ];
        let piece_offsets = [0, 8, 4, 4, 8, 8];
        let sliding_piece = [false, false, true, true, true, false];

        return MoveGenerator { 
            piece_offset,
            piece_offsets,
            sliding_piece
         }
    }

    fn generate_pseudo_legal_moves(&self, game: &Game, square: usize) -> Vec<Move> {
        if game.board[square].get_type() == EMPTY || game.board[square].get_color() != game.turn {
            return vec![];
        }
        if game.board[square].get_type() == PAWN {
            return self.generate_pawn_moves(game, square);
        }
        else {
            return self.generate_non_pawn_moves(game, square);
        }
    }

    fn generate_pawn_moves(&self, game: &Game, square: usize) -> Vec<Move> {
        let mut pseudo_legal_moves = vec![];

        let forward_offset: isize;
        if game.turn == WHITE {
            forward_offset = -8;
        }
        else {
            forward_offset = 8;
        }
        
        let next_square = square as isize + forward_offset;
        if game.get_row(next_square as usize) == 0 || game.get_row(next_square as usize) == 7 {
            if self.pawn_can_capture_left(game, next_square as usize) {
                pseudo_legal_moves.append(
                    &mut vec![
                        Move::new(square, (next_square - 1) as usize, KNIGHT_PROMOTION_CAP),
                        Move::new(square, (next_square - 1) as usize, BISHOP_PROMOTION_CAP),
                        Move::new(square, (next_square - 1) as usize, ROOK_PROMOTION_CAP),
                        Move::new(square, (next_square - 1) as usize, QUEEN_PROMOTION_CAP)
                    ]
                );
            }

            if self.pawn_can_capture_right(game, next_square as usize) {
                pseudo_legal_moves.append(
                    &mut vec![
                        Move::new(square, (next_square + 1) as usize, KNIGHT_PROMOTION_CAP),
                        Move::new(square, (next_square + 1) as usize, BISHOP_PROMOTION_CAP),
                        Move::new(square, (next_square + 1) as usize, ROOK_PROMOTION_CAP),
                        Move::new(square, (next_square + 1) as usize, QUEEN_PROMOTION_CAP)
                    ]
                );
            }
        }
        else {
            if self.pawn_can_capture_left(game, next_square as usize) {
                pseudo_legal_moves.push(Move::new(square, (next_square - 1) as usize, CAPTURE));
            }
            if self.pawn_can_capture_right(game, next_square as usize) {
                pseudo_legal_moves.push(Move::new(square, (next_square + 1) as usize, CAPTURE));
            }
        }

        if game.board[next_square as usize].get_type() == EMPTY {

            if game.get_row(next_square as usize) == 0 || game.get_row(next_square as usize) == 7 {
                pseudo_legal_moves.append(
                    &mut vec![
                        Move::new(square, next_square as usize, KNIGHT_PROMOTION),
                        Move::new(square, next_square as usize, BISHOP_PROMOTION),
                        Move::new(square, next_square as usize, ROOK_PROMOTION),
                        Move::new(square, next_square as usize, QUEEN_PROMOTION)
                    ]
                );
            }
            else {
                pseudo_legal_moves.push(Move::new(square, next_square as usize, QUIET_MOVE));

                let next_square = next_square + forward_offset;

                if (game.get_row(square) == 1 || game.get_row(square) == 6) &&
                game.board[next_square as usize].get_type() == EMPTY {
                    pseudo_legal_moves.push(Move::new(square, next_square as usize, DOUBLE_PAWN_PUSH));
                }
            }
        }

        if game.possible_ep_capture < 64 {
            if game.get_column(square) != 0 && square - 1 == game.possible_ep_capture { 
                pseudo_legal_moves.push(Move::new(square, next_square as usize - 1, EP_CAPTURE));
            }
            if game.get_column(square) != 7 && square + 1 == game.possible_ep_capture { 
                pseudo_legal_moves.push(Move::new(square, next_square as usize + 1, EP_CAPTURE));
            }
        }

        return pseudo_legal_moves;
    }

    fn generate_non_pawn_moves(&self, game: &Game, square: usize) -> Vec<Move> {
        let mut pseudo_legal_moves = vec![];

        let mailbox = Mailbox::new();
        for j in 0..self.piece_offsets[game.board[square].get_type() - 1] {
            let mut to_square: isize = square as isize;
            loop {
                to_square = mailbox.get_square_with_offset(to_square as usize, 
                    self.piece_offset[game.board[square].get_type() - 1][j]);

                if to_square == -1 {
                    break;
                }
                
                if game.board[to_square as usize].get_type() != EMPTY {
                    if game.board[to_square as usize].get_color() != game.turn {
                        pseudo_legal_moves.push(Move::new(square, to_square as usize, CAPTURE));
                    }
                    break;
                }

                pseudo_legal_moves.push(Move::new(square, to_square as usize, QUIET_MOVE));

                if !self.sliding_piece[game.board[square].get_type() - 1] {
                    break;
                }
            }
        }

        if game.board[square].get_type() == KING && !game.board[square].has_moved() {
            let king_rook;
            let queen_rook;
            if game.turn == WHITE {
                king_rook = game.board[7 * 8 + 7];
                queen_rook = game.board[7 * 8];
            }
            else {
                king_rook = game.board[0 * 8 + 7];
                queen_rook = game.board[0 * 8];
            }

            let mut king_side_empty = true;
            let mut queen_side_empty = true;

            if queen_rook.get_type() == ROOK && !queen_rook.has_moved() {
                for j in 0..3 {
                    if game.board[square - j - 1].get_type() != EMPTY {
                        queen_side_empty = false;
                        break;
                    }
                }
                if queen_side_empty {
                    pseudo_legal_moves.push(Move::new(square, square - 2, QUEEN_CASTLE));
                }
            }

            if king_rook.get_type() == ROOK && !king_rook.has_moved() {
                for j in 0..2 {
                    if game.board[square + j + 1].get_type() != EMPTY {
                        king_side_empty = false;
                        break;
                    }
                }
                if king_side_empty {
                    pseudo_legal_moves.push(Move::new(square, square + 2, KING_CASTLE));
                }
            }
        }

        return pseudo_legal_moves;
    }

    fn filter_pseudo_legal_moves(&self, game: &Game, pseudo_legal_moves: Vec<Move>) -> Vec<Move> {
        let mut legal_moves = vec![];
        for mv in pseudo_legal_moves {
            
            if mv.is_castle() {
                let square_besides_king;
                if mv.is_queen_castle() {
                    square_besides_king = mv.get_from() - 1;
                }
                else {
                    square_besides_king = mv.get_from() + 1;
                }
                if self.is_attacked(game, mv.get_from(), game.turn) || 
                self.is_attacked(game, square_besides_king, game.turn) {
                    continue;
                }
            }

            let mut game_copy = game.clone();
            game_copy.make_move(mv);
            if self.is_attacked(&game_copy, game_copy.king_square[game_copy.turn ^ 1], game_copy.turn ^ 1) {
                continue;
            }

            legal_moves.push(mv);
        }
        return legal_moves;
    }

    fn is_attacked(&self, game: &Game, square: usize, color: usize) -> bool {

        let mailbox = Mailbox::new();
        for piece in KNIGHT..=KING {
            for i in 0..self.piece_offsets[piece - 1] {
                let mut to_square: isize = square as isize;
                loop {
                    to_square = mailbox.get_square_with_offset(to_square as usize, 
                        self.piece_offset[piece - 1][i]);

                    if to_square == -1 {
                        break;
                    }
                    
                    let attacking_piece = &game.board[to_square as usize];
                    if attacking_piece.get_type() != EMPTY {
                        if attacking_piece.get_color() != color && attacking_piece.get_type() == piece {
                            return true;
                        }
                        break;
                    }

                    if !self.sliding_piece[piece - 1] {
                        break;
                    }
                }
            }
        }
        if !((color == WHITE && game.get_row(square) <= 1) || (color == BLACK && game.get_row(square) >= 6)) {
            let forward_offset: isize;
            if color == WHITE {
                forward_offset = -8;
            }
            else {
                forward_offset = 8;
            }
            let possible_pawn_cap1: &Piece = &game.board[(square as isize + forward_offset) as usize + 1];
            let possible_pawn_cap2: &Piece = &game.board[(square as isize + forward_offset) as usize - 1];

            if (possible_pawn_cap1.get_type() == PAWN && possible_pawn_cap1.get_color() != color) ||
            (possible_pawn_cap2.get_type() == PAWN && possible_pawn_cap2.get_color() != color) {
                return true;
            }
        }

        return false;
    }

    fn pawn_can_capture_left(&self, game: &Game, next_square: usize) -> bool {
        return game.get_column(next_square) != 0 && game.board[(next_square - 1) as usize].get_color() != game.turn &&
        game.board[(next_square - 1) as usize].get_type() != EMPTY;
    }

    fn pawn_can_capture_right(&self, game: &Game, next_square: usize) -> bool {
        return game.get_column(next_square) != 7 && game.board[(next_square + 1) as usize].get_color() != game.turn &&
        game.board[(next_square + 1) as usize].get_type() != EMPTY;
    }   
}

/// Converts an algebraic notation, example 'e3' to an integer between 0 and 63 inclusive
pub fn convert_algebraic_notation_to_number(alg_not: &str) -> usize {
    let mut square = 0;
    square += match alg_not.chars().nth(0).unwrap() {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => 0
    };
    square += 8 * match alg_not.chars().nth(1).unwrap() {
        '1' => 7,
        '2' => 6,
        '3' => 5,
        '4' => 4,
        '5' => 3,
        '6' => 2,
        '7' => 1,
        '8' => 0,
        _ => 0
    };
    return square;
}

fn convert_fen_to_game(fen: &str) -> Game {

    let fen_parts = fen.split(" ").collect::<Vec<&str>>();
    let board_rows = fen_parts[0].split("/").collect::<Vec<&str>>();

    let mut board = [Piece::empty(); 64];
    for row in 0..8 {
        let mut collumn = 0;
        let mut cur = 0;
        while collumn < 8 {
            board[row * 8 + collumn] =  match board_rows[row].chars().nth(cur).unwrap() {
                'r' => Piece::new(ROOK, BLACK, EMPTY),
                'R' => Piece::new(ROOK, WHITE, EMPTY),
                'b' => Piece::new(BISHOP, BLACK, EMPTY),
                'B' => Piece::new(BISHOP, WHITE, EMPTY),
                'k' => Piece::new(KING, BLACK, EMPTY),
                'K' => Piece::new(KING, WHITE, EMPTY),
                'q' => Piece::new(QUEEN, BLACK, EMPTY),
                'Q' => Piece::new(QUEEN, WHITE, EMPTY),
                'n' => Piece::new(KNIGHT, BLACK, EMPTY),
                'N' => Piece::new(KNIGHT, WHITE, EMPTY),
                'p' => Piece::new(PAWN, BLACK, EMPTY),
                'P' => Piece::new(PAWN, WHITE, EMPTY),
                _ => {
                    collumn += board_rows[row].chars().nth(cur).unwrap().to_digit(10).unwrap() as usize;
                    cur += 1;
                    continue;
                },
            };
            collumn += 1;
            cur += 1;
        }
    }

    let mut king_square = [0; 2];
    for i in 0..64 {
        if board[i].get_type() == KING {
            king_square[board[i].get_color()] = i;
        }
    }

    let turn = match fen_parts[1] {
        "w" => WHITE,
        "b" => BLACK,
        _ => EMPTY
    };

    let castle_rights = fen_parts[2];
    if !castle_rights.contains('K') {
        board[7 * 8 + 7].set_flags(HAS_MOVED);
    }
    if !castle_rights.contains('Q') {
        board[7 * 8].set_flags(HAS_MOVED);
    }
    if !castle_rights.contains('k') {
        board[0 * 8 + 7].set_flags(HAS_MOVED);
    }
    if !castle_rights.contains('q') {
        board[0 * 8].set_flags(HAS_MOVED);
    }

    let mut possible_ep_capture = 64;
    if fen_parts[3].len() == 2 {
        possible_ep_capture = convert_algebraic_notation_to_number(fen_parts[3]);
        if possible_ep_capture > 32 {
            possible_ep_capture -= 8;
        }
        else {
            possible_ep_capture += 8;
        }
    }
    let half_move_clock = fen_parts[4].parse::<usize>().unwrap();

    return Game {
        board,
        turn,
        possible_ep_capture,
        king_square,
        half_move_clock
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameState {
    InProgress,
    Check,
    Checkmate,
    Stalemate,
    InsufficientMaterial,
    DrawBy50MoveRule
}

/// The chess game
#[derive(Copy, Clone)]
pub struct Game {
    pub board: [Piece; 64],
    pub turn: usize,
    possible_ep_capture: usize,
    king_square: [usize; 2],
    half_move_clock: usize
}

impl Game {

    /// Creates a new game representing given FEN string
    pub fn new(fen: &str) -> Game {
        convert_fen_to_game(fen)
    }

    /// Creates a new game initialized to the starting position
    pub fn starting_position() -> Game {
        Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    /// Updates the game's current board state
    /// 
    /// # Arguments
    /// * 'fen' - An entire FEN string representing some board
    pub fn set_board_state(&mut self, fen: &str) {
        let new_game = convert_fen_to_game(fen);
        self.board = new_game.board;
        self.turn = new_game.turn;
        self.possible_ep_capture = new_game.possible_ep_capture;
        self.king_square = new_game.king_square;
        self.half_move_clock = new_game.half_move_clock;
    }

    /// Returns all legal moves in the current position
    pub fn get_all_legal_moves(&self) -> Vec<Move> {
        let move_gen = MoveGenerator::new();
        let mut pseudo_legal_moves = vec![];

        for square in 0..64 {
            if self.board[square].get_type() != EMPTY && self.board[square].get_color() == self.turn {
                pseudo_legal_moves.append(&mut move_gen.generate_pseudo_legal_moves(self, square));
            }
        }
        let legal_moves = move_gen.filter_pseudo_legal_moves(self, pseudo_legal_moves);
        return legal_moves;
    }

    /// Returns the legal moves from the given square, in the current position
    pub fn get_legal_moves(&self, square: usize) -> Vec<Move> {
        let move_gen = MoveGenerator::new();
        let pseudo_legal_moves = move_gen.generate_pseudo_legal_moves(self, square);
        return move_gen.filter_pseudo_legal_moves(self, pseudo_legal_moves);
    }

    /// Returns the game state of the current position, everything but 3-fold repetition is included
    pub fn get_game_state(&self) -> GameState {
        let move_gen = MoveGenerator::new();
        let mut game_state = GameState::InProgress;
        
        if move_gen.is_attacked(self, self.king_square[self.turn], self.turn) {
            game_state = GameState::Check;

            let legal_moves = self.get_all_legal_moves();
            if legal_moves.len() == 0 {
                return GameState::Checkmate;
            }
        }
        else {
            let legal_moves = self.get_all_legal_moves();
            if legal_moves.len() == 0 {
                return GameState::Stalemate;
            }
        }

        let mut n_pieces = [[0; 7]; 2];
        for square in 0..64 {
            if self.board[square].get_type() != EMPTY {
                n_pieces[self.board[square].get_color()][0] += 1;
                n_pieces[self.board[square].get_color()][self.board[square].get_type()] += 1;
            }
        }
        if n_pieces[WHITE][0] <= 3 && n_pieces[BLACK][0] <= 3 && 
			(n_pieces[WHITE][0] == 1 || 
			(n_pieces[WHITE][0] == 2 && (n_pieces[WHITE][BISHOP] == 1 || n_pieces[WHITE][KNIGHT] == 1)) ||
			(n_pieces[WHITE][0] == 3 && n_pieces[WHITE][KNIGHT] == 2))
			&&
			(n_pieces[BLACK][0] == 1 ||
			(n_pieces[BLACK][0] == 2 && (n_pieces[BLACK][BISHOP] == 1 || n_pieces[BLACK][KNIGHT] == 1)) ||
			(n_pieces[BLACK][0] == 3 && n_pieces[BLACK][KNIGHT] == 2)) {
                return GameState::InsufficientMaterial;
            }
        
        if self.half_move_clock >= 100 {
            return GameState::DrawBy50MoveRule;
        }

        return game_state;
    }

    /// Makes a move from a given square to another given square
    /// 
    /// # Arguments
    /// * 'from' - the square the move is made from
    /// * 'to' - the square the made is made to
    /// * 'promotion' the selected promotion if the move is a promotion, otherwise leave as EMPTY
    /// 
    /// # Returns
    /// * bool - True if the move is legal and false otherwise
    /// 
    /// # Examples
    /// 
    /// ```
    /// game.make_move_from_to(3, 11, EMPTY);
    /// game.make_move_from_to(8, 0, QUEEN_PROMOTION);
    /// ```
    /// 
    pub fn make_move_from_to(&mut self, from: usize, to: usize, promotion: usize) -> bool {
        let legal_moves = self.get_all_legal_moves();
        for mv in legal_moves {
            if mv.get_from() == from && mv.get_to() == to {
                if mv.is_promotion() && (mv.get_flags() & !CAPTURE) != promotion {
                    continue;
                }
                self.make_move(mv);
                return true;
            }
        }
        return false;
    }
    
    /// Makes the given move on the current board. 
    /// The move struct is given by either 'Game::generate_all_legal_moves' or 'Game::generate_legal_moves'.
    pub fn make_move(&mut self, mv: Move) {

        self.half_move_clock += 1;
        if self.board[mv.get_from()].get_type() == KING {
            self.king_square[self.turn] = mv.get_to();
        }
        if self.board[mv.get_from()].get_type() == PAWN {
            self.half_move_clock = 0;
        }

        if mv.is_capture() {
            self.half_move_clock = 0;
            let mut captured_square = mv.get_to();
            if mv.is_ep_capture() {
                captured_square = (mv.get_from() as isize + 
                (self.get_column(mv.get_to()) as isize - self.get_column(mv.get_from()) as isize)) as usize;
            }
            self.board[captured_square].set_type(EMPTY);
        }
        if self.possible_ep_capture < 64 {
            self.possible_ep_capture = 64;
        }
        if mv.is_double_pawn_push() {
            self.possible_ep_capture = mv.get_to();
        }
        self.board[mv.get_to()] = self.board[mv.get_from()];
        self.board[mv.get_from()].set_type(EMPTY);
        self.board[mv.get_to()].set_flags(HAS_MOVED);
        
        if mv.is_castle() {
            let rook_move;
            if mv.is_queen_castle() {
                rook_move = (mv.get_from() - 4, mv.get_from() - 1);
            }
            else {  
                rook_move = (mv.get_from() + 3, mv.get_from() + 1);
            }
            self.board[rook_move.1] = self.board[rook_move.0];
            self.board[rook_move.0].set_type(EMPTY);
        }
        
        if mv.is_promotion() {
            let promotion_type = mv.get_flags() & !(CAPTURE);

            if promotion_type == BISHOP_PROMOTION {
                self.board[mv.get_to()].set_type(BISHOP);
            }
            if promotion_type == KNIGHT_PROMOTION {
                self.board[mv.get_to()].set_type(KNIGHT);
            }
            if promotion_type == ROOK_PROMOTION {
                self.board[mv.get_to()].set_type(ROOK);
            }
            if promotion_type == QUEEN_PROMOTION {
                self.board[mv.get_to()].set_type(QUEEN);
            }
        }
        self.turn ^= 1;
    }

    /// Returns the collumn of the given square, indexed from left to right
    pub fn get_column(&self, square: usize) -> usize {
        return square % 8;
    }

    /// Returns the row of the given square, indexed from up to down
    pub fn get_row(&self, square: usize) -> usize {
        return square / 8;
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut board_string: String = "".to_string();
        for i in 0..64 {
            if i != 0 && i % 8 == 0 {
                board_string.push_str("\n");
            }

            if self.board[i].get_type() != EMPTY {
                board_string.push_str(
                    match self.board[i].get_color() {
                        WHITE => "W",
                        BLACK => "B",
                        _ => "_"
                    }
                );
            }
            
            board_string.push_str(
                match self.board[i].get_type() {
                    PAWN => "P ",
                    KNIGHT => "N ",
                    BISHOP => "B ",
                    ROOK => "R ",
                    QUEEN => "Q ",
                    KING => "K ",
                    _ => ".. "
                }
            );
        }
        write!(f, "{}", board_string)
    }
}