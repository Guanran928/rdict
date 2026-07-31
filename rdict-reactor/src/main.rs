#![windows_subsystem = "windows"]

mod components;
mod render;
mod theme;

use std::sync::{Arc, OnceLock};

use crate::components::list_item;

use directories_next::ProjectDirs;
use rdict_core::model::Language;
use rdict_core::rdict::{FetchedResult, Rdict};
use windows_reactor::*;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
    })
}

#[derive(Clone, PartialEq)]
enum TranslationState {
    Empty,
    Loading,
    Error(String),
    Translation(FetchedResult),
}

/// The rdict client never changes after initialization. Equality is by
/// instance identity, which is exactly what the framework's change detection
/// needs: only a new client counts as a change.
#[derive(Clone)]
struct Client(Arc<Rdict>);

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

const LANGUAGES: [Language; 4] = [
    Language::English,
    Language::French,
    Language::Korean,
    Language::Japanese,
];

fn app(cx: &mut RenderCx) -> Element {
    let (text_input_content, set_text_input_content) = cx.use_state(String::new());
    let (selected_language, set_selected_language) = cx.use_state(Language::English);
    let (translation_result, set_translation_result) = cx.use_async_state(TranslationState::Empty);
    let (client, set_client) = cx.use_async_state(None::<Client>);

    // Initialize the rdict client once, off the UI thread.
    cx.use_effect((), {
        let set_client = set_client.clone();
        let set_translation_result = set_translation_result.clone();
        move || {
            runtime().spawn(async move {
                let cache_db_path = ProjectDirs::from("dev", "ny4", "rdict")
                    .map(|proj_dirs| proj_dirs.cache_dir().join("cache.db"));

                match Rdict::new("https://m.youdao.com", cache_db_path).await {
                    Ok(client) => set_client.call(Some(Client(Arc::new(client)))),
                    Err(err) => {
                        set_translation_result.call(TranslationState::Error(err.to_string()))
                    }
                }
            });
        }
    });

    let submit = {
        let text_input_content = text_input_content.clone();
        let client = client.clone();
        let set_translation_result = set_translation_result.clone();
        let translation_result = translation_result.clone();
        move || {
            if text_input_content.trim().is_empty()
                || matches!(&translation_result, TranslationState::Loading)
            {
                return;
            }
            let Some(client) = client.as_ref() else {
                return;
            };
            let client = client.clone();

            set_translation_result.call(TranslationState::Loading);

            let text_input_content = text_input_content.clone();
            let set_translation_result = set_translation_result.clone();
            runtime().spawn(async move {
                match client
                    .0
                    .get_results(&text_input_content, selected_language)
                    .await
                {
                    Ok(msg) => set_translation_result.call(TranslationState::Translation(msg)),
                    Err(err) => {
                        set_translation_result.call(TranslationState::Error(err.to_string()))
                    }
                }
            });
        }
    };

    let content: Element = match &translation_result {
        TranslationState::Empty => Element::Empty,

        TranslationState::Loading => centered(ProgressRing::indeterminate()),

        TranslationState::Error(error) => centered(
            vstack((
                text_block("Lookup Error")
                    .font_size(20.0)
                    .bold()
                    .foreground(ThemeRef::SystemCritical),
                text_block(error)
                    .font_size(14.0)
                    .foreground(ThemeRef::SecondaryText)
                    .wrap(),
            ))
            .spacing(10.0),
        ),

        TranslationState::Translation(fetched_result) => match &fetched_result.data {
            rdict_core::rdict::TranslationData::FromEnglish(tc) => {
                render::en::to_chinese(tc, fetched_result.is_cached)
            }
            rdict_core::rdict::TranslationData::ToEnglish(te) => {
                render::en::to_english(te, fetched_result.is_cached)
            }
            rdict_core::rdict::TranslationData::FromFrench(tc) => {
                render::fr::to_chinese(tc, fetched_result.is_cached)
            }
            rdict_core::rdict::TranslationData::ToFrench(te) => {
                render::fr::to_french(te, fetched_result.is_cached)
            }
            rdict_core::rdict::TranslationData::FromKorean(tc) => {
                render::ko::to_chinese(tc, fetched_result.is_cached)
            }
            rdict_core::rdict::TranslationData::ToKorean(te) => {
                render::ko::to_korean(te, fetched_result.is_cached)
            }
            rdict_core::rdict::TranslationData::FromJapanese(tc) => {
                render::ja::to_chinese(tc, fetched_result.is_cached)
            }
            rdict_core::rdict::TranslationData::ToJapanese(te) => {
                render::ja::to_japanese(te, fetched_result.is_cached)
            }

            rdict_core::rdict::TranslationData::NotFound(nf) => {
                let suggestions: Vec<Element> = nf
                    .suggestions
                    .iter()
                    .map(|suggestion| list_item(text_block(suggestion).wrap()))
                    .collect();

                centered(
                    vstack((
                        vstack((
                            text_block("Translation not found")
                                .font_size(20.0)
                                .bold()
                                .foreground(ThemeRef::SystemCritical),
                            text_block("Did you mean:")
                                .font_size(14.0)
                                .foreground(ThemeRef::SecondaryText),
                        ))
                        .spacing(6.0),
                        vstack(suggestions).spacing(4.0),
                    ))
                    .spacing(10.0),
                )
            }
        },
    };

    let language_names: Vec<String> = LANGUAGES.iter().map(|lang| lang.to_string()).collect();
    let selected_language_index = LANGUAGES
        .iter()
        .position(|lang| *lang == selected_language)
        .expect("selected language is always in LANGUAGES")
        as i32;

    let input_bar = grid((
        text_box(text_input_content.clone())
            .placeholder_text("Type something here...")
            .on_text_changed(set_text_input_content)
            .height(theme::CONTROL_HEIGHT)
            .grid_column(0),
        ComboBox::new(language_names)
            .selected_index(selected_language_index)
            .on_selection_changed({
                let set_selected_language = set_selected_language.clone();
                move |index: i32| {
                    if let Some(lang) = LANGUAGES.get(index as usize) {
                        set_selected_language.call(*lang);
                    }
                }
            })
            .width(theme::LANGUAGE_PICKER_WIDTH)
            .height(theme::CONTROL_HEIGHT)
            .grid_column(1),
        button("")
            .icon(Symbol::Find)
            .accent()
            .tooltip("Translate")
            .on_click(submit.clone())
            .height(theme::CONTROL_HEIGHT)
            .grid_column(2),
    ))
    .columns([GridLength::Star(1.0), GridLength::Auto, GridLength::Auto])
    .column_spacing(theme::INPUT_BAR_SPACING)
    .grid_row(0);

    let mut rows: Vec<Element> = vec![input_bar.into(), content.grid_row(1)];
    let mut row_lengths: Vec<GridLength> = vec![GridLength::Auto, GridLength::Star(1.0)];

    if cfg!(debug_assertions) {
        rows.push(
            text_block("rdict_reactor dev")
                .font_size(12.0)
                .foreground(ThemeRef::TertiaryText)
                .horizontal_alignment(HorizontalAlignment::Center)
                .grid_row(2)
                .into(),
        );
        row_lengths.push(GridLength::Auto);
    }

    grid(rows)
        .rows(row_lengths)
        .row_spacing(10.0)
        .padding(theme::PAGE_PADDING)
        .keyboard_accelerator(KeyboardAccelerator::new(
            VirtualKey::Enter,
            VirtualKeyModifiers::None,
            submit,
        ))
        .into()
}

fn centered(children: impl Into<Element>) -> Element {
    vstack((children.into(),))
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}

fn main() -> Result<()> {
    bootstrap()?;
    App::new()
        .title("Rdict")
        .backdrop(Backdrop::Mica)
        .inner_size(400.0, 600.0)
        .inner_constraints(InnerConstraints {
            min_width: Some(200.0),
            min_height: Some(200.0),
            ..InnerConstraints::default()
        })
        .render(app)
}
