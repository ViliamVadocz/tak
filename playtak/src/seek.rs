use std::time::Duration;

use tokio_takconnect::{
    data_types::{Color, GameParameters, SeekParameters},
    Client,
};

use crate::HALF_KOMI;

pub async fn create_seek(client: &mut Client, color: Color, initial_time: Duration, increment: Duration) {
    client
        .seek(
            SeekParameters::new(
                None,
                Some(color),
                GameParameters::new(6, initial_time, increment, HALF_KOMI, 30, 1, true, false).unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
}
