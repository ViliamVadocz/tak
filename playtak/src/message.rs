use tak::Move;
use tokio_takconnect::data_types::Game;

#[derive(Debug)]
pub enum Message {
    GameInfo(Game),
    MoveRequest,
    Move(Move),
    GameEnded,
}
