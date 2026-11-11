// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Alignment, Color, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, about::About, image::Handle, menu};
use ctp_cosmic::look::{self, Dirs, Look};
use ctp_cosmic::{palette, styles, wallpaper};

use crate::config::Config;
use crate::fl;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// Size of a card in the wallpaper grid, and of the large preview.
const THUMB: (usize, usize) = (224, 126);
const BIG: (usize, usize) = (960, 540);

/// The colors shown as the palette, in the order they are drawn.
const SWATCHES: [&str; 8] = [
    "crust", "mantle", "base", "surface0", "surface1", "overlay0", "subtext0", "text",
];

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    about: About,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    /// The mode being edited.
    mode: Mode,
    /// What COSMIC has now.
    cosmic: Cosmic,
    /// The dark flavors: the light look is always Latte.
    flavors: Vec<&'static str>,
    accents: Vec<&'static str>,
    /// Styles in display order.
    styles: Vec<&'static str>,
    thumbs: HashMap<&'static str, Handle>,
    big: Option<Handle>,
    /// Bumped on every change of flavor or accent, so a slow render of an
    /// earlier choice never replaces the current one.
    generation: u64,
    rendering: bool,
    status: Status,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    Dark,
    Light,
}

/// The state of COSMIC shown next to the editor.
#[derive(Debug, Clone, Default)]
struct Cosmic {
    dark: Option<String>,
    light: Option<String>,
    is_dark: bool,
    auto_switch: bool,
}

impl Cosmic {
    fn read() -> Self {
        let Ok(dirs) = Dirs::home() else {
            return Self::default();
        };
        Self {
            dark: look::active_name(&dirs.config, true),
            light: look::active_name(&dirs.config, false),
            is_dark: look::is_dark_now(&dirs.config),
            auto_switch: look::auto_switch(&dirs.config),
        }
    }
}

#[derive(Debug, Clone, Default)]
enum Status {
    #[default]
    Idle,
    Applying,
    Done,
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Mode(Mode),
    Flavor(usize),
    Accent(usize),
    Style(&'static str),
    AutoSwitch(bool),
    OpenAppearance,
    /// Every style at thumbnail size, for one generation.
    Thumbs(u64, Vec<(&'static str, Handle)>),
    /// The selected style at preview size, for one generation.
    Big(u64, Handle),
    Apply,
    Applied(Result<(), String>),
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
}

/// RGB pixels as iced wants them.
fn handle(size: (usize, usize), rgb: &[u8]) -> Handle {
    let rgba: Vec<u8> = rgb
        .as_chunks::<3>()
        .0
        .iter()
        .flat_map(|[r, g, b]| [*r, *g, *b, 255])
        .collect();
    Handle::from_rgba(size.0 as u32, size.1 as u32, rgba)
}

/// Runs blocking work off the update loop.
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    tokio::task::spawn_blocking(f).await.ok()
}

fn save(config: &Config) {
    if let Ok(context) =
        cosmic_config::Config::new(<AppModel as cosmic::Application>::APP_ID, Config::VERSION)
    {
        let _ = config.write_entry(&context);
    }
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.Lucas_BRT.CtpCosmic";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        let mut config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();
        // A stale or hand-edited value must not reach the renderer.
        let styles: Vec<_> = styles::list().into_iter().map(|(name, _)| name).collect();
        let valid = palette::check(&config.dark_flavor, Some(&config.dark_accent))
            .is_ok_and(|p| p.is_dark())
            && palette::check("latte", Some(&config.light_accent)).is_ok()
            && styles.contains(&config.dark_style.as_str())
            && styles.contains(&config.light_style.as_str());
        if !valid {
            config = Config::default();
        }

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            key_binds: HashMap::new(),
            config,
            mode: Mode::Dark,
            cosmic: Cosmic::read(),
            flavors: palette::flavors()
                .into_iter()
                .filter(|f| palette::check(f, None).is_ok_and(|p| p.is_dark()))
                .collect(),
            accents: palette::accents(),
            styles,
            thumbs: HashMap::new(),
            big: None,
            generation: 0,
            rendering: false,
            status: Status::Idle,
        };
        let title = app.update_title();
        let render = app.render_all();
        (app, Task::batch([title, render]))
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);
        vec![menu_bar.into()]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }
        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let cosmic::cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            space_s,
            space_m,
            ..
        } = cosmic::theme::spacing();

        let look = self.look();
        let dark = self.mode == Mode::Dark;
        let mode_button = |label: String, mode: Mode| {
            let button = widget::button::text(label).on_press(Message::Mode(mode));
            if self.mode == mode {
                button.class(cosmic::theme::Button::Suggested)
            } else {
                button.class(cosmic::theme::Button::Standard)
            }
        };
        let modes = widget::row::with_capacity(2)
            .push(mode_button(fl!("dark"), Mode::Dark))
            .push(mode_button(fl!("light"), Mode::Light))
            .spacing(space_xs);

        let index = |list: &[&str], value: &str| list.iter().position(|v| *v == value);
        let mut palette_section = widget::settings::section().title(fl!("palette"));
        if dark {
            palette_section = palette_section.add(
                widget::settings::item::builder(fl!("flavor")).control(widget::dropdown(
                    &self.flavors,
                    index(&self.flavors, &look.flavor),
                    Message::Flavor,
                )),
            );
        } else {
            palette_section = palette_section.add(
                widget::settings::item::builder(fl!("flavor"))
                    .control(widget::text::body(look.flavor.clone())),
            );
        }
        palette_section = palette_section.add(
            widget::settings::item::builder(fl!("accent")).control(widget::dropdown(
                &self.accents,
                index(&self.accents, &look.accent),
                Message::Accent,
            )),
        );

        let status: Element<_> = match &self.status {
            Status::Idle => widget::text::caption(String::new()).into(),
            Status::Applying => widget::text::caption(fl!("applying")).into(),
            Status::Done => widget::text::caption(fl!("applied")).into(),
            Status::Failed(error) => {
                widget::text::caption(fl!("failed", error = error.clone())).into()
            }
        };
        let apply = widget::button::suggested(fl!("apply"))
            .on_press_maybe((!matches!(self.status, Status::Applying)).then_some(Message::Apply));

        let none = fl!("none");
        let now = widget::settings::section()
            .title(fl!("in-cosmic"))
            .add(
                widget::settings::item::builder(self.mode_label(true)).control(widget::text::body(
                    self.cosmic.dark.clone().unwrap_or_else(|| none.clone()),
                )),
            )
            .add(
                widget::settings::item::builder(self.mode_label(false)).control(
                    widget::text::body(self.cosmic.light.clone().unwrap_or_else(|| none.clone())),
                ),
            )
            .add(
                widget::settings::item::builder(fl!("auto-switch"))
                    .description(fl!("auto-switch-desc"))
                    .toggler(self.cosmic.auto_switch, Message::AutoSwitch),
            );

        let sidebar = widget::column::with_capacity(8)
            .push(self.preview())
            .push(self.swatches())
            .push(modes)
            .push(palette_section)
            .push(widget::text::caption(fl!("saved-as", name = look.name())))
            .push(
                widget::row::with_children(vec![
                    apply.into(),
                    widget::button::standard(fl!("open-appearance"))
                        .on_press(Message::OpenAppearance)
                        .into(),
                ])
                .spacing(space_xs),
            )
            .push(status)
            .push(now)
            .spacing(space_s)
            .width(Length::Fixed(340.0));

        let cards = self.styles.iter().map(|&name| self.card(name)).collect();
        let mut grid = widget::column::with_capacity(3)
            .push(widget::text::title4(fl!("wallpapers")))
            .spacing(space_xs);
        if self.rendering {
            grid = grid.push(widget::text::caption(fl!("rendering")));
        }
        let grid = grid.push(
            widget::flex_row(cards)
                .row_spacing(space_xs)
                .column_spacing(space_xs),
        );

        widget::row::with_capacity(2)
            .push(widget::scrollable(sidebar).height(Length::Fill))
            .push(
                widget::scrollable(grid)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .spacing(space_m)
            .padding([space_xxs, space_m])
            .align_y(Alignment::Start)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config))
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Mode(mode) => {
                if self.mode != mode {
                    self.mode = mode;
                    return self.render_all();
                }
            }
            Message::Flavor(i) => {
                if let Some(&flavor) = self.flavors.get(i) {
                    self.config.dark_flavor = flavor.into();
                    save(&self.config);
                    return self.render_all();
                }
            }
            Message::Accent(i) => {
                if let Some(&accent) = self.accents.get(i) {
                    match self.mode {
                        Mode::Dark => self.config.dark_accent = accent.into(),
                        Mode::Light => self.config.light_accent = accent.into(),
                    }
                    save(&self.config);
                    return self.render_all();
                }
            }
            Message::Style(name) => {
                match self.mode {
                    Mode::Dark => self.config.dark_style = name.into(),
                    Mode::Light => self.config.light_style = name.into(),
                }
                save(&self.config);
                return self.render_big();
            }
            Message::AutoSwitch(on) => {
                if let Ok(dirs) = Dirs::home()
                    && let Err(error) = look::set_auto_switch(&dirs.config, on)
                {
                    self.status = Status::Failed(error);
                }
                self.cosmic = Cosmic::read();
            }
            Message::OpenAppearance => {
                if let Err(err) = std::process::Command::new("cosmic-settings")
                    .arg("appearance")
                    .spawn()
                {
                    self.status = Status::Failed(format!("cosmic-settings: {err}"));
                }
            }
            Message::Thumbs(generation, thumbs) => {
                if generation == self.generation {
                    self.thumbs = thumbs.into_iter().collect();
                    self.rendering = false;
                }
            }
            Message::Big(generation, big) => {
                if generation == self.generation {
                    self.big = Some(big);
                }
            }
            Message::Apply => {
                self.status = Status::Applying;
                let look = self.look();
                return Task::perform(
                    blocking(move || Dirs::home().and_then(|dirs| look::apply(&look, &dirs, true))),
                    |result| {
                        cosmic::Action::App(Message::Applied(
                            result.unwrap_or_else(|| Err("the task stopped".into())),
                        ))
                    },
                );
            }
            Message::Applied(result) => {
                self.status = match result {
                    Ok(()) => Status::Done,
                    Err(error) => Status::Failed(error),
                };
                self.cosmic = Cosmic::read();
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }
            Message::UpdateConfig(config) => self.config = config,
            Message::LaunchUrl(url) => {
                if let Err(err) = open::that_detached(&url) {
                    eprintln!("failed to open {url:?}: {err}");
                }
            }
        }
        Task::none()
    }
}

impl AppModel {
    /// "Dark" or "Light", marked when COSMIC is in that mode right now.
    fn mode_label(&self, dark: bool) -> String {
        let name = if dark { fl!("dark") } else { fl!("light") };
        if self.cosmic.is_dark == dark {
            format!("{name} ●")
        } else {
            name
        }
    }

    /// The look being edited.
    fn look(&self) -> Look {
        match self.mode {
            Mode::Dark => Look {
                flavor: self.config.dark_flavor.clone(),
                accent: self.config.dark_accent.clone(),
                style: self.config.dark_style.clone(),
            },
            Mode::Light => Look {
                flavor: "latte".into(),
                accent: self.config.light_accent.clone(),
                style: self.config.light_style.clone(),
            },
        }
    }

    fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        match self.core.main_window_id() {
            Some(id) => self.set_window_title(fl!("app-title"), id),
            None => Task::none(),
        }
    }

    /// Re-renders the thumbnails and the large preview for the current palette.
    fn render_all(&mut self) -> Task<cosmic::Action<Message>> {
        self.generation += 1;
        self.rendering = true;
        self.status = Status::Idle;
        let look = self.look();
        let (generation, flavor, accent) = (self.generation, look.flavor, look.accent);
        let names = self.styles.clone();
        let thumbs = Task::perform(
            blocking(move || wallpaper::thumbnails(&flavor, &accent, THUMB)),
            move |done| {
                let thumbs = done
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(name, rgb)| {
                        let name = *names.iter().find(|n| **n == name)?;
                        Some((name, handle(THUMB, &rgb)))
                    })
                    .collect();
                cosmic::Action::App(Message::Thumbs(generation, thumbs))
            },
        );
        Task::batch([thumbs, self.render_big()])
    }

    fn render_big(&self) -> Task<cosmic::Action<Message>> {
        let look = self.look();
        let (generation, flavor, accent, style) =
            (self.generation, look.flavor, look.accent, look.style);
        Task::perform(
            blocking(move || wallpaper::render_one(&style, &flavor, &accent, BIG)),
            move |rgb| match rgb {
                Some(rgb) => cosmic::Action::App(Message::Big(generation, handle(BIG, &rgb))),
                None => cosmic::Action::None,
            },
        )
    }

    fn preview(&self) -> Element<'_, Message> {
        match &self.big {
            Some(big) => widget::image(big.clone())
                .width(Length::Fill)
                .height(Length::Fixed(190.0))
                .content_fit(cosmic::iced::ContentFit::Cover)
                .into(),
            None => widget::Space::new().width(Length::Fill).height(190).into(),
        }
    }

    /// The flavor's neutral colors and the accent, as a strip of squares.
    fn swatches(&self) -> Element<'_, Message> {
        let Some(p) = palette::Palette::new(&self.look().flavor) else {
            return widget::Space::new().into();
        };
        let square = |name: &str| {
            let [r, g, b] = p.get(name).map(|v| v as u8);
            let color = Color::from_rgb8(r, g, b);
            widget::container(widget::Space::new().width(32).height(32)).class(
                cosmic::theme::Container::custom(move |_| widget::container::Style {
                    background: Some(cosmic::iced::Background::Color(color)),
                    border: cosmic::iced::Border {
                        radius: 8.0.into(),
                        width: 1.0,
                        color: Color::from_rgba8(128, 128, 128, 0.4),
                    },
                    ..Default::default()
                }),
            )
        };
        let mut row = widget::row::with_capacity(SWATCHES.len() + 1).spacing(6);
        for name in SWATCHES {
            row = row.push(square(name));
        }
        row.push(square(&self.look().accent)).into()
    }

    /// One wallpaper style: its thumbnail and name, selected when chosen.
    fn card(&self, name: &'static str) -> Element<'_, Message> {
        let picture: Element<_> = match self.thumbs.get(name) {
            Some(thumb) => widget::image(thumb.clone())
                .width(Length::Fixed(THUMB.0 as f32))
                .height(Length::Fixed(THUMB.1 as f32))
                .into(),
            None => widget::Space::new()
                .width(Length::Fixed(THUMB.0 as f32))
                .height(Length::Fixed(THUMB.1 as f32))
                .into(),
        };
        let content = widget::column::with_capacity(2)
            .push(picture)
            .push(widget::text::caption(name))
            .spacing(4)
            .align_x(Alignment::Center);
        widget::button::custom(content)
            .class(cosmic::theme::Button::Image)
            .selected(self.look().style == name)
            .on_press(Message::Style(name))
            .into()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_light_look_is_latte_and_the_dark_one_is_not() {
        let config = Config::default();
        let dark = Look {
            flavor: config.dark_flavor,
            accent: config.dark_accent,
            style: config.dark_style,
        };
        assert!(dark.is_dark() && dark.name().starts_with("Catppuccin Mocha"));
        let light = Look {
            flavor: "latte".into(),
            accent: config.light_accent,
            style: config.light_style,
        };
        assert!(!light.is_dark());
    }
}
