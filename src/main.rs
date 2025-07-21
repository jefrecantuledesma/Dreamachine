use std::time::{Duration, Instant};

use eframe::{App, CreationContext, Frame, NativeOptions, egui, run_native};
use egui::containers::menu::MenuBar;
use egui::{Color32, Pos2, Rect};

use webbrowser;

enum Mode {
    Flash,
    Sweep,
    Lighthouse,
}

struct DreamApp {
    // blink mode
    flashing: bool,
    last_toggle: Instant,
    show_white: bool,
    interval: Duration,

    // UI text
    start_stop_text: String,
    spin_start: Instant,
    spin_speed: f32,

    // sweep mode
    mode: Mode,
    sweep_start: Instant,
    beam_width_norm: f32, // fraction of window width
    frequency_hz: f32,
    sweep_speed: f32, // cycles per second

    confirm_quit: bool,

    fullscreen: bool,
}

impl Default for DreamApp {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            flashing: false,
            last_toggle: now,
            show_white: false,
            frequency_hz: 10.0,
            interval: Duration::from_secs_f32(1.0 / 10.0), // ~10 Hz blink

            start_stop_text: "Start".into(),
            spin_start: now,
            spin_speed: 10.0,
            mode: Mode::Sweep,
            sweep_start: now,
            beam_width_norm: 0.4, // 20% of screen width

            sweep_speed: 10.0, // half sweep per second
            //
            confirm_quit: false,
            fullscreen: false,
        }
    }
}

impl DreamApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        let mut s = Self::default();
        s.sweep_speed = s.frequency_hz;
        s.interval = Duration::from_secs_f32(1.0 / s.frequency_hz);
        s
    }
}

impl App for DreamApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        // at the top of update():
        let show_menu = !self.fullscreen
    // read hover_pos() inside the closure:
    || ctx.input(|i| i.pointer.hover_pos().map_or(false, |pos| pos.y <= 60.0));

        if show_menu {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button(&self.start_stop_text).clicked() {
                            self.flashing = !self.flashing;
                            self.start_stop_text =
                                if self.flashing { "Stop" } else { "Start" }.into();
                            self.last_toggle = Instant::now();
                            self.show_white = false;
                        }
                        if ui.button("Quit").clicked() {
                            self.confirm_quit = true;
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        ui.menu_button("Mode", |ui| {
                            if ui
                                .button(format!(
                                    "Flash{}",
                                    if let Mode::Flash = self.mode {
                                        " *"
                                    } else {
                                        ""
                                    }
                                ))
                                .clicked()
                            {
                                self.mode = Mode::Flash;
                                self.sweep_start = Instant::now();
                            }
                            if ui
                                .button(format!(
                                    "Sweep{}",
                                    if let Mode::Sweep = self.mode {
                                        " *"
                                    } else {
                                        ""
                                    }
                                ))
                                .clicked()
                            {
                                self.mode = Mode::Sweep;
                                self.sweep_start = Instant::now();
                            }
                            if ui
                                .button(format!(
                                    "Lighthouse{}",
                                    if let Mode::Lighthouse = self.mode {
                                        " *"
                                    } else {
                                        ""
                                    }
                                ))
                                .clicked()
                            {
                                self.mode = Mode::Lighthouse;
                                self.spin_start = Instant::now();
                            }
                        });
                        ui.menu_button("Hertz", |ui| {
                            for &hz in &[8.0, 9.0, 10.0, 11.0, 12.0, 13.0] {
                                let label = format!(
                                    "{:.0} Hz{}",
                                    hz,
                                    if (self.frequency_hz - hz).abs() < 0.1 {
                                        " *"
                                    } else {
                                        ""
                                    }
                                );
                                if ui.button(label).clicked() {
                                    self.frequency_hz = hz;
                                    self.interval = Duration::from_secs_f32(1.0 / hz);
                                    self.sweep_speed = hz;
                                }
                            }
                        });
                    });
                    ui.menu_button("View", |ui| {
                        let label =
                            format!("Fullscreen{}", if self.fullscreen { " *" } else { "" });
                        if ui.button(label).clicked() {
                            self.fullscreen = !self.fullscreen;
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Learn More").clicked() {
                            let url = "https://en.wikipedia.org/wiki/Dreamachine";
                            if let Err(err) = webbrowser::open(url) {
                                eprintln!("Failed to open browser at {}: {}", url, err);
                            }
                        }
                    })
                });
            });
        }

        // === BLINK STATE ===
        if self.flashing {
            let now = Instant::now();
            if now.duration_since(self.last_toggle) >= self.interval {
                self.show_white = !self.show_white;
                self.last_toggle = now;
            }
        }

        if self.confirm_quit {
            egui::Window::new("Confirm Quit")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to quit?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("No").clicked() {
                            self.confirm_quit = false;
                        }
                    });
                });
        }

        // === DRAW ===
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            if self.flashing {
                match self.mode {
                    Mode::Flash => {
                        // full‑screen blink
                        let color = if self.show_white {
                            Color32::WHITE
                        } else {
                            Color32::BLACK
                        };
                        painter.rect_filled(rect, 0.0, color);
                    }

                    Mode::Sweep => {
                        // horizontal sweep beam (your existing code)
                        let t = Instant::now().duration_since(self.spin_start).as_secs_f32();
                        let period = 1.0 + self.beam_width_norm;
                        let tmod = (t * self.spin_speed) % period;
                        let center_norm = tmod - self.beam_width_norm * 0.5;
                        let cx = rect.left() + center_norm * rect.width();

                        let beam_w = rect.width() * self.beam_width_norm;
                        let half = beam_w * 0.5;
                        let start_x = cx - half;
                        let slices = 60;
                        let slice_w = beam_w / slices as f32;
                        for i in 0..slices {
                            let f = i as f32 / (slices - 1) as f32;
                            let dist = (f - 0.5).abs() * 2.0;
                            let alpha = ((1.0 - dist) * 255.0) as u8;

                            let x0 = start_x + f * (beam_w - slice_w);
                            let x1 = x0 + slice_w;
                            painter.rect_filled(
                                Rect::from_min_max(
                                    Pos2 {
                                        x: x0,
                                        y: rect.top(),
                                    },
                                    Pos2 {
                                        x: x1,
                                        y: rect.bottom(),
                                    },
                                ),
                                0.0,
                                Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                            );
                        }
                    }

                    Mode::Lighthouse => {
                        // radial‑wedge beam
                        let t = Instant::now().duration_since(self.spin_start).as_secs_f32();
                        let angle =
                            (t * self.frequency_hz * std::f32::consts::TAU) % std::f32::consts::TAU;
                        let center = rect.center();
                        let radius = (rect.width().hypot(rect.height())) * 0.6;
                        let half_w = 0.3; // beam angular half‑width in radians

                        // outer soft wedge
                        let a1 = angle - half_w;
                        let a2 = angle + half_w;
                        let p1 = center + egui::Vec2::new(a1.cos(), a1.sin()) * radius;
                        let p2 = center + egui::Vec2::new(a2.cos(), a2.sin()) * radius;
                        painter.add(egui::Shape::convex_polygon(
                            vec![center, p1, p2],
                            Color32::from_rgba_unmultiplied(255, 255, 255, 80),
                            egui::Stroke::default(),
                        ));

                        // inner bright wedge
                        let hw2 = half_w * 0.5;
                        let b1 = center
                            + egui::Vec2::new((angle - hw2).cos(), (angle - hw2).sin()) * radius;
                        let b2 = center
                            + egui::Vec2::new((angle + hw2).cos(), (angle + hw2).sin()) * radius;
                        painter.add(egui::Shape::convex_polygon(
                            vec![center, b1, b2],
                            Color32::WHITE,
                            egui::Stroke::default(),
                        ));
                    }
                }
            } else {
                // not flashing → always black
                painter.rect_filled(rect, 0.0, Color32::BLACK);
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let opts = NativeOptions::default();
    run_native(
        "Dreamachine",
        opts,
        Box::new(|cc| Ok(Box::new(DreamApp::new(cc)))),
    )
}
