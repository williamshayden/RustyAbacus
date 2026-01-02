/// Viewport for coordinate transformation between screen and world space

#[derive(Clone, Copy)]
pub struct Viewport {
    pub center_x: f64,
    pub center_y: f64,
    pub scale: f64, // pixels per unit
    pub width: f32,
    pub height: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            scale: 50.0, // 50 pixels per unit
            width: 400.0,
            height: 400.0,
        }
    }
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> (f64, f64) {
        let world_x = self.center_x + (screen_x as f64 - self.width as f64 / 2.0) / self.scale;
        let world_y = self.center_y - (screen_y as f64 - self.height as f64 / 2.0) / self.scale;
        (world_x, world_y)
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_x: f64, world_y: f64) -> (f32, f32) {
        let screen_x = (self.width as f64 / 2.0 + (world_x - self.center_x) * self.scale) as f32;
        let screen_y = (self.height as f64 / 2.0 - (world_y - self.center_y) * self.scale) as f32;
        (screen_x, screen_y)
    }

    /// Get the visible x range in world coordinates
    pub fn x_range(&self) -> (f64, f64) {
        let half_width = self.width as f64 / (2.0 * self.scale);
        (self.center_x - half_width, self.center_x + half_width)
    }

    /// Get the visible y range in world coordinates
    pub fn y_range(&self) -> (f64, f64) {
        let half_height = self.height as f64 / (2.0 * self.scale);
        (self.center_y - half_height, self.center_y + half_height)
    }

    /// Pan the viewport by screen delta
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.center_x -= dx as f64 / self.scale;
        self.center_y += dy as f64 / self.scale;
    }

    /// Zoom the viewport centered at screen point
    pub fn zoom(&mut self, factor: f64, screen_x: f32, screen_y: f32) {
        // Get world position before zoom
        let (world_x, world_y) = self.screen_to_world(screen_x, screen_y);

        // Apply zoom
        self.scale *= factor;
        self.scale = self.scale.clamp(5.0, 500.0); // Limit zoom range

        // Adjust center to keep mouse position fixed
        let (new_world_x, new_world_y) = self.screen_to_world(screen_x, screen_y);
        self.center_x += world_x - new_world_x;
        self.center_y += world_y - new_world_y;
    }

    /// Reset viewport to default view
    pub fn reset(&mut self) {
        self.center_x = 0.0;
        self.center_y = 0.0;
        self.scale = 50.0;
    }

    /// Update viewport size
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}
