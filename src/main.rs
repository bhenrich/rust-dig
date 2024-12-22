use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use noise::{NoiseFn, Perlin};
use std::{
    error::Error,
    io,
    time::{Duration, SystemTime},
};
use std::thread::sleep;
use rand::random;

const GAME_WIDTH: usize = 60;
const GAME_HEIGHT: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CellState {
    Empty,
    Player,
    Stone,
    Water,
    Wall,
}

struct GameState {
    player_x: usize,
    player_y: usize,
    inventory: usize,
    grid: [[CellState; GAME_WIDTH]; GAME_HEIGHT],
}

impl GameState {
    // create a new game state with the player at the center
    fn new() -> Self {
        let mut grid = [[CellState::Empty; GAME_WIDTH]; GAME_HEIGHT];
        // initialize the grid with some walls and stones
        for i in 0..GAME_WIDTH {
            grid[0][i] = CellState::Wall;
            grid[GAME_HEIGHT - 1][i] = CellState::Wall;
        }
        for i in 0..GAME_HEIGHT {
            grid[i][0] = CellState::Wall;
            grid[i][GAME_WIDTH - 1] = CellState::Wall;
        }

        // first layer river generation using perlin noise
        let perlin = Perlin::new(random());
        for y in 1..GAME_HEIGHT - 1 {
            for x in 1..GAME_WIDTH - 1 {
                let noise_value = perlin.get([x as f64 / 10.0, y as f64 / 10.0, 0.0]);
                if noise_value > 0.4 {
                    grid[y][x] = CellState::Water;
                }
            }
        }
        
        
        // second layer stone generaation
        let perlin = Perlin::new(random());
        for y in 1..GAME_HEIGHT - 1 {
            for x in 1..GAME_WIDTH - 1 {
                let noise_value = perlin.get([x as f64 / 10.0, y as f64 / 10.0, 0.0]);
                if noise_value > 0.2 {
                    grid[y][x] = CellState::Stone;
                }
            }
        }
        
        // make sure 1x1 ring around player is clear including corners
        grid[GAME_HEIGHT / 2]    [GAME_WIDTH / 2]     = CellState::Empty;
        grid[GAME_HEIGHT / 2 - 1][GAME_WIDTH / 2]     = CellState::Empty;
        grid[GAME_HEIGHT / 2 + 1][GAME_WIDTH / 2]     = CellState::Empty;
        grid[GAME_HEIGHT / 2]    [GAME_WIDTH / 2 - 1] = CellState::Empty;
        grid[GAME_HEIGHT / 2]    [GAME_WIDTH / 2 + 1] = CellState::Empty;
        grid[GAME_HEIGHT / 2 - 1][GAME_WIDTH / 2 - 1] = CellState::Empty;
        grid[GAME_HEIGHT / 2 + 1][GAME_WIDTH / 2 - 1] = CellState::Empty;
        grid[GAME_HEIGHT / 2 - 1][GAME_WIDTH / 2 + 1] = CellState::Empty;
        grid[GAME_HEIGHT / 2 + 1][GAME_WIDTH / 2 + 1] = CellState::Empty;
        
        
        grid[GAME_HEIGHT / 2][GAME_WIDTH / 2] = CellState::Player;
        
        GameState {
            player_x: GAME_WIDTH / 2,
            player_y: GAME_HEIGHT / 2,
            inventory: 0,
            grid,
        }
    }

    // handle player input
    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('w') | KeyCode::Char('W') => {
                self.move_player(0, -1)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.move_player(0, 1)
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.move_player(-1, 0)
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.move_player(1, 0)
            }
            // arrow keys to build
            KeyCode::Up => {
                self.place_stone(self.player_x, self.player_y - 1)
            }
            KeyCode::Down => {
                self.place_stone(self.player_x, self.player_y + 1)
            }
            KeyCode::Left => {
                self.place_stone(self.player_x - 1, self.player_y)
            }
            KeyCode::Right => {
                self.place_stone(self.player_x + 1, self.player_y)
            }
            _ => {}
        }
    }

    // move the player in the given direction
    fn move_player(&mut self, dx: i32, dy: i32) {
        let new_x = (self.player_x as i32 + dx) as usize;
        let new_y = (self.player_y as i32 + dy) as usize;
        if new_x > 0
            && new_x < GAME_WIDTH - 1
            && new_y > 0
            && new_y < GAME_HEIGHT - 1
        {
            match self.grid[new_y][new_x] {
                CellState::Empty => {
                    self.grid[self.player_y][self.player_x] = CellState::Empty;
                    self.player_x = new_x;
                    self.player_y = new_y;
                    self.grid[self.player_y][self.player_x] = CellState::Player;
                }
                CellState::Stone => {
                    self.grid[self.player_y][self.player_x] = CellState::Empty;
                    self.player_x = new_x;
                    self.player_y = new_y;
                    self.grid[self.player_y][self.player_x] = CellState::Player;
                    self.inventory += 1;
                }
                _ => {}
            }
        }
    }
    
    fn update_cell(&mut self, x: usize, y: usize, cell: CellState) {
        self.grid[y][x] = cell;
    }
    
    pub fn regenerate_game_state(&mut self) {
        let perlin = Perlin::new(random());
        for y in 1..GAME_HEIGHT - 1 {
            for x in 1..GAME_WIDTH - 1 {
                let noise_value = perlin.get([x as f64 / 10.0, y as f64 / 10.0, 0.0]);
                if noise_value > 0.2 {
                    self.update_cell(x, y, CellState::Stone);
                } else {
                    self.update_cell(x, y, CellState::Empty);
                }
            }
        }
        
        self.update_cell(self.player_x, self.player_y, CellState::Empty);
        self.player_x = GAME_WIDTH / 2;
        self.player_y = GAME_HEIGHT / 2;
        self.update_cell(self.player_x, self.player_y, CellState::Player);
        
        self.inventory = 0;

        // make sure 1x1 ring around player is clear including corners
        self.update_cell(self.player_x, self.player_y, CellState::Empty);
        self.update_cell(self.player_x - 1, self.player_y, CellState::Empty);
        self.update_cell(self.player_x + 1, self.player_y, CellState::Empty);
        self.update_cell(self.player_x, self.player_y - 1, CellState::Empty);
        self.update_cell(self.player_x, self.player_y + 1, CellState::Empty);
        self.update_cell(self.player_x - 1, self.player_y - 1, CellState::Empty);
        self.update_cell(self.player_x + 1, self.player_y - 1, CellState::Empty);
        self.update_cell(self.player_x - 1, self.player_y + 1, CellState::Empty);
        self.update_cell(self.player_x + 1, self.player_y + 1, CellState::Empty);
    }
    
    fn place_stone(&mut self, tx: usize, ty: usize) {
        let x = self.player_x;
        let y = self.player_y;
        
        if (tx == x && ty == y) || (tx == x - 1 && ty == y) || (tx == x + 1 && ty == y) || (tx == x && ty == y - 1) || (tx == x && ty == y + 1) {
            if self.inventory <= 0 {
                return;
            } else {
                self.update_cell(tx, ty, CellState::Stone);
                self.inventory -= 1;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut game_state = GameState::new();
    run_app(&mut terminal, &mut game_state)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    game_state: &mut GameState,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, game_state))?;

        if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
            if kind == KeyEventKind::Press { 
                match code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') => game_state.regenerate_game_state(),
                    _ => game_state.handle_input(code),
                }
                
                // sleep(Duration::from_millis(100));
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, game_state: &GameState) {
    // create two chunks with equal horizontal space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(f.size());

    // Iterate through grid and create list items
    let mut game_items = Vec::new();
    for y in 0..GAME_HEIGHT {
        let mut row = String::new();
        for x in 0..GAME_WIDTH {
            let cell = game_state.grid[y][x];
            let symbol = match cell {
                CellState::Empty => " ",
                CellState::Player => "@",
                CellState::Stone => "#",
                CellState::Water => ".",
                CellState::Wall => "█",
            };
            row.push_str(symbol);
        }
        game_items.push(ListItem::new(row));
    }

    // Create a List from all list items
    let game_list = List::new(game_items)
        .block(Block::default().borders(Borders::ALL).title("MAP"))
        .style(Style::default().fg(Color::White));

    // Render the game list
    f.render_widget(game_list, chunks[0]);

    let inventory_items = vec![
        ListItem::new(format!("Stone: {}", game_state.inventory)),
    ];

    let inventory_list = List::new(inventory_items)
        .block(Block::default().borders(Borders::ALL).title("INV"))
        .style(Style::default().fg(Color::White));

    // Render the inventory list
    f.render_widget(inventory_list, chunks[1]);
}