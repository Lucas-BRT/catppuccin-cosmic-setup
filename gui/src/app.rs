// SPDX-License-Identifier: MIT

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Subscription, event, keyboard};
use cosmic::prelude::*;
use cosmic::widget::{
    self,
    about::About,
    image::Handle,
    segmented_button,
    toaster::{Toast, ToastId, Toasts},
};
use ctp_cosmic::look::{self, Dirs, Look};
use ctp_cosmic::{palette, styles, wallpaper};

use crate::catalog;
use crate::config::Config;
use crate::fl;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// Gallery thumbnails and the large preview.
pub const THUMB: (usize, usize) = (320, 180);
const BIG: (usize, usize) = (720, 405);

pub struct AppModel {
    pub core: cosmic::Core,
    pub context_page: ContextPage,
    about: About,
    pub config: Config,
    /// The mode being edited.
    pub mode: Mode,
    /// What COSMIC has now.
    pub cosmic: Cosmic,
    pub accents: Vec<&'static str>,
    /// Thumbnails of every style, per "flavor/accent". The ones of an earlier
    /// palette stay on screen until the new ones are in.
    pub cache: HashMap<String, HashMap<&'static str, Handle>>,
    pub shown: Option<String>,
    rendering: HashSet<String>,
    /// Large previews, by look.
    pub bigs: HashMap<Look, Handle>,
    pub query: String,
    pub search_id: widget::Id,
    pub flavors: segmented_button::SingleSelectModel,
    pub categories: segmented_button::SingleSelectModel,
    pub status: Status,
    pub details_open: bool,
    toasts: Toasts<Message>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub fn name(self) -> String {
        match self {
            Mode::Dark => fl!("dark"),
            Mode::Light => fl!("light"),
        }
    }
}

/// The state of COSMIC shown in the drawer and on the mode cards.
#[derive(Debug, Clone, Default)]
pub struct Cosmic {
    pub dark: Option<String>,
    pub light: Option<String>,
    pub is_dark: bool,
    pub auto_switch: bool,
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

    pub fn name(&self, mode: Mode) -> Option<&String> {
        match mode {
            Mode::Dark => self.dark.as_ref(),
            Mode::Light => self.light.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Step {
    Render,
    Import,
    Wallpaper,
}

#[derive(Debug, Clone, Default)]
pub enum Status {
    #[default]
    Idle,
    Applying(Step),
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Mode(Mode),
    Flavor(segmented_button::Entity),
    Accent(&'static str),
    Style(&'static str),
    Category(segmented_button::Entity),
    Query(String),
    FocusSearch,
    AutoSwitch(bool),
    OpenAppearance,
    /// Thumbnails of every style for one "flavor/accent".
    Thumbs(String, Vec<(&'static str, Handle)>),
    Big(Look, Handle),
    Apply,
    Rendered(Look, Result<PathBuf, String>),
    Imported(Look, PathBuf, Result<(), String>),
    Applied(Look, Result<(), String>),
    ToggleDetails,
    ToastClosed(ToastId),
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
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

fn stopped<T>(result: Option<Result<T, String>>) -> Result<T, String> {
    result.unwrap_or_else(|| Err("the task stopped".into()))
}

fn save(config: &Config) {
    if let Ok(context) =
        cosmic_config::Config::new(<AppModel as cosmic::Application>::APP_ID, Config::VERSION)
    {
        let _ = config.write_entry(&context);
    }
}

pub fn key(look: &Look) -> String {
    format!("{}/{}", look.flavor, look.accent)
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
        let known: Vec<_> = styles::list().into_iter().map(|(name, _)| name).collect();
        let valid = palette::check(&config.dark_flavor, Some(&config.dark_accent))
            .is_ok_and(|p| p.is_dark())
            && palette::check("latte", Some(&config.light_accent)).is_ok()
            && known.contains(&config.dark_style.as_str())
            && known.contains(&config.light_style.as_str());
        if !valid {
            config = Config::default();
        }

        let cosmic_state = Cosmic::read();
        let mut flavors = segmented_button::SingleSelectModel::default();
        for flavor in ["mocha", "macchiato", "frappe"] {
            let entity = flavors
                .insert()
                .text(crate::ui::flavor_label(flavor))
                .data::<&'static str>(flavor)
                .id();
            if flavor == config.dark_flavor {
                flavors.activate(entity);
            }
        }
        let mut categories = segmented_button::SingleSelectModel::default();
        categories
            .insert()
            .text(fl!("cat-all"))
            .data::<Option<usize>>(None)
            .activate();
        for (i, (id, _)) in catalog::CATEGORIES.iter().enumerate() {
            categories
                .insert()
                .text(crate::ui::category_label(id))
                .data::<Option<usize>>(Some(i));
        }

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            config,
            // Start on the mode COSMIC is in.
            mode: if cosmic_state.is_dark {
                Mode::Dark
            } else {
                Mode::Light
            },
            cosmic: cosmic_state,
            accents: palette::accents(),
            cache: HashMap::new(),
            shown: None,
            rendering: HashSet::new(),
            bigs: HashMap::new(),
            query: String::new(),
            search_id: widget::Id::unique(),
            flavors,
            categories,
            status: Status::Idle,
            details_open: false,
            toasts: Toasts::new(Message::ToastClosed),
        };
        let title = app.update_title();
        let render = app.render_all();
        (app, Task::batch([title, render]))
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        crate::ui::header_end()
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
            ContextPage::Cosmic => context_drawer::context_drawer(
                self.cosmic_drawer(),
                Message::ToggleContextPage(ContextPage::Cosmic),
            )
            .title(fl!("in-cosmic")),
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        widget::toaster(&self.toasts, self.layout())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen_with(|event, _status, _window| match event {
            event::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if modifiers.command() && c.as_str() == "f" => Some(Message::FocusSearch),
            event::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Enter),
                modifiers,
                ..
            }) if modifiers.command() => Some(Message::Apply),
            _ => None,
        })
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Mode(mode) => {
                if self.mode != mode {
                    self.mode = mode;
                    self.sync_flavor_model();
                    return self.render_all();
                }
            }
            Message::Flavor(entity) => {
                if let Some(&flavor) = self.flavors.data::<&'static str>(entity) {
                    self.flavors.activate(entity);
                    self.config.dark_flavor = flavor.into();
                    save(&self.config);
                    return self.render_all();
                }
            }
            Message::Accent(accent) => {
                match self.mode {
                    Mode::Dark => self.config.dark_accent = accent.into(),
                    Mode::Light => self.config.light_accent = accent.into(),
                }
                save(&self.config);
                return self.render_all();
            }
            Message::Style(name) => {
                match self.mode {
                    Mode::Dark => self.config.dark_style = name.into(),
                    Mode::Light => self.config.light_style = name.into(),
                }
                save(&self.config);
                return self.render_big();
            }
            Message::Category(entity) => self.categories.activate(entity),
            Message::Query(query) => self.query = query,
            Message::FocusSearch => return widget::text_input::focus(self.search_id.clone()),
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
            Message::Thumbs(key, thumbs) => {
                self.rendering.remove(&key);
                self.cache.insert(key, thumbs.into_iter().collect());
                self.refresh_shown();
            }
            Message::Big(look, big) => {
                self.bigs.insert(look, big);
            }
            Message::Apply => {
                if !matches!(self.status, Status::Applying(_)) && !self.is_applied() {
                    self.status = Status::Applying(Step::Render);
                    self.details_open = false;
                    let look = self.look();
                    let job = look.clone();
                    return Task::perform(
                        blocking(move || {
                            Dirs::home().and_then(|d| look::render_wallpaper(&job, &d))
                        }),
                        move |r| cosmic::Action::App(Message::Rendered(look.clone(), stopped(r))),
                    );
                }
            }
            Message::Rendered(look, result) => match result {
                Ok(png) => {
                    self.status = Status::Applying(Step::Import);
                    let job = look.clone();
                    let file = png.clone();
                    return Task::perform(
                        blocking(move || {
                            Dirs::home().and_then(|d| look::import_theme(&job, &d, true))
                        }),
                        move |r| {
                            cosmic::Action::App(Message::Imported(
                                look.clone(),
                                file.clone(),
                                stopped(r),
                            ))
                        },
                    );
                }
                Err(error) => self.status = Status::Failed(error),
            },
            Message::Imported(look, png, result) => match result {
                Ok(()) => {
                    self.status = Status::Applying(Step::Wallpaper);
                    let job = look.clone();
                    return Task::perform(
                        blocking(move || {
                            Dirs::home().and_then(|d| look::set_wallpaper(&job, &d, &png))
                        }),
                        move |r| cosmic::Action::App(Message::Applied(look.clone(), stopped(r))),
                    );
                }
                Err(error) => self.status = Status::Failed(error),
            },
            Message::Applied(look, result) => {
                self.cosmic = Cosmic::read();
                match result {
                    Ok(()) => {
                        self.status = Status::Idle;
                        let mode = if look.is_dark() {
                            Mode::Dark
                        } else {
                            Mode::Light
                        };
                        let toast = Toast::new(fl!(
                            "toast-applied",
                            name = crate::ui::short_name(&look),
                            mode = mode.name().to_lowercase()
                        ))
                        .action(fl!("open-appearance-short"), |_| Message::OpenAppearance);
                        return self.toasts.push(toast).map(cosmic::Action::App);
                    }
                    Err(error) => self.status = Status::Failed(error),
                }
            }
            Message::ToggleDetails => self.details_open = !self.details_open,
            Message::ToastClosed(id) => self.toasts.remove(id),
            Message::ToggleContextPage(page) => {
                if self.context_page == page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = page;
                    self.core.window.show_context = true;
                }
            }
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

    /// The look of a mode.
    pub fn look_of(&self, mode: Mode) -> Look {
        match mode {
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

    /// The look being edited.
    pub fn look(&self) -> Look {
        self.look_of(self.mode)
    }

    /// COSMIC already has this look in its mode.
    pub fn is_applied(&self) -> bool {
        self.cosmic.name(self.mode) == Some(&self.look().name())
    }

    fn sync_flavor_model(&mut self) {
        let want = self.config.dark_flavor.clone();
        let found = self.flavors.iter().find(|e| {
            self.flavors
                .data::<&'static str>(*e)
                .is_some_and(|f| *f == want)
        });
        if let Some(entity) = found {
            self.flavors.activate(entity);
        }
    }

    fn refresh_shown(&mut self) {
        let current = key(&self.look());
        if self.cache.contains_key(&current) {
            self.shown = Some(current);
        }
    }

    /// Renders what is missing for both modes (thumbnails of every style per
    /// palette, cached) and the large preview of the look being edited.
    fn render_all(&mut self) -> Task<cosmic::Action<Message>> {
        self.refresh_shown();
        let mut tasks = vec![self.render_big()];
        let other = match self.mode {
            Mode::Dark => Mode::Light,
            Mode::Light => Mode::Dark,
        };
        for mode in [self.mode, other] {
            let look = self.look_of(mode);
            let k = key(&look);
            if self.cache.contains_key(&k) || !self.rendering.insert(k.clone()) {
                continue;
            }
            let (flavor, accent) = (look.flavor, look.accent);
            tasks.push(Task::perform(
                blocking(move || wallpaper::thumbnails(&flavor, &accent, THUMB)),
                move |done| {
                    let thumbs = done
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|(name, rgb)| {
                            let name = styles::list().into_iter().find(|(n, _)| *n == name)?.0;
                            Some((name, handle(THUMB, &rgb)))
                        })
                        .collect();
                    cosmic::Action::App(Message::Thumbs(k.clone(), thumbs))
                },
            ));
        }
        Task::batch(tasks)
    }

    fn render_big(&self) -> Task<cosmic::Action<Message>> {
        let look = self.look();
        if self.bigs.contains_key(&look) {
            return Task::none();
        }
        let job = look.clone();
        Task::perform(
            blocking(move || wallpaper::render_one(&job.style, &job.flavor, &job.accent, BIG)),
            move |rgb| match rgb {
                Some(rgb) => cosmic::Action::App(Message::Big(look.clone(), handle(BIG, &rgb))),
                None => cosmic::Action::None,
            },
        )
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    Cosmic,
    About,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_palette_is_cached_by_flavor_and_accent_only() {
        let a = Look {
            flavor: "mocha".into(),
            accent: "teal".into(),
            style: "glow".into(),
        };
        let b = Look {
            style: "mesh".into(),
            ..a.clone()
        };
        assert_eq!(key(&a), key(&b));
        assert_ne!(
            key(&a),
            key(&Look {
                accent: "red".into(),
                ..a
            })
        );
    }
}
