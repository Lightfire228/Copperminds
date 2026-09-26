
use std::marker::PhantomData;

use iced::border::Radius;
use iced::{Border, Element, Length, Shadow, Size, color};
use iced::advanced::{Renderer, Widget, renderer};
use iced::advanced::widget::Tree;
use iced::advanced::layout::{Limits, Node};

const CELL_WIDTH:  f32 = 50.0;
const CELL_HEIGHT: f32 = 50.0;

pub struct Table<'a> {
    _p: PhantomData<&'a ()>,
    rows: usize,
    cols: usize,
}

enum Message {}

impl<'a> Table<'a> {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            _p: PhantomData,
            rows,
            cols,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Table<'a>
where
    Renderer: renderer::Renderer,
{

    fn size(&self) -> Size<Length> {
        Size {
            width:  Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        _tree:     &mut Tree,
        _renderer: &Renderer,
        _limits:   &Limits,
    )
        -> Node
    {
        Node::new(Size {
            width:  self.cols as f32 * CELL_WIDTH,
            height: self.rows as f32 * CELL_HEIGHT,
        })
    }

    fn draw(
        &self,
        _tree:     &iced::advanced::widget::Tree,
        renderer:  &mut Renderer,
        _theme:    &Theme,
        _style:    &iced::advanced::renderer::Style,
        _layout:    iced::advanced::Layout<'_>,
        _cursor:    iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {

        let bounds = _layout.bounds();

        if !bounds.intersects(_viewport) {
            return;
        }

        let width  = bounds.width  / self.cols as f32;
        let height = bounds.height / self.rows as f32;

        for y in 0..self.rows {
            for x in 0..self.cols {


                renderer.fill_quad(
                    renderer::Quad {
                        bounds: iced::Rectangle {
                            x:      bounds.x + width  * x as f32,
                            y:      bounds.y + height * y as f32,
                            width,
                            height,
                        },
                        border: Border {
                            color:  color!(255, 255, 255),
                            width:  1.0,
                            radius: Radius::new(0.3),
                        },
                        shadow: Shadow::default(),
                        snap:   true,
                    },
                    color!(0, 0, 0, 1.0),
                );
            }
        }
    }
}

impl<'a, Message> From<Table<'a>> for Element<'a, Message> {
    fn from(value: Table<'a>) -> Self {
        Element::new(value)
    }
}
