use ggez::{
    Context,
    GameResult,
    glam::*,
    graphics::{self, Color, TextFragment, PxScale},
    event::{self, EventHandler},
    input::keyboard::KeyCode, audio::{self, Source, SoundSource}
};

use std::{ path, env, time::{Instant, Duration} };

const SPEED: f32 = 10.0;
const BALL_SPEED: f32 = 6.5;

fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("pong", "Maarten van Keulen").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build().expect("Could not build context");
    
    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = MyGame::new(&mut ctx);

    // Run!
    event::run(ctx, event_loop, my_game);
}

enum Collision {
    None,
    Hit,
    Miss
}

struct Ball {
    x: f32,
    y: f32,
    xvel: f32,
    yvel: f32,
    continue_timer: Instant,
}

impl Ball {
    pub fn new() -> Ball {
        Ball {
            x: 50.0,
            y: 50.0,
            xvel: BALL_SPEED,
            yvel: BALL_SPEED,
            continue_timer: Instant::now()
        }
    }

    pub fn update(&mut self, player_a: &mut Player, player_b: &mut Player) -> Collision {
        self.x += self.xvel;
        self.y += self.yvel;

        if self.y < 0.0 || self.y > 590.0 {
            self.yvel *= -1.0;
        }

        let mut collision: Collision = Collision::None;
        if self.x < 10.0 {
            collision = if player_a.collided(self as &Ball) { Collision::Hit } else { Collision::Miss };
        } else if self.x > 780.0 {
            collision = if player_b.collided(self as &Ball) { Collision::Hit } else { Collision::Miss };
        }

        match collision {
            Collision::Hit => self.xvel *= -1.0,
            Collision::Miss => {
                self.continue_timer = Instant::now();
                self.x = 50.0;
                self.y = 50.0;

                self.xvel = BALL_SPEED;
                self.yvel = BALL_SPEED;

                player_a.y = 50.0;
                player_b.y = 50.0;
            },
            Collision::None => {}
        }

        collision
    }
}

struct Player {
    y: f32,
    points: i32
}

impl Player {
    pub fn new() -> Player {
        Player {
            y: 0.0,
            points: 0
        }
    }

    pub fn up(&mut self) {
        self.y -= SPEED;
        if self.y < 0.0 {
            self.y = 0.0;
        }
    }

    pub fn down(&mut self) {
        self.y += SPEED;
        if self.y > 550.0 {
            self.y = 550.0;
        }
    }

    pub fn collided(&mut self, ball: &Ball) -> bool {
        let hit = ball.y >= (self.y - 10.0) && ball.y <= (self.y + 60.0);
        if hit {
            self.points += 1;
        }

        return hit;
    }
}

struct MyGame {
    player_a: Player,
    player_b: Player,
    ball: Ball,
    miss: Source,
    hit: Source,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        _ctx.gfx.add_font(
            "NotoMono",
            graphics::FontData::from_path(_ctx, "/NotoMono-Regular.ttf").unwrap(),
        );

        let hit = audio::Source::new(_ctx, "/240.ogg").unwrap();
        let miss = audio::Source::new(_ctx, "/440.ogg").unwrap();

        // Load/create resources such as images here.
        MyGame {
            player_a: Player::new(),
            player_b: Player::new(),
            ball: Ball::new(),
            miss,
            hit
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if Instant::now().duration_since(self.ball.continue_timer) < Duration::from_millis(400) {
            return Ok(());
        }

        // Update code here...
        if _ctx.keyboard.is_key_pressed(KeyCode::W) {
            self.player_a.up()
        } else if _ctx.keyboard.is_key_pressed(KeyCode::S) {
            self.player_a.down()
        }

        if _ctx.keyboard.is_key_pressed(KeyCode::Up) {
            self.player_b.up()
        } else if _ctx.keyboard.is_key_pressed(KeyCode::Down) {
            self.player_b.down()
        }

        let _ = match self.ball.update(&mut self.player_a, &mut self.player_b) {
            Collision::Hit => self.hit.play_detached(_ctx),
            Collision::Miss => self.miss.play_detached(_ctx),
            Collision::None => Ok(())
        };
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        
        // Player A
        let pa = &mut graphics::MeshBuilder::new();
        pa.rectangle(
            graphics::DrawMode::Fill(graphics::FillOptions::DEFAULT),
            graphics::Rect::new(1.0, self.player_a.y + 1.0, 10.0, 50.0),
            graphics::Color::new(1.0, 0.7, 0.7, 1.0),
        )?;

        // Player B
        let pb = &mut graphics::MeshBuilder::new();
        pb.rectangle(
            graphics::DrawMode::Fill(graphics::FillOptions::DEFAULT),
            graphics::Rect::new(790.0, self.player_b.y + 1.0, 10.0, 50.0),
            graphics::Color::new(0.7, 0.7, 1.0, 1.0),
        )?;

        // Ball
        let b = &mut graphics::MeshBuilder::new();
        b.circle(
            graphics::DrawMode::Fill(graphics::FillOptions::DEFAULT),
            Vec2::new(self.ball.x, self.ball.y),
            10.0,
            1.0,
            graphics::Color::new(1.0, 1.0, 1.0, 1.0)
        )?;
        
        let mut rect_a = graphics::Mesh::from_data(ctx, pa.build());
        let mut rect_b = graphics::Mesh::from_data(ctx, pb.build());
        let mut circ_b = graphics::Mesh::from_data(ctx, b.build());

        canvas.draw(&mut rect_a, graphics::DrawParam::new());
        canvas.draw(&mut rect_b, graphics::DrawParam::new());
        canvas.draw(&mut circ_b, graphics::DrawParam::new());
        
        // Player A points
        canvas.draw(
            &graphics::Text::new(TextFragment {
                text: format!("{}", self.player_a.points).to_string(),
                color: Some(graphics::Color::new(1.0, 0.7, 0.7, 1.0)),
                font: Some("NotoMono".into()),
                scale: Some(PxScale::from(24.0)),
                ..Default::default()
            }),
            Vec2::new(290.0, 0.0),
        );

        // Player B points
        canvas.draw(
            &graphics::Text::new(TextFragment {
                text: format!("{}", self.player_b.points).to_string(),
                color: Some(graphics::Color::new(0.7, 0.7, 1.0, 1.0)),
                font: Some("NotoMono".into()),
                scale: Some(PxScale::from(24.0)),
                ..Default::default()
            }),
            Vec2::new(500.0, 570.0),
        );

        canvas.finish(ctx)
    }
}
