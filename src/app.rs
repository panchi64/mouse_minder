use egui::{Color32, Context, CornerRadius, RichText, Stroke, Ui, Vec2};
use core::f32;
use std::sync::mpsc::{channel, Receiver};
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
            ("TRACKING", Color32::from_rgb(76, 175, 80))
        } else {
            ("PAUSED", Color32::from_rgb(255, 152, 0))
        };

        let status_bg = if self.tracker.is_tracking() {
            Color32::from_rgb(40, 70, 40) // Darker green for dark mode
        } else {
            Color32::from_rgb(70, 50, 20) // Darker orange for dark mode
        };

        ui.horizontal_centered(|ui| {
            // Create a layout that only occupies the space needed
            let circle_radius = 8.0;
            let padding = 12;
            
            // Manually create a tight container
            let text_size = ui.text_style_height(&egui::TextStyle::Body) * 1.2;
            
            // Calculate an appropriate width for the status box
            let status_width = status_text.len() as f32 * 10.0 + circle_radius * 3.0 + padding as f32 * 1.5;
            
            // Create frame with fixed width
            egui::Frame::new()
                .fill(status_bg)
                .corner_radius(CornerRadius::same(16))
                .inner_margin(egui::Margin::symmetric(padding, padding))
                .outer_margin(egui::Margin::same(0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.set_width(status_width);
                        
                        // Draw status circle
                        let circle_pos = ui.cursor().min + Vec2::new(circle_radius, text_size * 0.5);
                        ui.painter().circle_filled(circle_pos, circle_radius, status_color);
                        ui.painter().circle_stroke(circle_pos, circle_radius, Stroke::new(1.0, Color32::GRAY));
                        
                        // Add spacing and text
                        ui.add_space(circle_radius * 2.5);
                        ui.label(
                            RichText::new(status_text)
                                .size(16.0)
                                .color(status_color)
                                .strong(),
                        );
                    });
                });
        });
}

    // Update and render the UI
    pub fn update(&mut self, ctx: &Context) {
        // Handle any pending hotkey actions
        self.handle_hotkeys();

        // Request a repaint to keep the UI responsive
        ctx.request_repaint_after(Duration::from_millis(config::UI_REFRESH_INTERVAL_MS));

        // Custom colors
        let app_bg = Color32::from_rgb(30, 30, 35);
        let panel_bg = Color32::from_rgb(45, 45, 50);
        let text_color = Color32::from_rgb(220, 220, 230);

        // Render the UI
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(app_bg))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Main content area
                    egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(20,0))
                    .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    // Status indicator
                                    ui.add_space(-550.0);
                                    self.status_indicator(ui);
                                    ui.add_space(16.0);

                                    // Position info
                                    egui::Frame::new()
                                        .fill(panel_bg)
                                        .corner_radius(CornerRadius::same(8))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(230, 230, 240)))
                                        .shadow(egui::epaint::Shadow {
                                            offset: [0, 2],
                                            blur: 4,
                                            spread: 0,
                                            color: Color32::from_rgb(0, 0, 0).linear_multiply(0.1),
                                        })
                                        .inner_margin(egui::Margin::same(16))
                                        .show(ui, |ui| {
                                            ui.vertical_centered(|ui| {
                                                ui.heading(
                                                    RichText::new("Last Saved Position")
                                                        .color(text_color)
                                                        .size(18.0),
                                                );
                                                ui.add_space(10.0);

                                                if let Some(pos) = self.tracker.get_saved_position() {
                                                    // Coordinates
                                                    let coords_text = format!("X: {}, Y: {}", pos.x, pos.y);
                                                    ui.label(
                                                        RichText::new(coords_text)
                                                            .size(20.0)
                                                            .color(text_color)
                                                    );

                                                    ui.add_space(4.0);

                                                    // Timestamp
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "Saved at: {}",
                                                            Self::format_time(pos.timestamp)
                                                        ))
                                                        .color(Color32::from_rgb(120, 120, 140))
                                                        .size(14.0),
                                                    );
                                                } else {
                                                    ui.label(
                                                        RichText::new("No position saved yet")
                                                            .italics()
                                                            .color(Color32::from_rgb(150, 150, 170))
                                                            .size(16.0),
                                                    );
                                                }
                                            });
                                        });

                                    // Restore feedback
                                    if self.restore_feedback_visible {
                                        ui.add_space(16.0);
                                        egui::Frame::new()
                                            .fill(Color32::from_rgb(232, 245, 233))
                                            .corner_radius(CornerRadius::same(8))
                                            .inner_margin(egui::Margin::same(10))
                                            .show(ui, |ui| {
                                                ui.vertical_centered(|ui| {
                                                    ui.label(
                                                        RichText::new("Position Restored!")
                                                            .color(Color32::from_rgb(46, 125, 50))
                                                            .size(16.0)
                                                            .strong(),
                                                    );
                                                });
                                            });
                                    }

                                    ui.add_space(24.0);

                                    // Control buttons - centered
                                    egui::Frame::new()
                                        .show(ui, |ui| {
                                            ui.vertical_centered(|ui| {
                                                // First row - start/stop button
                                                let track_button_text;
                                                let track_button_color;
                                                let track_button_text_color;

                                                if self.tracker.is_tracking() {
                                                    track_button_text = "⏹ Stop Tracking";
                                                    track_button_color = Color32::from_rgb(239, 83, 80);
                                                    track_button_text_color = Color32::WHITE;
                                                } else {
                                                    track_button_text = "▶ Start Tracking";
                                                    track_button_color = Color32::from_rgb(76, 175, 80);
                                                    track_button_text_color = Color32::LIGHT_GRAY;
                                                }

                                                let track_button = egui::Button::new(
                                                    RichText::new(track_button_text)
                                                        .color(track_button_text_color)
                                                        .size(16.0),
                                                )
                                                .fill(track_button_color)
                                                .corner_radius(CornerRadius::same(6))
                                                .min_size(egui::Vec2::new(180.0, 40.0));

                                                if self.tracker.is_tracking() {
                                                    if ui.add(track_button).clicked() {
                                                        self.tracker.stop_tracking();
                                                    }
                                                } else if ui.add(track_button).clicked() {
                                                    self.tracker.start_tracking();
                                                }

                                                ui.add_space(12.0);

                                                // Second row - reset
                                                ui.vertical_centered(|ui| {
                                                    let button_height = 36.0;
                                                    let button_width = 150.0;
                                                    
                                                    // Reset button
                                                    let reset_button = egui::Button::new(
                                                        RichText::new("🗑 Reset Position")
                                                            .color(Color32::GRAY)
                                                            .size(14.0),
                                                    )
                                                    .min_size(egui::Vec2::new(button_width, button_height))
                                                    .corner_radius(CornerRadius::same(6))
                                                    .fill(Color32::from_rgb(47, 54, 64));

                                                    if ui.add(reset_button).clicked() {
                                                        self.tracker.reset_position();
                                                    }

                                                    ui.add_space(12.0);
                                                });
                                            });
                                        });

                                    ui.add_space(24.0);

                                    // Instructions
                                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                        egui::Frame::new()
                                            .fill(Color32::from_rgb(47, 54, 64))
                                            .corner_radius(CornerRadius::same(8))
                                            .inner_margin(egui::Margin::same(16))
                                            .show(ui, |ui| {
                                                ui.vertical_centered(|ui| {
                                                    ui.heading(
                                                        RichText::new("Instructions")
                                                            .color(Color32::from_rgb(220, 221, 225))
                                                            .size(16.0),
                                                    );
                                                    ui.add_space(8.0);

                                                    // Add a subtle separator
                                                    let separator_stroke =
                                                        Stroke::new(1.0, Color32::from_rgb(220, 220, 230));
                                                    let y = ui.cursor().min.y;
                                                    let rect = ui.max_rect();
                                                    let line_start = egui::Pos2::new(rect.min.x, y);
                                                    let line_end = egui::Pos2::new(rect.max.x, y);
                                                    ui.painter().line_segment([line_start, line_end], separator_stroke);
                                                    ui.add_space(10.0);

                                                    ui.label(
                                                        RichText::new("• Mouse position is saved after 2 seconds of inactivity")
                                                            .color(text_color)
                                                            .size(14.0),
                                                    );

                                                    let hotkey_text = if cfg!(target_os = "macos") {
                                                        "• Press ⌘+Shift+R to restore mouse position"
                                                    } else {
                                                        "• Press Ctrl+Shift+R to restore mouse position"
                                                    };

                                                    ui.label(
                                                        RichText::new(hotkey_text)
                                                            .color(text_color)
                                                            .size(14.0),
                                                    );

                                                    ui.label(
                                                        RichText::new("• Click 'Start Tracking' to begin watching for idle positions")
                                                            .color(text_color)
                                                            .size(14.0),
                                                    );
                                                });
                                        });
                                    });

                                    // Footer with app name and version
                                    ui.add_space(16.0);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgb(25, 25, 30))
                                        .inner_margin(egui::Margin::same(8))
                                        .show(ui, |ui| {
                                            ui.vertical_centered(|ui| {
                                                ui.label(
                                                    RichText::new(format!("{} v{}", config::APP_NAME, config::APP_VERSION))
                                                        .size(12.0)
                                                        .color(Color32::from_rgb(150, 150, 160)),
                                                );
                                            });
                                        });
                                });
                    });
                });
            });
    }
}
