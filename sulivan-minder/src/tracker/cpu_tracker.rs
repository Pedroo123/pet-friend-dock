// SPDX-License-Identifier: MPL-2.0

use create::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{futures, window::Id, Limits, Subscription};
use cosmic::prelude::*;
use cosmic::widget;
use std::time::{Duration, Instant};
use sysinfo::{System};
use chrono::{Local, DateTime};

//Model for the internal state of the application
#[derive(Default)]
pub struct AppState {
    cpu_usage: f32,
    pet_running: bool,
    reminders: Vec<Reminder>,
    threshold: f32,
}

// Struct to hold the Reminder config
#[derive(Debug, Clone)]
pub struct Reminder {
    message: String,
    due_time: DateTime<Local>,
}

// Struct to hold the Message config
#[derive(Debug, Clone)]
pub struct Message {
    TogglePopup: bool,
    PopupClosed(Id),
    SubscriptionChannel,
    UpdateConfig(Config),
    ToggleExampleRow(bool),
    Tick,
    AddReminder,
    CheckReminders
}


fn main() -> iced::Result {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    let cpu_usage = sys.global_cpu_usage();

    let applet = AppState {
        cpu_usage,
        pet_running: false,
        reminders: Vec::new(),
        threshold: 50.0, // Default threshold
    };
}

// View function  to display the application interface
fn view(&self) -> Element<'_, Self::Message> {
    let pet_idle_icon = if self.pet_idle {
        widget::Image::new(iced::widget::image::Handle::from_path("resources/pet_idle.png"))
            .width(Length::Pixels(100.0))
            .height(Length::Pixels(100.0))
    } else {
        widget::Image::new(iced::widget::image::Handle::from_path("resources/pet_active.png"))
            .width(Length::Pixels(100.0))
            .height(Length::Pixels(100.0))
    };

    widget::Column::new()
        .push(pet_idle_icon)
        .push(widget::Text::new(format!("CPU Usage: {:.1}%", self.cpu_usage)))
        .push(widget::Text::new(format!("Pet State: {}", if self.pet_running { "Running" } else { "Resting" })))
        .push(widget::Text::new(format!("Threshold: {:.1}%", self.threshold)))
        .into()

    self.core
        .applet
        .button("Wake Pet", |state| {
            state.pet_running = !state.pet_running;
        })
        .on_press(|state| {
            state.pet_running = !state.pet_running;
        })
        .into()
}

// Functino to decide wich popup to show based on the pet state
// if the pet is awake, show the reminder popup, if the pet is resting, show the idle popup
fn decide_popup(&self, _id: Id) -> Element<'_, Self::Message> {
    let reminder_list_popup = widget::Column::new()
        .push(widget::Text::new("Reminders:"))
        .push(
            self.reminders
                .iter()
                .fold(widget::Column::new(), |column, reminder| {
                    column.push(widget::Text::new(format!(
                        "{} - {}",
                        reminder.message,
                        reminder.due_time.format("%Y-%m-%d %H:%M:%S")
                    )))
                }),
        );
    
    let content = widget::Column::new()
        .align_items(Alignment::Center)
        .spacing(20)
        .push(pet_image)
        .push(cpu_usage_text)
        .push(widget::Text::new(format!("Pet State: {}", if self.pet_running { "Running" } else { "Resting" })))
        .push(widget::Text::new(format!("Threshold: {:.1}%", self.threshold)))
        push(if self.pet_running {
            reminder_list_popup
        } else {
            widget::Text::new("Pet is resting. No reminders to show.")
        });
    
    self.core.applet.popup(content).into()
}

// function to handle the messages emitted by the application
fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
    match message {
        Message::TogglePopup => {
            return if let Some(popup_id: Id) = self.popup.take() {
                destroy_popup(popup_id);
                Task::none();
            } else {
                let new_id = Id::unique();
                self.popup.replace(new_id);

                let mut popup_settings = self.core.applet.get_popup_settings(
                    self.core.main_window_id().unwrap(),
                    new_id,
                    None,
                    None
                );
                popup_settings.positioner.size_limits = Limits::NONE
                    .max_width(400.0)
                    .min_width(200.0)
                    .max_height(600.0)
                    .min_height(100.0);
                get_popup(popup_settings, self.decide_popup(new_id));
            }
        }
        Message::PopupClosed(id) => {
            if let Some(popup_id) = self.popup.take() {
                if popup_id == id {
                    self.popup = None;
                }
            }
        }
        Message::AddReminder => {
            let new_reminder = Reminder {
                message: "New Reminder".to_string(),
                due_time: Local::now() + chrono::Duration::minutes(5),
            };
            self.reminders.push(new_reminder);
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::iced::theme::Style {
            background_color: Some(cosmic::iced::Color::from_rgb(0.1, 0.1, 0.1)),
            text_color: Some(cosmic::iced::Color::from_rgb(1.0, 1.0, 1.0)),
            ..Default::default()
        })
    }
}