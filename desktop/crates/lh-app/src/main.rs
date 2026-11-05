//! LuxuryHub desktop client.
//!
//! The whole application is one window: a navigation column and the content of
//! the selected section. Screens are placeholders for now — the point of this
//! file is the shape: state lives in [`AppShell`], rendering is pure, and the
//! next section is one match arm away.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use gpui::{
    App, AsyncApp, Bounds, ClickEvent, Context, IntoElement, ParentElement, Render, Styled, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_platform::application;
use lh_ui::{
    SIDEBAR_WIDTH, badge, card, eyebrow, heading, muted, nav_item, palette, separator, space,
};

/// How often the live theme file is checked while the client runs.
const THEME_POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Sections of the left navigation. Order matters: the index is the state.
const SECTIONS: [&str; 4] = ["Каталог", "Сделки", "Консьерж", "Настройки"];

/// Root view: owns the selection and renders the shell.
struct AppShell {
    active: usize,
}

impl AppShell {
    fn new() -> Self {
        Self { active: 0 }
    }

    fn select(&mut self, index: usize) {
        self.active = index;
    }
}

impl Render for AppShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = palette();

        div()
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            .bg(p.background)
            .text_color(p.text)
            .child(self.sidebar(cx))
            .child(self.content())
    }
}

impl AppShell {
    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let p = palette();

        let mut nav = div().flex().flex_col().gap_1();
        for (index, title) in SECTIONS.iter().enumerate() {
            nav = nav.child(
                div()
                    .id(("nav", index))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                        this.select(index);
                        cx.notify();
                    }))
                    .child(nav_item(*title, index == self.active)),
            );
        }

        div()
            .flex()
            .flex_col()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .p_6()
            .gap_6()
            .bg(p.surface)
            .border_1()
            .border_color(p.border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("LUXURYHUB"),
                    )
                    .child(muted("закрытый маркетплейс")),
            )
            .child(separator())
            .child(nav)
            .child(div().flex_1())
            .child(separator())
            .child(muted("v0.1.0 · skeleton"))
    }

    fn content(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .p_6()
            .gap_6()
            .child(self.header())
            .child(self.placeholder_grid())
    }

    fn header(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(eyebrow("Раздел"))
                    .child(heading(SECTIONS[self.active])),
            )
            .child(badge("каркас"))
    }

    fn placeholder_grid(&self) -> impl IntoElement {
        let placeholders: [(&str, &str); 3] = [
            ("Первый экран", "Здесь появится список лотов из API."),
            ("Второй экран", "Карточка лота, галерея, цена."),
            ("Третий экран", "Сделка: оффер, статус, история."),
        ];

        let mut grid = div().flex().flex_row().gap_4();
        for (title, description) in placeholders {
            grid = grid.child(
                card()
                    .flex_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(title),
                    )
                    .child(muted(description))
                    .child(div().h(px(space::XXXL)))
                    .child(muted("данные подключаются на фазе 2")),
            );
        }
        grid
    }
}

fn main() {
    application().run(|cx: &mut App| {
        // The theme is read before the first frame, so the window opens already
        // in the colours of `theme.ini` instead of flashing the built-in ones.
        let theme = theme_path();
        if let Some(path) = theme.as_deref() {
            apply_theme(path);
        }

        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

        cx.open_window(
            WindowOptions::new().window_bounds(Some(WindowBounds::Windowed(bounds))),
            |window, cx| {
                window.set_window_title("LuxuryHub");
                cx.new(|_| AppShell::new())
            },
        )
        .expect("окно не создалось");

        cx.activate(true);

        if let Some(path) = theme {
            watch_theme(cx, path);
        }
    });
}

/// Where the live theme lives: `LUXURYHUB_THEME` if it is set, otherwise
/// `desktop/theme.ini` beside the crates in a debug build. Release builds ship
/// with the built-in palette and read a file only when told to.
fn theme_path() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("LUXURYHUB_THEME") {
        return Some(PathBuf::from(explicit));
    }

    if cfg!(debug_assertions) {
        // `crates/lh-app` → `desktop/theme.ini`; `ancestors` keeps the path free
        // of `..` so it reads plainly in the log when the file has a typo.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)?
            .join("theme.ini");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn apply_theme(path: &Path) {
    match lh_ui::load_palette(path) {
        Ok(theme) => {
            lh_ui::set_palette(theme.palette);
            for warning in theme.warnings {
                eprintln!("тема · {}: {warning}", path.display());
            }
        }
        Err(error) => eprintln!("тема · {}: {error}", path.display()),
    }
}

/// Watches the theme file and repaints the window when it changes on disk.
///
/// Rust code cannot be swapped into a running native binary, so colours are the
/// one layer this can cover; everything else needs a rebuild (`make desktop-watch`).
fn watch_theme(cx: &mut App, path: PathBuf) {
    let mut last = modified(&path);

    cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            cx.background_executor().timer(THEME_POLL_INTERVAL).await;

            let current = modified(&path);
            if current == last {
                continue;
            }
            last = current;

            apply_theme(&path);
            cx.update(|cx| cx.refresh_windows());
        }
    })
    .detach();
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}
