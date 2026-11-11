// SPDX-License-Identifier: MIT

//! The views: twin mode cards, the desktop preview, the look controls and the
//! style gallery.

use cosmic::iced::widget::Stack;
use cosmic::iced::{Alignment, Background, Border, Color, ContentFit, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, image::Handle};
use ctp_cosmic::look::Look;
use ctp_cosmic::palette::Palette;

use crate::app::{AppModel, ContextPage, Message, Mode, Status, Step, key};
use crate::catalog;
use crate::fl;

/// Content width at which the window gets two panes.
const WIDE: f32 = 860.0;
const LOOK_W: f32 = 360.0;
const GAP: f32 = 12.0;
const CARD_MIN: f32 = 168.0;

pub fn flavor_label(flavor: &str) -> String {
    match flavor {
        "frappe" => "Frappé".into(),
        other => cap(other),
    }
}

pub fn category_label(id: &str) -> String {
    match id {
        "calm" => fl!("cat-calm"),
        "patterns" => fl!("cat-patterns"),
        "fractals" => fl!("cat-fractals"),
        "lines" => fl!("cat-lines"),
        _ => fl!("cat-organic"),
    }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// "Mocha · Mauve · Glow".
pub fn short_name(look: &Look) -> String {
    format!(
        "{} · {} · {}",
        flavor_label(&look.flavor),
        cap(&look.accent),
        catalog::display(&look.style)
    )
}

pub fn header_end<'a>() -> Vec<Element<'a, Message>> {
    let icon = |name: &'static str, page, tip: String| {
        widget::tooltip(
            widget::button::icon(widget::icon::from_name(name))
                .on_press(Message::ToggleContextPage(page)),
            widget::text::body(tip),
            widget::tooltip::Position::Bottom,
        )
        .into()
    };
    vec![
        icon(
            "preferences-desktop-theme-symbolic",
            ContextPage::Cosmic,
            fl!("status-tooltip"),
        ),
        icon(
            "help-about-symbolic",
            ContextPage::About,
            fl!("about-tooltip"),
        ),
    ]
}

fn color(p: &Palette, name: &str) -> Color {
    let [r, g, b] = p.get(name).map(|v| v as u8);
    Color::from_rgb8(r, g, b)
}

/// A filled, optionally outlined box.
fn fill(
    color: Color,
    radius: f32,
    outline: Option<(f32, Color)>,
) -> cosmic::theme::Container<'static> {
    cosmic::theme::Container::custom(move |_| widget::container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: radius.into(),
            width: outline.map_or(0.0, |o| o.0),
            color: outline.map_or(Color::TRANSPARENT, |o| o.1),
        },
        ..Default::default()
    })
}

fn block<'a>(w: f32, h: f32, class: cosmic::theme::Container<'static>) -> Element<'a, Message> {
    widget::container(widget::Space::new().width(w).height(h))
        .class(class)
        .into()
}

/// A mock desktop in the look's colors over its wallpaper: panel, one window
/// with the accent, and the dock. `w` is its width; the shape is 16:9.
pub fn desk<'a>(look: &Look, wall: Option<&Handle>, w: f32) -> Element<'a, Message> {
    let h = w * 9.0 / 16.0;
    let Some(p) = Palette::new(&look.flavor) else {
        return widget::Space::new().width(w).height(h).into();
    };
    let (base, mantle, text, accent) = (
        color(&p, "base"),
        color(&p, "mantle"),
        color(&p, "text"),
        color(&p, &look.accent),
    );
    let faded = |c: Color, a: f32| Color { a, ..c };

    let ground: Element<_> = match wall {
        Some(handle) => widget::image(handle.clone())
            .width(w)
            .height(h)
            .content_fit(ContentFit::Cover)
            .into(),
        None => block(w, h, fill(mantle, 0.0, None)),
    };

    let dot = |c: Color| block(w * 0.016, w * 0.016, fill(c, w * 0.008, None));
    let panel = widget::container(
        widget::row::with_capacity(3)
            .push(dot(accent))
            .push(dot(faded(text, 0.55)))
            .push(dot(faded(text, 0.55)))
            .spacing(w * 0.016),
    )
    .width(Length::Fixed(w))
    .height(Length::Fixed(h * 0.08))
    .padding([0.0, w * 0.024])
    .align_y(Alignment::Center)
    .class(fill(faded(mantle, 0.92), 0.0, None));

    let title_bar = widget::container(
        widget::row::with_capacity(3)
            .push(dot(faded(text, 0.4)))
            .push(dot(faded(text, 0.4)))
            .push(dot(faded(text, 0.4)))
            .spacing(w * 0.012),
    )
    .width(Length::Fill)
    .height(Length::Fixed(h * 0.09))
    .padding([0.0, w * 0.03])
    .align_y(Alignment::Center)
    .class(fill(mantle, 0.0, None));
    let line = |frac: f32, c: Color| block(w * frac, h * 0.03, fill(c, h * 0.015, None));
    let window = widget::container(
        widget::column::with_capacity(2).push(title_bar).push(
            widget::container(
                widget::column::with_capacity(3)
                    .push(line(0.4, faded(text, 0.8)))
                    .push(line(0.24, faded(text, 0.45)))
                    .push(block(w * 0.15, h * 0.07, fill(accent, h * 0.035, None)))
                    .spacing(h * 0.04),
            )
            .padding(w * 0.035),
        ),
    )
    .width(Length::Fixed(w * 0.52))
    .height(Length::Fixed(h * 0.52))
    .class(fill(base, w * 0.02, Some((w * 0.005, accent))));

    let upper = widget::column::with_capacity(3)
        .push(panel)
        .push(widget::Space::new().height(h * 0.11))
        .push(
            widget::row::with_capacity(2)
                .push(widget::Space::new().width(w * 0.11))
                .push(window),
        );
    let dock = widget::container(
        widget::row::with_capacity(3)
            .push(block(
                w * 0.042,
                w * 0.042,
                fill(faded(text, 0.3), w * 0.012, None),
            ))
            .push(block(w * 0.042, w * 0.042, fill(accent, w * 0.012, None)))
            .push(block(
                w * 0.042,
                w * 0.042,
                fill(faded(text, 0.3), w * 0.012, None),
            ))
            .spacing(w * 0.02),
    )
    .padding([h * 0.02, w * 0.025])
    .class(fill(faded(mantle, 0.92), 999.0, None));
    let lower = widget::container(dock)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::End)
        .padding(h * 0.04);

    Stack::with_children(vec![ground, upper.into(), lower.into()])
        .width(Length::Fixed(w))
        .height(Length::Fixed(h))
        .into()
}

impl AppModel {
    fn thumb_for(&self, look: &Look) -> Option<&Handle> {
        self.cache.get(&key(look))?.get(look.style.as_str())
    }

    /// The whole window: two panes when wide, one column with a footer when not.
    pub fn layout(&self) -> Element<'_, Message> {
        widget::responsive(move |size| self.layout_for(size.width)).into()
    }

    fn layout_for(&self, width: f32) -> Element<'_, Message> {
        let space_m = cosmic::theme::spacing().space_m as f32;
        let space_l = cosmic::theme::spacing().space_l as f32;
        // Room kept for a scrollbar next to a scrolling pane.
        let bar = 12.0;
        if width >= WIDE {
            let gallery_w = width - 2.0 * space_m - space_l - LOOK_W - 2.0 * bar;
            let look_pane = widget::column::with_capacity(2)
                .push(
                    widget::scrollable(
                        widget::container(self.look_controls(LOOK_W - bar))
                            .padding([0.0, bar, 0.0, 0.0]),
                    )
                    .height(Length::Fill),
                )
                .push(self.apply_block())
                .spacing(space_m as u16)
                .width(Length::Fixed(LOOK_W + bar));
            let gallery = widget::column::with_capacity(2)
                .push(self.gallery_bar())
                .push(
                    widget::scrollable(
                        widget::container(self.gallery_grid(gallery_w))
                            .padding([0.0, bar, 0.0, 0.0]),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill),
                )
                .spacing(space_m as u16)
                .width(Length::Fill);
            widget::row::with_capacity(2)
                .push(look_pane)
                .push(gallery)
                .spacing(space_l as u16)
                .padding([0.0, space_m, space_m, space_m])
                .height(Length::Fill)
                .into()
        } else {
            let pane = (width - 2.0 * space_m - bar).clamp(240.0, 480.0);
            let body = widget::column::with_capacity(3)
                .push(
                    widget::container(self.look_controls(pane))
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                )
                .push(self.gallery_bar())
                .push(self.gallery_grid(width - 2.0 * space_m - bar))
                .spacing(space_m as u16)
                .padding([0.0, space_m, space_m, space_m]);
            let footer = widget::row::with_capacity(2)
                .push(widget::text::heading(short_name(&self.look())).width(Length::Fill))
                .push(self.apply_button(false))
                .spacing(space_m as u16)
                .align_y(Alignment::Center)
                .padding(space_m);
            widget::column::with_capacity(2)
                .push(widget::scrollable(body).height(Length::Fill))
                .push(footer)
                .into()
        }
    }

    /// Twin mode cards, preview, name, flavor and accent.
    fn look_controls(&self, w: f32) -> Element<'_, Message> {
        let space_m = cosmic::theme::spacing().space_m;
        let look = self.look();
        let card_w = (w - GAP) / 2.0;
        let twin = widget::row::with_capacity(2)
            .push(self.mode_card(Mode::Dark, card_w))
            .push(self.mode_card(Mode::Light, card_w))
            .spacing(GAP as u16);

        let wall = self.bigs.get(&look).or_else(|| self.thumb_for(&look));
        let preview = desk(&look, wall, w);
        let names = widget::column::with_capacity(2)
            .push(widget::text::title4(short_name(&look)))
            .push(widget::text::caption(look.name()));

        let flavor_row: Element<_> = if self.mode == Mode::Dark {
            widget::column::with_capacity(2)
                .push(widget::text::body(fl!("flavor")))
                .push(
                    widget::segmented_control::horizontal(&self.flavors)
                        .on_activate(Message::Flavor)
                        .width(Length::Fill),
                )
                .spacing(cosmic::theme::spacing().space_s)
                .into()
        } else {
            widget::column::with_capacity(2)
                .push(
                    widget::row::with_capacity(2)
                        .push(widget::text::body(fl!("flavor")).width(Length::Fill))
                        .push(widget::text::body(flavor_label(&look.flavor))),
                )
                .push(widget::text::caption(fl!("latte-only")))
                .spacing(4)
                .into()
        };
        let flavor_item = widget::container(flavor_row).padding([
            cosmic::theme::spacing().space_s,
            cosmic::theme::spacing().space_m,
        ]);
        let settings = widget::settings::section()
            .add(flavor_item)
            .add(self.accent_row(&look));

        let mut col = widget::column::with_capacity(6)
            .push(twin)
            .push(preview)
            .push(names)
            .push(settings)
            .spacing(space_m);
        if let Status::Failed(error) = &self.status {
            col = col.push(self.failure(error));
        }
        col.into()
    }

    fn mode_card(&self, mode: Mode, w: f32) -> Element<'_, Message> {
        let look = self.look_of(mode);
        let current = self.cosmic.is_dark == (mode == Mode::Dark);
        let mut label = widget::row::with_capacity(2)
            .push(widget::text::heading(mode.name()))
            .spacing(8)
            .align_y(Alignment::Center);
        if current {
            label = label.push(widget::text::caption(format!("● {}", fl!("active"))));
        }
        let inner = (w - 16.0).max(60.0);
        // Only the preview is the button; text inside a button takes its accent color.
        let preview = widget::button::custom(desk(&look, self.thumb_for(&look), inner))
            .class(cosmic::theme::Button::Image)
            .selected(self.mode == mode)
            .on_press(Message::Mode(mode));
        widget::column::with_capacity(3)
            .push(preview)
            .push(label)
            .push(widget::text::caption(short_name(&look)))
            .spacing(4)
            .width(Length::Fixed(w))
            .into()
    }

    /// The accent label with the chosen name, over a row of swatches.
    fn accent_row(&self, look: &Look) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let Some(p) = Palette::new(&look.flavor) else {
            return widget::Space::new().into();
        };
        let base = color(&p, "base");
        let swatches = self.accents.iter().map(|&accent| {
            let chosen = look.accent == accent;
            let mark: Element<_> = if chosen {
                widget::text::body("✓")
                    .class(cosmic::theme::Text::Color(base))
                    .into()
            } else {
                widget::Space::new().into()
            };
            let face = widget::container(mark)
                .width(Length::Fixed(32.0))
                .height(Length::Fixed(32.0))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .class(fill(color(&p, accent), 8.0, None));
            widget::tooltip(
                widget::button::custom(face)
                    .padding(0)
                    .class(cosmic::theme::Button::Image)
                    .selected(chosen)
                    .on_press(Message::Accent(accent)),
                widget::text::body(cap(accent)),
                widget::tooltip::Position::Bottom,
            )
            .into()
        });
        widget::container(
            widget::column::with_capacity(2)
                .push(
                    widget::row::with_capacity(2)
                        .push(widget::text::body(fl!("accent")).width(Length::Fill))
                        .push(widget::text::body(cap(&look.accent))),
                )
                .push(
                    widget::flex_row(swatches.collect())
                        .row_spacing(6)
                        .column_spacing(6),
                )
                .spacing(space_s),
        )
        .padding([space_s, cosmic::theme::spacing().space_m])
        .into()
    }

    fn failure(&self, error: &str) -> Element<'_, Message> {
        let mut col = widget::column::with_capacity(3)
            .push(
                widget::warning(format!("{}\n{}", fl!("failed-title"), fl!("failed-body")))
                    .into_widget(),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(widget::button::text(fl!("try-again")).on_press(Message::Apply))
                    .push(widget::button::text(fl!("details")).on_press(Message::ToggleDetails)),
            )
            .spacing(4);
        if self.details_open {
            col = col.push(widget::text::caption(error.to_string()));
        }
        col.into()
    }

    fn apply_button(&self, wide: bool) -> Element<'_, Message> {
        let applying = matches!(self.status, Status::Applying(_));
        let applied = self.is_applied() && !applying;
        let label = if applying {
            fl!("applying")
        } else if applied {
            format!("✓ {}", fl!("applied"))
        } else if self.mode == Mode::Dark {
            fl!("apply-dark")
        } else {
            fl!("apply-light")
        };
        // A custom button: the text variants keep their natural width.
        let face = widget::container(widget::text::heading(label))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding([
                cosmic::theme::spacing().space_xs,
                cosmic::theme::spacing().space_m,
            ]);
        let button = widget::button::custom(face)
            .class(if applied {
                cosmic::theme::Button::Standard
            } else {
                cosmic::theme::Button::Suggested
            })
            .on_press_maybe((!applying && !applied).then_some(Message::Apply));
        if wide {
            button.width(Length::Fill).into()
        } else {
            button.into()
        }
    }

    /// Apply, with the step it is on or where the look already stands.
    fn apply_block(&self) -> Element<'_, Message> {
        let note = match &self.status {
            Status::Applying(Step::Render) => fl!("step-render"),
            Status::Applying(Step::Import) => fl!("step-import"),
            Status::Applying(Step::Wallpaper) => fl!("step-wallpaper"),
            _ if self.is_applied() => {
                fl!("now-in-cosmic", mode = self.mode.name().to_lowercase())
            }
            _ => String::new(),
        };
        widget::column::with_capacity(2)
            .width(Length::Fill)
            .push(self.apply_button(true))
            .push(
                widget::container(widget::text::caption(note))
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .spacing(4)
            .into()
    }

    fn gallery_bar(&self) -> Element<'_, Message> {
        widget::column::with_capacity(2)
            .push(
                widget::search_input(fl!("search"), &self.query)
                    .id(self.search_id.clone())
                    .on_input(Message::Query)
                    .on_clear(Message::Query(String::new()))
                    .width(Length::Fill),
            )
            .push(
                widget::segmented_control::horizontal(&self.categories)
                    .on_activate(Message::Category)
                    .width(Length::Fill),
            )
            .spacing(GAP as u16)
            .into()
    }

    /// The styles as rows of equal cards that fill `w`.
    fn gallery_grid(&self, w: f32) -> Element<'_, Message> {
        let wanted = self
            .categories
            .active_data::<Option<usize>>()
            .copied()
            .flatten();
        let query = self.query.trim().to_lowercase();
        let thumbs = self.shown.as_ref().and_then(|k| self.cache.get(k));
        let cols = (((w + GAP) / (CARD_MIN + GAP)).floor() as usize).max(1);
        let card_w = (w - GAP * (cols as f32 - 1.0)) / cols as f32;

        let mut out = widget::column::with_capacity(8).spacing(GAP as u16);
        let mut shown = 0;
        for (i, (id, items)) in catalog::CATEGORIES.iter().enumerate() {
            if wanted.is_some_and(|c| c != i) {
                continue;
            }
            let hits: Vec<_> = items
                .iter()
                .filter(|(raw, name)| {
                    query.is_empty() || raw.contains(&query) || name.to_lowercase().contains(&query)
                })
                .collect();
            if hits.is_empty() {
                continue;
            }
            if wanted.is_none() {
                out = out.push(widget::text::heading(category_label(id)));
            }
            for chunk in hits.chunks(cols) {
                let mut row = widget::row::with_capacity(cols).spacing(GAP as u16);
                for (raw, name) in chunk {
                    shown += 1;
                    row = row.push(self.style_card(raw, name, card_w, thumbs));
                }
                for _ in chunk.len()..cols {
                    row = row.push(widget::Space::new().width(card_w));
                }
                out = out.push(row);
            }
        }
        if shown == 0 {
            out = out.push(widget::text::body(fl!(
                "no-match",
                query = self.query.clone()
            )));
        }
        out.into()
    }

    fn style_card<'a>(
        &'a self,
        raw: &'static str,
        name: &'static str,
        w: f32,
        thumbs: Option<&'a std::collections::HashMap<&'static str, Handle>>,
    ) -> Element<'a, Message> {
        let h = w * 9.0 / 16.0;
        let picture: Element<_> = match thumbs.and_then(|m| m.get(raw)) {
            Some(handle) => widget::image(handle.clone())
                .width(w)
                .height(h)
                .content_fit(ContentFit::Cover)
                .into(),
            None => widget::container(widget::Space::new().width(w).height(h))
                .class(cosmic::theme::Container::Card)
                .into(),
        };
        let chosen = self.look().style == raw;
        let label: Element<_> = if chosen {
            widget::text::heading(name).into()
        } else {
            widget::text::body(name).into()
        };
        let picture = widget::tooltip(
            widget::button::custom(picture)
                .padding(0)
                .class(cosmic::theme::Button::Image)
                .selected(chosen)
                .width(Length::Fixed(w))
                .on_press(Message::Style(raw)),
            widget::text::body(raw),
            widget::tooltip::Position::Bottom,
        );
        widget::column::with_capacity(2)
            .push(picture)
            .push(label)
            .spacing(4)
            .width(Length::Fixed(w))
            .into()
    }

    /// The drawer: what COSMIC has now in each mode.
    pub fn cosmic_drawer(&self) -> Element<'_, Message> {
        let none = fl!("none");
        let theme_row = |title: String, mode: Mode| {
            let name = self
                .cosmic
                .name(mode)
                .cloned()
                .unwrap_or_else(|| none.clone());
            let mut control = widget::column::with_capacity(2).push(widget::text::body(name));
            if self.cosmic.is_dark == (mode == Mode::Dark) {
                control = control.push(widget::text::caption(format!("● {}", fl!("active"))));
            }
            widget::settings::item::builder(title).control(control.align_x(Alignment::End))
        };
        let section = widget::settings::section()
            .add(theme_row(fl!("dark-theme"), Mode::Dark))
            .add(theme_row(fl!("light-theme"), Mode::Light))
            .add(
                widget::settings::item::builder(fl!("auto-switch"))
                    .description(fl!("auto-switch-desc"))
                    .toggler(self.cosmic.auto_switch, Message::AutoSwitch),
            );
        widget::column::with_capacity(2)
            .push(section)
            .push(
                widget::button::standard(fl!("open-appearance")).on_press(Message::OpenAppearance),
            )
            .spacing(cosmic::theme::spacing().space_m)
            .into()
    }
}
