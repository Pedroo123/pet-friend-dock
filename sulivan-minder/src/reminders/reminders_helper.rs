use chrono::{DateTime, Local};

pub mod remindersHelper {
    // Struct to represent a reminder with a due date and message
    #[derive(Debug, Clone, PartialEq)]
    pub struct Reminder {
        pub message: String,
        pub due_time: DateTime<Local>,
    }

    impl Reminder {
        pub fn from_config(&(timestamp, ref message): &(i64, String)) -> Option<Self> {
            Some(Self {
                message: message.clone(),
                due_time: Local.timestamp_opt(*timestamp, 0).single()?,
            })
        }

        pub fn to_config(&self) -> (i64, String) {
            (self.due_time.timestamp(), self.message.clone())
        }
    }

    // Function to convert a vector of reminders from config format to Reminder structs
    pub fn reminders_from_config(config_reminders: &Vec<(i64, String)>) -> Vec<Reminder> {
        config_reminders.iter().filter_map(Reminder::from_config).collect()
    }

    // Separates reminders into (due and pending) based on the current time (now)
    pub fn partition_reminders(reminders: Vec<Reminder>, now: DateTime<Local>) -> (Vec<Reminder>, Vec<Reminder>) {
        let mut due = Vec::new();
        let mut pending = Vec::new();

        for reminder in reminders {
            if reminder.due_time <= now {
                due.push(reminder);
            } else {
                pending.push(reminder);
            }
        }

        (due, pending)
    }
}