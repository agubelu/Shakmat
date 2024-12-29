use rocket::http::{Status, ContentType};
use rocket::serde::json::serde_json::json;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Value;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket::request::Request;

use shakmat_engine::SearchResult;
use shakmat_core::{Move, Color, Board};

// Generic API response with an arbitraty HTTP status code and json payload
// kudos to https://stackoverflow.com/a/54867136
pub struct ApiResponse {
    status: Status,
    payload: Value,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ApiResponse {
    fn respond_to(self, req: &'r Request) -> response::Result<'o> {
        Response::build_from(self.payload.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

impl ApiResponse {
    pub fn bad_request(msg: String) -> Self {
        Self { status: Status::BadRequest, payload: json!({"msg": msg}) }
    }

    pub fn not_found(msg: String) -> Self {
        Self { status: Status::NotFound, payload: json!({"msg": msg}) }
    }

    pub fn game_created(key: String, turn_info: TurnInfo) -> Self {
        Self { status: Status::Created, payload: json!({"key": key, "turn_info": turn_info}) }
    }

    pub fn turn_info(turn_info: TurnInfo) -> Self {
        Self { status: Status::Ok, payload: json!({"turn_info": turn_info}) }
    }

    pub fn move_suggestion(sr: &SearchResult) -> Self {
        Self { status: Status::Ok, payload: json!({
            "move": sr.best_move.unwrap().to_string(),
            "eval": sr.score.to_string(), 
        }) }
    }

    pub fn no_content() -> Self {
        Self { status: Status::NoContent, payload: json!({}) }
    }
}

// Info for the current turn
#[derive(Serialize)]
#[serde(rename = "turn_info")]
pub struct TurnInfo {
    turn_number: u16,
    color: Color,
    moves: Vec<Move>,
    in_check: bool,
    fen: String,
}

impl TurnInfo {
    pub fn from_board(board: &Board, history: &[u64]) -> Self {
        let moves = if shakmat_engine::is_draw_by_repetition(board, 0, history) {
            vec![]
        } else {
            board.legal_moves()
        };

        Self {
            turn_number: board.turn_number(),
            color: board.turn_color(),
            in_check: board.is_check(),
            fen: board.fen(),
            moves
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Structs for deserializing stuff sent by the clients

#[derive(Deserialize)]
pub struct FenData {
    pub fen: String
}

#[derive(Deserialize, Serialize)]
pub struct MoveData {
    pub r#move: String,
}

#[derive(Deserialize, Serialize)]
pub struct ConfigOptions {
    pub use_book: bool,
    pub always_top_line: bool
}

///////////////////////////////////////////////////////////////////////////////

