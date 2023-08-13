#![allow(unused, dead_code)]
use derivative::Derivative;

#[derive(Derivative, Debug, Clone, Copy)]
#[derivative(Default)]
pub struct CursorCoords {
    #[derivative(Default(value = "1"))]
    pub x: u16,
    #[derivative(Default(value = "1"))]
    pub y: u16,
    #[derivative(Default(value = "None"))]
    pub min_x: Option<u16>,
    #[derivative(Default(value = "None"))]
    pub min_y: Option<u16>,
    #[derivative(Default(value = "None"))]
    pub max_x: Option<u16>,
    #[derivative(Default(value = "None"))]
    pub max_y: Option<u16>,
}

impl CursorCoords {
    pub fn set(&mut self, x: u16, y: u16) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }
    pub fn get(&self) -> (u16, u16) {
        (self.x, self.y)
    }
    pub fn set_min(&mut self, (min_x, min_y): (u16, u16)) -> &mut Self {
        self.min_x = Some(min_x);
        self.min_y = Some(min_y);
        self
    }
    pub fn get_min(&self) -> (Option<u16>, Option<u16>) {
        (self.min_x, self.min_y)
    }
    pub fn set_max(&mut self, (max_x, max_y): (u16, u16)) -> &mut Self {
        self.max_x = Some(max_x);
        self.max_y = Some(max_y);
        self
    }
    pub fn get_max(&self) -> (Option<u16>, Option<u16>) {
        (self.max_x, self.max_y)
    }
    pub fn up(&mut self) {
        if self.y > 0 {
            match self.min_y {
                Some(min) if min < self.y => self.y -= 1,
                None => self.y -= 1,
                _ => {}
            }
        }
    }
    pub fn down(&mut self) {
        if let Some(max_y) = self.max_y {
            if max_y > self.y {
                self.y += 1;
            }
        }
        match self.max_y {
            Some(max) if max > self.y => self.y += 1,
            None => self.y += 1,
            _ => {}
        }
    }
    pub fn right(&mut self) {
        match self.max_x {
            Some(max) if max > self.x => self.x += 1,
            None => self.x += 1,
            _ => {}
        }
        self.x += 1;
    }
    pub fn left(&mut self) {
        if self.x > 0 {
            match self.min_x {
                Some(min) if min < self.x => self.x -= 1,
                None => self.x -= 1,
                _ => {}
            }
        }
        self.x -= 1;
    }
}

impl From<(u16, u16)> for CursorCoords {
    fn from((x, y): (u16, u16)) -> Self {
        Self {
            x,
            y,
            ..Default::default()
        }
    }
}

impl ToString for CursorCoords {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Default, Debug)]
pub struct CursorCoordsState {
    pub normal: CursorCoords,
    pub insert: CursorCoords,
}
