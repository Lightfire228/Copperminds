use iced::Length::Fill;
use iced::keyboard;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Button, Column, button, column, pick_list, row, text, tooltip};

use crate::ui::{Message, QueueType, UIMode};
use crate::vault::command::VaultCommand;
use crate::vault::md_file::FileView;


#[derive(Debug, Clone)]
pub struct SortQueue {
    pub queue_type:   QueueType,
    pub files:        Vec<FileView>,

}

macro_rules! table {
    ($( ($action:ident, $key:literal, $name:literal) ),*$(,)? ) => {[

        $(
            MenuAction {
                key:    $key,
                name:   $name,
                action: Action::$action
            },
        )*
    ]}
}


static NEEDS_TYPE: &'static [MenuAction] = &table!(
    (SetTypeInfo,           "i", "type - info"),
    (SetTypeAction,         "a", "type - action"),
);

static NEEDS_ACTION: &'static [MenuAction] = &table!(
    (SetActionTodo,         "t", "action - todo"),
    (SetActionWaitingFor,   "w", "action - waiting for"),
    // (SetActionCalendar,     "c", "action - calendar"),
    (SetActionProject,      "p", "action - project"),
    (SetActionMaybeSomeday, "m", "action - maybe someday"),
    (SetTypeInfo,           "i", "type   - info"),
    (SetStatusComplete,     "c", "status - complete"),
    (SetStatusArchived,     "a", "status - archived"),
);


impl SortQueue {

    pub fn view(&self) -> Element<'_, Message> {

        let options = match self.queue_type {
            QueueType::NeedsType   => NEEDS_TYPE,
            QueueType::NeedsAction => NEEDS_ACTION,
        }
            .iter()
            .map (|o| text!("{}", o.name).into())
        ;

        container(
            row![
                container(
                    column![
                        text!("{}", self.queue_type),
                        text!("==="),
                        column(options),
                    ]
                )
                    .width  (300)
                    .padding(10)
                ,
                container(
                    column(
                        self.files.iter().map(|f|
                            text!("{}", &f.name).into()
                        )
                    ),
                )
                    .width  (Fill)
                    .padding(10)
            ]
            .spacing(40)
        )
        .into()
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn handle_key_event(&self, key: Key) -> Task<Message> {

        type N = keyboard::key::Named;

        match key {
              Key::Named(N::ArrowLeft)
            | Key::Named(N::Escape)    => {

                Task::future(async {
                    Message::NavigateBack
                })
            },
            Key::Character(_) => {
                Task::none()
            },
            _ => Task::none()
        }
    }

}

impl Into<UIMode> for SortQueue {
    fn into(self) -> UIMode {
        UIMode::SortQueue(self)
    }
}


struct MenuAction {
    pub key:     &'static str,
    pub name:    &'static str,
    pub action:  Action,
}

enum Action {
    SetTypeInfo,
    SetTypeAction,
    SetActionWaitingFor,
    // SetActionCalendar,
    SetActionProject,
    SetActionTodo,
    SetActionMaybeSomeday,
    SetStatusComplete,
    SetStatusArchived,
}
