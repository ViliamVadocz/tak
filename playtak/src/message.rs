#[derive(Debug)]
pub enum Message {
    MoveRequest,
    Turn(String),
    GameEnded,
}
