use std::time::Duration;

use tokio_takconnect::{Client, Color, GameParameters, SeekParameters};

use crate::HALF_KOMI;

pub async fn create_seek(client: &mut Client, color: Color, initial_time: Duration, increment: Duration) {
    client
        .seek(
            SeekParameters::new(
                None,
                color,
                GameParameters::new(6, initial_time, increment, HALF_KOMI, 30, 1, false, false).unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
}
