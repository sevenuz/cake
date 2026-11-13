use crate::config::Config;
use crate::markdown;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Wrap};
use std::error::Error;
use std::io::IsTerminal;

/// full screen pager: j/k or arrows scroll, J/K or PageUp/Down by page,
/// g/G top/bottom, any other key quits
fn scroll_view(text: &str) -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let mut offset: u16 = 0;
    let result = (|| -> Result<(), Box<dyn Error>> {
        loop {
            let area = terminal.size()?;
            let lines = markdown::render(text, area.width);
            let w = area.width.max(1) as usize;
            let total = lines.iter().map(|l| l.width().max(1).div_ceil(w)).sum::<usize>() as u16;
            let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
            let max = total.saturating_sub(area.height);
            offset = offset.min(max);
            terminal.draw(|f| f.render_widget(paragraph.scroll((offset, 0)), f.area()))?;
            let page = area.height.saturating_sub(1).max(1);
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                    KeyCode::Down | KeyCode::Char('j') => offset = offset.saturating_add(1),
                    KeyCode::Up | KeyCode::Char('k') => offset = offset.saturating_sub(1),
                    KeyCode::PageDown | KeyCode::Char('J') | KeyCode::Char(' ') => offset = offset.saturating_add(page),
                    KeyCode::PageUp | KeyCode::Char('K') => offset = offset.saturating_sub(page),
                    KeyCode::Char('g') | KeyCode::Home => offset = 0,
                    KeyCode::Char('G') | KeyCode::End => offset = max,
                    _ => return Ok(()),
                },
                _ => (),
            }
        }
    })();
    ratatui::restore();
    result
}

/// renders markdown, silent in `--json` mode; long text in a pager
pub fn print(config: &Config, text: String) -> Result<(), Box<dyn Error>> {
    if crate::out::json() {
        return Ok(());
    }
    // pipes and files get the plain markdown, the scroll view needs a terminal
    if !std::io::stdout().is_terminal() {
        print!("{}", text);
        return Ok(());
    }
    if config.scrollview_threshold > -1 && text.lines().count() > config.scrollview_threshold.try_into()? {
        return scroll_view(&text);
    }
    let width = ratatui::crossterm::terminal::size().map_or(80, |(w, _)| w);
    for line in markdown::render(&text, width) {
        println!("{}", markdown::ansi(&line));
    }
    Ok(())
}
