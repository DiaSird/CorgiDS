/// CorgiDS - Nintendo DS Emulator
/// Copyright PSISP 2017
/// Licensed under the GPLv3
/// See LICENSE.txt for details
use druid::widget::{Container, Label};
use druid::{AppLauncher, Color, Data, Lens, LocalizedString, UnitData, WidgetExt, WindowDesc};

mod emu_window;
use emu_window::EmuWindow;

/// Application state
#[derive(Clone, Data)]
struct AppState {
    /// Emulator window state
    window_initialized: bool,
}

/// Entry point for CorgiDS emulator
fn main() {
    /// Initialize the main window descriptor
    let main_window = WindowDesc::new(ui_builder())
        .title(LocalizedString::new("CorgiDS"))
        .window_size((800.0, 600.0));

    /// Initial application state
    let initial_state = AppState {
        window_initialized: false,
    };

    /// Launch the application with Druid framework
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch CorgiDS application");
}

/// Build the main UI
fn ui_builder() -> impl druid::Widget<AppState> {
    /// Create main emulator window
    let emu_window = EmuWindow::new();

    /// Initialize emulator
    match emu_window.initialize() {
        Ok(_) => {
            /// Successfully initialized - show the emulator window
            Container::new(Label::new("CorgiDS Emulator Running"))
                .background(Color::rgb8(0x3d, 0x3d, 0x42))
                .expand()
        }
        Err(e) => {
            /// Failed to initialize - show error
            eprintln!("Failed to initialize emulator: {}", e);
            Container::new(Label::new(format!("Error: {}", e)))
                .background(Color::rgb8(0xff, 0x00, 0x00))
                .expand()
        }
    }
}
