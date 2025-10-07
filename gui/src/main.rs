use std::fs::write;
use std::path::PathBuf;

use chrono::{NaiveTime, TimeDelta, Timelike};
use fit2srt_core::SrtGenerator;
// use iced::widget::qr_code::{Data, QRCode};
use iced::widget::{
    button, column, container, horizontal_space, image, rich_text, row, scrollable, span, text,
};
use iced::widget::{Button, Column};
use iced::{color, font::Weight, Color, Element, Fill, Font};
use native_dialog::DialogBuilder;

// static BTC_ADDR: &[u8; 34] = b"3QQ6vmEvjznxqSub4hCQRymicT2kKCcLzd";
// static PAYPAL_ADDR: &[u8; 48] = b"https://www.paypal.com/ncp/payment/EH3BJ4MSTFQN4";

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
    // btc_qr_data: Data,
    // paypal_qr_data: Data,
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
            Screen::CryptoDonate => "CryptoDonate",
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
            for (_, _, srt) in generator.open(f)? {
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
            Screen::CryptoDonate => self.crypto_donate(),
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
            Screen::End => true,
            Screen::CryptoDonate => false,
        }
    }

    fn welcome(&self) -> Column<Message> {
        Self::container("Welcome!")
            .push(column![
                text("This is a simple tool for you to make your diving log as video subtitles."),
                text("You can see some sample video here:"),
                rich_text![span("https://www.youtube.com/@yanganto/videos").color(color!(0x0000FF))]
            ])
            // TODO: https://github.com/squidowl/halloy/blob/main/src/widget/selectable_rich_text.rs
            .push(column![
                text("If you want to any scuba diving crouse, please contact with me."),
                rich_text![span("yanganto@gmail.com").color(color!(0x0000FF))]
            ])
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
        .push(
            rich_text![span("PayPal").font(Font { weight: Weight::Bold, ..Font::default() })]
        )
        .push(
            // container(qr_img(&self.paypal_qr_data))
            container(image(format!("{}/../assets/paypal-qrcode.png", env!("CARGO_MANIFEST_DIR"))))
            .center_x(Fill)
        )
    }
    fn crypto_donate(&self) -> Column<Message> {
        Self::container("Help us")
            .push("If you want to donate with crypto.")
            .push("Please help us with Bitcoin.")
            .push(rich_text![span("BTC").font(Font {
                weight: Weight::Bold,
                ..Font::default()
            })])
            .push(
                // container(qr_img(&self.btc_qr_data)).center(600)
                container(image(format!(
                    "{}/../assets/btc-qrcode.jpg",
                    env!("CARGO_MANIFEST_DIR")
                )))
                .center_x(Fill),
            )
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
    CryptoDonate,
}

impl Screen {
    const ALL: &'static [Self] = &[Self::Intro, Self::Input, Self::End, Self::CryptoDonate];

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

// TODO Fix QRCode generating bug
// fn qr_img(data: &Data) -> Element<'_, Message> {
//     QRCode::new(data).cell_size(10).into()
// }

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
            // btc_qr_data: Data::new(BTC_ADDR).unwrap(),
            // paypal_qr_data: Data::new(PAYPAL_ADDR).unwrap(),
        }
    }
}
