use std::time::Duration;

use alpha_tak::config::KOMI;
use tokio_takconnect::{Client, Color, GameParameters, SeekParameters};

pub async fn create_seek(client: &mut Client, color: Color, initial_time: Duration, increment: Duration) {
    client
        .seek(
            SeekParameters::new(
                None,
                color,
                GameParameters::new(5, initial_time, increment, 2 * KOMI, 21, 1, false, false).unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
}
