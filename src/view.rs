// https://docs.rs/crate/termimad/latest/source/examples/scrollable/main.rs
use std::error::Error;
use std::io::{stdout, Write};
use termimad::crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode::*, KeyEvent},
    queue,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use termimad::terminal_size;

use crate::config::Config;

fn view_area() -> termimad::Area {
    let mut area = termimad::Area::full_screen();
    let (width, _) = terminal_size();
    area.pad_for_max_width(width); // we don't want a too wide text column
    area
}

fn scroll_view(skin: termimad::MadSkin, text: String) -> Result<(), termimad::Error> {
    let mut w = stdout(); // we could also have used stderr
    queue!(w, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    queue!(w, Hide)?; // hiding the cursor
    let mut view = termimad::MadView::from(text.clone(), view_area(), skin.clone());
    loop {
        view.write_on(&mut w)?;
        w.flush()?;
        match event::read() {
            Ok(Event::Key(KeyEvent { code, .. })) => match code {
                Up => view.try_scroll_lines(-1),
                Down => view.try_scroll_lines(1),
                PageUp => view.try_scroll_pages(-1),
                PageDown => view.try_scroll_pages(1),
                Char('j') => view.try_scroll_lines(1),
                Char('k') => view.try_scroll_lines(-1),
                Char('J') => view.try_scroll_pages(1),
                Char('K') => view.try_scroll_pages(-1),
                Char('g') => view.try_scroll_pages(-1000),
                Char('G') => view.try_scroll_pages(1000),
                _ => break,
            },
            Ok(Event::Resize(..)) => {
                queue!(w, Clear(ClearType::All))?;
                view.resize(&view_area());
            }
            _ => {}
        }
    }
    terminal::disable_raw_mode()?;
    queue!(w, Show)?; // we must restore the cursor
    queue!(w, LeaveAlternateScreen)?;
    w.flush()?;
    Ok(())
}

pub fn print(config: &Config, text: String) -> Result<(), Box<dyn Error>> {
    let skin = config.build_skin()?;
    if config.scrollview_threshold > -1
        && text.lines().count() > config.scrollview_threshold.try_into()?
    {
        scroll_view(skin, text)?;
    } else {
        skin.print_text(&text);
    }
    Ok(())
}
