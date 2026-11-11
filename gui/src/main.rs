// SPDX-License-Identifier: MIT

mod app;
mod catalog;
mod config;
mod i18n;
mod ui;

fn main() -> cosmic::iced::Result {
    i18n::init(&i18n_embed::DesktopLanguageRequester::requested_languages());

    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(1100.0, 720.0))
        .size_limits(
            cosmic::iced::Limits::NONE
                .min_width(480.0)
                .min_height(480.0),
        );
    cosmic::app::run::<app::AppModel>(settings, ())
}
