use std::io;
use std::panic::PanicInfo;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen};
use crossterm::{event, execute};
use crossterm::{event::EnableMouseCapture, terminal::EnterAlternateScreen};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::canvas::{Canvas, MapResolution, Rectangle};
use tui::widgets::{Block, Borders, List, ListItem, Row, Table};
use tui::{symbols, Frame, Terminal};

use instagram_api::client::InstagramClient;
use instagram_api::models::explore::Location;
use instagram_api::InstagramCredentials;

///
/// The app holds useful information for the ui, such as which accounts are currently logged in, the logs and geotags.
pub struct App<'a> {
    known_credentials: Vec<InstagramCredentials<'a>>,
    logged_in_accounts: Vec<InstagramClient>,
    log: Vec<String>,
    locations: Vec<Location>,
    should_close: bool,
}

/// self explanatory
impl<'a> App<'a> {
    pub fn new(
        known_credentials: Vec<InstagramCredentials<'a>>,
        logged_in_accounts: Vec<InstagramClient>,
        log: Vec<String>,
        locations: Vec<Location>,
    ) -> Self {
        App {
            known_credentials,
            logged_in_accounts,
            log,
            locations,
            should_close: false,
        }
    }

    pub fn add_location(&mut self, location: Location) {
        self.locations.push(location);
    }

    pub fn log(&mut self, response: impl ToString) {
        self.log.push(response.to_string());
    }

    pub fn on_key(&mut self, key: char) {
        match key {
            'c' => self.should_close = true,
            _ => {}
        }
    }
    pub fn on_right(&self) {}
    pub fn on_up(&self) {}
    pub fn on_down(&self) {}

    pub fn on_left(&self) {}
    pub fn logged_in_accounts(&mut self) -> &mut Vec<InstagramClient> {
        &mut self.logged_in_accounts
    }
}

/// <h1>UI Main render loop</h1>
/// In this method, the terminal is being prepared to draw on
/// and the main event and drawing loop is being created.
/// If the users presses q on his keyboard, the app state "should_close" is being set to true
/// and the method will break out of the loop to restore the terminal state and exit the application with exit code 0
pub fn setup_ui(app: &Arc<Mutex<App>>) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    loop {
        if crossterm::event::poll(Duration::new(0, 0))? {
            if let Event::Key(key) = event::read()? {
                let mut app = app.lock().unwrap();
                match key.code {
                    KeyCode::Char(c) => app.on_key(c),
                    KeyCode::Left => app.on_left(),
                    KeyCode::Up => app.on_up(),
                    KeyCode::Right => app.on_right(),
                    KeyCode::Down => app.on_down(),
                    _ => {}
                }
            }
        }

        if app.lock().unwrap().should_close {
            break;
        }

        terminal.draw(|f| {
            ui(f, app);
        })?;
    }

    shutdown_ui();
    Ok(())
}

/// Main function for drawing the ui in here the layouts are created
/// and the corresponding ui components are being called to draw
fn ui<B: Backend>(f: &mut Frame<B>, app: &Arc<Mutex<App>>) {
    let parent_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .margin(1)
        .split(f.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(parent_layout[0]);

    accounts(f, app, chunks.clone());
    post_map(f, app, parent_layout.clone());
    requests(f, app, chunks.clone());
}

/// Draws a table of current accounts (determines which are logged in and which are not)
pub fn accounts<B: Backend>(f: &mut Frame<B>, app: &Arc<Mutex<App>>, chunks: Vec<Rect>) {
    let app = app.lock().unwrap();
    let rows = app
        .known_credentials
        .iter()
        .map(|f| {
            Row::new(vec![format!("{}", f), "✓".to_owned()]).style(Style::default().fg(Color::Cyan))
        })
        .collect::<Vec<_>>();
    let table = Table::new(rows)
        .header(
            Row::new(vec!["Account", "Logged in"])
                .style(Style::default().fg(Color::Cyan))
                .bottom_margin(1),
        )
        .block(Block::default().title("Accounts").borders(Borders::ALL))
        .column_spacing(5)
        .widths(&[Constraint::Length(20), Constraint::Length(20)]);
    f.render_widget(table, chunks[0]);
}

///
/// Draw a world map to display the found Geotags of posts.
/// The rectangle describes the range where posts can origin from. (Simply done by taking the min of the latitudes and longitudes etc.)
/// There are two modes for drawing this map, one is braille and the other is Dot (based on personal preference I chose braille
pub fn post_map<B: Backend>(f: &mut Frame<B>, app: &Arc<Mutex<App>>, chunks: Vec<Rect>) {
    let app = app.lock().unwrap();
    let canvas = Canvas::default()
        .block(Block::default().title("World").borders(Borders::ALL))
        .paint(|ctx| {
            ctx.draw(&tui::widgets::canvas::Map {
                color: Color::White,
                resolution: MapResolution::High,
            });
            ctx.layer();

            let min_x = app
                .locations
                .iter()
                .cloned()
                .map(|f| f.lng)
                .fold(180f64, |acc, x| acc.min(x as f64));
            let min_y = app
                .locations
                .iter()
                .cloned()
                .map(|f| f.lat)
                .fold(180f64, |acc, x| acc.min(x as f64));

            let max_x = app
                .locations
                .iter()
                .cloned()
                .map(|f| f.lng)
                .fold(-180f64, |acc, x| acc.max(x as f64));
            let max_y = app
                .locations
                .iter()
                .cloned()
                .map(|f| f.lat)
                .fold(-180f64, |acc, x| acc.max(x as f64));

            ctx.draw(&Rectangle {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
                color: Color::Yellow,
            });
            for location in &app.locations {
                ctx.print(
                    location.lng as f64,
                    location.lat as f64,
                    Span::styled(
                        format!("X: {}", location.short_name),
                        Style::default().fg(Color::Green),
                    ),
                );
            }
        })
        .marker(symbols::Marker::Dot)
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0]);
    f.render_widget(canvas, chunks[1]);
}

///
/// Print the logs which can be mutated so there might be more each render loop
///
pub fn requests<B: Backend>(f: &mut Frame<B>, app: &Arc<Mutex<App>>, chunks: Vec<Rect>) {
    let items = app
        .lock()
        .unwrap()
        .log
        .iter()
        .map(|f| ListItem::new(f.clone()))
        .collect::<Vec<_>>();
    let list = List::new(items).block(Block::default().title("Log").borders(Borders::ALL));
    f.render_widget(list, chunks[1]);
}

///
/// When something panics, restore the original terminal state and print the error!
///
pub fn ui_panic_hook(info: &PanicInfo<'_>) {
    shutdown_ui();
    println!("{}", info)
}
///
/// Capture ctrl c to restore the terminal state before exiting the application
///
pub fn ctrl_c_hook() {
    shutdown_ui();
    exit(0);
}

/// Restores the original terminal state
pub fn shutdown_ui() {
    // restore terminal
    disable_raw_mode().unwrap();
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
}
