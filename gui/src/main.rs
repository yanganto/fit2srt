use iced::widget::{
    button, checkbox, column, container, horizontal_space, image, radio, row, scrollable, slider,
    text, text_input, toggler, vertical_space,
};
use iced::widget::{Button, Column, Container, Slider};
use iced::{Center, Color, Element, Fill, Font, Pixels};

pub fn main() -> iced::Result {
    // #[cfg(target_arch = "wasm32")]
    // {
    //     console_log::init().expect("Initialize logger");
    //     std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    // }

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::application(Tour::title, Tour::update, Tour::view)
        .centered()
        .run()
}

pub struct Tour {
    screen: Screen,
    slider: u8,
    spacing: u16,
    text_size: u16,
    text_color: Color,
    toggler: bool,
    image_width: u16,
    image_filter_method: image::FilterMethod,
    input_value: String,
    input_is_secure: bool,
    input_is_showing_icon: bool,
    debug: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackPressed,
    NextPressed,
    InputChanged(String),
}

impl Tour {
    fn title(&self) -> String {
        let screen = match self.screen {
            Screen::Intro => "Introduction",
            Screen::TextInput => "Text input",
            Screen::End => "End",
        };

        format!("Fit2srt - {}", screen)
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::BackPressed => {
                if let Some(screen) = self.screen.previous() {
                    self.screen = screen;
                }
            }
            Message::NextPressed => {
                if let Some(screen) = self.screen.next() {
                    self.screen = screen;
                }
            }
            Message::InputChanged(input_value) => {
                self.input_value = input_value;
            }
        }
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
            Screen::TextInput => self.text_input(),
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
            Screen::TextInput => !self.input_value.is_empty(),
            Screen::End => false,
        }
    }

    fn welcome(&self) -> Column<Message> {
        Self::container("Welcome!")
            .push("This is a simple tool for you to make your diving log as video subtitles.")
    }

    fn text_input(&self) -> Column<Message> {
        let value = &self.input_value;
        let is_secure = self.input_is_secure;
        let is_showing_icon = self.input_is_showing_icon;

        let mut text_input = text_input("Type something to continue...", value)
            .on_input(Message::InputChanged)
            .padding(10)
            .size(30);

        if is_showing_icon {
            text_input = text_input.icon(text_input::Icon {
                font: Font::default(),
                code_point: '🚀',
                size: Some(Pixels(28.0)),
                spacing: 10.0,
                side: text_input::Side::Right,
            });
        }

        Self::container("Text input")
            .push("Use a text input to ask for different kinds of information.")
            .push(text_input.secure(is_secure))
            .push(
                "A text input produces a message every time it changes. It is \
                 very easy to keep track of its contents:",
            )
            .push(
                text(if value.is_empty() {
                    "You have not typed anything yet..."
                } else {
                    value
                })
                .width(Fill)
                .align_x(Center),
            )
    }

    fn end(&self) -> Column<Message> {
        Self::container("All Done!")
            .push("The .srt file is created in the same folder for .fit file.")
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
    TextInput,
    End,
}

impl Screen {
    const ALL: &'static [Self] = &[Self::Intro, Self::TextInput, Self::End];

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
                "{}/assets/paypal-qrcode.png",
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
                "{}/assets/btc-qrcode.jpg",
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

impl Default for Tour {
    fn default() -> Self {
        Self {
            screen: Screen::Intro,
            slider: 50,
            spacing: 20,
            text_size: 30,
            text_color: Color::BLACK,
            toggler: false,
            image_width: 300,
            image_filter_method: image::FilterMethod::Linear,
            input_value: String::new(),
            input_is_secure: false,
            input_is_showing_icon: false,
            debug: false,
        }
    }
}
