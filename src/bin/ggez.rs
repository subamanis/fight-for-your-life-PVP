use std::ops::Index;
use std::path::{PathBuf};
use std::process::exit;
use std::time::Instant;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::input::keyboard;
use ggez::{Context, ContextBuilder, GameResult, timer};
use ggez::graphics::{self, Color, DrawParam, FillOptions, MeshBuilder, Rect, StrokeOptions};
use ggez::event::{self, EventHandler, KeyCode, KeyMods, quit};

use lazy_static::lazy_static;

type Point2f = ggez::mint::Point2<f32>;
type Point2u = ggez::mint::Point2<usize>;
type Point2i = ggez::mint::Point2<isize>;

// BLOCK_SIZE is a common perfect divisor of WINDOW_X - 2*HP_BAR_WIDTH and WINDOW_Y.
// Horizontal and vertical blocks are the division (WINDOW_X - 2*HP_BAR_WIDTH) / BLOCK_SIZE and WINDOW_Y / BLOCK_SIZE respectively
// These hardcoded values should only be changed in the above conditions are met.
const WINDOW_X     : f32 = 1502.0;
const WINDOW_Y     : f32 = 952.0;
const HP_BAR_WIDTH : f32 = 20.0;
const BLOCK_SIZE   : f32 = 34.0;
const HORIZONTAL_BLOCKS : usize = 43;
const VERTICAL_BLOCKS   : usize = 28;

const AREA_1_X : f32 = (HORIZONTAL_BLOCKS/8) as f32 * BLOCK_SIZE + HP_BAR_WIDTH;
const AREA_2_X_OFFSET : f32 = if HORIZONTAL_BLOCKS % 2 == 0 {1.0} else {0.0};
const AREA_2_X : f32 = ((2*HORIZONTAL_BLOCKS/3) as f32 + AREA_2_X_OFFSET) * BLOCK_SIZE + HP_BAR_WIDTH;
const AREA_WIDTH  : f32 = (HORIZONTAL_BLOCKS/4) as f32 * BLOCK_SIZE;
const AREA_LENGTH : f32 = (VERTICAL_BLOCKS-2) as f32 * BLOCK_SIZE;

const DESIRED_FPS: u32 = 60;
const GENERATION_CALCULATION_DELAY: f32 = 0.6;


lazy_static! {
    static ref LIFE_COLORS:[Color; 6] = [Color::from_rgb(105, 212, 76), Color::from_rgb(151, 212, 76), Color::from_rgb(203, 212, 76),
                                         Color::from_rgb(219, 190, 75), Color::from_rgb(219, 157, 75), Color::from_rgb(217, 80, 56)];
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

#[derive(Debug)]
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
    game_state: GameState,
    player1: Player,
    player2: Player,
    board: [[bool; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]
}


fn calculate_next_generation(board: &mut [[bool; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]) {

}

fn draw_pause_menu(ctx: &Context, game: &Game) {
    
}

fn draw_winner_screen(ctx: &Context, game: &Game) {

}

impl EventHandler<ggez::GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.game_state != GameState::PLAYING {return Ok(())}

        let seconds = 1.0 / DESIRED_FPS as f32;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            calculate_next_generation(&mut self.board);

            if self.player1.is_dead() {
                // self.winner = Some(self.player2);
                println!("player 2 won");
                ggez::event::quit(ctx);
            }
            if self.player2.is_dead() {
                return Ok(())
            } 
        }

        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb(170,170,170));

        match self.game_state {
            GameState::PLAYING => draw_board(ctx, self)?,
            GameState::PAUSE_MENU => draw_pause_menu(ctx, self),
            GameState::WINNER_SCREEN => draw_winner_screen(ctx, self)
        }
        
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, repeat: bool) {
        if repeat {return}
        
        match key {
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
                for p in self.player2.selected_squares.iter() {
                    self.board[p.x][p.y] = true;
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

fn draw_board(ctx: &mut Context, game_state: &mut Game) -> GameResult<()> {
    let mut mb = MeshBuilder::new();
    let rect_draw_fill_mode = graphics::DrawMode::Fill(FillOptions::default());
    let rect_draw_stroke_mode = graphics::DrawMode::Stroke(StrokeOptions::default().with_line_width(1.0));

    // the 2 HP bars
    mb.rectangle(
        rect_draw_fill_mode,
        Rect::new(0.0, 0.0, HP_BAR_WIDTH, WINDOW_Y), 
        Color::GREEN
    )?;
    mb.rectangle(
        rect_draw_fill_mode,
        Rect::new(WINDOW_X - HP_BAR_WIDTH, 0.0, HP_BAR_WIDTH, WINDOW_Y), 
        Color::GREEN
    )?;

    // the board
    game_state.board[3][25] = true;
    for y in 0..VERTICAL_BLOCKS {
        for x in 0..HORIZONTAL_BLOCKS {
            let color = if game_state.board[y][x] { Color::WHITE} else {Color::BLACK};
            mb.rectangle(
                rect_draw_fill_mode,
                Rect::new(HP_BAR_WIDTH + x as f32 * BLOCK_SIZE, y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE), 
                color
            )?;
        }
    }

    // selectable square area bounds
    mb.rectangle(
        rect_draw_stroke_mode,
        Rect::new(AREA_1_X, BLOCK_SIZE,AREA_WIDTH,AREA_LENGTH),
        Color::from_rgb(105, 105, 105)
    )?;
    mb.rectangle(
        rect_draw_stroke_mode,
        Rect::new(AREA_2_X, BLOCK_SIZE, AREA_WIDTH, AREA_LENGTH),
        Color::from_rgb(105, 105, 105)
    )?;

    // player selected squares
    for p in game_state.player1.selected_squares.iter() {
        mb.rectangle(
            rect_draw_stroke_mode,
            Rect::new(p.x as f32 * BLOCK_SIZE + HP_BAR_WIDTH, p.y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
            Color::from_rgb(94, 199, 255)
        )?;
    }
    for p in game_state.player2.selected_squares.iter() {
        mb.rectangle(
            rect_draw_stroke_mode,
            Rect::new(p.x as f32 * BLOCK_SIZE + HP_BAR_WIDTH, p.y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
            Color::from_rgb(94, 199, 255)
        )?;
    }

    // player hovering squares 
    mb.rectangle(
        rect_draw_stroke_mode,
        Rect::new(game_state.player1.hovering_square.x as f32 * BLOCK_SIZE + HP_BAR_WIDTH, game_state.player1.hovering_square.y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
        Color::from_rgb(255, 94, 207)
    )?;
    mb.rectangle(
        rect_draw_stroke_mode,
        Rect::new(game_state.player2.hovering_square.x as f32 * BLOCK_SIZE + HP_BAR_WIDTH, game_state.player2.hovering_square.y as f32 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
        Color::from_rgb(255, 94, 207)
    )?;

    //debug line
    // for i in 0..HORIZONTAL_BLOCKS {
    //     mb.rectangle(
    //         rect_draw_fill_mode,
    //         Rect::new(i as f32 * BLOCK_SIZE + HP_BAR_WIDTH, 6.0 * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
    //         if i % 2==0 {Color::from_rgb(66, 66, 66)} else {Color::from_rgb(145, 145, 145)}
    //     )?;
    // }
  
    // mb.line(&[point![20_f32,400_f32],point![700_f32,400_f32]], 2.0, graphics::Color::BLACK)?;

    let mesh = &mb.build(ctx)?;

    graphics::draw(ctx, mesh, DrawParam::default())?;
    
    Ok(())
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
        self.life_color_index == 5
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
            game_state: GameState::PLAYING,
            player1:  Player::new(PlayerNum::ONE),
            player2:  Player::new(PlayerNum::TWO),
            board: [[false; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]
        }
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


fn move_right_checked(player: &mut Player) {
    if player.hovering_square.x + 1 > player._x_right_bound {
        player.hovering_square.x = player._x_left_bound;
    } else {
        player.hovering_square.x += 1; 
    }
}

fn move_up_checked(player: &mut Player) {
    if player.hovering_square.y -1 < player._y_upper_bound {
        player.hovering_square.y = player._y_lower_bound;
    } else {
        player.hovering_square.y -= 1; 
    }
}

fn move_down_checked(player: &mut Player) {
    if player.hovering_square.y + 1 > player._y_lower_bound {
        player.hovering_square.y = player._y_upper_bound;
    } else {
        player.hovering_square.y += 1; 
    }
}

fn move_left_checked(player: &mut Player) {
    if player.hovering_square.x - 1 < player._x_left_bound {
        player.hovering_square.x = player._x_right_bound;
    } else {
        player.hovering_square.x -= 1; 
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

    let game_state = Game::new(&mut ctx);

    event::run(ctx, event_loop, game_state);
}