
use std::marker::PhantomData;
use std::mem;

use iced::advanced::graphics::core::event;
use iced::advanced::renderer::Quad;
use iced::border::Radius;
use iced::widget::{Text, text};
use iced::{Border, Color, Element, Length, Padding, Shadow, Size, color};
use iced::advanced::{self, Widget, renderer};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::layout::{Limits, Node};
use crate::prelude::*;

const ALT_ROW_COLOR: Color = color!(0, 0, 0, 0.25);

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


macro_rules! xy_to_i {
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
            width:  Length::Shrink,
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

        if self.cells.is_empty() {
            return Node::new(limits.min());
        }

        let rows = (self.cells.len() / self.cols.len()).min(state.rows_visible);

        let mut cols_sizes: Vec<f32>  = vec![0.0;             self.cols .len()];
        let mut children:   Vec<Node> = vec![Node::default(); self.cells.len()];

        let mut fixed_cols  = vec![];
        let mut shrink_cols = vec![];
        let mut grow_cols   = vec![];

        let width_available = limits.max().width;

        for (i, col) in self.cols.iter().enumerate() {
            match col {
                Length::Fill           => grow_cols  .push(i),
                Length::FillPortion(_) => grow_cols  .push(i),
                Length::Shrink         => shrink_cols.push(i),
                Length::Fixed(x)       => fixed_cols .push((i, x)),
            }
        }

        // Layout fixed width cols
        for (x, width) in fixed_cols {
            for y in 0..rows {
                let i = xy_to_i!(self, x, y);

                let limits = limits.height(self.row_height).width(Length::Fixed(*width));
                let node   = self
                    .cells[i]
                    .as_widget_mut()
                    .layout       (&mut tree.children[i], renderer, &limits)
                ;

                children[i] = node;
            }

            cols_sizes[x] = *width;
        }


        // Layout shrink and min width grow cols
        for x in shrink_cols.iter().chain(&grow_cols) {

            let consumed_width: f32 = cols_sizes.iter().sum();

            let mut col_width = 0.0;

            for y in 0..rows {
                let i = xy_to_i!(self, x, y);

                let limits = limits.height(self.row_height).max_width(width_available - consumed_width);
                let node   = self
                    .cells[i]
                    .as_widget_mut()
                    .layout       (&mut tree.children[i], renderer, &limits)
                ;

                if col_width < node.size().width {
                    col_width = node.size().width;
                }

                children[i] = node;
            }

            cols_sizes[*x] = col_width;
        }

        // Resize grow cols

        let consumed_width: f32 = cols_sizes.iter().sum();
        let remaining           = width_available - consumed_width;

        // TODO:
        if grow_cols.len() > 1 {
            panic!("Mutliple grow cols not yet supported");
        }


        for x in grow_cols {
            cols_sizes[x] += remaining;

            for node in children.iter_mut().skip(x).step_by(self.cols.len()) {
                grow_node(node, cols_sizes[x]);
            }
        }

        // Position cols
        for (i, node) in children.iter_mut().enumerate() {
            let (x, y) = i_to_xy!(self, i);

            let x_offset = cols_sizes.iter().take(x).sum();

            node.move_to_mut((x_offset, y as f32 * self.row_height));

        }

        let limits = limits.min_width(cols_sizes.iter().sum()).min_height(rows as f32 * self.row_height);

        Node::with_children(limits.min(), children)
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

            if y % 2 == 0 {
                draw_alt_row_background(renderer, viewport);
            }

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


fn draw_alt_row_background<'a, Renderer>(
    renderer: &mut Renderer,
    viewport: &iced::Rectangle
)
where
    Renderer: 'a + advanced::Renderer + iced::advanced::text::Renderer,
{
    draw_background(renderer, viewport, ALT_ROW_COLOR);
}

fn draw_background<'a, Renderer>(
    renderer: &mut Renderer,
    viewport: &iced::Rectangle,
    color:    Color,
)
where
    Renderer: 'a + advanced::Renderer + iced::advanced::text::Renderer,
{
    renderer.fill_quad(
        Quad {
            bounds: *viewport,
            ..Default::default()
        },
        color,
    );
}


// TODO: 0.15 new layout nodes have grow support
fn grow_node(node: &mut Node, new_width: f32) {
    let mut tmp = Node::default();

    mem::swap(node, &mut tmp);
    let padding = new_width - tmp.size().width;

    *node = Node::container(tmp, Padding::ZERO.right(padding));
}
