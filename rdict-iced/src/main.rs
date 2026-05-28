#![windows_subsystem = "windows"]

mod components;
mod render;

use anyhow::Result;
use directories_next::ProjectDirs;
use iced::font;
use iced::widget::{button, column, container, pick_list, row, space, text, text_input};
use iced::window::Settings;
use iced::{Alignment, Element, Font, Length, Size, Task};
use rdict_core::model::{Language, Voice};
use rdict_core::rdict::TranslationData;
use rdict_core::rdict::{FetchedResult, Rdict};
use std::io::Cursor;

use crate::components::list_item;

#[derive(Default)]
struct State {
    text_input_content: String,
    selected_language: Language,
    translation_result: TranslationState,
    client: Option<Rdict>,
}

#[derive(Default)]
enum TranslationState {
    #[default]
    Empty,
    Loading,
    Error(String),
    Translation(FetchedResult),
}

#[derive(Debug, Clone)]
enum Message {
    ContentChanged(String),
    LanguageChanged(Language),
    Submit,
    TranslationResult(FetchedResult),
    TranslationError(String),
    PlayVoice(Voice),
    VoicePlayed(Result<(), String>),
}

fn main() -> Result<()> {
    // https://www.reddit.com/r/learnrust/comments/jaqfcx/windows_print_to_hidden_console_window/
    #[cfg(target_os = "windows")]
    {
        use winapi::um::wincon::{ATTACH_PARENT_PROCESS, AttachConsole};
        unsafe {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }

    iced::application(State::default, update, view)
        .title("Rdict")
        .window(Settings {
            size: Size::new(400.0, 600.0),
            min_size: Some(Size::new(200.0, 200.0)),
            ..Settings::default()
        })
        .run()?;

    Ok(())
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ContentChanged(content) => {
            state.text_input_content = content;
            Task::none()
        }
        Message::LanguageChanged(lang) => {
            state.selected_language = lang;
            Task::none()
        }
        Message::Submit => {
            if state.text_input_content.trim().is_empty() {
                return Task::none();
            }
            state.translation_result = TranslationState::Loading;

            // Clone only what is required for the async move block
            let text_input_content = state.text_input_content.clone();
            let selected_language = state.selected_language;
            let client = state.client.clone();

            Task::perform(
                async move { fetch_translation(client, text_input_content, selected_language).await },
                |res| match res {
                    Ok(msg) => Message::TranslationResult(msg),
                    Err(e) => Message::TranslationError(e.to_string()),
                },
            )
        }
        Message::TranslationResult(msg) => {
            state.translation_result = TranslationState::Translation(msg);
            Task::none()
        }
        Message::TranslationError(err) => {
            state.translation_result = TranslationState::Error(err);
            Task::none()
        }
        Message::PlayVoice(voice) => Task::perform(
            async move { play_voice(voice).await.map_err(|e| e.to_string()) },
            Message::VoicePlayed,
        ),
        Message::VoicePlayed(result) => {
            if let Err(error) = result {
                eprintln!("Voice playback failed: {error}");
            }

            Task::none()
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let main: Element<'_, Message> = match &state.translation_result {
        TranslationState::Empty => space().width(Length::Fill).height(Length::Fill).into(),

        TranslationState::Error(error) => container(column![
            text("Lookup Error")
                .style(text::danger)
                .font(Font {
                    weight: font::Weight::Bold,
                    ..Font::default()
                })
                .size(20),
            text(error).size(16).style(text::secondary),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into(),

        TranslationState::Loading => container(text("Loading...").size(16).style(text::secondary))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into(),

        TranslationState::Translation(fetched_result) => match &fetched_result.data {
            TranslationData::FromEnglish(tc) => render::en::to_chinese(tc, &fetched_result.voices),
            TranslationData::ToEnglish(te) => render::en::to_english(te),

            TranslationData::FromFrench(tc) => render::fr::to_chinese(tc, &fetched_result.voices),
            TranslationData::ToFrench(te) => render::fr::to_french(te),
            TranslationData::FromKorean(tc) => render::ko::to_chinese(tc),
            TranslationData::ToKorean(te) => render::ko::to_korean(te),
            TranslationData::FromJapanese(tc) => render::ja::to_chinese(tc, &fetched_result.voices),
            TranslationData::ToJapanese(te) => render::ja::to_japanese(te),

            TranslationData::NotFound(nf) => {
                let suggestions_col = {
                    let mut col = column![];
                    for suggestion in &nf.suggestions {
                        col = col.push(list_item(text(suggestion)));
                    }
                    col
                };

                container(
                    column![
                        column![
                            text("Translation not found")
                                .style(text::danger)
                                .font(Font {
                                    weight: font::Weight::Bold,
                                    ..Font::default()
                                })
                                .size(20),
                            text("Did you mean:").style(text::secondary),
                        ],
                        suggestions_col,
                    ]
                    .spacing(10),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
            }
        },
    };

    const LANGUAGES: [Language; 4] = [
        Language::English,
        Language::French,
        Language::Korean,
        Language::Japanese,
    ];

    let mut layout = column![
        row![
            text_input("Type something here...", &state.text_input_content)
                .on_input(Message::ContentChanged)
                .on_submit(Message::Submit),
            pick_list(
                &LANGUAGES[..],
                Some(state.selected_language),
                Message::LanguageChanged
            ),
            button("Translate").on_press(Message::Submit),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        main
    ]
    .padding(10)
    .spacing(10);

    if cfg!(debug_assertions) {
        layout = layout.push(text("rdict_iced dev").align_x(iced::alignment::Horizontal::Center));
    }

    layout.into()
}

async fn fetch_translation(
    client: Option<Rdict>,
    text_input_content: String,
    language: Language,
) -> Result<FetchedResult, rdict_core::Error> {
    let client = if let Some(c) = client {
        c
    } else {
        let cache_db_path = ProjectDirs::from("dev", "ny4", "rdict")
            .map(|proj_dirs| proj_dirs.cache_dir().join("cache.db"));
        Rdict::new("https://m.youdao.com", cache_db_path).await?
    };

    client.get_results(&text_input_content, language).await
}

async fn play_voice(voice: Voice) -> Result<()> {
    let client = Rdict::new("https://m.youdao.com", None).await?;
    let bytes = client.fetch_voice(&voice).await?;

    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut stream = rodio::DeviceSinkBuilder::open_default_sink()?;
        stream.log_on_drop(false);

        let player = rodio::play(stream.mixer(), Cursor::new(bytes))?;
        player.sleep_until_end();

        Ok(())
    })
    .await??;

    Ok(())
}
