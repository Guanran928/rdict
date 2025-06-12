use anyhow::Result;
use rdict_core::parse::TranslationData;
use rdict_core::rdict::{self, Rdict};
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::ParseMode;
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger, etc.
    pretty_env_logger::init();
    log::info!("Starting translation bot...");

    let bot = Bot::from_env();

    Command::repl(bot, answer).await;

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

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Translate(text) => {
            // TODO: Don't create a `rdict` client on every request
            let client = Rdict::new("https://m.youdao.com", None).await.unwrap();
            let res = client.get_results(&text).await;

            match res {
                Ok(result) => {
                    let output_result = match result.data {
                        TranslationData::ToChinese(tc) => rdict::render_chinese_plain(&tc),
                        TranslationData::ToEnglish(te) => rdict::render_english_plain(&te),
                    };

                    match output_result {
                        Ok(output) => {
                            let wrapped_output = format!(
                                "<pre><code class=\"language-markdown\">{}</code></pre>",
                                html_escape::encode_text(&output)
                            );
                            bot.send_message(msg.chat.id, wrapped_output)
                                .reply_to(msg)
                                .parse_mode(ParseMode::Html)
                                .await?
                        }
                        Err(err) => {
                            bot.send_message(msg.chat.id, format!("Format error: {err}"))
                                .reply_to(msg)
                                .await?
                        }
                    }
                }
                Err(err) => {
                    bot.send_message(msg.chat.id, format!("Lookup failed: {err}"))
                        .await?
                }
            }
        }
    };

    Ok(())
}
