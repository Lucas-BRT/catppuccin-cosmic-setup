// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Alignment, Color, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, about::About, image::Handle, menu};
use ctp_cosmic::{apply, palette, styles, wallpaper};

use crate::config::Config;
use crate::fl;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// Size of a card in the wallpaper grid, and of the large preview.
const THUMB: (usize, usize) = (288, 162);
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
    Flavor(usize),
    Accent(usize),
    Style(&'static str),
    LightMode(bool),
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
        if palette::check(&config.flavor, Some(&config.accent)).is_err() {
            config = Config::default();
        }
        if !styles.contains(&config.style.as_str()) {
            config.style = Config::default().style;
        }

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            key_binds: HashMap::new(),
            config,
            flavors: palette::flavors(),
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

        let index = |list: &[&str], value: &str| list.iter().position(|v| *v == value);
        let controls = widget::settings::section()
            .title(fl!("palette"))
            .add(
                widget::settings::item::builder(fl!("flavor")).control(widget::dropdown(
                    &self.flavors,
                    index(&self.flavors, &self.config.flavor),
                    Message::Flavor,
                )),
            )
            .add(
                widget::settings::item::builder(fl!("accent")).control(widget::dropdown(
                    &self.accents,
                    index(&self.accents, &self.config.accent),
                    Message::Accent,
                )),
            )
            .add(
                widget::settings::item::builder(fl!("light-mode"))
                    .description(fl!("light-mode-desc"))
                    .toggler(self.config.light_mode, Message::LightMode),
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

        let sidebar = widget::column::with_capacity(5)
            .push(self.preview())
            .push(self.swatches())
            .push(controls)
            .push(apply)
            .push(status)
            .spacing(space_s)
            .width(Length::Fixed(380.0));

        let cards = self.styles.iter().map(|&name| self.card(name)).collect();
        let mut grid = widget::column::with_capacity(2)
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
            Message::Flavor(i) => {
                if let Some(&flavor) = self.flavors.get(i) {
                    self.config.flavor = flavor.into();
                    save(&self.config);
                    return self.render_all();
                }
            }
            Message::Accent(i) => {
                if let Some(&accent) = self.accents.get(i) {
                    self.config.accent = accent.into();
                    save(&self.config);
                    return self.render_all();
                }
            }
            Message::Style(name) => {
                self.config.style = name.into();
                save(&self.config);
                return self.render_big();
            }
            Message::LightMode(on) => {
                self.config.light_mode = on;
                save(&self.config);
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
                let opts = options(&self.config);
                return Task::perform(blocking(move || apply::run(&opts, true)), |result| {
                    cosmic::Action::App(Message::Applied(
                        result.unwrap_or_else(|| Err("the task stopped".into())),
                    ))
                });
            }
            Message::Applied(result) => {
                self.status = match result {
                    Ok(()) => Status::Done,
                    Err(error) => Status::Failed(error),
                };
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
        let (generation, flavor, accent) = (
            self.generation,
            self.config.flavor.clone(),
            self.config.accent.clone(),
        );
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
        let (generation, flavor, accent, style) = (
            self.generation,
            self.config.flavor.clone(),
            self.config.accent.clone(),
            self.config.style.clone(),
        );
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
                .content_fit(cosmic::iced::ContentFit::Cover)
                .into(),
            None => widget::Space::new().width(Length::Fill).height(214).into(),
        }
    }

    /// The flavor's neutral colors and the accent, as a strip of squares.
    fn swatches(&self) -> Element<'_, Message> {
        let Some(p) = palette::Palette::new(&self.config.flavor) else {
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
        row.push(square(&self.config.accent)).into()
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
            .selected(self.config.style == name)
            .on_press(Message::Style(name))
            .into()
    }
}

/// What `ctp-cosmic apply` runs for these choices: the desktop theme and the
/// wallpaper, the two things this app previews.
fn options(config: &Config) -> apply::Opts {
    let dark = palette::check(&config.flavor, None).is_ok_and(|p| p.is_dark());
    apply::Opts {
        flavor: config.flavor.clone(),
        accent: config.accent.clone(),
        style: config.style.clone(),
        light_flavor: (dark && config.light_mode).then(|| "latte".to_string()),
        steps: vec!["cosmic".into(), "wallpaper".into()],
        ..apply::Opts::default()
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
    fn latte_in_light_mode_only_with_a_dark_flavor() {
        let mut c = Config {
            light_mode: true,
            ..Config::default()
        };
        assert_eq!(options(&c).light_flavor.as_deref(), Some("latte"));
        assert_eq!(options(&c).steps, ["cosmic", "wallpaper"]);
        c.flavor = "latte".into();
        assert_eq!(options(&c).light_flavor, None);
    }
}
