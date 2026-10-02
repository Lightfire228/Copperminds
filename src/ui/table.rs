
use std::marker::PhantomData;

use iced::border::Radius;
use iced::widget::{Text, text};
use iced::{Border, Element, Length, Shadow, Size, color};
use iced::advanced::{self, Widget, renderer};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::layout::{Limits, Node};
use crate::prelude::*;

const CELL_HEIGHT: f32 = 50.0;


pub struct Table<'a, Message, Theme, Renderer> {
    _rows:      usize,
    cols:       usize,
    cells:      Vec<Element<'a, Message, Theme, Renderer>>

}

macro_rules! i_to_xy {
    ($self:expr, $i:expr) => {(
        $i % $self.cols,
        $i / $self.cols,
    )};
}


macro_rules! _xy_to_i {
    ($self:expr, $x:expr, $y:expr) => {
        $self.cols * $y + $x
    };
}

pub struct RowInfo {
    pub row:  usize,
}

macro_rules! get_data {
    () => {
        &[Box<dyn Fn(&'a Data, RowInfo) -> Element<'widget, Message, Theme, Renderer>>]
    };
}

impl<'widget, Message, Theme, Renderer>
    Table<'widget, Message, Theme, Renderer>
where
    Renderer: 'widget + advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    'widget + iced::widget::text::Catalog,
{

    pub fn new<'a, Data>(
        col_select: get_data!(),
        data:       impl Iterator<Item = &'a Data>
    )
        -> Self
    where
        'widget:  'a,
        Data:     'a,
    {

        let cells: Vec<_> = data
            .enumerate()
            .flat_map (|(i, row)|

                col_select
                    .iter()
                    .map (move |col| col(row, RowInfo {
                        row:  i,
                    }))
            )
            .collect ()
        ;

        Self {
            _rows: cells     .len(),
            cols:  col_select.len(),

            cells,
        }
    }

}

impl<'a, Message, Theme, Renderer>
    Widget<Message, Theme, Renderer>
    for Table<'a, Message, Theme, Renderer>
where
    Renderer: 'a + advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    'a + iced::widget::text::Catalog,
{

    fn size(&self) -> Size<Length> {
        Size {
            width:  Length::Fill,
            height: Length::Fill,
        }
    }

    fn children(&self) -> Vec<Tree> {
        self
            .cells
            .iter   ()
            .map    (Tree::new)
            .collect()
    }

    fn state(&self) -> tree::State {
        tree::State::new(())
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.cells);
    }


    fn layout(
        &mut self,
        tree:     &mut Tree,
        renderer: &Renderer,
        limits:   &Limits,
    )
        -> Node
    {

        let cell_size = Size {
            width:  limits.max().width / self.cols as f32,
            height: CELL_HEIGHT,
        };

        let cell_limits = Limits::new(cell_size, cell_size);

        let children = self
            .cells
            .iter_mut ()
            .enumerate()
            .map      (|(i, cell)| {

                let (x, y) = i_to_xy!(self, i);

                let x = x as f32 * cell_size.width;
                let y = y as f32 * cell_size.height;

                cell
                    .as_widget_mut()
                    .layout       (&mut tree.children[i], renderer, &cell_limits)
                    .move_to      ((x, y))
            })
            .collect()
        ;

        Node::with_children(limits.max(), children)
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

        let mut layout_iter = layout.children();


        let children = self
            .cells
            .iter     ()
            .enumerate()
        ;

        for (i, cell) in children {
            let (x, y) = i_to_xy!(self, i);

            let layout = layout_iter.next().unwrap();

            let width  = layout.bounds().width;
            let height = layout.bounds().height;

            // calculate new viewport to clip content overflow
            let viewport = &iced::Rectangle {
                x: bounds.x + (x as f32 * layout.bounds().width),
                y: bounds.y + (y as f32 * layout.bounds().height),
                width,
                height,
            };

            cell.as_widget().draw(
                &tree.children[i],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }
    }
}

fn _cell(rect: iced::Rectangle) -> renderer::Quad {
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

impl<'a, Message: 'a, Theme, Renderer>
    From<Table<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a + advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    'a + iced::widget::text::Catalog,

{
    fn from(value: Table<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
