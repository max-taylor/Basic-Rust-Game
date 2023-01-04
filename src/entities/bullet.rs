use crate::{
    api::display::{
        element::{DEFAULT_BACKGROUND, DEFAULT_FOREGROUND},
        Element, Layout, Point,
    },
    components::{Drawable, DrawableState, DrawableType},
};

pub struct Bullet {
    pub drawable: DrawableState,
}

const ARROW_ELEMENT: Element = Element::new('^', DEFAULT_BACKGROUND, DEFAULT_FOREGROUND);

impl Bullet {
    pub fn new(location: Point<u32>) -> Self {
        let map = Layout::new(
            &Point {
                width: 1,
                height: 1,
            },
            Some(ARROW_ELEMENT),
        );

        Self {
            drawable: DrawableState::new(map, location, DrawableType::Enemy),
        }
    }
}

impl Drawable for Bullet {
    fn set_position(&mut self, updated_position: Point<u32>) -> &mut Self {
        self.drawable.location = updated_position;

        self
    }

    fn get_drawable_state(&self) -> &DrawableState {
        &self.drawable
    }
}
