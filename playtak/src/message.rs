use tak::Move;

#[derive(Debug)]
pub enum Message {
    MoveRequest,
    Move(Move),
    GameEnded,
}
