import unittest
import time
from core.cpu_tracker import ReminderManager, Pet2D

class TestReminderManager(unittest.TestCase):
    def test_add_and_trigger_reminder(self):
        rm = ReminderManager()
        now = time.time()
        rm.add_reminder("Water plants", now - 10)
        rm.add_reminder("Future meeting", now + 3600)

        triggered = rm.check_due_reminders(now)
        self.assertEqual(len(triggered), 1)
        self.assertEqual(triggered[0]["text"], "Water plants")

        pending = rm.get_pending_reminders()
        self.assertEqual(len(pending), 1)
        self.assertEqual(pending[0]["text"], "Future meeting")

    def test_no_duplicate_triggers(self):
        rm = ReminderManager()
        now = time.time()
        rm.add_reminder("Take break", now - 5)

        triggered1 = rm.check_due_reminders(now)
        self.assertEqual(len(triggered1), 1)

        triggered2 = rm.check_due_reminders(now + 1)
        self.assertEqual(len(triggered2), 0)


class TestPet2D(unittest.TestCase):
    def test_pet_states_based_on_cpu(self):
        pet = Pet2D(dock_min_x=0, dock_max_x=10)

        state_sleeping = pet.update_state(5.0)
        self.assertEqual(state_sleeping["state"], "SLEEPING")
        self.assertEqual(state_sleeping["speed"], 0)

        state_idle = pet.update_state(25.0)
        self.assertEqual(state_idle["state"], "IDLE")

        state_walking = pet.update_state(50.0)
        self.assertEqual(state_walking["state"], "WALKING")

        state_running = pet.update_state(80.0)
        self.assertEqual(state_running["state"], "RUNNING")

        state_sprinting = pet.update_state(95.0)
        self.assertEqual(state_sprinting["state"], "SPRINTING")

    def test_movement_within_dock_bounds(self):
        pet = Pet2D(dock_min_x=0, dock_max_x=5)
        pet.position_x = 4
        pet.direction = 1

        # High CPU causes movement forward and bounce back at dock boundary
        res1 = pet.update_state(80.0)
        self.assertTrue(0 <= res1["position"]["x"] <= 5)
        self.assertEqual(pet.direction, -1)

        res2 = pet.update_state(80.0)
        self.assertTrue(0 <= res2["position"]["x"] <= 5)


if __name__ == "__main__":
    unittest.main()
