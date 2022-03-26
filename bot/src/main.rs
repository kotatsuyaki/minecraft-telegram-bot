use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use teloxide::{prelude2::*, types::ParseMode, utils::command::BotCommand};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    join,
    sync::Mutex,
};

#[derive(Debug, Serialize, Deserialize)]
struct JavaInEvent {
    event: JavaEventKind,
    name: Option<String>,
    msg: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum JavaEventKind {
    PlayerJoin,
    PlayerLeave,
    ChatMsg,
    PlayerDeath,
}

#[derive(Debug, Serialize, Deserialize)]
struct JavaOutEvent {
    event: String,
    name: String,
    msg: String,
}

#[derive(BotCommand, Debug, Clone, Copy)]
#[command(rename = "lowercase", description = "Bot commands")]
enum Command {
    #[command(description = "Ping the server")]
    Ping,
}

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::from_env();
    let out_mtx = Arc::new(Mutex::new(()));

    send_maybe_report(
        bot.send_message(
            std::env::var("CHATID")
                .context("Unable to get CHATID")
                .unwrap(),
            "Server 開ら",
        ),
    )
    .await;

    let bot_clone = bot.clone();
    let handler = Update::filter_message()
        // Respond to `/ping` command
        .branch(dptree::entry().filter_command::<Command>().endpoint(
            |msg: Message, bot: Bot, cmd: Command| async move {
                match cmd {
                    Command::Ping => {
                        let mut req = bot.send_message(msg.chat.id, "有ら server 有開ら");
                        req.reply_to_message_id = Some(msg.id);
                        send_maybe_report(req).await;
                    }
                };

                anyhow::Result::<()>::Ok(())
            },
        ))
        // Send message from minecraft server to telegram group
        .branch(dptree::entry().endpoint(move |msg: Message, _bot: Bot| {
            let out_mtx = out_mtx.clone();
            async move {
                let json_str = serde_json::to_string(&JavaOutEvent {
                    event: "chat_msg".into(),
                    name: msg
                        .from()
                        .and_then(|u| u.username.clone())
                        .unwrap_or("unknown".into()),
                    msg: msg.text().unwrap_or("").into(),
                });
                if json_str.is_err() {
                    return Ok(());
                }
                let json_str = json_str.unwrap();
                let _ = out_mtx.lock().await;
                println!("{}", json_str);

                anyhow::Result::<()>::Ok(())
            }
        }));

    let mut dispatcher = Dispatcher::builder(bot_clone, handler)
        .default_handler(|_upd| async move {})
        .build();
    let (input_loop_res, _) = join!(input_loop(bot), dispatcher.setup_ctrlc_handler().dispatch());
    input_loop_res?;

    Ok(())
}

async fn input_loop(bot: Bot) -> Result<()> {
    let mut stdin = BufReader::new(stdin()).lines();

    while let Some(input_line) = stdin.next_line().await? {
        let event = serde_json::from_str(&input_line);
        if event.is_err() {
            continue;
        }

        let event: JavaInEvent = event.unwrap();
        match event.event {
            JavaEventKind::PlayerJoin => {
                let mut req = bot.send_message(
                    std::env::var("CHATID")
                        .context("Unable to get CHATID")
                        .unwrap(),
                    format!(
                        "`[{} 上線ら]`",
                        event.name.as_deref().unwrap_or("<unknown>")
                    ),
                );
                req.parse_mode = Some(ParseMode::MarkdownV2);
                req.disable_notification = Some(true);
                send_maybe_report(req).await;
            }
            JavaEventKind::PlayerLeave => {
                let mut req = bot.send_message(
                    std::env::var("CHATID")
                        .context("Unable to get CHATID")
                        .unwrap(),
                    format!(
                        "`[{} 跑路ら]`",
                        event.name.as_deref().unwrap_or("<unknown>")
                    ),
                );
                req.parse_mode = Some(ParseMode::MarkdownV2);
                req.disable_notification = Some(true);
                send_maybe_report(req).await;
            }
            JavaEventKind::ChatMsg => {
                let mut req = bot.send_message(
                    std::env::var("CHATID")
                        .context("Unable to get CHATID")
                        .unwrap(),
                    format!(
                        "`[{}] {}`",
                        event.name.as_deref().unwrap_or("<unknown>"),
                        event.msg.as_deref().unwrap_or_default()
                    ),
                );
                req.parse_mode = Some(ParseMode::MarkdownV2);
                req.disable_notification = Some(true);
                send_maybe_report(req).await;
            }
            JavaEventKind::PlayerDeath => {
                let mut req = bot.send_message(
                    std::env::var("CHATID")
                        .context("Unable to get CHATID")
                        .unwrap(),
                    format!(
                        "`[{} dieら: {}]`",
                        event.name.as_deref().unwrap_or("<unknown>"),
                        event.msg.as_deref().unwrap_or_default()
                    ),
                );
                req.parse_mode = Some(ParseMode::MarkdownV2);
                req.disable_notification = Some(true);
                send_maybe_report(req).await;
            }
        }
    }

    Ok(())
}

/// Send a request, optionally report failure to stderr upon errors
async fn send_maybe_report<R: Request>(req: R) {
    if let Err(e) = req.send().await {
        eprintln!("Error sending: {:?}", e);
    }
}
