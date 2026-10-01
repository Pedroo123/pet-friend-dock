# Pet Friend Dock Widget for COSMIC Desktop

A desktop dock widget designed for Pop!_OS and the COSMIC desktop environment. Cute pixelated pets rest above your dock when idle, run across your dock when your CPU is in use, and run even faster when CPU usage crosses a threshold! You can switch pets and save reminders displayed via COSMIC notifications.

## Features

1. **Rest Above Dock when Idle**: When CPU usage is low (< 15%), your pet rests peacefully.
2. **Run Across Dock**: When CPU is active (>= 15%), your pet runs across the dock.
3. **Run Faster at High CPU**: When CPU usage exceeds threshold X (50%), your pet runs at maximum speed.
4. **Reminders & Notifications**: Create and manage reminders with COSMIC desktop notification alerts (`notify-send`).
5. **Switch Pets**: Choose from a list of pixelated pets inspired by "Let's Build Your Zoo" (Capybara, Panda, Red Fox, Penguin, Koala, Duck).

---

## Prerequisites

Ensure you have the following installed on Pop!_OS / Linux:

- **Python 3.10+**
- **psutil** Python library
- **Rust / Cargo** (for the COSMIC applet frontend)
- **libnotify** (`notify-send` command for notifications)

### Installing Dependencies

```bash
# Install Python dependency
pip install psutil

# Install libnotify on Pop!_OS / Ubuntu
sudo apt install libnotify-bin
```

---

## Running the Project

### 1. Running the Core CPU Tracker (Python Backend)

The Python backend tracks system CPU usage, manages pet states, handles pet switching, and manages reminders.

```bash
python3 core/cpu_tracker.py
```

### 2. Running the COSMIC Dock Applet (Rust Frontend)

The Rust applet connects asynchronously to the Python backend to receive real-time pet status and render the dock widget in the COSMIC environment.

```bash
cd cosmic-applet
cargo run
```

---

## Running Tests

To execute the unit test suite covering CPU calculations, pet switching, and reminder storage:

```bash
python3 -m unittest discover -s tests
```
