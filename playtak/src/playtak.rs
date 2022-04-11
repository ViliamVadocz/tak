use std::{
    error::Error,
    sync::mpsc::{Receiver, Sender},
};

use tokio::{select, signal::ctrl_c};
use tokio_takconnect::{connect_as, connect_guest, Client, Color, GameUpdate};

use crate::{cli::Args, message::Message, seek::create_seek};

pub async fn seek_loop(args: Args, tx: Sender<Message>, rx: Receiver<Message>) -> Result<(), Box<dyn Error>> {
    // Connect to PlayTak
    let mut client = if let (Some(username), Some(password)) = (args.username, args.password) {
        println!("Connecting as {username}");
        connect_as(username, password).await
    } else {
        println!("Connecting as guest");
        connect_guest().await
    }?;

    let mut seek_as_white = false;
    select! {
        _ = ctrl_c() => (),
        _ = async move {
            loop {
                create_seek(&mut client, if seek_as_white {Color::White} else {Color::Black}).await;
                println!("Created seek (white: {seek_as_white})");

                if run_playtak_game(&mut client, &tx, &rx, seek_as_white).await.is_err() {
                    break;
                }

                // Alternate seek colours.
                seek_as_white = !seek_as_white;
            }
        } => (),
    }

    println!("Shutting down...");
    Ok(())
}

async fn run_playtak_game(
    client: &mut Client,
    tx: &Sender<Message>,
    rx: &Receiver<Message>,
    seek_as_white: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut game = client.game().await?;
    println!("Game started!");

    let mut take_my_turn = seek_as_white;
    loop {
        if take_my_turn {
            tx.send(Message::MoveRequest)?;
            match rx.recv()? {
                Message::Turn(m) => {
                    if game.play(m.parse()?).await.is_err() {
                        println!("Failed to play move!");
                    }
                }
                Message::GameEnded => {}
                _ => {}
            }
        }

        match game.update().await? {
            GameUpdate::Played(m) => {
                tx.send(Message::Turn(m.to_string()))?;
            }
            GameUpdate::Ended(_result) => {
                tx.send(Message::GameEnded)?;
                break Ok(());
            }
            _ => {}
        }

        take_my_turn = true;
    }
}
