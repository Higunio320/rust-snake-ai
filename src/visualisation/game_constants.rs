use once_cell::sync::Lazy;

pub const GRID_SIZE: (i16, i16) = (10, 10);
pub const GRID_CELL_SIZE: (i16, i16) = (48, 48);

pub const MAX_Y_DISTANCE: f64 = GRID_SIZE.1 as f64 - 1.0;

pub const MAX_X_DISTANCE: f64 = GRID_SIZE.0 as f64 - 1.0;

pub static MAX_DISTANCE: Lazy<f64> = Lazy::new(|| ((GRID_SIZE.0.pow(2) + GRID_SIZE.1.pow(2)) as f64).sqrt());

pub const GAME_SCREEN_SIZE: (f32, f32) = (
    (GRID_SIZE.0 * GRID_CELL_SIZE.0) as f32,
    (GRID_SIZE.1 * GRID_CELL_SIZE.1) as f32
);

pub const SCREEN_SIZE: (f32, f32) = (
    GAME_SCREEN_SIZE.0 + 1000.0,
    GAME_SCREEN_SIZE.1 + 500.0
);

pub const FPS: u32 = 10;