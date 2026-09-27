
use std::marker::PhantomData;

use iced::border::Radius;
use iced::widget::{Text, text};
use iced::{Border, Element, Length, Shadow, Size, color};
use iced::advanced::{self, Widget, renderer};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::layout::{Limits, Node};
use log::warn;

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
                .map(|_| text!("{TEXT}").into())
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

    fn children(&self) -> Vec<Tree> {
        self.cells.iter().map(|x| Tree::new(x)).collect()
    }

    fn state(&self) -> tree::State {
        tree::State::new(())
    }

    // this is what populates `tree.children`
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
        // let mut limits = *limits;

        // for cell in self.cells.iter_mut() {
        //     cell.as_widget_mut().layout(tree, renderer, limits);

        // }

        let size = Size {
            width:  self.cols as f32 * CELL_WIDTH,
            height: self.rows as f32 * CELL_HEIGHT,
        };


        let mut limits = *limits;

        let children = self
            .cells
            .iter_mut ()
            .enumerate()
            .map      (|(i, cell)| {
                cell.as_widget_mut().layout(&mut tree.children[i], renderer, &limits)
            })
            .collect  ()
        ;

        Node::with_children(size, children)
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

        for y in 0..self.rows {
            for x in 0..self.cols {

                renderer.fill_quad(
                    cell(iced::Rectangle {
                        x:      bounds.x + width  * x as f32,
                        y:      bounds.y + height * y as f32,
                        width,
                        height,
                    }),
                    color!(0, 0, 0, 0.0),
                );


                let i = y * self.cols + x;
                self.cells[i].as_widget().draw(&tree.children[i], renderer, theme, style, layout, cursor, viewport);
            }
        }
    }
}

fn cell(rect: iced::Rectangle) -> renderer::Quad {
    renderer::Quad {
        bounds: rect,
        border: Border {
            color:  color!(255, 255, 255),
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


struct HeadNodeArgs {

}

fn cell_node(args: HeadNodeArgs) {

}
