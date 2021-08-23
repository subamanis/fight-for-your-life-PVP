use std::ops::Index;
use std::path::{PathBuf};

use ggez::conf::{WindowMode, WindowSetup};
use ggez::{Context, ContextBuilder, GameResult, timer};
use ggez::graphics::{self, Color, DrawParam, FillOptions, MeshBuilder, PxScale, Rect, StrokeOptions};
use ggez::event::{self, EventHandler, KeyCode, KeyMods, quit};

use lazy_static::lazy_static;

type Point2f = ggez::mint::Point2<f32>;
type Point2u = ggez::mint::Point2<usize>;
type Point2i = ggez::mint::Point2<isize>;

// BLOCK_SIZE is a common perfect divisor of WINDOW_X - 2*HP_BAR_WIDTH and WINDOW_Y.
// Horizontal and vertical blocks are the division (WINDOW_X - 2*HP_BAR_WIDTH) / BLOCK_SIZE and WINDOW_Y / BLOCK_SIZE respectively
// These hardcoded values should only be changed if the above conditions are met.
const WINDOW_X     : f32 = 1502.0;
const WINDOW_Y     : f32 = 952.0;
const HP_BAR_WIDTH : f32 = 20.0;
const BLOCK_SIZE   : f32 = 34.0;
const HORIZONTAL_BLOCKS : usize = 43;
const VERTICAL_BLOCKS   : usize = 28;

// const WINDOW_X     : f32 = 1479.0;
// const WINDOW_Y     : f32 = 957.0;
// const HP_BAR_WIDTH : f32 = 20.0;
// const BLOCK_SIZE   : f32 = 29.0;
// const HORIZONTAL_BLOCKS : usize = 51;
// const VERTICAL_BLOCKS   : usize = 33;

const AREA_1_X : f32 = (HORIZONTAL_BLOCKS/8) as f32 * BLOCK_SIZE + HP_BAR_WIDTH;
const AREA_2_X_OFFSET : f32 = if HORIZONTAL_BLOCKS % 2 == 0 {1.0} else {0.0};
const AREA_2_X : f32 = ((2*HORIZONTAL_BLOCKS/3) as f32 + AREA_2_X_OFFSET) * BLOCK_SIZE + HP_BAR_WIDTH;
const AREA_WIDTH  : f32 = (HORIZONTAL_BLOCKS/4) as f32 * BLOCK_SIZE;
const AREA_LENGTH : f32 = (VERTICAL_BLOCKS-2) as f32 * BLOCK_SIZE;

const DESIRED_FPS: u32 = 60;
const GENERATION_CALCULATION_DELAY: f32 = 0.15;


lazy_static! {
    static ref LIFE_COLORS:[Color; 6] = [Color::from_rgb(105, 212, 76), Color::from_rgb(151, 212, 76), Color::from_rgb(203, 212, 76),
                                         Color::from_rgb(219, 190, 75), Color::from_rgb(219, 157, 75), Color::from_rgb(217, 80, 56)];

    static ref STROKE_MODE_1: graphics::DrawMode = graphics::DrawMode::Stroke(StrokeOptions::default().with_line_width(1.0));
    static ref STROKE_MODE_2: graphics::DrawMode = graphics::DrawMode::Stroke(StrokeOptions::default().with_line_width(2.0));
    static ref FILL_MODE    : graphics::DrawMode = graphics::DrawMode::Fill(FillOptions::default());
}

#[macro_use]
macro_rules! pointu {
    ($x:expr,$y:expr) => {
        Point2u{x:$x,y:$y}
    }
}
#[macro_use]
macro_rules! pointi {
    ($x:expr,$y:expr) => {
        Point2i{x:$x,y:$y}
    }
}
#[macro_use]
macro_rules! pointf {
    ($x:expr,$y:expr) => {
        Point2f{x:$x,y:$y}
    }
}

#[derive(Debug, PartialEq, Clone)]
enum PlayerNum {
    ONE,
    TWO
}

#[derive(Debug)]
enum Direction {
    UP,
    RIGHT,
    LEFT,
    DOWN
}

#[derive(Debug,PartialEq)]
enum GameState {
    PLAYING,
    PAUSE_MENU,
    WINNER_SCREEN
}

#[derive(Debug)]
struct Player {
    pub player_num: PlayerNum,
    pub input_state: InputState,
    pub movement_cooldown_time: f32,
    pub life_color_index: usize,
    pub hovering_square: Point2u,
    pub selected_squares: Vec<Point2u>,
    _x_left_bound: usize,
    _x_right_bound: usize,
    _y_upper_bound: usize,
    _y_lower_bound: usize,
}

#[derive(Debug)]
struct InputState {
    movement_vector: Point2i,
    mark_pressed: bool,
    deploy_pressed: bool
}

#[derive(Debug)]
struct Game {
    state: GameState,
    timer: f32,
    player1: Player,
    player2: Player,
    winner: Option<PlayerNum>,
    board: [[bool; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]
}


impl EventHandler<ggez::GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.state != GameState::PLAYING {return Ok(())}

        let seconds = 1.0 / DESIRED_FPS as f32;
        self.timer += seconds;
        
        while timer::check_update_time(ctx, DESIRED_FPS) {
            if self.timer < GENERATION_CALCULATION_DELAY {continue}
            self.timer = 0.0;

            let next_gen_info = calculate_next_generation(&mut self.board);
            
            make_damage_calculations(ctx, self, next_gen_info.1);

            self.board = next_gen_info.0;
        }

        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb(170,170,170));

        match self.state {
            GameState::PLAYING => draw_board(ctx, self)?,
            GameState::PAUSE_MENU => draw_pause_menu(ctx, self)?,
            GameState::WINNER_SCREEN => draw_winner_screen(ctx, self)?
        }
        
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, repeat: bool) {
        if repeat {return}
        
        match key {
            KeyCode::Escape => {
                ggez::event::quit(ctx)
            },
            KeyCode::P => {
                if self.state == GameState::PLAYING {
                    self.state = GameState::PAUSE_MENU 
                } else if self.state == GameState::PAUSE_MENU {
                    self.state = GameState::PLAYING 
                }
            },
            KeyCode::R => { 
                if self.state == GameState::WINNER_SCREEN {
                    self.reset();
                }
            },
            KeyCode::B => { 
                if self.state == GameState::WINNER_SCREEN {
                    self.state = GameState::PLAYING
                } else if self.state == GameState::PLAYING {
                    self.state = GameState::WINNER_SCREEN 
                }
            },
            // Player1
            KeyCode::W => {
                let amount = if mods.contains(KeyMods::ALT) {3} else {1};
                self.player1.move_hover(Direction::UP, amount)
            },
            KeyCode::D => {
                let amount = if mods.contains(KeyMods::ALT) {3} else {1};
                self.player1.move_hover(Direction::RIGHT, amount)
            },
            KeyCode::S => {
                let amount = if mods.contains(KeyMods::ALT) {3} else {1};
                self.player1.move_hover(Direction::DOWN, amount)
            },
            KeyCode::A => {
                let amount = if mods.contains(KeyMods::ALT) {3} else {1};
                self.player1.move_hover(Direction::LEFT, amount)
            },
            KeyCode::C => {
                let index = self.player1.selected_squares.iter().position(|x| *x == self.player1.hovering_square);
                if let Some(i) = index {
                    self.player1.selected_squares.remove(i);
                } else {
                    self.player1.selected_squares.push(self.player1.hovering_square);
                }
            },
            KeyCode::Space => {
                for p in self.player1.selected_squares.iter() {
                    self.board[p.y][p.x] = true;
                }
                self.player1.selected_squares.clear();
            },
            //Player2
            KeyCode::Up => {
                let amount = if mods.contains(KeyMods::CTRL) {3} else {1};
                self.player2.move_hover(Direction::UP, amount)
            },
            KeyCode::Right => {
                let amount = if mods.contains(KeyMods::CTRL) {3} else {1};
                self.player2.move_hover(Direction::RIGHT, amount)
            },
            KeyCode::Down => {
                let amount = if mods.contains(KeyMods::CTRL) {3} else {1};
                self.player2.move_hover(Direction::DOWN, amount)
            },
            KeyCode::Left => {
                let amount = if mods.contains(KeyMods::CTRL) {3} else {1};
                self.player2.move_hover(Direction::LEFT, amount)
            },
            KeyCode::RShift => {
                let index = self.player2.selected_squares.iter().position(|x| *x == self.player2.hovering_square);
                if let Some(i) = index {
                    self.player2.selected_squares.remove(i);
                } else {
                    self.player2.selected_squares.push(self.player2.hovering_square);
                }
            },
            KeyCode::Return => {
                for p in self.player2.selected_squares.iter() {
                    self.board[p.y][p.x] = true;
                }
                self.player2.selected_squares.clear();
            },
            _ => ()
        }
    }
}


fn draw_board(ctx: &mut Context, game: &mut Game) -> GameResult<()> {
    let mut mb = MeshBuilder::new();

    // the 2 HP bars
    mb.rectangle(
        *FILL_MODE,
        Rect::new(0.0, 0.0, HP_BAR_WIDTH, WINDOW_Y), 
        LIFE_COLORS[game.player1.life_color_index] 
    )?;
    mb.rectangle(
        *FILL_MODE,
        Rect::new(WINDOW_X - HP_BAR_WIDTH, 0.0, HP_BAR_WIDTH, WINDOW_Y), 
        LIFE_COLORS[game.player2.life_color_index] 
    )?;

    // the board
    for y in 0..VERTICAL_BLOCKS {
        for x in 0..HORIZONTAL_BLOCKS {
            let color = if game.board[y][x] { Color::WHITE} else {Color::BLACK};
            mb.rectangle(
                *FILL_MODE,
                Rect::new(HP_BAR_WIDTH + x as f32 * BLOCK_SIZE, y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE), 
                color
            )?;
        }
    }

    // selectable square area bounds
    mb.rectangle(
        *STROKE_MODE_1,
        Rect::new(AREA_1_X, BLOCK_SIZE,AREA_WIDTH,AREA_LENGTH),
        Color::from_rgb(105, 105, 105)
    )?;
    mb.rectangle(
        *STROKE_MODE_1,
        Rect::new(AREA_2_X, BLOCK_SIZE, AREA_WIDTH, AREA_LENGTH),
        Color::from_rgb(105, 105, 105)
    )?;

    // player selected squares
    for p in game.player1.selected_squares.iter() {
        mb.rectangle(
            *STROKE_MODE_2,
            Rect::new(p.x as f32 * BLOCK_SIZE + HP_BAR_WIDTH, p.y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
            Color::from_rgb(94, 199, 255)
        )?;
    }
    for p in game.player2.selected_squares.iter() {
        mb.rectangle(
            *STROKE_MODE_2,
            Rect::new(p.x as f32 * BLOCK_SIZE + HP_BAR_WIDTH, p.y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
            Color::from_rgb(94, 199, 255)
        )?;
    }

    // player hovering squares 
    mb.rectangle(
        *STROKE_MODE_1,
        Rect::new(game.player1.hovering_square.x as f32 * BLOCK_SIZE + HP_BAR_WIDTH, game.player1.hovering_square.y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
        Color::from_rgb(255, 94, 207)
    )?;
    mb.rectangle(
        *STROKE_MODE_1,
        Rect::new(game.player2.hovering_square.x as f32 * BLOCK_SIZE + HP_BAR_WIDTH, game.player2.hovering_square.y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
        Color::from_rgb(255, 94, 207)
    )?;

    let mesh = &mb.build(ctx)?;

    graphics::draw(ctx, mesh, DrawParam::default())?;
    
    Ok(())
}

fn draw_pause_menu(ctx: &mut Context, game: &Game) -> GameResult<()> {
    let mut mb = MeshBuilder::new();

    let (menu_x, menu_y, menu_width, menu_height) = (WINDOW_X/4.0, 100.0, WINDOW_X/2.0, 400.0);

    mb.rounded_rectangle(
        *FILL_MODE,
        Rect::new(menu_x, menu_y, menu_width, menu_height),
        5.0, 
        Color::from_rgb(80, 80, 80)
    )?;

    let mesh = &mb.build(ctx)?;

    graphics::draw(ctx, mesh, DrawParam::default())?;

    let title = graphics::Text::new("Fight for your life!")
            .set_bounds(pointf![menu_width,100.0], graphics::Align::Center)
            .set_font(graphics::Font::default(), PxScale{x: 40.0, y: 40.0 })
            .to_owned();
    graphics::draw(
        ctx, 
        &title,
        DrawParam::default().dest(pointf![menu_x + 20.0, menu_y + 10.0])
    )?;

    let descr = graphics::Text::new("Try to create shapes that follow the rules of the 'game of life',
    and make them reach your opponent's health bar to damage it!")
            .set_bounds(pointf![menu_width - 10.0,100.0], graphics::Align::Center)
            .set_font(graphics::Font::default(), PxScale{x: 18.0, y: 18.0 })
            .to_owned();
    graphics::draw(
        ctx, 
        &descr,
        DrawParam::default().dest(pointf![menu_x + 5.0, menu_y + 60.0]).color(Color::from_rgb(224, 142, 40))
    )?;

    let start = graphics::Text::new("pause/unpause (PRESS TO START) - P")
            .set_bounds(pointf![menu_width - 10.0,100.0], graphics::Align::Center)
            .set_font(graphics::Font::default(), PxScale{x: 22.0, y: 22.0 })
            .to_owned();
    graphics::draw(
        ctx, 
        &start,
        DrawParam::default().dest(pointf![menu_x + 5.0, menu_y + 130.0]).color(Color::from_rgb(219, 68, 46))
    )?;

    let keys = graphics::Text::new("move selected tile :  W A S D - (Player1) , Arrows (Player2)\n
select/deselect tile : C - (Player1) , Shift - (Player2)\n
faster movement: hold Alt - (Player1) , hold Ctrl - (Player2)\n
finilize selected tiles : Space - (Player1) , Enter - (Player2)")
            .set_bounds(pointf![menu_width - 10.0,200.0], graphics::Align::Left)
            .set_font(graphics::Font::default(), PxScale{x: 22.0, y: 22.0 })
            .to_owned();
    graphics::draw(
        ctx, 
        &keys,
        DrawParam::default().dest(pointf![menu_x + 5.0, menu_y + 185.0])
    )?;

    Ok(())
}

fn draw_winner_screen(ctx: &mut Context, game: &Game) -> GameResult<()> {
    let mut mb = MeshBuilder::new();

    mb.rectangle(
        *FILL_MODE,
        Rect::new(0.0, 0.0, WINDOW_X, WINDOW_Y),
        Color::from_rgb(106, 181, 98)
    )?;

    let mesh = &mb.build(ctx)?;

    graphics::draw(ctx, mesh, DrawParam::default())?;

    let winner = game.winner.clone().unwrap();
    let player_name = if winner == PlayerNum::ONE {"Player 1!"} else {"Player 2!"};
    let title = graphics::Text::new("Congratulations ".to_string() + player_name)
    .set_bounds(pointf![600.0,100.0], graphics::Align::Center)
    .set_font(graphics::Font::default(), PxScale{x: 65.0, y: 65.0 })
    .to_owned();

    graphics::draw(
        ctx, 
        &title,
        DrawParam::default().dest(pointf![WINDOW_X/4.0 + 45.0, 100.0]).color(Color::from_rgb(237, 191, 104))
    )?;

    let replay = graphics::Text::new("Press R to replay! ".to_string())
    .set_bounds(pointf![400.0,100.0], graphics::Align::Center)
    .set_font(graphics::Font::default(), PxScale{x: 30.0, y: 30.0 })
    .to_owned();

    graphics::draw(
        ctx, 
        &replay,
        DrawParam::default().dest(pointf![WINDOW_X/4.0 + 150.0, 280.0])
    )?;

    Ok(())
}

//1) Any live cell with fewer than two live neighbours dies, as if by underpopulation.
//2) Any live cell with two or three live neighbours lives on to the next generation.
//3) Any live cell with more than three live neighbours dies, as if by overpopulation.
//4) Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
fn calculate_next_generation(board: &mut [[bool; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]) -> ([[bool; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS],(bool,bool)) {
    let mut next_gen_board = [[false; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS];
    for (y,line) in board.iter().enumerate() {
        for (x, cell) in line.iter().enumerate() {
            let alive_neighbours = count_alive_neighbours(x,y,board);
            if *cell {
                if alive_neighbours == 3 || alive_neighbours == 2 {
                    next_gen_board[y][x] = true;
                }
            } else {
                if alive_neighbours == 3 {
                    next_gen_board[y][x] = true;
                }
            }
        }
    }

    (next_gen_board, check_for_damage(board))
}

fn count_alive_neighbours(x: usize, y: usize, board: &[[bool; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]) -> usize {
    let mut count = 0;
    if y == 0 || y == VERTICAL_BLOCKS - 1 || x == 0 || x == HORIZONTAL_BLOCKS - 1 {
        if y == 0 {
            if x == 0 {
                if board[y+1][x]   {count += 1}
                if board[y+1][x+1] {count += 1}
                if board[y][x+1]   {count += 1}
                return count
            } else if x == HORIZONTAL_BLOCKS - 1 {
                if board[y+1][x]   {count += 1}
                if board[y+1][x-1] {count += 1}
                if board[y][x-1]   {count += 1}
                return count
            } else {
                if board[y+1][x-1] {count += 1}
                if board[y+1][x]   {count += 1}
                if board[y+1][x+1] {count += 1}
                if board[y][x-1]   {count += 1}
                if board[y][x+1]   {count += 1}
                return count
            }
        } else if y == VERTICAL_BLOCKS - 1 {
            if x == 0 {
                if board[y-1][x]   {count += 1}
                if board[y-1][x+1] {count += 1}
                if board[y][x+1]   {count += 1}
                return count
            } else if x == HORIZONTAL_BLOCKS - 1 {
                if board[y-1][x]   {count += 1}
                if board[y-1][x-1] {count += 1}
                if board[y][x-1]   {count += 1}
                return count
            } else {
                if board[y-1][x-1] {count += 1}
                if board[y-1][x]   {count += 1}
                if board[y-1][x+1] {count += 1}
                if board[y][x-1]   {count += 1}
                if board[y][x+1]   {count += 1}
                return count
            }
        } 

        if x == 0 {
            if board[y-1][x+1] {count += 1}
            if board[y][x+1]   {count += 1}
            if board[y+1][x+1] {count += 1}
            if board[y-1][x]   {count += 1}
            if board[y+1][x]   {count += 1}
            return count
        } else if x == HORIZONTAL_BLOCKS - 1 {
            if board[y-1][x-1] {count += 1}
            if board[y][x-1]   {count += 1}
            if board[y+1][x-1] {count += 1}
            if board[y-1][x]   {count += 1}
            if board[y+1][x]   {count += 1}
            return count
        } 
    } else { // is not near a corner
        if board[y-1][x-1] {count += 1}
        if board[y-1][x]   {count += 1}
        if board[y-1][x+1] {count += 1}
        
        if board[y+1][x-1] {count += 1}
        if board[y+1][x]   {count += 1}
        if board[y+1][x+1] {count += 1}

        if board[y][x+1] {count += 1}
        if board[y][x-1] {count += 1}
    }

    count
}

fn check_for_damage(board: &[[bool; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]) -> (bool,bool) {
    let (mut player1_damage, mut player2_damage) = (false,false);
    let (mut consecutive_alive_count_1, mut consecutive_alive_count_2) = (0,0);
    for row in board.iter() {
        if !player1_damage {
            if row[0] {
                consecutive_alive_count_1 += 1;
                if consecutive_alive_count_1 == 3 {
                    player1_damage = true;
                }
            }
        }

        if !player2_damage {
            if row[HORIZONTAL_BLOCKS - 1] {
                consecutive_alive_count_2 += 1;
                if consecutive_alive_count_2 == 3 {
                    player2_damage = true;
                }
            }
        }

        if player1_damage && player2_damage {
            return (true, true)
        }
    } 

    (player1_damage,player2_damage)
}

fn make_damage_calculations(ctx: &mut Context, game: &mut Game, players_damage: (bool,bool)) {
    if players_damage.0 {
        game.player1.take_damage()
    }
    if players_damage.1 {
        game.player2.take_damage()
    }

    if game.player1.is_dead() {
        println!("player 2 won");
        game.state = GameState::WINNER_SCREEN;
    }
    if game.player2.is_dead() {
        println!("player 1 won");
        game.state = GameState::WINNER_SCREEN;
    } 
}


impl Player {
    pub fn new(player_num: PlayerNum) -> Self {
        let _x_left_bound = match player_num {
            PlayerNum::ONE => (AREA_1_X / BLOCK_SIZE) as usize,
            PlayerNum::TWO => (AREA_2_X / BLOCK_SIZE) as usize
        };
        let _x_right_bound = _x_left_bound + (AREA_WIDTH / BLOCK_SIZE) as usize - 1;
        let _y_upper_bound = 1usize;
        let _y_lower_bound = VERTICAL_BLOCKS - 2;

        let hovering_square_point = match player_num {
            PlayerNum::ONE => pointu![(AREA_1_X + AREA_WIDTH/2.0) as usize / BLOCK_SIZE as usize, (VERTICAL_BLOCKS/2)],
            PlayerNum::TWO => pointu![(AREA_2_X + AREA_WIDTH/2.0) as usize / BLOCK_SIZE as usize, (VERTICAL_BLOCKS/2)]
        };

        Player {
            player_num,
            input_state: InputState::default(),
            movement_cooldown_time: 0.0,
            life_color_index: 0,
            hovering_square : hovering_square_point,
            selected_squares: Vec::with_capacity(20),
            _x_left_bound,
            _x_right_bound,
            _y_upper_bound,
            _y_lower_bound
        }
    }

    pub fn is_dead(&self) -> bool {
        self.life_color_index == LIFE_COLORS.len() - 1
    }

    pub fn take_damage(&mut self) {
        self.life_color_index +=1; 
    }

    pub fn move_hover(&mut self, dir: Direction, mut amount: usize) {
        match dir {
            Direction::UP => {
                if amount > self.hovering_square.y {amount = self.hovering_square.y};
                if self.hovering_square.y - amount < self._y_upper_bound {
                    self.hovering_square.y = self._y_lower_bound;
                } else {
                    self.hovering_square.y -= amount; 
                }
            },
            Direction::RIGHT => {
                if self.hovering_square.x + amount > self._x_right_bound {
                    self.hovering_square.x = self._x_left_bound;
                } else {
                    self.hovering_square.x += amount; 
                }
            },
            Direction::DOWN => {
                if self.hovering_square.y + amount > self._y_lower_bound {
                    self.hovering_square.y = self._y_upper_bound;
                } else {
                    self.hovering_square.y += amount; 
                }
            },
            Direction::LEFT => {
                if amount > self.hovering_square.x {amount = self.hovering_square.y};
                if self.hovering_square.x - amount < self._x_left_bound {
                    self.hovering_square.x = self._x_right_bound;
                } else {
                    self.hovering_square.x -= amount; 
                }
            }
        }
    }
}

impl Game {
    pub fn new(ctx: &mut Context) -> Game {
        Game {
            state: GameState::PAUSE_MENU,
            timer: 0.0,
            player1:  Player::new(PlayerNum::ONE),
            player2:  Player::new(PlayerNum::TWO),
            winner: Some(PlayerNum::ONE),
            board: [[false; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]
        }
    }

    pub fn reset(&mut self) {
        self.state = GameState::PLAYING;
        self.timer = 0.0;
        self.player1 = Player::new(PlayerNum::ONE);
        self.player2 = Player::new(PlayerNum::TWO);
        self.winner = Some(PlayerNum::ONE);
        self.board = [[false; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]
    }
}

impl InputState {
    pub fn default() -> Self {
        InputState {
            movement_vector: pointi![0,0],
            mark_pressed: false,
            deploy_pressed: false
        }
    }
}




fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("fight_for_your_life", "Petros Papatheodorou")
        .add_resource_path(PathBuf::from("./res"))
        .window_setup(WindowSetup::default()
            .title("Fight for your life!")
            .vsync(true))
        .window_mode(WindowMode::default()
            .dimensions(WINDOW_X, WINDOW_Y))
        .build()
        .expect("aieee, could not create ggez context!");

    let window = graphics::window(&ctx);
    if let Some(monitor) = window.current_monitor() {
        let pos_x = (monitor.size().width as f32 - WINDOW_X) /2f32;

        let mut pos = monitor.position();
        pos.x = pos_x as i32;
        pos.y = 10;
        window.set_outer_position(pos);
    }

    let game = Game::new(&mut ctx);

    event::run(ctx, event_loop, game);
}