import time
import json
import sys
import select
import psutil
from typing import List, Dict, Any, Optional

class ReminderManager:
    def __init__(self):
        self.reminders: List[Dict[str, Any]] = []

    def add_reminder(self, text: str, due_timestamp: float) -> Dict[str, Any]:
        reminder = {
            "id": len(self.reminders) + 1,
            "text": text,
            "due_timestamp": due_timestamp,
            "triggered": False
        }
        self.reminders.append(reminder)
        return reminder

    def check_due_reminders(self, current_time: Optional[float] = None) -> List[Dict[str, Any]]:
        if current_time is None:
            current_time = time.time()
        triggered_now = []
        for r in self.reminders:
            if not r["triggered"] and current_time >= r["due_timestamp"]:
                r["triggered"] = True
                triggered_now.append(r)
        return triggered_now

    def get_pending_reminders(self) -> List[Dict[str, Any]]:
        return [r for r in self.reminders if not r["triggered"]]


class Pet2D:
    # 2D ASCII Sprite Frames for Pet States
    SPRITES = {
        "SLEEPING": ["(z_z)", "(Z_Z)", "( -_-)z"],
        "IDLE": ["(o.o)", "(o_o)", "(.-.)"],
        "WALKING": [">^._.^<", " >^._.^<", "  >^._.^<"],
        "RUNNING": ["c(-.-'c)", " c(-.-'c)", "  c(-.-'c)"],
        "SPRINTING": ["=~(x.x)=~", " =~(-.-)=~", "  =~(>.<)=~"]
    }

    def __init__(self, dock_min_x: int = 0, dock_max_x: int = 20):
        self.dock_min_x = dock_min_x
        self.dock_max_x = dock_max_x
        self.position_x = (dock_min_x + dock_max_x) // 2
        self.position_y = 0  # 2D Y coordinate on dock plane
        self.direction = 1  # 1 for right, -1 for left
        self.frame_index = 0

    def update_state(self, cpu_usage: float) -> Dict[str, Any]:
        if cpu_usage < 15.0:
            state = "SLEEPING"
            speed = 0
            activity = "resting in dock space"
        elif cpu_usage < 40.0:
            state = "IDLE"
            speed = 1
            activity = "chilling in dock space"
        elif cpu_usage < 70.0:
            state = "WALKING"
            speed = int(cpu_usage // 10)
            activity = "walking along dock"
        elif cpu_usage < 90.0:
            state = "RUNNING"
            speed = int(cpu_usage // 10) + 1
            activity = "running fast across dock"
        else:
            state = "SPRINTING"
            speed = int(cpu_usage // 10) + 3
            activity = "sprinting frantically across dock"

        # Update 2D x-position bounded within the dock resting space
        if speed > 0:
            self.position_x += self.direction * speed
            if self.position_x >= self.dock_max_x:
                self.position_x = self.dock_max_x
                self.direction = -1
            elif self.position_x <= self.dock_min_x:
                self.position_x = self.dock_min_x
                self.direction = 1

        # Cycle animation frame
        frames = self.SPRITES.get(state, self.SPRITES["IDLE"])
        self.frame_index = (self.frame_index + 1) % len(frames)
        sprite = frames[self.frame_index]

        # 2D dock visual canvas string
        dock_width = self.dock_max_x - self.dock_min_x + 1
        rel_pos = max(0, min(self.position_x - self.dock_min_x, dock_width - 1))
        dock_canvas = list("_" * dock_width)
        dock_canvas[rel_pos] = "P"
        dock_visual = f"[{''.join(dock_canvas)}]"

        return {
            "state": state,
            "speed": speed,
            "activity": activity,
            "position": {"x": self.position_x, "y": self.position_y},
            "sprite": sprite,
            "dock_visual": dock_visual
        }


def check_incoming_reminders(reminder_manager: ReminderManager):
    """Check stdin non-blockingly for incoming reminder JSON commands."""
    try:
        rlist, _, _ = select.select([sys.stdin], [], [], 0)
        if rlist:
            line = sys.stdin.readline().strip()
            if line:
                data = json.loads(line)
                if data.get("action") == "add_reminder":
                    text = data.get("text", "")
                    delay = float(data.get("delay", 0))
                    due_ts = time.time() + delay
                    reminder_manager.add_reminder(text, due_ts)
    except Exception:
        pass


def get_pet_state(interval: float = 1.0, reminder_manager: Optional[ReminderManager] = None, pet: Optional[Pet2D] = None):
    if reminder_manager is None:
        reminder_manager = ReminderManager()
    if pet is None:
        pet = Pet2D()

    while True:
        check_incoming_reminders(reminder_manager)
        cpu_usage = psutil.cpu_percent(interval=interval)
        pet_dynamics = pet.update_state(cpu_usage)

        due = reminder_manager.check_due_reminders()
        notifications = [r["text"] for r in due]

        status = {
            "cpu": cpu_usage,
            "state": pet_dynamics["state"],
            "speed": pet_dynamics["speed"],
            "activity": pet_dynamics["activity"],
            "position": pet_dynamics["position"],
            "sprite": pet_dynamics["sprite"],
            "dock_visual": pet_dynamics["dock_visual"],
            "notifications": notifications
        }

        print(json.dumps(status))
        sys.stdout.flush()

if __name__ == "__main__":
    try:
        get_pet_state()
    except KeyboardInterrupt:
        sys.exit(0)
