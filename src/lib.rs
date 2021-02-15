mod events;
mod pixel;
mod renderer;
mod viewport;
mod texture;

// -----------------------------------------------------------------------------
//     - Reexports -
// -----------------------------------------------------------------------------
pub use events::{start, EventLoop, Event};
pub use pixel::{Pixel, PixelBuffer};
pub use renderer::Renderer;
pub use viewport::Viewport;

// -----------------------------------------------------------------------------
//     - Winit -
// -----------------------------------------------------------------------------
pub use winit::event::{VirtualKeyCode, KeyboardInput, ElementState};
pub use winit::event_loop::EventLoop as WinitEventLoop;
pub use winit::window::WindowBuilder;

// -----------------------------------------------------------------------------
//     - Euclid -
// -----------------------------------------------------------------------------
pub type Vec2D<T> = euclid::default::Vector2D<T>;

/// Constraining units to screen space
pub struct Screen;

/// Constraining units to world space
pub struct World;

/// A position on screen, where 0,0 is the top left corner
pub type ScreenPos = euclid::Point2D<u32, Screen>;

/// A position in the world
pub type WorldPos = euclid::Point2D<f32, World>;

/// A rect on screen
pub type ScreenRect = euclid::Rect<u16, Screen>;

/// A rect in the world
pub type WorldRect = euclid::Rect<f32, World>;

/// A size on screen
pub type ScreenSize = euclid::Size2D<u32, Screen>;

/// A size in the world
pub type WorldSize = euclid::Size2D<f32, World>;
