import time
import json
import sys
import os
import psutil
import subprocess

# List of pixelated pets inspired by "Let's Build Your Zoo"
PETS = [
    {"id": "capybara", "name": "Capybara", "style": "pixelated", "icon": "🦫"},
    {"id": "panda", "name": "Panda", "style": "pixelated", "icon": "🐼"},
    {"id": "fox", "name": "Red Fox", "style": "pixelated", "icon": "🦊"},
    {"id": "penguin", "name": "Penguin", "style": "pixelated", "icon": "🐧"},
    {"id": "koala", "name": "Koala", "style": "pixelated", "icon": "🐨"},
    {"id": "duck", "name": "Duck", "style": "pixelated", "icon": "🦆"}
]

CPU_HIGH_THRESHOLD = 50.0  # Threshold X above which pet runs faster

def calculate_pet_state(cpu_usage, high_threshold=CPU_HIGH_THRESHOLD):
    """
    1 - Rest above the dock when idle (cpu_usage < 15)
    2 - Run across the dock while the CPU is in use (15 <= cpu_usage < high_threshold)
    3 - Run faster if the CPU goes above X (cpu_usage >= high_threshold)
    """
    if cpu_usage < 15.0:
        state = "RESTING"
        speed = 0
    elif cpu_usage < high_threshold:
        state = "RUNNING"
        speed = int(cpu_usage // 10) + 1
    else:
        state = "FAST_RUNNING"
        # Speed multiplier when CPU goes above threshold X
        speed = int(cpu_usage // 10) * 2

    return state, speed


class PetTracker:
    def __init__(self, high_threshold=CPU_HIGH_THRESHOLD):
        self.current_pet_index = 0
        self.high_threshold = high_threshold
        self.reminders = []
        self.reminders_file = os.path.join(os.path.dirname(__file__), "reminders.json")
        self.load_reminders()

    def get_current_pet(self):
        return PETS[self.current_pet_index]

    def next_pet(self):
        self.current_pet_index = (self.current_pet_index + 1) % len(PETS)
        return self.get_current_pet()

    def select_pet(self, pet_id):
        for idx, pet in enumerate(PETS):
            if pet["id"] == pet_id:
                self.current_pet_index = idx
                return pet
        return self.get_current_pet()

    def load_reminders(self):
        if os.path.exists(self.reminders_file):
            try:
                with open(self.reminders_file, "r") as f:
                    self.reminders = json.load(f)
            except Exception:
                self.reminders = []

    def save_reminders(self):
        try:
            with open(self.reminders_file, "w") as f:
                json.dump(self.reminders, f, indent=2)
        except Exception as e:
            sys.stderr.write(f"Error saving reminders: {e}\n")

    def add_reminder(self, title, text):
        reminder = {
            "id": len(self.reminders) + 1,
            "title": title,
            "text": text,
            "created_at": time.time(),
            "triggered": False
        }
        self.reminders.append(reminder)
        self.save_reminders()
        self.send_cosmic_notification(f"Reminder Set: {title}", text)
        return reminder

    def send_cosmic_notification(self, title, message):
        """Sends a desktop/COSMIC notification."""
        try:
            subprocess.run(["notify-send", title, message], check=False)
        except Exception as e:
            sys.stderr.write(f"Notification error: {e}\n")

    def get_status(self, cpu_usage):
        state, speed = calculate_pet_state(cpu_usage, self.high_threshold)
        pet = self.get_current_pet()
        return {
            "cpu": cpu_usage,
            "state": state,
            "speed": speed,
            "pet": pet,
            "reminders_count": len(self.reminders),
            "high_threshold": self.high_threshold
        }


def get_pet_state():
    tracker = PetTracker()
    while True:
        cpu_usage = psutil.cpu_percent(interval=1.0)
        status = tracker.get_status(cpu_usage)
        print(json.dumps(status))
        sys.stdout.flush()


if __name__ == "__main__":
    try:
        get_pet_state()
    except KeyboardInterrupt:
        sys.exit(0)
