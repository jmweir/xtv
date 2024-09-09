use std::io;
use tui::{
    Frame,
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders},
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use client_lib::{
    KeyCode as XTVKeyCode,
    XTVClient
};

type CrosstermTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = create_terminal()?;
    run_app(&mut terminal).await?;
    shutdown(&mut terminal)?;
    Ok(())
}

fn create_terminal() -> Result<CrosstermTerminal, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    return Ok(Terminal::new(CrosstermBackend::new(stdout))?);
}

fn shutdown(terminal: &mut CrosstermTerminal) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_app(terminal: &mut CrosstermTerminal) -> Result<(), Box<dyn std::error::Error>> {
    let client = XTVClient::new()?;
    let device = client.lookup_device("Media Room").await?;

    loop {
        terminal.draw(ui)?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => { return Ok(()); }
                KeyCode::Char(' ') => { client.press_key(XTVKeyCode::Pause, &device).await?; }
                _ => {}
            }            
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>) {
    let size = f.size();

    let block = Block::default()
        .title("XTV")
        .borders(Borders::ALL);

    f.render_widget(block, size);
}
