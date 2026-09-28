
use std::marker::PhantomData;

use iced::border::Radius;
use iced::widget::{Text, text};
use iced::{Border, Element, Length, Shadow, Size, color};
use iced::advanced::{self, Widget, renderer};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::layout::{Limits, Node};
use crate::prelude::*;

const CELL_WIDTH:  f32 = 50.0;
const CELL_HEIGHT: f32 = 50.0;

const TEXT: &'static str = "asdf";

pub struct Table<'a, Message, Theme, Renderer> {
    // _p: PhantomData<&'a ()>,
    rows:  usize,
    cols:  usize,
    cells: Vec<Element<'a, Message, Theme, Renderer>>,
}

enum Message {}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer>
where
    Renderer: 'a + advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    'a + iced::widget::text::Catalog,

{
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            rows,
            cols,
            cells: (0..rows * cols)
                .into_iter()
                .map(|_| text!("{TEXT}").wrapping(text::Wrapping::None).into())
                .collect()
            ,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Table<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    iced::widget::text::Catalog,
{

    fn size(&self) -> Size<Length> {
        Size {
            width:  Length::Fill,
            height: Length::Fill,
        }
    }

    // This is responsible for calculating the arangement of the widget, represented as a `Node` tree,
    // and is used to calculate the `Layout` tree passed to `draw()`
    //
    // limits specifies the minimum and maximum bounds of real estate available to the widget
    fn layout(
        &mut self,
        tree:     &mut Tree,
        renderer: &Renderer,
        limits:   &Limits,
    )
        -> Node
    {

        let cell_size = Size {
            width:  CELL_WIDTH,
            height: CELL_HEIGHT,
        };

        let cell_limits = Limits::new(cell_size, cell_size);

        let children = self
            .cells
            .iter_mut ()
            .enumerate()
            .map      (|(i, cell)| {

                let x = i % self.cols;
                let y = i / self.cols;

                let x = x as f32 * CELL_WIDTH;
                let y = y as f32 * CELL_HEIGHT;

                cell
                    .as_widget_mut()
                    .layout       (&mut tree.children[i], renderer, &cell_limits)
                    .move_to      ((x, y))
            })
            .collect()
        ;

        let size = Size {
            width:  self.cols as f32 * CELL_WIDTH,
            height: self.rows as f32 * CELL_HEIGHT,
        };

        Node::with_children(size, children)
    }

    // this sets the state held by the state tree for this current node
    fn state(&self) -> tree::State {
        tree::State::new(())
    }

    // this is responsible for walking the children and populating their nodes in the state tree
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.cells);
    }


    fn draw(
        &self,
        tree:     &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme:    &Theme,
        style:    &iced::advanced::renderer::Style,
        layout:    iced::advanced::Layout<'_>,
        cursor:    iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {

        let bounds = layout.bounds();

        if !bounds.intersects(viewport) {
            return;
        }

        let width  = bounds.width  / self.cols as f32;
        let height = bounds.height / self.rows as f32;

        let mut layout_iter = layout.children();
        for y in 0..self.rows {
            for x in 0..self.cols {
                let i = y * self.cols + x;

                self.cells[i].as_widget().draw(
                    &tree.children[i],
                    renderer,
                    theme,
                    style,
                    layout_iter.next().unwrap(),
                    cursor,
                    viewport,
                );

                renderer.fill_quad(
                    cell(iced::Rectangle {
                        x:      bounds.x + width  * x as f32,
                        y:      bounds.y + height * y as f32,
                        width,
                        height,
                    }),
                    color!(0, 0, 0, 1.0),
                );
            }
        }
    }
}

fn cell(rect: iced::Rectangle) -> renderer::Quad {
    renderer::Quad {
        bounds: rect,
        border: Border {
            color:  color!(125, 255, 255),
            width:  1.0,
            radius: Radius::new(0.3),
        },
        shadow: Shadow::default(),
        snap:   true,
    }
}

impl<'a, Message: 'a, Theme, Renderer> From<Table<'a, Message, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a + advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    'a + iced::widget::text::Catalog,

{
    fn from(value: Table<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
