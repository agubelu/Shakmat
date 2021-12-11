use rocket::http::{Status, ContentType};
use rocket::serde::json::serde_json::json;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Value;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket::request::Request;

use crate::chess::{Move, Color, Board};

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
}

// Info for the current turn
#[derive(Debug, Serialize)]
#[serde(rename = "turn_info")]
pub struct TurnInfo {
    turn_number: u16,
    color: Color,
    moves: Vec<Move>,
    in_check: bool
}

impl TurnInfo {
    pub fn from_board(board: &Board) -> Self {
        Self {
            turn_number: board.turn_number(),
            color: board.turn_color(),
            moves: board.legal_moves(),
            in_check: board.is_check(board.turn_color())
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Structs for deserializing stuff sent by the clients

#[derive(Debug, Deserialize)]
pub struct FenData {
    pub fen: String
}

#[derive(Debug, Deserialize)]
pub struct MoveData {
    pub r#move: String,
}

///////////////////////////////////////////////////////////////////////////////


