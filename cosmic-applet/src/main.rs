use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};
use tokio::sync::mpsc;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct PetStatus {
    cpu: f32,
    state: String,
    speed: u23,
}

// Funcao async para inicializar o script em python e enviar para a inerface Rust
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
                    let _ = tx.blocking_send(staus);
                }
            }
        }
    })
}

fn main() {
    let (tx, mut rx) = mpsc::channel::<PetStatus>(32);

    //Inicia processo python em background
    iniciar_tracker_python(tx);

    println!("Everyting set up on COSMIC, Waiting on pet data....");
}