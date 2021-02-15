use tinypixel::*;

enum Direction {
    Up,
    Right,
    Down,
    Left,
}

struct Game {
    viewport: Viewport,
    pix_pos: ScreenPos,
    direction: Option<Direction>,
}

impl EventLoop for Game {
    fn draw(&mut self, renderer: &mut Renderer) {
        let pixel = Pixel {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        };

        self.viewport.draw_pixel(pixel, self.pix_pos);
        renderer.draw(&self.viewport);
    }

    fn update(&mut self) {
        match self.direction {
            Some(Direction::Up) => self.pix_pos.y -= 1,
            Some(Direction::Right) => self.pix_pos.x += 1,
            Some(Direction::Down) => self.pix_pos.y += 1,
            Some(Direction::Left) => self.pix_pos.x -= 1,
            _ => ()
        }
    }

    fn input(&mut self, event: Event) {
        match event {
            Event::Key(KeyboardInput {
                virtual_keycode, 
                state,
                ..
            }) => match (virtual_keycode, state) {
                (_, ElementState::Released) => self.direction = None,
                (Some(VirtualKeyCode::H), ElementState::Pressed) => { self.direction = Some(Direction::Left) }
                (Some(VirtualKeyCode::J), ElementState::Pressed) => { self.direction = Some(Direction::Down) }
                (Some(VirtualKeyCode::K), ElementState::Pressed) => { self.direction = Some(Direction::Up) }
                (Some(VirtualKeyCode::L), ElementState::Pressed) => { self.direction = Some(Direction::Right) }
                _ => {}
            },
            _ => {}
        }
    }

    fn resize(&mut self, new_size: ScreenSize) {
        self.viewport.resize(new_size);
    }
}

fn main() {
    let event_loop = WinitEventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let size = window.inner_size();
    eprintln!("{} | {}", size.width, size.height);

    let mut game = Game {
        viewport: Viewport::new(ScreenPos::zero(), ScreenSize::new(size.width, size.height)),
        pix_pos: ScreenPos::new(size.width / 2, size.height / 2),
        direction: None,
    };

    start(game, window, event_loop);
}
