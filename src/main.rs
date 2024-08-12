use std::sync::{Arc, Mutex};
use std::time::Duration;
use clipboard::{ClipboardContext, ClipboardProvider};
use chrono::Local;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{executor, Command, Element, Length, Subscription, Application, Settings, Theme};

#[derive(Clone, Debug)]
struct ClipboardEntry {
    content: String,
    timestamp: chrono::DateTime<Local>,
}

struct ClipboardManager {
    history: Arc<Mutex<Vec<ClipboardEntry>>>,
    max_entries: usize,
}

impl ClipboardManager {
    fn new(max_entries: usize) -> Self {
        ClipboardManager {
            history: Arc::new(Mutex::new(Vec::new())),
            max_entries,
        }
    }

    fn start_monitoring(&self) {
        let history = Arc::clone(&self.history);
        let max_entries = self.max_entries;

        std::thread::spawn(move || {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            let mut last_content = String::new();

            loop {
                if let Ok(content) = ctx.get_contents() {
                    if !content.is_empty() && content != last_content {
                        let mut history = history.lock().unwrap();
                        history.push(ClipboardEntry {
                            content: content.clone(),
                            timestamp: Local::now(),
                        });
                        if history.len() > max_entries {
                            history.remove(0);
                        }
                        last_content = content;
                    }
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        });
    }

    fn get_history(&self) -> Vec<ClipboardEntry> {
        self.history.lock().unwrap().clone()
    }

    fn copy_to_clipboard(&self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        let history = self.history.lock().unwrap();
        if let Some(entry) = history.get(index) {
            let mut ctx: ClipboardContext = ClipboardProvider::new()?;
            ctx.set_contents(entry.content.clone())?;
        }
        Ok(())
    }
}

struct ClipboardHistoryApp {
    clipboard_manager: ClipboardManager,
    entries: Vec<ClipboardEntry>,
}

#[derive(Debug, Clone)]
enum Message {
    Refresh,
    CopyToClipboard(usize),
    Tick,
}

impl Application for ClipboardHistoryApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let clipboard_manager = ClipboardManager::new(100);
        clipboard_manager.start_monitoring();

        let app = ClipboardHistoryApp {
            clipboard_manager,
            entries: Vec::new(),
        };

        (app, Command::perform(async {}, |_| Message::Refresh))
    }

    fn title(&self) -> String {
        String::from("Clipboard History Manager")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Refresh | Message::Tick => {
                self.entries = self.clipboard_manager.get_history();
                Command::none()
            }
            Message::CopyToClipboard(index) => {
                if let Err(e) = self.clipboard_manager.copy_to_clipboard(index) {
                    eprintln!("Error copying to clipboard: {}", e);
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = scrollable(
            column(
                self.entries
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| {
                        row![
                            text(&entry.content).width(Length::FillPortion(3)),
                            text(&entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
                                .width(Length::FillPortion(1)),
                            button("Copy")
                                .on_press(Message::CopyToClipboard(i))
                                .width(Length::FillPortion(1))
                        ]
                        .spacing(10)
                        .padding(5)
                        .into()
                    })
                    .collect(),
            )
            .spacing(5),
        );

        let refresh_button = button("Refresh").on_press(Message::Refresh);

        container(column![refresh_button, content].spacing(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(5)).map(|_| Message::Tick)
    }
}

fn main() -> iced::Result {
    ClipboardHistoryApp::run(Settings::default())
}