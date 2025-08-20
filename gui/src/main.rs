use std::fs::write;
use std::path::PathBuf;

use chrono::{NaiveTime, TimeDelta, Timelike};
use fit2srt_core::SrtGenerator;
use iced::widget::{button, column, container, horizontal_space, image, row, scrollable, text};
use iced::widget::{Button, Column, Container};
use iced::{Color, Element, Fill};
use native_dialog::DialogBuilder;

pub fn main() -> iced::Result {
    // #[cfg(target_arch = "wasm32")]
    // {
    //     console_log::init().expect("Initialize logger");
    //     std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    // }

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::application(App::title, App::update, App::view)
        .centered()
        .run()
}

pub struct App {
    screen: Screen,
    debug: bool,
    fitfile: Option<PathBuf>,
    starting_time: NaiveTime,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackPressed,
    NextPressed,
    SelectFile,
    StartingTimeChange(i64),
}

impl App {
    fn title(&self) -> String {
        let screen = match self.screen {
            Screen::Intro => "Introduction",
            Screen::Input => "Setup inputs",
            Screen::End => "End",
        };

        format!("Fit2srt - {screen}")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::BackPressed => {
                if let Some(screen) = self.screen.previous() {
                    self.screen = screen;
                }
            }
            Message::NextPressed => {
                if let Err(_msg) = self.gen_srt() {
                    return;
                }

                if let Some(screen) = self.screen.next() {
                    self.screen = screen;
                }
            }
            Message::SelectFile => {
                // #[cfg(target_arch = "wasm32")]
                // TODO

                #[cfg(not(target_arch = "wasm32"))]
                let path = DialogBuilder::file()
                    .set_location("~/")
                    .add_filter("Fit File", ["fit"])
                    .open_single_file()
                    .show();

                self.fitfile = path.unwrap_or_default();
            }
            Message::StartingTimeChange(t) => {
                self.starting_time += TimeDelta::try_seconds(t).unwrap();
            }
        }
    }

    fn gen_srt(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
        let mut generator = SrtGenerator::default();
        generator.after_second(self.starting_time.num_seconds_from_midnight());
        if let Some(f) = &self.fitfile {
            let mut srt_content = String::new();
            for srt in generator.open(f)? {
                srt_content += &format!("{srt:}\n\n");
            }
            write(self.srt_file().as_ref().unwrap(), srt_content)?;
        }
        Ok(())
    }

    fn srt_file(&self) -> Option<PathBuf> {
        let mut srt_path = self.fitfile.clone();
        if let Some(ref mut p) = srt_path {
            p.set_extension("srt");
        }
        srt_path
    }

    fn view(&self) -> Element<Message> {
        let controls = row![]
            .push_maybe(self.screen.previous().is_some().then(|| {
                padded_button("Back")
                    .on_press(Message::BackPressed)
                    .style(button::secondary)
            }))
            .push(horizontal_space())
            .push_maybe(
                self.can_continue()
                    .then(|| padded_button("Next").on_press(Message::NextPressed)),
            );

        let screen = match self.screen {
            Screen::Intro => self.welcome(),
            Screen::Input => self.inputs(),
            Screen::End => self.end(),
        };

        let content: Element<_> = column![screen, controls,]
            .max_width(540)
            .spacing(20)
            .padding(20)
            .into();

        let scrollable = scrollable(
            container(if self.debug {
                content.explain(Color::BLACK)
            } else {
                content
            })
            .center_x(Fill),
        );

        container(scrollable).center_y(Fill).into()
    }

    fn can_continue(&self) -> bool {
        match self.screen {
            Screen::Intro => true,
            Screen::Input => self.fitfile.is_some(),
            Screen::End => false,
        }
    }

    fn welcome(&self) -> Column<Message> {
        Self::container("Welcome!")
            .push("This is a simple tool for you to make your diving log as video subtitles.")
            .push("You can see some sample video here:")
            // TODO: https://github.com/squidowl/halloy/blob/main/src/widget/selectable_rich_text.rs
            .push("https://www.youtube.com/@yanganto/videos")
            .push("If you want to any scuba diving crouse, please contact with me.")
            .push("yanganto@gmail.com")
    }

    fn inputs(&self) -> Column<Message> {
        Self::container("Setup inputs")
            .push(text(if let Some(f) = &self.fitfile {
                format!("1. Fit file loaded: {}", f.display())
            } else {
                "1. Select the fit file from you diving computer.".to_string()
            }))
            .push(padded_button("Open").on_press(Message::SelectFile))
            .push("2. Setup the starting time of the video")
            .push(
                row![
                    button("+").on_press(Message::StartingTimeChange(3600)),
                    button("+").on_press(Message::StartingTimeChange(60)),
                    button("+").on_press(Message::StartingTimeChange(1)),
                ]
                .spacing(10),
            )
            .push(text(self.starting_time.format("%H:%M:%S").to_string()).size(24))
            .push(
                row![
                    button("-").on_press(Message::StartingTimeChange(-3600)),
                    button("-").on_press(Message::StartingTimeChange(-60)),
                    button("-").on_press(Message::StartingTimeChange(-1)),
                ]
                .spacing(10),
            )
    }

    fn end(&self) -> Column<Message> {
        Self::container("All Done!")
            .push(text(format!("The .srt file is created: {}", self.srt_file().unwrap().display())))
            .push("You can upload .srt to youtube or use it in video editor.")
            .push("If you like this project, please buy me a coffee via paypal or bitcoin to support me.")
            .push(paypal_donate(300, image::FilterMethod::Linear))
            .push("BTC address")
            .push(btc_donate(250, image::FilterMethod::Linear))
    }

    fn container(title: &str) -> Column<'_, Message> {
        column![text(title).size(50)].spacing(20)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Intro,
    Input,
    End,
}

impl Screen {
    const ALL: &'static [Self] = &[Self::Intro, Self::Input, Self::End];

    pub fn next(self) -> Option<Screen> {
        Self::ALL
            .get(
                Self::ALL
                    .iter()
                    .copied()
                    .position(|screen| screen == self)
                    .expect("Screen must exist")
                    + 1,
            )
            .copied()
    }

    pub fn previous(self) -> Option<Screen> {
        let position = Self::ALL
            .iter()
            .copied()
            .position(|screen| screen == self)
            .expect("Screen must exist");

        if position > 0 {
            Some(Self::ALL[position - 1])
        } else {
            None
        }
    }
}

fn paypal_donate<'a>(width: u16, filter_method: image::FilterMethod) -> Container<'a, Message> {
    container(
        // This should go away once we unify resource loading on native
        // platforms
        if cfg!(target_arch = "wasm32") {
            image("/paypal-qrcode.png")
        } else {
            image(format!(
                "{}/../assets/paypal-qrcode.png",
                env!("CARGO_MANIFEST_DIR")
            ))
        }
        .filter_method(filter_method)
        .width(width),
    )
    .center_x(Fill)
}

fn btc_donate<'a>(width: u16, filter_method: image::FilterMethod) -> Container<'a, Message> {
    container(
        // This should go away once we unify resource loading on native
        // platforms
        if cfg!(target_arch = "wasm32") {
            image("/btc-qrcode.jpg")
        } else {
            image(format!(
                "{}/../assets/btc-qrcode.jpg",
                env!("CARGO_MANIFEST_DIR")
            ))
        }
        .filter_method(filter_method)
        .width(width),
    )
    .center_x(Fill)
}

fn padded_button<Message: Clone>(label: &str) -> Button<'_, Message> {
    button(text(label)).padding([12, 24])
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Intro,
            debug: false,
            fitfile: None,
            starting_time: NaiveTime::default(),
        }
    }
}
