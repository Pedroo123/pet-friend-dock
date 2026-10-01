use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PetInfo {
    pub id: String,
    pub name: String,
    pub style: String,
    pub icon: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PetStatus {
    pub cpu: f32,
    pub state: String,
    pub speed: u32,
    pub pet: Option<PetInfo>,
    pub reminders_count: Option<usize>,
    pub high_threshold: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum AppletMessage {
    PetStatusUpdated(PetStatus),
    SwitchPetPressed,
    AddReminderPressed(String, String),
}

pub struct CosmicPetApplet {
    pub current_status: Option<PetStatus>,
}

impl CosmicPetApplet {
    pub fn new() -> Self {
        Self {
            current_status: None,
        }
    }

    pub fn handle_update(&mut self, status: PetStatus) {
        println!(
            "[COSMIC Dock] Pet: {} ({}) | State: {} | Speed: {} | CPU: {:.1}%",
            status.pet.as_ref().map_or("Unknown", |p| &p.name),
            status.pet.as_ref().map_or("?", |p| &p.icon),
            status.state,
            status.speed,
            status.cpu
        );
        self.current_status = Some(status);
    }

    pub fn render_dock_widget(&self) -> String {
        match &self.current_status {
            Some(status) => {
                let icon = status.pet.as_ref().map_or("🐾", |p| &p.icon);
                let name = status.pet.as_ref().map_or("Pet", |p| &p.name);
                format!(
                    "[Dock Widget] {} {} | State: {} | Speed: {} | CPU: {:.1}%",
                    icon, name, status.state, status.speed, status.cpu
                )
            }
            None => "[Dock Widget] Initializing Pet...".to_string(),
        }
    }
}

// Funcao async para inicializar o script em python e enviar para a interface Rust
fn iniciar_tracker_python(tx: mpsc::Sender<PetStatus>) {
    std::thread::spawn(move || {
        let mut child = Command::new("python3")
            .arg("../core/cpu_tracker.py")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Error spawning the python script");

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        for l in reader.lines() {
            if let Ok(texto) = l {
                if let Ok(status) = serde_json::from_str::<PetStatus>(&texto) {
                    let _ = tx.blocking_send(status);
                }
            }
        }
    });
}

fn main() {
    let (tx, mut rx) = mpsc::channel::<PetStatus>(32);

    // Inicia processo python em background
    iniciar_tracker_python(tx);

    let mut applet = CosmicPetApplet::new();

    println!("Everything set up on COSMIC, Waiting on pet data....");

    // Continuous loop listening for incoming status updates
    while let Some(status) = rx.blocking_recv() {
        applet.handle_update(status);
        println!("{}", applet.render_dock_widget());
    }
}
