use egui::{Color32, Context, RichText, Stroke, Ui, Vec2};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config;
use crate::hotkeys::{HotKeyAction, HotKeySystem};
use crate::tracker::MouseTracker;

// Main application state
pub struct MouseMinderApp {
    tracker: MouseTracker,
    hotkey_receiver: Receiver<HotKeyAction>,
    last_restore_time: Option<SystemTime>,
    restore_feedback_visible: bool,
}

impl MouseMinderApp {
    pub fn new(ctx: &Context) -> Self {
        // Create action channel for hotkey events
        let (tx, rx) = channel();

        // Initialize tracker
        let tracker = MouseTracker::new();

        // Initialize hotkey system
        let _ = HotKeySystem::new(tx).expect("Failed to initialize hotkey system");

        // Request continuous repaints to keep UI responsive
        ctx.request_repaint_after(Duration::from_millis(config::UI_REFRESH_INTERVAL_MS));

        Self {
            tracker,
            hotkey_receiver: rx,
            last_restore_time: None,
            restore_feedback_visible: false,
        }
    }

    // Handle hotkey actions
    fn handle_hotkeys(&mut self) {
        while let Ok(action) = self.hotkey_receiver.try_recv() {
            match action {
                HotKeyAction::RestorePosition => {
                    if self.tracker.restore_position() {
                        // Show feedback that position was restored
                        self.last_restore_time = Some(SystemTime::now());
                        self.restore_feedback_visible = true;
                    }
                }
            }
        }

        // Clear restore feedback after configured duration
        if self.restore_feedback_visible {
            if let Some(time) = self.last_restore_time {
                if time.elapsed().unwrap().as_millis() >= config::FEEDBACK_DURATION_MS as u128 {
                    self.restore_feedback_visible = false;
                }
            }
        }
    }

    // Format time for display
    fn format_time(time: SystemTime) -> String {
        let duration = time.duration_since(UNIX_EPOCH).unwrap();
        let secs = duration.as_secs();

        let hours = (secs % 86400) / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    // Create color-coded status indicator
    fn status_indicator(&self, ui: &mut Ui) {
        let (status_text, status_color) = if self.tracker.is_tracking() {
            ("Status: Tracking", Color32::from_rgb(50, 180, 50))
        } else {
            ("Status: Paused", Color32::from_rgb(220, 170, 50))
        };

        ui.horizontal(|ui| {
            // Draw a status circle
            let circle_radius = 8.0;
            let circle_pos = ui.cursor().min + Vec2::new(circle_radius, circle_radius);
            ui.painter()
                .circle_filled(circle_pos, circle_radius, status_color);
            ui.painter()
                .circle_stroke(circle_pos, circle_radius, Stroke::new(1.0, Color32::GRAY));

            // Add some space then show the status text
            ui.add_space(circle_radius * 2.5);
            ui.label(RichText::new(status_text).size(16.0));
        });
    }

    // Update and render the UI
    pub fn update(&mut self, ctx: &Context) {
        // Handle any pending hotkey actions
        self.handle_hotkeys();

        // Request a repaint to keep the UI responsive
        ctx.request_repaint_after(Duration::from_millis(config::UI_REFRESH_INTERVAL_MS));

        // Render the UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Title
                ui.add_space(10.0);
                ui.heading(RichText::new(config::APP_NAME).size(24.0));
                ui.label(RichText::new(format!("v{}", config::APP_VERSION)).weak());
                ui.add_space(15.0);

                // Status indicator
                self.status_indicator(ui);
                ui.add_space(15.0);

                // Position info
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(240, 240, 250))
                    .show(ui, |ui| {
                        ui.heading("Last Saved Position");
                        ui.add_space(5.0);

                        if let Some(pos) = self.tracker.get_saved_position() {
                            ui.label(format!("X: {}, Y: {}", pos.x, pos.y));
                            ui.label(format!("Saved at: {}", Self::format_time(pos.timestamp)));
                        } else {
                            ui.label(RichText::new("No position saved yet").italics());
                        }
                    });

                // Restore feedback
                if self.restore_feedback_visible {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("✓ Position Restored!")
                            .color(Color32::from_rgb(30, 150, 30))
                            .size(16.0)
                            .strong(),
                    );
                }

                ui.add_space(20.0);

                // Control buttons
                ui.horizontal(|ui| {
                    let button_height = 32.0;

                    if self.tracker.is_tracking() {
                        if ui
                            .add_sized([140.0, button_height], egui::Button::new("⏹ Stop Tracking"))
                            .clicked()
                        {
                            self.tracker.stop_tracking();
                        }
                    } else if ui
                        .add_sized(
                            [140.0, button_height],
                            egui::Button::new("▶ Start Tracking"),
                        )
                        .clicked()
                    {
                        self.tracker.start_tracking();
                    }

                    ui.add_space(5.0);

                    if ui
                        .add_sized(
                            [120.0, button_height],
                            egui::Button::new("🗑 Reset Position"),
                        )
                        .clicked()
                    {
                        self.tracker.reset_position();
                    }

                    ui.add_space(5.0);

                    if ui
                        .add_sized([120.0, button_height], egui::Button::new("↩ Restore"))
                        .clicked()
                        && self.tracker.restore_position()
                    {
                        self.last_restore_time = Some(SystemTime::now());
                        self.restore_feedback_visible = true;
                    }
                });

                ui.add_space(20.0);

                // Instructions
                egui::Frame::group(ui.style())
                    .fill(ui.style().visuals.extreme_bg_color)
                    .show(ui, |ui| {
                        ui.heading("Instructions");
                        ui.add_space(5.0);
                        ui.label("• Mouse position is saved after 2 seconds of inactivity");

                        let hotkey_text = if cfg!(target_os = "macos") {
                            "• Press ⌘+Shift+R to restore mouse position"
                        } else {
                            "• Press Ctrl+Shift+R to restore mouse position"
                        };
                        ui.label(hotkey_text);
                        ui.label("• Click 'Start Tracking' to begin watching for idle positions");
                    });

                ui.add_space(10.0);
            });
        });
    }
}
