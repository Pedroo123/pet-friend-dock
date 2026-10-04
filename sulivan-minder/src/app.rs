// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::fl;
use crate::pet::{Direction, FRAME_COUNT, Pet};
use crate::tracker::cpu_tracker::{self, CpuTracker};
use chrono::{DateTime, Local, TimeZone};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{Alignment, Length, Limits, Padding, Subscription, window::Id};
use cosmic::prelude::*;
use cosmic::widget;
use std::time::Duration;

/// Dog sprites facing left, in walk-cycle order.
const SPRITES: [&[u8]; FRAME_COUNT] = [
    include_bytes!("assets/goldenDefault_0002(3).png"),
    include_bytes!("assets/goldenDefault_0003(2).png"),
    include_bytes!("assets/goldenDefault_0004(1).png"),
    include_bytes!("assets/goldenDefault_0005.png"),
];

/// How many sprite widths long the pet's walking track on the dock is.
const TRACK_SPRITES: f32 = 5.0;

const CPU_POLL_INTERVAL: Duration = Duration::from_secs(1);
const ANIMATION_INTERVAL: Duration = Duration::from_millis(100);
const REMINDER_INTERVAL: Duration = Duration::from_secs(1);
const DEFAULT_REMINDER_MINUTES: i64 = 5;

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Handle used to persist the configuration.
    config_handler: Option<cosmic_config::Config>,
    /// Samples the CPU load.
    tracker: CpuTracker,
    /// Whether a CPU sample is currently in flight.
    sampling: bool,
    /// Latest CPU usage percentage.
    cpu_usage: f32,
    /// True while the CPU usage is above the configured threshold.
    pet_running: bool,
    /// Position and animation state of the pet.
    pet: Pet,
    /// Reminders whose time has passed and that were not dismissed yet.
    due: Vec<Reminder>,
    /// Text of the reminder being typed.
    new_message: String,
    /// Minutes of the reminder being typed.
    new_minutes: String,
    /// Sprites facing left (the direction they were drawn in).
    left_handles: Vec<widget::image::Handle>,
    /// Horizontally mirrored sprites, used when walking right.
    right_handles: Vec<widget::image::Handle>,
}

/// Loads the sprite as a handle, optionally mirrored horizontally.
fn sprite_handle(bytes: &[u8], mirrored: bool) -> widget::image::Handle {
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let img = if mirrored { img.fliph() } else { img };
            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            widget::image::Handle::from_rgba(width, height, rgba.into_raw())
        }
        Err(why) => {
            eprintln!("failed to decode sprite: {why}");
            widget::image::Handle::from_bytes(bytes.to_vec())
        }
    }
}

/// A saved reminder.
#[derive(Debug, Clone, PartialEq)]
struct Reminder {
    message: String,
    due_time: DateTime<Local>,
}

impl Reminder {
    fn from_config(&(timestamp, ref message): &(i64, String)) -> Option<Self> {
        Some(Self {
            message: message.clone(),
            due_time: Local.timestamp_opt(timestamp, 0).single()?,
        })
    }

    fn to_config(&self) -> (i64, String) {
        (self.due_time.timestamp(), self.message.clone())
    }
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    UpdateConfig(Config),
    /// Time to sample the CPU.
    PollCpu,
    /// A CPU sample finished.
    CpuSampled(f32),
    /// Advance the sprite animation.
    Animate,
    /// Look for reminders that became due.
    CheckReminders,
    ReminderTextChanged(String),
    ReminderMinutesChanged(String),
    AddReminder,
    RemoveReminder(usize),
    DismissDue(usize),
}

impl AppModel {
    fn reminders(&self) -> Vec<Reminder> {
        self.config
            .reminders
            .iter()
            .filter_map(Reminder::from_config)
            .collect()
    }

    fn save_reminders(&mut self, reminders: Vec<(i64, String)>) {
        match &self.config_handler {
            Some(handler) => {
                if let Err(why) = self.config.set_reminders(handler, reminders) {
                    eprintln!("failed to save reminders: {why}");
                }
            }
            None => self.config.reminders = reminders,
        }
    }

    fn sprite_size(&self) -> f32 {
        f32::from(self.core.applet.suggested_size(true).0)
    }

    fn track_length(&self) -> f32 {
        self.sprite_size() * TRACK_SPRITES
    }

    fn open_popup(&mut self) -> Task<cosmic::Action<Message>> {
        let new_id = Id::unique();
        self.popup.replace(new_id);
        let mut popup_settings = self.core.applet.get_popup_settings(
            self.core.main_window_id().unwrap(),
            new_id,
            None,
            None,
            None,
        );
        popup_settings.positioner.size_limits = Limits::NONE
            .max_width(372.0)
            .min_width(300.0)
            .min_height(200.0)
            .max_height(1080.0);
        get_popup(popup_settings)
    }

    fn sprite(&self, size: f32) -> Element<'_, Message> {
        let handles = match self.pet.direction {
            Direction::Forward => &self.right_handles,
            Direction::Backward => &self.left_handles,
        };
        widget::image(handles[self.pet.frame % FRAME_COUNT].clone())
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into()
    }
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.pedroo123.sulivanMinder";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config_handler = cosmic_config::Config::new(Self::APP_ID, Config::VERSION).ok();
        let config = config_handler
            .as_ref()
            .map(|context| match Config::get_entry(context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        let app = AppModel {
            core,
            popup: None,
            config,
            config_handler,
            tracker: CpuTracker::default(),
            sampling: false,
            cpu_usage: 0.0,
            pet_running: false,
            pet: Pet::default(),
            due: Vec::new(),
            new_message: String::new(),
            new_minutes: DEFAULT_REMINDER_MINUTES.to_string(),
            left_handles: SPRITES.iter().map(|b| sprite_handle(b, false)).collect(),
            right_handles: SPRITES.iter().map(|b| sprite_handle(b, true)).collect(),
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// The applet's button in the panel shows the animated pet.
    fn view(&self) -> Element<'_, Self::Message> {
        let size = self.sprite_size();
        let button = widget::button::custom(self.sprite(size))
            .padding(0)
            .class(cosmic::theme::Button::AppletIcon)
            .on_press(Message::TogglePopup);

        // The pet walks along a track on the dock; its offset along the panel's main axis
        // is the pet's current position.
        let offset = self.pet.position;
        let track = self.track_length();
        let track_view: Element<'_, Message> = if self.core.applet.is_horizontal() {
            widget::container(button)
                .padding(Padding::ZERO.left(offset))
                .width(Length::Fixed(track))
                .height(Length::Fixed(size))
                .into()
        } else {
            widget::container(button)
                .padding(Padding::ZERO.top(offset))
                .width(Length::Fixed(size))
                .height(Length::Fixed(track))
                .into()
        };
        self.core.applet.autosize_window(track_view).into()
    }

    /// The applet's popup: CPU status, due reminders and reminder management.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let state = if self.pet_running {
            fl!("pet-running")
        } else {
            fl!("pet-resting")
        };

        let mut content = widget::Column::new()
            .spacing(12)
            .padding(12)
            .align_x(Alignment::Center)
            .push(self.sprite(100.0))
            .push(widget::text::body(state))
            .push(widget::text::body(fl!(
                "cpu-usage",
                usage = format!("{:.1}", self.cpu_usage),
                threshold = format!("{:.0}", self.config.cpu_threshold)
            )));

        if !self.due.is_empty() {
            let mut due = widget::Column::new()
                .spacing(6)
                .push(widget::text::heading(fl!("due-reminders")));
            for (i, reminder) in self.due.iter().enumerate() {
                due = due.push(
                    widget::Row::new()
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .push(widget::text::body(reminder.message.clone()).width(Length::Fill))
                        .push(
                            widget::button::standard(fl!("dismiss"))
                                .on_press(Message::DismissDue(i)),
                        ),
                );
            }
            content = content.push(due);
        }

        let mut list = widget::Column::new()
            .spacing(6)
            .push(widget::text::heading(fl!("reminders")));
        let reminders = self.reminders();
        if reminders.is_empty() {
            list = list.push(widget::text::caption(fl!("no-reminders")));
        }
        for (i, reminder) in reminders.iter().enumerate() {
            list = list.push(
                widget::Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        widget::text::body(format!(
                            "{} – {}",
                            reminder.due_time.format("%H:%M"),
                            reminder.message
                        ))
                        .width(Length::Fill),
                    )
                    .push(
                        widget::button::icon(widget::icon::from_name("edit-delete-symbolic"))
                            .on_press(Message::RemoveReminder(i)),
                    ),
            );
        }
        content = content.push(list);

        let form = widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(
                widget::text_input(fl!("reminder-placeholder"), &self.new_message)
                    .on_input(Message::ReminderTextChanged)
                    .on_submit(|_| Message::AddReminder)
                    .width(Length::Fill),
            )
            .push(
                widget::text_input(fl!("minutes-placeholder"), &self.new_minutes)
                    .on_input(Message::ReminderMinutesChanged)
                    .on_submit(|_| Message::AddReminder)
                    .width(Length::Fixed(80.0)),
            );
        content = content
            .push(form)
            .push(widget::button::suggested(fl!("add-reminder")).on_press(Message::AddReminder));

        self.core.applet.popup_container(content).into()
    }

    /// Register subscriptions for this application.
    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
            cosmic::iced::time::every(CPU_POLL_INTERVAL).map(|_| Message::PollCpu),
            cosmic::iced::time::every(REMINDER_INTERVAL).map(|_| Message::CheckReminders),
        ];
        subscriptions.push(cosmic::iced::time::every(ANIMATION_INTERVAL).map(|_| Message::Animate));
        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::UpdateConfig(config) => {
                self.config = config;
                self.pet_running =
                    cpu_tracker::is_over_threshold(self.cpu_usage, self.config.cpu_threshold);
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    self.open_popup()
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::PollCpu => {
                if self.sampling {
                    return Task::none();
                }
                self.sampling = true;
                let tracker = self.tracker.clone();
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || tracker.sample())
                            .await
                            .unwrap_or(0.0)
                    },
                    |usage| cosmic::Action::App(Message::CpuSampled(usage)),
                );
            }
            Message::CpuSampled(usage) => {
                self.sampling = false;
                self.cpu_usage = usage;
                self.pet_running = cpu_tracker::is_over_threshold(usage, self.config.cpu_threshold);
            }
            Message::Animate => {
                let (track, size) = (self.track_length(), self.sprite_size());
                self.pet.tick(
                    ANIMATION_INTERVAL.as_secs_f32(),
                    self.pet_running,
                    track,
                    size,
                );
            }
            Message::CheckReminders => {
                let now = Local::now();
                let (due, pending): (Vec<_>, Vec<_>) = self
                    .reminders()
                    .into_iter()
                    .partition(|r| r.due_time <= now);
                if due.is_empty() {
                    return Task::none();
                }
                let pending = pending.iter().map(Reminder::to_config).collect();
                self.save_reminders(pending);
                self.due.extend(due);
                if self.popup.is_none() {
                    return self.open_popup();
                }
            }
            Message::ReminderTextChanged(text) => self.new_message = text,
            Message::ReminderMinutesChanged(text) => self.new_minutes = text,
            Message::AddReminder => {
                let message = self.new_message.trim().to_string();
                if message.is_empty() {
                    return Task::none();
                }
                let minutes = self
                    .new_minutes
                    .trim()
                    .parse::<i64>()
                    .ok()
                    .filter(|m| *m >= 0)
                    .unwrap_or(DEFAULT_REMINDER_MINUTES);
                let due = Local::now() + chrono::Duration::minutes(minutes);
                let mut reminders = self.config.reminders.clone();
                reminders.push((due.timestamp(), message));
                reminders.sort_by_key(|(timestamp, _)| *timestamp);
                self.save_reminders(reminders);
                self.new_message.clear();
            }
            Message::RemoveReminder(index) => {
                let mut reminders = self.config.reminders.clone();
                if index < reminders.len() {
                    reminders.remove(index);
                    self.save_reminders(reminders);
                }
            }
            Message::DismissDue(index) => {
                if index < self.due.len() {
                    self.due.remove(index);
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
