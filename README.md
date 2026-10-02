# pet-friend-dock

`pet-friend-dock` is an interactive desktop pet applet designed for the COSMIC desktop environment. The applet features a pixelated pet attached to your dock that reacts to system CPU usage and manages reminders/notifications.

The project combines a **Rust** frontend applet (`cosmic-applet`) and a **Python** backend tracker (`core/cpu_tracker.py`) communicating over standard asynchronous JSON IPC.

---

## Table of Contents

- [Features](#features)
- [Project Architecture](#project-architecture)
- [Prerequisites](#prerequisites)
- [Building & Running Locally](#building--running-locally)
  - [1. Setting Up Python Environment](#1-setting-up-python-environment)
  - [2. Running Python Unit Tests](#2-running-python-unit-tests)
  - [3. Building and Running the COSMIC Applet](#3-building-and-running-the-cosmic-applet)
- [Contributing Guidelines](#contributing-guidelines)
  - [Development Workflow](#development-workflow)
  - [Coding Standards](#coding-standards)
  - [Submitting Pull Requests](#submitting-pull-requests)
- [License](#license)

---

## Features

- **Interactive Pet States**: Pet behavior changes dynamically based on CPU load (RESTING when idle, RUNNING when busy, FAST_RUNNING under high load).
- **Multiple Pets**: Select from various pixelated pet personas (Capybara, Panda, Red Fox, Penguin, Koala, Duck).
- **Reminders & Notifications**: Set reminders and receive desktop notifications (`notify-send`).
- **COSMIC Dock Integration**: Async IPC between Rust COSMIC applet and Python backend.

---

## Project Architecture

```text
pet-friend-dock/
├── core/
│   └── cpu_tracker.py       # Python backend service tracking CPU and pet state
├── cosmic-applet/
│   ├── Cargo.toml           # Rust dependencies and package configuration
│   ├── manifest.ron         # COSMIC applet manifest
│   └── src/
│       └── main.rs          # Rust COSMIC applet entrypoint & IPC interface
├── tests/
│   └── test_cpu_tracker.py  # Unit tests for the Python backend
└── README.md                # Project documentation
```

---

## Prerequisites

Before building the project locally, ensure you have the following installed:

- **Python**: Python 3.10+ and `pip`
- **Rust & Cargo**: Rust toolchain (2021 edition)
- **Optional / System Utilities**: `notify-send` (for desktop notification support on Linux/COSMIC)

---

## Building & Running Locally

### 1. Setting Up Python Environment

Install the required Python dependencies (`psutil` is required by `cpu_tracker.py`):

```bash
pip install psutil
```

*(Optional)* You can also use a virtual environment:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install psutil
```

### 2. Running Python Unit Tests

Before compiling or running the Rust applet, ensure all backend unit tests pass:

```bash
python3 -m unittest discover -s tests
```

### 3. Building and Running the COSMIC Applet

Navigate to the `cosmic-applet` directory to build and run the Rust applet:

```bash
cd cosmic-applet

# Verify/check code without producing binary
cargo check

# Build debug binary
cargo build

# Run the applet locally (spawns Python core automatically)
cargo run
```

---

## Contributing Guidelines

We welcome contributions! To keep code maintainable and reliable, please follow these steps when contributing:

### Development Workflow

1. **Fork & Clone** the repository.
2. **Create a Feature Branch**:
   ```bash
   git checkout -b feature/my-cool-feature
   ```
3. **Make Your Changes**: Keep changes focused and clear.
4. **Run Tests**:
   - Run Python unit tests: `python3 -m unittest discover -s tests`
   - Check Rust code: `cd cosmic-applet && cargo check`
5. **Commit Your Work**: Write descriptive, concise commit messages.

### Coding Standards

- **Python**: Follow PEP 8 guidelines. Write unit tests in `tests/` for any new logic added to `core/`.
- **Rust**: Ensure code formats cleanly with `cargo fmt` and compiles without warnings (`cargo check`).

### Submitting Pull Requests

- Push your branch to your fork and submit a Pull Request against `main`.
- Describe the motivation, changes made, and test steps performed in your PR description.

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
