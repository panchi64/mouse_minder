use device_query::{DeviceQuery, DeviceState};
use enigo::{Enigo, Mouse, Settings};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime};

// Structure to hold saved position information
#[derive(Clone, Debug)]
pub struct SavedPosition {
    pub x: i32,
    pub y: i32,
    pub timestamp: SystemTime,
}

// Core tracker functionality
pub struct MouseTracker {
    is_tracking: Arc<Mutex<bool>>,
    saved_position: Arc<Mutex<Option<SavedPosition>>>,
    _tracking_thread: Option<JoinHandle<()>>, // Store thread handle but don't expose it
}

impl MouseTracker {
    pub fn new() -> Self {
        let is_tracking = Arc::new(Mutex::new(false));
        let saved_position = Arc::new(Mutex::new(None));

        let tracking_thread =
            Self::spawn_tracking_thread(Arc::clone(&is_tracking), Arc::clone(&saved_position));

        Self {
            is_tracking,
            saved_position,
            _tracking_thread: Some(tracking_thread),
        }
    }

    // Spawn a background thread to track mouse movement
    fn spawn_tracking_thread(
        is_tracking: Arc<Mutex<bool>>,
        saved_position: Arc<Mutex<Option<SavedPosition>>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let device_state = DeviceState::new();
            let mut last_position = (0, 0);
            let mut last_movement_time = Instant::now();

            loop {
                // Check if tracking is enabled
                let tracking = { *is_tracking.lock().unwrap() };

                if tracking {
                    // Get current mouse position
                    let current_position = device_state.get_mouse().coords;

                    // If position changed, update the last movement time
                    if current_position.0 != last_position.0
                        || current_position.1 != last_position.1
                    {
                        last_movement_time = Instant::now();
                        last_position = current_position;
                    } else {
                        // Check if mouse has been still for the threshold time
                        let elapsed = last_movement_time.elapsed().as_millis() as u64;
                        if elapsed >= crate::config::INACTIVITY_THRESHOLD_MS {
                            // Save the position if different from the last saved one
                            let mut pos_guard = saved_position.lock().unwrap();
                            let should_update = match pos_guard.as_ref() {
                                None => true,
                                Some(p) => p.x != current_position.0 || p.y != current_position.1,
                            };

                            if should_update {
                                *pos_guard = Some(SavedPosition {
                                    x: current_position.0,
                                    y: current_position.1,
                                    timestamp: SystemTime::now(),
                                });
                            }
                        }
                    }
                }

                // Sleep to avoid high CPU usage
                thread::sleep(Duration::from_millis(crate::config::POLL_INTERVAL_MS));
            }
        })
    }

    // Start tracking mouse movement
    pub fn start_tracking(&self) {
        let mut tracking = self.is_tracking.lock().unwrap();
        *tracking = true;
    }

    // Stop tracking mouse movement
    pub fn stop_tracking(&self) {
        let mut tracking = self.is_tracking.lock().unwrap();
        *tracking = false;
    }

    // Check if currently tracking
    pub fn is_tracking(&self) -> bool {
        *self.is_tracking.lock().unwrap()
    }

    // Get the last saved position
    pub fn get_saved_position(&self) -> Option<SavedPosition> {
        self.saved_position.lock().unwrap().clone()
    }

    // Reset (clear) the saved position
    pub fn reset_position(&self) {
        let mut pos = self.saved_position.lock().unwrap();
        *pos = None;
    }

    // Restore cursor to saved position
    pub fn restore_position(&self) -> bool {
        if let Some(pos) = self.get_saved_position() {
            if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                // Add the enigo::Coordinate enum to specify absolute positioning
                let _ = enigo.move_mouse(pos.x, pos.y, enigo::Coordinate::Abs);
                return true;
            }
        }
        false
    }
}

impl Drop for MouseTracker {
    fn drop(&mut self) {
        // Clean up by stopping tracking before dropping
        let mut tracking = self.is_tracking.lock().unwrap();
        *tracking = false;
    }
}
