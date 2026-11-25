//! dd_emailforge window. Disk commands print and return. A path, or no
//! command, opens the editor.

mod outline;
mod settings;
mod shell;
mod shortcuts;
mod theme_load;
mod windows;

#[cfg(test)]
mod tests;

use std::process::ExitCode;

use dd_emailforge::cli;
use gpui::{App, AppContext, Focusable};
use gpui_component::Root;
use shell::{Shell, ShellOptions};

fn main() -> ExitCode {
    let cli = cli::parse();
    match cli.command {
        None => open_window(cli.path),
        Some(cli::Command::Tui { path }) => open_window(path),
        Some(command) => cli::run_command(command),
    }
}

fn open_window(path: Option<std::path::PathBuf>) -> ExitCode {
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_component::init(cx);
        let path = path.clone();
        cx.spawn(async move |cx| {
            cx.update(|cx| {
                let bounds = windows::centered_bounds(cx);
                cx.open_window(windows::main_window_options(bounds), |window, cx| {
                    let view = cx.new(|cx| {
                        let shell = Shell::open(ShellOptions::production(path), cx);
                        window.focus(&shell.focus_handle(cx), cx);
                        shell
                    });
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("open window");
            });
        })
        .detach();
    });
    ExitCode::SUCCESS
}
