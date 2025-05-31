use anyhow::Result;
use rdict_core::parse::TranslationData;
use rdict_core::rdict::{self, Format, Rdict};
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
            let rdict = Rdict::new("https://m.youdao.com", Format::Markdown, None)
                .await
                .unwrap();
            let res = rdict.get_results(&text).await;

            match res {
                Ok(result) => {
                    let output_result = match result.data {
                        TranslationData::ToChinese(tc) => rdict::output_chinese_plain(&tc),
                        TranslationData::ToEnglish(te) => rdict::output_english_plain(&te),
                    };

                    match output_result {
                        Ok(output) => {
                            let wrapped_output =
                                format!("<pre language=\"c++\">\n{output}\n</pre>");
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
