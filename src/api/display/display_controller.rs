use std::io::{self, stdout, ErrorKind, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen},
    ErrorKind as CrosstermError,
};

use super::{display_controller_error::DisplayControllerError, Map};
use super::{
    element::{Element, DEFAULT_BACKGROUND, DEFAULT_FOREGROUND},
    Point,
};

pub struct DisplayController<'dimensions> {
    dimensions: &'dimensions Point,
    offset: Point,
    screen_size: Point,
    display: Map,
    default_element: Element,
    pub target: io::Stdout,
}

pub enum Direction {
    Vertical,
    Horizontal,
}

const BORDER_ELEMENT: Element = Element::new('x', Color::Black, Color::Black);
const PADDING: Point = Point::new(10, 10);

// TODO -> This x/y business is annoying, change to rows/columns or make it clearer

impl<'dimensions> DisplayController<'dimensions> {
    /// Creates a new display controller, a display controller fills the entire screen but the provided dimensions will be the controllable area
    ///
    /// # Arguments
    ///
    /// * `dimensions` - The controllable area
    ///
    /// ```
    pub fn new(dimensions: &'dimensions Point) -> Result<Self, DisplayControllerError> {
        let (columns, rows) = size().unwrap();

        if dimensions.x > rows.into() || dimensions.y > columns.into() {
            return Err(DisplayControllerError::DisplayTooSmallForDimensions);
        }

        let screen_size = Point::new(columns as u32, rows as u32);

        dbg!(&screen_size);

        // Make game size of terminal and draw dimensions in middle

        let mut controller = DisplayController {
            display: Map::new(&screen_size),
            target: stdout(),
            dimensions: &dimensions,
            default_element: Element::default(),
            screen_size,
            offset: (screen_size - *dimensions) / Point::new(2, 2),
        };

        dbg!(controller.offset);

        controller
            .setup()
            .draw_borders()?
            .print_display()?
            // .print_display()?
            .flush();

        Ok(controller)
    }

    fn draw_borders(&mut self) -> Result<&mut Self, DisplayControllerError> {
        self.draw_rect(
            &Point::new(0, 0),
            self.dimensions,
            Element::new('x', Color::Blue, Color::Green),
        )
    }

    fn setup(&mut self) -> &mut Self {
        queue!(self.target, EnterAlternateScreen, Hide).unwrap();

        self
    }

    /// Flushing the target publishes all queued writes
    fn flush(&mut self) -> &mut Self {
        self.target.flush().unwrap();

        self
    }

    pub fn reset_cursor(&mut self) -> &mut Self {
        queue!(
            self.target,
            SetForegroundColor(DEFAULT_FOREGROUND),
            SetBackgroundColor(DEFAULT_BACKGROUND),
            MoveTo(0, 0)
        )
        .unwrap();

        self
    }

    fn draw_rect(
        &mut self,
        start_position: &Point,
        dimensions: &Point,
        element: Element,
    ) -> Result<&mut Self, DisplayControllerError> {
        self.draw_line(element, dimensions.x, start_position, Direction::Horizontal)?
            .draw_line(
                element,
                dimensions.x,
                &start_position.addY(dimensions.y - 1),
                Direction::Horizontal,
            )?
            .draw_line(element, dimensions.y, start_position, Direction::Vertical)?
            .draw_line(
                element,
                dimensions.y,
                &start_position.addX(dimensions.x - 1),
                Direction::Vertical,
            )?;

        Ok(self)
    }

    // TODO: Add docs describing that the line draws from top->bottom
    pub fn draw_line(
        &mut self,
        element: Element,
        len: u32,
        start_position: &Point,
        direction: Direction,
    ) -> Result<&mut Self, DisplayControllerError> {
        for position_change in 0..len {
            let new_position = match direction {
                Direction::Horizontal => start_position.addX(position_change),
                Direction::Vertical => start_position.addY(position_change),
            };

            self.draw_item(element, &new_position)?;
        }

        Ok(self)
    }

    fn draw_item(
        &mut self,
        element: Element,
        position: &Point,
    ) -> Result<&mut Self, DisplayControllerError> {
        // Position is exclusive of the dimension borders
        if position.x >= self.dimensions.x || position.y >= self.dimensions.y {
            return Err(DisplayControllerError::PositionOutOfRange);
        }

        let updated_positions = self.offset + *position;

        dbg!(updated_positions);

        let row = self
            .display
            .map
            .get_mut(updated_positions.y as usize)
            .ok_or(DisplayControllerError::PositionOutOfRange)?;

        // This could instead just have the .insert chained on the above expression to replace the item, but this is a bit more verbose for my learning
        if let Some(existing_item) = row[updated_positions.x as usize].as_mut() {
            *existing_item = element;
        } else {
            row[updated_positions.x as usize] = Some(element);
        }

        Ok(self)
    }

    pub fn print_display(&mut self) -> Result<&mut Self, DisplayControllerError> {
        self.reset_cursor();

        for row in self.display.map.iter() {
            for element in row.iter() {
                match element {
                    Some(element) => {
                        DisplayController::print_element(&mut self.target, element, None)?;
                    }
                    None => {
                        DisplayController::print_element(
                            &mut self.target,
                            &self.default_element,
                            None,
                        )?;
                    }
                }
            }
        }

        Ok(self)
    }

    pub fn print_element(
        target: &mut io::Stdout,
        element: &Element,
        move_to: Option<&Point>,
    ) -> Result<(), DisplayControllerError> {
        if let Some(move_to_destination) = move_to {
            queue!(
                target,
                MoveTo(move_to_destination.x as u16, move_to_destination.y as u16)
            )
            .map_err(DisplayControllerError::from_crossterm_error)?;
        };

        queue!(
            target,
            SetForegroundColor(element.foreground),
            SetBackgroundColor(element.background),
            Print(element.value)
        )
        .map_err(DisplayControllerError::from_crossterm_error)?;

        Ok(())
    }

    pub fn close(target: &mut io::Stdout) -> Result<(), CrosstermError> {
        disable_raw_mode()?;
        execute!(target, LeaveAlternateScreen, Show)?;

        Ok(())
    }
}
