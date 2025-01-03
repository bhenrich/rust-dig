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
};
use rand::random;

const GAME_WIDTH: usize = 60;
const GAME_HEIGHT: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CellState {
    Empty,
    Player1Cell,
    Player2Cell,
    Stone,
    Water,
    Wall,
}

struct GameState {
    players: Vec<Player>,
    grid: [[CellState; GAME_WIDTH]; GAME_HEIGHT],
    debug_log: Vec<String>, // Stores debug messages
    debug_panel_enabled: bool, // Tracks whether the debug panel is enabled
}

struct Item {
    item_id: u32,
    item_name: String,
    item_count: usize,
}

struct Player {
    player_id: u32,
    x: usize,
    y: usize,
    inventory: Vec<Item>,
}

impl GameState {

    // Log a debug message
    fn log_debug_message(&mut self, message: String) {
        // Limit the number of messages
        if self.debug_log.len() >= 10 {
            self.debug_log.remove(0); // Keep the last 10 messages
        }
        self.debug_log.push(message);
    }

    // Clear all debug messages
    fn clear_debug_log(&mut self) {
        self.debug_log.clear();
    }

    fn generate_map(&mut self) {
        // Reset the grid to empty
        self.grid = [[CellState::Empty; GAME_WIDTH]; GAME_HEIGHT];

        // Add walls around the edges
        for i in 0..GAME_WIDTH {
            self.grid[0][i] = CellState::Wall;
            self.grid[GAME_HEIGHT - 1][i] = CellState::Wall;
        }
        for i in 0..GAME_HEIGHT {
            self.grid[i][0] = CellState::Wall;
            self.grid[i][GAME_WIDTH - 1] = CellState::Wall;
        }

        // Generate water using Perlin noise
        let water_perlin = Perlin::new(random());
        for y in 1..GAME_HEIGHT - 1 {
            for x in 1..GAME_WIDTH - 1 {
                let noise_value = water_perlin.get([x as f64 / 10.0, y as f64 / 10.0, 0.0]);
                if noise_value > 0.4 {
                    self.grid[y][x] = CellState::Water;
                }
            }
        }

        // Generate stones using Perlin noise
        let stone_perlin = Perlin::new(random());
        for y in 1..GAME_HEIGHT - 1 {
            for x in 1..GAME_WIDTH - 1 {
                let noise_value = stone_perlin.get([x as f64 / 10.0, y as f64 / 10.0, 0.0]);
                if noise_value > 0.2 && self.grid[y][x] == CellState::Empty {
                    self.grid[y][x] = CellState::Stone;
                }
            }
        }

        // Clear player surrounding area
        for player in &self.players {
            let offsets = [
                (0, 0),
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ];
            for (dy, dx) in offsets {
                let ny = (player.y as isize + dy) as usize;
                let nx = (player.x as isize + dx) as usize;
                if ny > 0 && ny < GAME_HEIGHT && nx > 0 && nx < GAME_WIDTH {
                    self.grid[ny][nx] = CellState::Empty;
                }
            }
        }

        // Place players back on the map
        self.grid[self.players[0].y][self.players[0].x] = CellState::Player1Cell;
        self.grid[self.players[1].y][self.players[1].x] = CellState::Player2Cell;
    }

    // Initializes a new game state
    fn new() -> Self {
        let mut game_state = GameState {
            players: vec![
                Player {
                    player_id: 1,
                    x: 2,
                    y: 2,
                    inventory: vec![
                        Item { item_id: 1, item_name: "Stone".to_string(), item_count: 0 },
                        Item { item_id: 2, item_name: "Wood".to_string(), item_count: 0 },
                    ],
                },
                Player {
                    player_id: 2,
                    x: GAME_WIDTH - 3,
                    y: GAME_HEIGHT - 3,
                    inventory: vec![
                        Item { item_id: 1, item_name: "Stone".to_string(), item_count: 0 },
                        Item { item_id: 2, item_name: "Wood".to_string(), item_count: 0 },
                    ],
                },
            ],
            grid: [[CellState::Empty; GAME_WIDTH]; GAME_HEIGHT],
            debug_log: Vec::new(),
            debug_panel_enabled: false,
        };
        game_state.generate_map();
        game_state
    }

    // Regenerate the game state while preserving player positions
    pub fn regenerate_game_state(&mut self) {
        // Reset player inventories
        for player in &mut self.players {
            player.inventory[0].item_count = 0;
        }
        // Regenerate the map
        self.generate_map();
    }

    // Handles player input
    fn handle_input(&mut self, key: KeyEvent) {
        let player1_modifiers = key.modifiers.contains(KeyModifiers::ALT);
        let player2_modifiers = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('w') => {
                if player1_modifiers {
                    self.place_stone(self.players[0].x, self.players[0].y - 1, 0);
                } else {
                    self.move_player(0, -1, 0);
                }
            }
            KeyCode::Char('s') => {
                if player1_modifiers {
                    self.place_stone(self.players[0].x, self.players[0].y + 1, 0);
                } else {
                    self.move_player(0, 1, 0);
                }
            }
            KeyCode::Char('a') => {
                if player1_modifiers {
                    self.place_stone(self.players[0].x - 1, self.players[0].y, 0);
                } else {
                    self.move_player(-1, 0, 0);
                }
            }
            KeyCode::Char('d') => {
                if player1_modifiers {
                    self.place_stone(self.players[0].x + 1, self.players[0].y, 0);
                } else {
                    self.move_player(1, 0, 0);
                }
            }
            KeyCode::Up => {
                if player2_modifiers {
                    self.place_stone(self.players[1].x, self.players[1].y - 1, 1);
                } else {
                    self.move_player(0, -1, 1);
                }
            }
            KeyCode::Down => {
                if player2_modifiers {
                    self.place_stone(self.players[1].x, self.players[1].y + 1, 1);
                } else {
                    self.move_player(0, 1, 1);
                }
            }
            KeyCode::Left => {
                if player2_modifiers {
                    self.place_stone(self.players[1].x - 1, self.players[1].y, 1);
                } else {
                    self.move_player(-1, 0, 1);
                }
            }
            KeyCode::Right => {
                if player2_modifiers {
                    self.place_stone(self.players[1].x + 1, self.players[1].y, 1);
                } else {
                    self.move_player(1, 0, 1);
                }
            }
            _ => {}
        }
    }

    // Moves the player in the given direction
    fn move_player(&mut self, dx: i32, dy: i32, player_index: usize) {
        let new_x = (self.players[player_index].x as i32 + dx) as usize;
        let new_y = (self.players[player_index].y as i32 + dy) as usize;

        if new_x > 0 && new_y > 0 && new_x < GAME_WIDTH - 1 && new_y < GAME_HEIGHT - 1 {
            match self.grid[new_y][new_x] {
                CellState::Empty => {
                    self.grid[self.players[player_index].y][self.players[player_index].x] = CellState::Empty;
                    self.players[player_index].x = new_x;
                    self.players[player_index].y = new_y;
                    self.grid[new_y][new_x] = if player_index == 0 {
                        CellState::Player1Cell
                    } else {
                        CellState::Player2Cell
                    };
                }
                CellState::Stone => {
                    self.players[player_index].inventory[0].item_count += 1; // Increment stone count
                    self.grid[new_y][new_x] = CellState::Empty; // Remove the stone from the map
                    self.grid[self.players[player_index].y][self.players[player_index].x] = CellState::Empty;
                    self.players[player_index].x = new_x;
                    self.players[player_index].y = new_y;
                    self.grid[new_y][new_x] = if player_index == 0 {
                        CellState::Player1Cell
                    } else {
                        CellState::Player2Cell
                    };
                    self.log_debug_message(format!(
                        "Player {} collected a stone. Total: {}",
                        player_index + 1,
                        self.players[player_index].inventory[0].item_count
                    ));
                }
                _ => {}
            }
        }
    }

    fn place_stone(&mut self, tx: usize, ty: usize, player_index: usize) {
        let x = self.players[player_index].x;
        let y = self.players[player_index].y;

        if tx >= GAME_WIDTH || ty >= GAME_HEIGHT {
            self.log_debug_message(format!("Out of bounds: ({}, {})", tx, ty));
            return;
        }

        if self.grid[ty][tx] != CellState::Empty {
            self.log_debug_message(format!("Cannot place stone: Cell not empty at ({}, {})", tx, ty));
            return;
        }

        if self.players[player_index].inventory[0].item_count <= 0 {
            self.log_debug_message(format!(
                "Player {} has no stones to use.",
                player_index + 1
            ));
            return;
        }

        if (tx as isize - x as isize).abs() <= 1 && (ty as isize - y as isize).abs() <= 1 {
            self.grid[ty][tx] = CellState::Stone;
            self.players[player_index].inventory[0].item_count -= 1;
            self.log_debug_message(format!(
                "Player {} placed stone at ({}, {}).",
                player_index + 1,
                tx,
                ty
            ));
        } else {
            self.log_debug_message(format!(
                "Placement at ({}, {}) is not adjacent to Player {}.",
                tx, ty, player_index + 1
            ));
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

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') => game_state.regenerate_game_state(),
                    KeyCode::F(1) => game_state.debug_panel_enabled = !game_state.debug_panel_enabled, // Toggle debug panel
                    _ => game_state.handle_input(key),
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, game_state: &GameState) {
    // Split the terminal area into main game area and debug/logging area
    let vertical_chunks = if game_state.debug_panel_enabled {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(3, 4), // Main game and inventories take 3/4 of space
                Constraint::Ratio(1, 4), // Debug panel takes 1/4 of space
            ])
            .split(f.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1)]) // Use all available vertical space
            .split(f.area())
    };

    // Split the main game area into the game map and player inventories
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(2, 3), // Game map takes 2/3 of space
            Constraint::Ratio(1, 3), // Inventory takes 1/3 of space
        ])
        .split(vertical_chunks[0]);

    // Split the inventory chunk into separate sections for Player 1 and Player 2
    let inventory_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 2), // Player 1 inventory takes top half
            Constraint::Ratio(1, 2), // Player 2 inventory takes bottom half
        ])
        .split(horizontal_chunks[1]);

    // Render the game map in the first horizontal chunk
    let mut game_items = Vec::new();
    for y in 0..GAME_HEIGHT {
        let mut row = String::new();
        for x in 0..GAME_WIDTH {
            let cell = game_state.grid[y][x];
            let symbol = match cell {
                CellState::Empty => " ",
                CellState::Player1Cell => "1",
                CellState::Player2Cell => "2",
                CellState::Stone => "#",
                CellState::Water => ".",
                CellState::Wall => "█",
            };
            row.push_str(symbol);
        }
        game_items.push(ListItem::new(row));
    }

    let game_list = List::new(game_items)
        .block(Block::default().borders(Borders::ALL).title("MAP"))
        .style(Style::default().fg(Color::White));

    f.render_widget(game_list, horizontal_chunks[0]);

    // Render Player 1's inventory in the top half of the inventory chunk
    let player1_inventory: Vec<ListItem> = game_state.players[0]
        .inventory
        .iter()
        .map(|item| {
            ListItem::new(format!(
                "{}: {}",
                item.item_name, item.item_count
            ))
        })
        .collect();

    let inventory_panel1 = List::new(player1_inventory)
        .block(Block::default().borders(Borders::ALL).title("P1 INV"))
        .style(Style::default().fg(Color::Blue));

    f.render_widget(inventory_panel1, inventory_chunks[0]);

    // Render Player 2's inventory in the bottom half of the inventory chunk
    let player2_inventory: Vec<ListItem> = game_state.players[1]
        .inventory
        .iter()
        .map(|item| {
            ListItem::new(format!(
                "{}: {}",
                item.item_name, item.item_count
            ))
        })
        .collect();

    let inventory_panel2 = List::new(player2_inventory)
        .block(Block::default().borders(Borders::ALL).title("P2 INV"))
        .style(Style::default().fg(Color::Red));

    f.render_widget(inventory_panel2, inventory_chunks[1]);

    // Render the debug panel at the bottom (if enabled)
    if game_state.debug_panel_enabled {
        let debug_items: Vec<ListItem> = game_state
            .debug_log
            .iter()
            .map(|msg| ListItem::new(msg.clone()))
            .collect();

        let debug_list = List::new(debug_items)
            .block(Block::default().borders(Borders::ALL).title("DEBUG"))
            .style(Style::default().fg(Color::Yellow));

        f.render_widget(debug_list, vertical_chunks[1]);
    }
}