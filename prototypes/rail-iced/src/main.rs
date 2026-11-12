//! ADR-0002 prototype: both Bayleaf rails as layer-shell surfaces in iced.

use std::cell::Cell;
use std::time::{Duration, Instant};

use iced::widget::{Space, button, column, container, text, vertical_slider};
use iced::window::Id;
use iced::{Alignment, Border, Color, Element, Length, Task};
use iced_layershell::daemon;
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;

const RAIL_WIDTH: u32 = 56;
const BG: Color = Color::from_rgb(0.118, 0.129, 0.118);
const HOVER: Color = Color::from_rgb(0.184, 0.227, 0.184);
const FG: Color = Color::from_rgb(0.902, 0.902, 0.902);
const ACCENT: Color = Color::from_rgb(0.561, 0.769, 0.561);

fn main() -> Result<(), iced_layershell::Error> {
    let started = Instant::now();
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(3));
        println!("rss_kb={}", rss_kb().unwrap_or(0));
    });

    daemon(move || Rails::new(started), namespace, Rails::update, Rails::view)
        .subscription(Rails::subscription)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                size: Some((RAIL_WIDTH, 0)),
                exclusive_zone: RAIL_WIDTH as i32,
                anchor: Anchor::Left | Anchor::Top | Anchor::Bottom,
                layer: Layer::Top,
                keyboard_interactivity: KeyboardInteractivity::None,
                start_mode: StartMode::Active,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}

fn namespace() -> String {
    "bayleaf-rail".into()
}

struct Rails {
    right: Option<Id>,
    /// Button flashed after a press. iced derives Pressed/Hovered from the cursor position,
    /// which touch input does not move, so touch presses need their own feedback.
    flash: Option<u8>,
    volume: f32,
    clock: String,
    battery: String,
    started: Instant,
    reported: Cell<bool>,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Message {
    Tick,
    Volume(f32),
    Pressed(u8),
    Unflash(u8),
}

impl Rails {
    fn new(started: Instant) -> (Self, Task<Message>) {
        let right = Id::unique();
        let open_right = Task::done(Message::NewLayerShell {
            settings: NewLayerShellSettings {
                size: Some((RAIL_WIDTH, 0)),
                exclusive_zone: Some(RAIL_WIDTH as i32),
                anchor: Anchor::Right | Anchor::Top | Anchor::Bottom,
                layer: Layer::Top,
                keyboard_interactivity: KeyboardInteractivity::None,
                output_option: OutputOption::Active,
                namespace: Some(namespace()),
                ..Default::default()
            },
            id: right,
        });
        (
            Self {
                right: Some(right),
                flash: None,
                volume: 50.0,
                clock: now_hhmm(),
                battery: battery_text(),
                started,
                reported: Cell::new(false),
            },
            open_right,
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_secs(30)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.clock = now_hhmm();
                self.battery = battery_text();
            }
            Message::Volume(v) => self.volume = v,
            Message::Pressed(n) => {
                self.flash = Some(n);
                return Task::perform(
                    async { smol::Timer::after(Duration::from_millis(150)).await },
                    move |_| Message::Unflash(n),
                );
            }
            Message::Unflash(n) => {
                if self.flash == Some(n) {
                    self.flash = None;
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self, id: Id) -> Element<'_, Message> {
        if !self.reported.replace(true) {
            println!("startup_ms={}", self.started.elapsed().as_millis());
        }
        let content = if Some(id) == self.right {
            self.right_rail()
        } else {
            self.left_rail()
        };
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(BG.into()),
                text_color: Some(FG),
                ..Default::default()
            })
            .into()
    }

    fn right_rail(&self) -> Element<'_, Message> {
        column![
            text(&self.clock).size(12),
            text(&self.battery).size(12),
            text("♪").size(18),
            vertical_slider(0.0..=100.0, self.volume, Message::Volume).height(160),
            Space::new().height(Length::Fill),
            self.rail_button("⌨", 3, false),
            self.rail_button("☰", 4, false),
        ]
        .spacing(10)
        .padding([12, 0])
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    }

    fn left_rail(&self) -> Element<'_, Message> {
        column![
            self.rail_button("◉", 1, false),
            self.rail_button("▤", 2, false),
            self.rail_button(">_", 5, false),
            self.rail_button("▶", 6, false),
            Space::new().height(Length::Fill),
            self.rail_button("⊞", 0, true),
        ]
        .spacing(8)
        .padding([12, 0])
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    }

    fn rail_button<'a>(&self, label: &'a str, n: u8, accent: bool) -> Element<'a, Message> {
        let flashed = self.flash == Some(n);
        button(text(label).size(28).center())
            .width(48)
            .height(48)
            .on_press(Message::Pressed(n))
            .style(move |_, status| rail_button_style(status, flashed, accent))
            .into()
    }
}

fn rail_button_style(status: button::Status, flashed: bool, accent: bool) -> button::Style {
    button::Style {
            background: match status {
                _ if flashed => Some(HOVER.into()),
                button::Status::Hovered | button::Status::Pressed => Some(HOVER.into()),
                _ => None,
            },
        text_color: if accent { ACCENT } else { FG },
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn now_hhmm() -> String {
    std::process::Command::new("date")
        .arg("+%H:%M")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn battery_text() -> String {
    std::fs::read_dir("/sys/class/power_supply")
        .ok()
        .and_then(|dir| {
            dir.flatten()
                .find(|e| e.file_name().to_string_lossy().starts_with("BAT"))
        })
        .and_then(|e| std::fs::read_to_string(e.path().join("capacity")).ok())
        .map(|c| format!("{}%", c.trim()))
        .unwrap_or_else(|| "—".into())
}

fn rss_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find(|l| l.starts_with("VmRSS:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}
