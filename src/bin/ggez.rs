use std::path::{PathBuf};

use ggez::conf::{WindowMode, WindowSetup};
use ggez::input::keyboard;
use ggez::{Context, ContextBuilder, GameResult, timer};
use ggez::graphics::{self, Color, DrawParam, FillOptions, MeshBuilder, Rect, StrokeOptions};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};

use lazy_static::lazy_static;

type Point2f = ggez::mint::Point2<f32>;
type Point2 = ggez::mint::Point2<usize>;

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
const PLAYER_MOVEMENT_DELAY: f32 = 0.3;


lazy_static! {
    static ref LIFE_COLORS:[Color; 6] = [Color::from_rgb(105, 212, 76), Color::from_rgb(151, 212, 76), Color::from_rgb(203, 212, 76),
                                         Color::from_rgb(219, 190, 75), Color::from_rgb(219, 157, 75), Color::from_rgb(217, 80, 56)];
}

#[macro_use]
macro_rules! point {
    ($x:expr,$y:expr) => {
        Point2{x:$x,y:$y}
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

#[derive(Debug)]
struct Player {
    pub player_num: PlayerNum,
    pub input_state: InputState,
    pub elapsed_time_since_move: f32,
    pub life_color_index: usize,
    pub hovering_square: Point2,
    pub selected_squares: Vec<Point2>,
    _x_left_bound: usize,
    _x_right_bound: usize,
    _y_upper_bound: usize,
    _y_lower_bound: usize,
}

#[derive(Debug,Default)]
struct InputState {
    x_axis_value: isize,
    y_axis_value: isize,
    mark_pressed: bool,
    deploy_pressed: bool
}

#[derive(Debug)]
struct GameState {
    player1: Player,
    player2: Player,
    board: [[bool; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]
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
            PlayerNum::ONE => point![(AREA_1_X + AREA_WIDTH/2.0) as usize / BLOCK_SIZE as usize, (VERTICAL_BLOCKS/2)],
            PlayerNum::TWO => point![(AREA_2_X + AREA_WIDTH/2.0) as usize / BLOCK_SIZE as usize, (VERTICAL_BLOCKS/2)]
        };

        Player {
            player_num,
            input_state: InputState::default(),
            elapsed_time_since_move: 0.0,
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

    pub fn move_hover(&mut self, dir: Direction) {
        match dir {
            Direction::UP => {
                if self.hovering_square.y -1 < self._y_upper_bound {
                    self.hovering_square.y = self._y_lower_bound;
                } else {
                    self.hovering_square.y -= 1; 
                }
            },
            Direction::RIGHT => {
                if self.hovering_square.x + 1 > self._x_right_bound {
                    self.hovering_square.x = self._x_left_bound;
                } else {
                    self.hovering_square.x += 1; 
                }
            },
            Direction::DOWN => {
                if self.hovering_square.y + 1 > self._y_lower_bound {
                    self.hovering_square.y = self._y_upper_bound;
                } else {
                    self.hovering_square.y += 1; 
                }
            },
            Direction::LEFT => {
                if self.hovering_square.x - 1 < self._x_left_bound {
                    self.hovering_square.x = self._x_right_bound;
                } else {
                    self.hovering_square.x -= 1; 
                }
            }
        }
    }
}


impl GameState {
    pub fn new(ctx: &mut Context) -> GameState {
        GameState {
            player1:  Player::new(PlayerNum::ONE),
            player2:  Player::new(PlayerNum::TWO),
            board: [[false; HORIZONTAL_BLOCKS]; VERTICAL_BLOCKS]
        }
    }
}


fn update_player_state(player: &mut Player, seconds_elapsed: f32) {
    player.elapsed_time_since_move += seconds_elapsed;
    println!("elapsed: {} , sum: {}",seconds_elapsed, player.elapsed_time_since_move);
    if player.elapsed_time_since_move < PLAYER_MOVEMENT_DELAY {return} 

    if player.input_state.mark_pressed {
        player.selected_squares.push(player.hovering_square);
    }

    if player.input_state.deploy_pressed {
        //game of life logic
    }

    if player.input_state.x_axis_value == -1 {
        move_left_checked(player);
        player.elapsed_time_since_move = 0.0;
    } else if player.input_state.x_axis_value == 1 {
        move_right_checked(player);
        player.elapsed_time_since_move = 0.0;
    }

    if player.input_state.y_axis_value == -1 {
        move_up_checked(player);
        player.elapsed_time_since_move = 0.0;
    } else if player.input_state.y_axis_value == 1 {
        mov3_down_checked(player);
        player.elapsed_time_since_move = 0.0;
    }

    player.input_state.mark_pressed = false; //remove
    player.input_state.deploy_pressed = false; //remove
}

impl EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let seconds = 1.0 / DESIRED_FPS as f32;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            update_player_state(&mut self.player1, seconds);
            update_player_state(&mut self.player2, seconds);
        }

        
        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::WHITE);

        draw_board(ctx, self)?;
        
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, repeat: bool) {
        match key {
            // Player1
            KeyCode::W => {
                self.player1.input_state.y_axis_value -=1;
            },
            KeyCode::D => {
                self.player1.input_state.x_axis_value +=1;
            },
            KeyCode::S => {
                self.player1.input_state.y_axis_value +=1;
            },
            KeyCode::A => {
                self.player1.input_state.x_axis_value -=1;
            },
            KeyCode::C => {
                self.player1.input_state.mark_pressed = true;
            },
            KeyCode::Space => {
                self.player1.input_state.deploy_pressed = true;
            },
            //Player2
            KeyCode::Up => {
                self.player2.input_state.y_axis_value -=1;
            },
            KeyCode::Right => {
                self.player2.input_state.x_axis_value +=1;
            },
            KeyCode::Down => {
                self.player2.input_state.y_axis_value +=1;
            },
            KeyCode::Left => {
                self.player2.input_state.x_axis_value -=1;
            },
            KeyCode::RShift => {
                self.player2.input_state.mark_pressed = true;
            },
            KeyCode::Return => {
                self.player2.input_state.deploy_pressed = true;
            },
            _ => ()
        }
    }

    
    fn key_up_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods) {
        match key {
            // Player1
            KeyCode::W => {
                self.player1.input_state.y_axis_value +=1;
            },
            KeyCode::D => {
                self.player1.input_state.x_axis_value -=1;
            },
            KeyCode::S => {
                self.player1.input_state.y_axis_value -=1;
            },
            KeyCode::A => {
                self.player1.input_state.x_axis_value +=1;
            },
            //Player2
            KeyCode::Up => {
                self.player2.input_state.y_axis_value +=1;
            },
            KeyCode::Right => {
                self.player2.input_state.x_axis_value -=1;
            },
            KeyCode::Down => {
                self.player2.input_state.y_axis_value -=1;
            },
            KeyCode::Left => {
                self.player2.input_state.x_axis_value +=1;
            },
            _ => ()
        }
    }
}


fn draw_board(ctx: &mut Context, game_state: &mut GameState) -> GameResult<()> {
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

fn mov3_down_checked(player: &mut Player) {
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

    let game_state = GameState::new(&mut ctx);

    event::run(ctx, event_loop, game_state);
}