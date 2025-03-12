use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

// Actions that can be triggered by hotkeys
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HotKeyAction {
    RestorePosition,
}

// Hotkey handling system
pub struct HotKeySystem {
    _listener_thread: JoinHandle<()>, // Keep thread alive with the struct
}

impl HotKeySystem {
    pub fn new(action_sender: Sender<HotKeyAction>) -> Result<Self, Box<dyn std::error::Error>> {
        // Start a thread to handle hotkey registration and events
        let listener_thread = thread::spawn(move || {
            if let Ok(manager) = GlobalHotKeyManager::new() {
                // Determine platform specific modifier (Cmd for macOS, Ctrl for others)
                let modifier = if cfg!(target_os = "macos") {
                    Modifiers::META | Modifiers::SHIFT // Change CMD to META
                } else {
                    Modifiers::CONTROL | Modifiers::SHIFT
                };

                // Create and register the restore position hotkey (R key)
                let restore_hotkey = HotKey::new(Some(modifier), Code::KeyR);
                if manager.register(restore_hotkey).is_ok() {
                    // Record the mapping of hotkey ID to action
                    let restore_id = restore_hotkey.id();

                    // Listen for hotkey events
                    let event_receiver = GlobalHotKeyEvent::receiver();
                    while let Ok(event) = event_receiver.recv() {
                        if event.state == HotKeyState::Pressed && event.id == restore_id {
                            let _ = action_sender.send(HotKeyAction::RestorePosition);
                        }
                    }
                }
            }
        });

        Ok(Self {
            _listener_thread: listener_thread,
        })
    }
}
