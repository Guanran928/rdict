#![windows_subsystem = "windows"]

mod components;
mod render;

use directories_next::ProjectDirs;
use iced::font;
use iced::time::Instant;
use iced::widget::button::{Status as ButtonStatus, Style as ButtonStyle};
use iced::widget::{Stack, column, container, row, space};
use iced::window::Settings;
use iced::{Alignment, Background, Color, Font, Length, Padding, Size, Subscription, Task, border};
use iced_material as material;
use material::widget::{button, progress_bar, select, text_input, theme_picker};
use rdict_core::Error;
use rdict_core::model::Language;
use rdict_core::rdict::TranslationData;
use rdict_core::rdict::{FetchedResult, Rdict};
use std::sync::Arc;

use crate::components::list_item;

const INITIAL_WINDOW_WIDTH: f32 = 450.0;
const INITIAL_WINDOW_HEIGHT: f32 = 600.0;
const THEME_PANEL_PADDING: f32 = 12.0;
const THEME_PANEL_RADIUS: f32 = 28.0;
const THEME_PICKER_PANEL_SPACING: f32 = 8.0;
const THEME_PANEL_SECTION_SPACING: f32 = 12.0;
const THEME_SWATCH_SIZE: f32 = 40.0;
const THEME_SWATCH_TARGET_SIZE: f32 = 48.0;
const THEME_SWATCH_COLUMNS: usize = 4;
const THEME_SELECTED_SWATCH_OUTLINE_WIDTH: f32 = 3.0;
const THEME_SWATCH_OUTLINE_WIDTH: f32 = 1.0;

struct State {
    text_input_content: String,
    selected_language: Language,
    translation_result: TranslationState,
    client: Option<Arc<Rdict>>,
    theme_controller: theme_picker::ThemeController,
    window_size: Size,
    loading_indicator: progress_bar::IndeterminateState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            text_input_content: String::new(),
            selected_language: Language::default(),
            translation_result: TranslationState::default(),
            client: None,
            theme_controller: theme_picker::ThemeController::new(
                theme_picker::MaterialColor::Purple,
                system_dark_mode(),
            ),
            window_size: initial_window_size(),
            loading_indicator: progress_bar::IndeterminateState::new(Instant::now()),
        }
    }
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
    ClientReady(Result<Arc<Rdict>, String>),
    Submit,
    ContentChanged(String),
    LanguageChanged(Language),
    TranslationResult(FetchedResult),
    TranslationError(String),
    ThemeChanged(theme_picker::ThemeAction),
    WindowResized(Size),
    Frame(Instant),
}

fn main() -> iced::Result {
    // https://www.reddit.com/r/learnrust/comments/jaqfcx/windows_print_to_hidden_console_window/
    #[cfg(target_os = "windows")]
    {
        use winapi::um::wincon::{ATTACH_PARENT_PROCESS, AttachConsole};
        // SAFETY: AttachConsole is safe to call even if there is no parent console;
        // it returns 0 on failure which we discard intentionally.
        unsafe {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }

    material::application(
        || {
            (
                State::default(),
                Task::perform(init_client(), Message::ClientReady),
            )
        },
        update,
        view,
    )
    .title("Rdict")
    .subscription(subscription)
    .theme(|state: &State| state.theme_controller.theme("Rdict"))
    .window(Settings {
        size: initial_window_size(),
        min_size: Some(initial_window_size()),
        ..Settings::default()
    })
    .run()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ClientReady(Ok(client)) => {
            state.client = Some(client);
            Task::none()
        }
        Message::ClientReady(Err(err)) => {
            state.translation_result = TranslationState::Error(err);
            Task::none()
        }
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

            let Some(client) = state.client.clone() else {
                return Task::none();
            };

            state.translation_result = TranslationState::Loading;
            state.loading_indicator = progress_bar::IndeterminateState::new(Instant::now());

            let text_input_content = state.text_input_content.clone();
            let selected_language = state.selected_language;

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
        Message::ThemeChanged(action) => {
            state.theme_controller.update(
                action,
                state.window_size,
                theme_picker::FLOATING_MARGIN,
                Instant::now(),
            );
            Task::none()
        }
        Message::WindowResized(size) => {
            state.window_size = size;
            Task::none()
        }
        Message::Frame(now) => {
            state.loading_indicator.advance(now);
            let _ = state.theme_controller.advance(now);
            Task::none()
        }
    }
}

fn subscription(state: &State) -> Subscription<Message> {
    let mut subscriptions =
        vec![iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size))];

    if matches!(state.translation_result, TranslationState::Loading)
        || state.theme_controller.is_animating()
    {
        subscriptions.push(iced::window::frames().map(Message::Frame));
    }

    Subscription::batch(subscriptions)
}

fn view(state: &State) -> material::Element<'_, Message> {
    let now = Instant::now();

    let main: material::Element<'_, Message> = match &state.translation_result {
        TranslationState::Empty => space().width(Length::Fill).height(Length::Fill).into(),

        TranslationState::Error(error) => container(column![
            material::text::title_medium("Lookup Error")
                .style(material::text::error)
                .size(20)
                .font(Font {
                    weight: font::Weight::Bold,
                    ..Font::default()
                }),
            material::text::body_large(error).style(material::text::surface_variant),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into(),

        TranslationState::Loading => container(progress_bar::loading_indicator(
            state.loading_indicator.loading_phase(),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into(),

        TranslationState::Translation(fetched_result) => match &fetched_result.data {
            TranslationData::FromEnglish(tc) => render::en::to_chinese(tc),
            TranslationData::ToEnglish(te) => render::en::to_english(te),
            TranslationData::FromFrench(tc) => render::fr::to_chinese(tc),
            TranslationData::ToFrench(te) => render::fr::to_french(te),
            TranslationData::FromKorean(tc) => render::ko::to_chinese(tc),
            TranslationData::ToKorean(te) => render::ko::to_korean(te),
            TranslationData::FromJapanese(tc) => render::ja::to_chinese(tc),
            TranslationData::ToJapanese(te) => render::ja::to_japanese(te),

            TranslationData::NotFound(nf) => {
                let suggestions_col = {
                    let mut col = column![];
                    for suggestion in &nf.suggestions {
                        col = col.push(list_item(material::text::body_large(suggestion)));
                    }
                    col
                };

                container(
                    column![
                        column![
                            material::text::title_medium("Translation not found")
                                .style(material::text::error)
                                .size(20)
                                .font(Font {
                                    weight: font::Weight::Bold,
                                    ..Font::default()
                                }),
                            material::text::body_large("Did you mean:")
                                .style(material::text::surface_variant),
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
            text_input::outlined_floating("Type something here", &state.text_input_content)
                .on_input(Message::ContentChanged)
                .on_submit(Message::Submit),
            select::outlined(
                &LANGUAGES[..],
                Some(state.selected_language),
                Message::LanguageChanged
            )
            .width(Length::Shrink),
            button::filled("Translate").on_press(Message::Submit),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        main
    ]
    .padding(10)
    .spacing(10);

    if cfg!(debug_assertions) {
        layout = layout.push(
            material::text::body_medium("rdict_iced dev")
                .style(material::text::surface_variant)
                .align_x(iced::alignment::Horizontal::Center),
        );
    }

    let content = theme_controls_over(layout.into(), state);

    state.theme_controller.reveal_over(content, now)
}

fn theme_controls_over<'a>(
    content: material::Element<'a, Message>,
    state: &'a State,
) -> material::Element<'a, Message> {
    let layer: material::Element<'a, Message> = container(floating_theme_controls(state))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: theme_picker::FLOATING_MARGIN,
            bottom: theme_picker::FLOATING_MARGIN,
            left: 0.0,
        })
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .into();

    Stack::with_children([content, layer])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn floating_theme_controls(state: &State) -> iced::widget::Column<'_, Message, material::Theme> {
    let mut controls = column![]
        .spacing(THEME_PICKER_PANEL_SPACING)
        .align_x(iced::alignment::Horizontal::Right);

    if state.theme_controller.is_picker_open() {
        controls = controls.push(theme_picker_panel(state));
    }

    controls.push(
        button::primary_fab("palette").on_press(Message::ThemeChanged(
            theme_picker::ThemeAction::TogglePicker,
        )),
    )
}

fn theme_picker_panel(state: &State) -> material::Container<'_, Message> {
    let mut swatch_rows = column![].spacing(THEME_PICKER_PANEL_SPACING);

    for colors in theme_picker::MaterialColor::ALL.chunks(THEME_SWATCH_COLUMNS) {
        let mut swatch_row = row![].spacing(THEME_PICKER_PANEL_SPACING);

        for color in colors {
            swatch_row = swatch_row.push(theme_swatch_button(
                *color,
                *color == state.theme_controller.selected_color(),
            ));
        }

        swatch_rows = swatch_rows.push(swatch_row);
    }

    let content = column![
        state
            .theme_controller
            .dark_mode_switch("Dark mode", Message::ThemeChanged),
        swatch_rows
    ]
    .spacing(THEME_PANEL_SECTION_SPACING);

    container(content)
        .padding(THEME_PANEL_PADDING)
        .style(theme_picker_panel_style)
}

fn theme_swatch_button(
    color: theme_picker::MaterialColor,
    selected: bool,
) -> button::Button<'static, Message> {
    button::Button::new(
        container(space())
            .width(Length::Fixed(THEME_SWATCH_SIZE))
            .height(Length::Fixed(THEME_SWATCH_SIZE)),
    )
    .width(Length::Fixed(THEME_SWATCH_TARGET_SIZE))
    .height(Length::Fixed(THEME_SWATCH_TARGET_SIZE))
    .padding(Padding::from([
        (THEME_SWATCH_TARGET_SIZE - THEME_SWATCH_SIZE) / 2.0,
        (THEME_SWATCH_TARGET_SIZE - THEME_SWATCH_SIZE) / 2.0,
    ]))
    .on_press(Message::ThemeChanged(
        theme_picker::ThemeAction::SelectColor(color),
    ))
    .style(move |theme, status| theme_swatch_style(theme, status, color, selected))
}

fn theme_picker_panel_style(theme: &material::Theme) -> iced::widget::container::Style {
    let colors = theme.colors();

    iced::widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text),
        border: border::rounded(THEME_PANEL_RADIUS),
        ..iced::widget::container::Style::default()
    }
}

fn theme_swatch_style(
    theme: &material::Theme,
    status: ButtonStatus,
    color: theme_picker::MaterialColor,
    selected: bool,
) -> ButtonStyle {
    let colors = theme.colors();
    let base = color.swatch();
    let background = match status {
        ButtonStatus::Active | ButtonStatus::Disabled => base,
        ButtonStatus::Hovered => mix_color(base, colors.surface.text, 0.08),
        ButtonStatus::Pressed => mix_color(base, colors.surface.text, 0.12),
    };

    ButtonStyle {
        background: Some(Background::Color(background)),
        text_color: colors.surface.text,
        border: border::rounded(THEME_SWATCH_SIZE)
            .color(if selected {
                colors.surface.text
            } else {
                colors.outline.variant
            })
            .width(if selected {
                THEME_SELECTED_SWATCH_OUTLINE_WIDTH
            } else {
                THEME_SWATCH_OUTLINE_WIDTH
            }),
        ..ButtonStyle::default()
    }
}

fn mix_color(from: Color, to: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);

    Color {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

fn system_dark_mode() -> bool {
    match dark_light::detect() {
        Ok(dark_light::Mode::Light) => false,
        Ok(dark_light::Mode::Dark | dark_light::Mode::Unspecified) | Err(_) => true,
    }
}

fn initial_window_size() -> Size {
    Size::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
}

async fn init_client() -> Result<Arc<Rdict>, String> {
    let cache_db_path = ProjectDirs::from("dev", "ny4", "rdict")
        .map(|proj_dirs| proj_dirs.cache_dir().join("cache.db"));

    Rdict::new("https://m.youdao.com", cache_db_path)
        .await
        .map(Arc::new)
        .map_err(|e| e.to_string())
}

async fn fetch_translation(
    client: Arc<Rdict>,
    text_input_content: String,
    language: Language,
) -> Result<FetchedResult, Error> {
    client.get_results(&text_input_content, language).await
}
