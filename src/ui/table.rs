
use std::marker::PhantomData;

use iced::advanced::graphics::core::event;
use iced::border::Radius;
use iced::widget::{Text, text};
use iced::{Border, Element, Length, Shadow, Size, color};
use iced::advanced::{self, Widget, renderer};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::layout::{Limits, Node};
use crate::prelude::*;


pub struct Table<'a, Message, Theme, Renderer, CallBack>
where
    CallBack: Fn(usize) -> Message
{
    cols:         Vec<Length>,
    cells:        Vec<Element<'a, Message, Theme, Renderer>>,

    rows_message: CallBack,
    row_height:   f32,
}

pub struct TableState {
    rows_visible:  usize,
    last_callback: usize,
}

macro_rules! i_to_xy {
    ($self:expr, $i:expr) => {(
        $i % $self.cols.len(),
        $i / $self.cols.len(),
    )};
}


macro_rules! _xy_to_i {
    ($self:expr, $x:expr, $y:expr) => {
        $self.cols.len() * $y + $x
    };
}

pub struct Row<'a, Data> {
    pub y:    usize,
    pub data: &'a Data,
}

pub struct ColInfo<'a, 'widget, Message, Theme, Renderer, Data> {
    pub size: Length,
    pub col:  &'a dyn Fn(Row<'a, Data>) -> Element<'widget, Message, Theme, Renderer>,
}


impl<'widget, Message, Theme, Renderer, CallBack>
    Table<'widget, Message, Theme, Renderer, CallBack>
where
    Renderer: 'widget + advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    'widget + iced::widget::text::Catalog,
    CallBack: Fn(usize) -> Message
{

    pub fn new<'a, Data>(
        row_height:    f32,
        col_select:    &[ColInfo<'a, 'widget, Message, Theme, Renderer, Data>],
        data:          impl Iterator<Item = &'a Data>,
        rows_message:  CallBack,
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
                    .map (move |col| (col.col)(Row {
                        y:    i,
                        data: row,
                    }))
            )
            .collect ()
        ;

        let cols = col_select.iter().map(|col| col.size).collect();

        Self {
            row_height,
            cols,

            cells,
            rows_message,
        }
    }

}

impl<'a, Message, Theme, Renderer, CallBack>
    Widget<Message, Theme, Renderer>
    for Table<'a, Message, Theme, Renderer, CallBack>
where
    Renderer: 'a + advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    'a + iced::widget::text::Catalog,
    CallBack: Fn(usize) -> Message
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
        tree::State::new(TableState {
            rows_visible:  0,
            last_callback: 0,
        })
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
        let state: &mut TableState = tree.state.downcast_mut();

        state.rows_visible = (limits.max().height / self.row_height).floor() as usize;

        let mut cols_sizes: Vec<_> = self.cols.iter().map(|_| 0.0).collect();

        // size fixed cols
        let fixed = self
            .cols
            .iter      ()
            .enumerate ()
            .filter_map(|c| match c.1 {
                Length::Fixed(f) => Some((c.0, *f)),
                _                => None
            })
        ;

        for (i, size) in fixed {
            cols_sizes[i] = size;
        }

        let fixed_size_sum = cols_sizes.iter().sum();

        let mut child_limits = limits.shrink(Size {
            width: fixed_size_sum,
            height: 0.0,
        });

        // minimium size non-fixed cols
        let shrink = self
            .cols
            .iter      ()
            .enumerate ()
            .filter_map(|c| match c.1 {
                Length::Shrink         => Some(c.0),
                Length::Fill           => Some(c.0),
                Length::FillPortion(_) => Some(c.0),
                _                      => None
            })
        ;

        for col in shrink {
            let mut max_width: f32 = 0.0;

            for i in (col..self.cells.len()).step_by(self.cols.len()) {
                let     cell  = &mut self.cells[i];
                let mut state = &mut tree.children[i];

                let size  = cell.as_widget_mut().layout(&mut state, renderer, &child_limits);

                max_width = max_width.max(size.size().width);
            }

            cols_sizes[col] = max_width;
            child_limits          = child_limits.shrink(Size {
                width:  max_width,
                height: 0.0,
            });
        }

        // BUG: multiple columns with Fill sizing don't work

        let children = self
            .cells
            .iter_mut ()
            .enumerate()
            .map      (|(i, cell)| {

                let (x, y) = i_to_xy!(self, i);

                let cell_size = Size {
                    width:  cols_sizes[x].min(limits.max().width),
                    height: self.row_height,
                };

                let cell_limits = Limits::new(cell_size, cell_size);

                let x = cols_sizes.iter().take(x).sum();
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

        let mut x_offset: f32 = 0.0;

        for (i, cell) in children {
            let (x, y) = i_to_xy!(self, i);

            let layout = layout_iter.next().unwrap();

            let width  = layout.bounds().width;
            let height = layout.bounds().height;

            if x == 0 {
                x_offset = 0.0;
            }

            // calculate new viewport to clip content overflow
            let viewport = &iced::Rectangle {
                x: bounds.x + x_offset,
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

            x_offset += width;
        }
    }

    fn update(
        &mut self,
        tree:       &mut Tree,
        _event:     &iced::Event,
        _layout:     advanced::Layout<'_>,
        _cursor:     advanced::mouse::Cursor,
        _renderer:  &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell:      &mut advanced::Shell<'_, Message>,
        _viewport:  &iced::Rectangle,
    )
    {
        let state: &mut TableState = tree.state.downcast_mut();

        if state.rows_visible != state.last_callback {
            state.last_callback = state.rows_visible;

            let callback = &self.rows_message;

            shell.publish(callback(state.rows_visible));
        }
    }
}

impl<'a, Message: 'a, Theme, Renderer, CallBack>
    From<Table<'a, Message, Theme, Renderer, CallBack>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a + advanced::Renderer + iced::advanced::text::Renderer,
    Theme:    'a + iced::widget::text::Catalog,
    CallBack: 'a + Fn(usize) -> Message

{
    fn from(value: Table<'a, Message, Theme, Renderer, CallBack>) -> Self {
        Element::new(value)
    }
}
