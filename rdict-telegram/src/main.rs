#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use rdict_core::rdict::Rdict;
use std::sync::Arc;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::ParseMode;
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting translation bot...");

    let client = Arc::new(Rdict::new("https://m.youdao.com", None).await?);
    let bot = Bot::from_env();

    let handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![client])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "translate the given text")]
    Translate(String),
}

async fn handle_command(bot: Bot, msg: Message, cmd: Command, client: Arc<Rdict>) -> Result<()> {
    let answer = async || match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;

            Ok(())
        }

        Command::Translate(text) => {
            let result = client
                .get_results(&text, rdict_core::model::Language::English)
                .await
                .context("Failed to get translation results")?;

            let output = result.render_plain();

            let wrapped_output = format!(
                "<pre><code class=\"language-markdown\">{}</code></pre>",
                html_escape::encode_text(&output)
            );

            bot.send_message(msg.chat.id, wrapped_output)
                .reply_to(&msg)
                .parse_mode(ParseMode::Html)
                .await
                .context("Failed to send message")?;

            Ok(())
        }
    };

    let res: Result<()> = answer().await;

    // TODO: Use `Dispatcher`'s `handle_error`.
    if let Err(e) = res {
        log::error!("{e}");
        bot.send_message(
            msg.chat.id,
            format!("❌ An error occurred. Please try again later.\n\n{e:?}"),
        )
        .await
        .ok();
    }

    Ok(())
}
