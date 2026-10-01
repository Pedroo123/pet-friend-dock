import unittest
import os
import json
import tempfile
import sys

# Ensure core directory is in python path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from core.cpu_tracker import calculate_pet_state, PetTracker, PETS


class TestCpuTracker(unittest.TestCase):
    def test_calculate_pet_state_idle(self):
        state, speed = calculate_pet_state(10.0, high_threshold=50.0)
        self.assertEqual(state, "RESTING")
        self.assertEqual(speed, 0)

    def test_calculate_pet_state_normal_running(self):
        state, speed = calculate_pet_state(30.0, high_threshold=50.0)
        self.assertEqual(state, "RUNNING")
        self.assertGreater(speed, 0)

    def test_calculate_pet_state_fast_running(self):
        state, speed = calculate_pet_state(75.0, high_threshold=50.0)
        self.assertEqual(state, "FAST_RUNNING")
        self.assertGreater(speed, 5)

    def test_pet_selection(self):
        tracker = PetTracker()
        initial_pet = tracker.get_current_pet()
        self.assertEqual(initial_pet["id"], PETS[0]["id"])

        next_pet = tracker.next_pet()
        self.assertEqual(next_pet["id"], PETS[1]["id"])

        selected_pet = tracker.select_pet("fox")
        self.assertEqual(selected_pet["id"], "fox")

    def test_reminder_management(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tracker = PetTracker()
            tracker.reminders_file = os.path.join(tmpdir, "test_reminders.json")

            reminder = tracker.add_reminder("Water Plants", "Remember to water the office plants")
            self.assertEqual(reminder["title"], "Water Plants")
            self.assertEqual(len(tracker.reminders), 1)

            # Check JSON file contents
            self.assertTrue(os.path.exists(tracker.reminders_file))
            with open(tracker.reminders_file, "r") as f:
                data = json.load(f)
                self.assertEqual(len(data), 1)
                self.assertEqual(data[0]["title"], "Water Plants")


if __name__ == "__main__":
    unittest.main()
