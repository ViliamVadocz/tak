use std::{error::Error, time::Duration};

use tokio::{
    select,
    signal::ctrl_c,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_takconnect::{
    connect_as,
    connect_guest,
    data_types::{Color, Update},
    Client,
};

use crate::{cli::Args, message::Message, seek::create_seek};

pub async fn seek_loop(
    args: Args,
    tx: UnboundedSender<Message>,
    mut rx: UnboundedReceiver<Message>,
) -> Result<(), Box<dyn Error>> {
    // Connect to PlayTak
    let mut client = if let (Some(username), Some(password)) = (args.username, args.password) {
        println!("Connecting as {username}");
        connect_as(username, password).await
    } else {
        println!("Connecting as guest");
        connect_guest().await
    }?;

    let mut seek_as_white = true;
    select! {
        _ = ctrl_c() => (),
        _ = async move {
            loop {
                create_seek(
                    &mut client, if seek_as_white {Color::White} else {Color::Black},
                    Duration::from_secs(args.initial_time),
                    Duration::from_secs(args.increment),
                ).await;
                println!("Created seek (white: {seek_as_white})");

                if let Err(err) = run_playtak_game(&mut client, &tx, &mut rx, seek_as_white).await {
                    println!("Error in run_playtak_game {err}");
                    break;
                }

                // Alternate seek colors.
                seek_as_white = !seek_as_white;
            }
        } => (),
    }

    println!("Shutting down...");
    Ok(())
}

async fn run_playtak_game(
    client: &mut Client,
    tx: &UnboundedSender<Message>,
    rx: &mut UnboundedReceiver<Message>,
    seek_as_white: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut game = client.game().await?;
    let game_info = game.game();
    tx.send(Message::GameInfo(game_info.clone()))?;

    println!("Game started! {} vs {}", game_info.white(), game_info.black());

    let mut take_my_turn = seek_as_white;
    loop {
        if take_my_turn {
            tx.send(Message::MoveRequest)?;
            match rx.recv().await {
                Some(Message::Move(my_move)) => {
                    if game.play(my_move).await.is_err() {
                        println!("Failed to play move!");
                    }
                }
                Some(Message::GameEnded(_)) => {}
                None => break Ok(()),
                _ => {}
            }
        }

        match game.update().await? {
            Update::Played(m) => {
                tx.send(Message::Move(m))?;
            }
            Update::GameEnded(result) => {
                tx.send(Message::GameEnded(Some(result)))?;
                break Ok(());
            }
            _ => {}
        }

        take_my_turn = true;
    }
}
