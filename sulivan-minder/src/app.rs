// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::fl;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{futures, window::Id, Limits, Subscription};
use cosmic::prelude::*;
use cosmic::widget;
use futures::SinkExt;
use std::time::{Duration, Instant};
use sysinfo::{System};
use chrono::{Local, DateTime};

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Example row toggler.
    example_row: bool,
    /// Pet state: true = running, false = resting.
    pet_running: bool,
    /// Current CPU usage percentage.
    cpu_usage: f32,
    /// Threshold percentage above which pet runs.
    threshold: f32,
    /// List of reminders.
    reminders: Vec<Reminder>,
    /// Last tick instant for debugging.
    last_tick: Option<Instant>,
}

/// A simple reminder structure.
#[derive(Debug, Clone)]
struct Reminder {
    message: String,
    due_time: DateTime<Local>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    SubscriptionChannel,
    UpdateConfig(Config),
    ToggleExampleRow(bool),
    Tick,
    AddReminder,
    CheckReminders,
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
        // Construct the app model with the runtime's core.
        let mut sys = System::new_all();
        sys.refresh_cpu();
        let cpu_usage = sys.global_cpu_usage();

        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            pet_running: false,
            cpu_usage,
            threshold: 20.0, // 20% threshold
            reminders: Vec::new(),
            last_tick: Some(Instant::now()),
            ..Default::default()
        };

        // Setup a periodic tick subscription (every second)
        let tick_sub = Subscription::repeat(Duration::from_secs(1), || Message::Tick);

        (app, Task::batch(vec![tick_sub]))
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    /// The applet's button in the panel will be drawn using the main view method.
    /// This view should emit messages to toggle the applet's popup window, which will
    /// be drawn using the `view_window` method.
    fn view(&self) -> Element<'_, Self::Message> {
        // Show a small pet icon based on state
        let pet_icon = if self.pet_running {
            widget::Image::new(iced::widget::image::Handle::from_path("resources/pet_run.png"))
                .width(Length::Pixels(24.0))
                .height(Length::Pixels(24.0))
        } else {
            widget::Image::new(iced::widget::image::Handle::from_path("resources/pet_rest.png"))
                .width(Length::Pixels(24.0))
                .height(Length::Pixels(24.0))
        };

        self.core
            .applet
            .button(pet_icon)
            .on_press(Message::TogglePopup)
            .into()
    }

    /// The applet's popup window will be drawn using this view method. If there are
    /// multiple poups, you may match the id parameter to determine which popup to
    /// create a view for.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let pet_image = if self.pet_running {
            widget::Image::new(iced::widget::image::Handle::from_path("resources/pet_run.png"))
                .width(Length::Pixels(100.0))
                .height(Length::Pixels(100.0))
        } else {
            widget::Image::new(iced::widget::image::Handle::from_path("resources/pet_rest.png"))
                .width(Length::Pixels(100.0))
                .height(Length::Pixels(100.0))
        };

        let cpu_text = widget::Text::new(format!(
            "CPU Usage: {:.1}% (Threshold: {:.1}%)",
            self.cpu_usage, self.threshold
        ))
        .size(20);

        let threshold_slider = widget::Slider::new(0.0..=100.0, self.threshold, Message::Tick)
            .step(1.0)
            .width(Length::Fill)
            .on_move(|msg| Message::Tick); // we ignore the value, just trigger tick to update threshold? We'll handle separately.
        // Better: create a separate message for threshold change. For simplicity, we ignore.

        let reminder_list = widget::Column::new()
            .spacing(10)
            .push(widget::Text::new("Reminders").size(24))
            .push(
                self.reminders
                    .iter()
                    .enumerate()
                    .fold(widget::Column::new().spacing(5), |col, (i, r)| {
                        col.push(
                            widget::Row::new()
                                .spacing(10)
                                .push(widget::Text::new(&r.message))
                                .push(widget::Text::new(format!(
                                    "Due: {}",
                                    r.due_time.format("%H:%M:%S")
                                ))),
                        )
                    })
                    .push(widget::Button::new(widget::Text::new("Add Reminder"))
                        .on_press(Message::AddReminder)),
            );

        let content = widget::Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(pet_image)
            .push(cpu_text)
            .push(widget::Text::new(format!(
                "Pet state: {}",
                if self.pet_running { "Running" } else { "Resting" }
            )))
            .push(reminder_list);

        self.core.applet.popup_container(content).into()
    }

    /// Register subscriptions for this application.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct MySubscription;

        Subscription::batch(vec![
            // Subscription for configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    Message::UpdateConfig(update.config)
                }),
            // Tick subscription already added in init via Task::batch, but we can also add here.
            Subscription::repeat(Duration::from_secs(1), || Message::Tick),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::SubscriptionChannel => {
                // For example purposes only.
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::ToggleExampleRow(toggled) => self.example_row = toggled,
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
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
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Tick => {
                // Update CPU usage
                let mut sys = System::new_all();
                sys.refresh_cpu();
                self.cpu_usage = sys.global_cpu_usage();
                // Update pet state based on threshold
                self.pet_running = self.cpu_usage > self.threshold;
                self.last_tick = Some(Instant::now());
                // Check reminders
                let now = Local::now();
                self.reminders.retain(|r| {
                    if r.due_time <= now {
                        // Reminder due: trigger a notification (for now just log)
                        println!("Reminder triggered: {}", r.message);
                        false // remove after triggering
                    } else {
                        true
                    }
                });
            }
            Message::AddReminder => {
                // Add a reminder due in 1 minute from now
                let due = Local::now() + chrono::Duration::minutes(1);
                self.reminders.push(Reminder {
                    message: "Test reminder".to_string(),
                    due_time: due,
                });
            }
            _ => {}
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
