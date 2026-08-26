mod components;
mod vault_subscription;


use std::fmt::Display;
use std::hash::Hash;

use iced::keyboard::{Event};
use iced::{Element, Font, Subscription, Theme, application};
use tokio::sync::mpsc::{Sender};
use tokio::sync::oneshot;

use crate::ui::components::select_queue::{self, SelectQueue};
use crate::ui::components::sort_queue::{self, SortQueue};
use crate::vault::{ENV};
use crate::vault::command::{Cmd, VaultCommand, VaultUpdate};
use crate::prelude::*;

use std::mem;


type Task = iced::Task<Message>;

pub fn main(tx: Sender<VaultCommand>) {
    info!("iced ui");


    application(
        move || App::new(tx.clone()),
        App::update,
        App::view
    )
        .theme       (Theme::Dark)
        .title       (App  ::title)
        .subscription(App  ::subscription)
        .default_font(Font ::MONOSPACE)
        .run         ()
        .unwrap      ()
    ;
}

struct App {
    vault:      Sender<VaultCommand>,
    ui_mode:    UIMode,
}

#[derive(Debug)]
enum Message {
    None,
    Event           (iced::Event),
    SelectQueue     (select_queue::Message),
    SortQueue       (sort_queue  ::Message),
    VaultUpdate     (VaultUpdate)
}

#[derive(Debug)]
enum Action {
    None,
    SelectQueue     (select_queue::Action),
    SortQueue       (sort_queue  ::Action),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueueType {
    NeedsType,
    NeedsAction,
}


#[derive(Debug)]
enum UIMode {
    SelectQueue(SelectQueue),
    SortQueue  (SortQueue),
}


impl App {
    fn new(tx: Sender<VaultCommand>) -> (Self, Task) {(

        Self {
            vault:   tx,
            ui_mode: SelectQueue::new().into(),
        },
        Self::on_startup(),

    )}

    fn on_startup() -> Task {
        Task::none()
    }

    fn title(&self) -> String {
        format!("Copperminds - {}", ENV.name())
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.get_keyboard_subscriber(),
            self.get_vault_subscriber   (),
        ])
    }

    fn get_keyboard_subscriber(&self) -> Subscription<Message> {
        iced::event::listen().map(Message::Event)
    }

    fn get_vault_subscriber(&self) -> Subscription<Message> {

        let vault = VaultSubscriber { sub: self.vault.clone() };

        Subscription::run_with(
            vault,
            |vault| vault_subscription::connect(
                vault.sub.clone()
            )
        )
            .map(Message::VaultUpdate)
    }


    fn update(&mut self, message: Message) -> Task {


        use iced::Event::Keyboard;

        match message {
            Message::Event(Keyboard(event)) => {
                trace!("keyboard event {event:?}");
                let action = self.handle_key_event(event);

                return self.handle_action(action)
            }

            Message::VaultUpdate(message) => {
                debug!("vault update event {message:?}");

                use sort_queue::Message::VaultUpdate;

                return self.update(match &self.ui_mode {
                    UIMode::SelectQueue(_) => Message::None,
                    UIMode::SortQueue  (_) => Message::SortQueue(VaultUpdate(message)),
                })
            },

            _ => {}
        }

        macro_rules! handle {
            ( $($type:ident,)*$(,)? ) => {

                match (&mut self.ui_mode, message) {$(
                    (UIMode::$type(component), Message::$type(message)) => {
                        let t = stringify!($type);
                        debug!("{t} event {message:?}");

                        Some(Action::$type(component.update(message)))

                    })*

                    _ => None,
                }
            };
        }

        let Some(action) = handle!(
            SelectQueue,
            SortQueue,
        ) else {
            return Task::none()
        };

        self.handle_action(action)
    }

    fn handle_action(&mut self, action: Action) -> Task {
        match action {
            Action::None                => Task::none(),
            Action::SelectQueue(action) => self.handle_action_select_queue(action),
            Action::SortQueue  (action) => self.handle_action_sort_queue  (action),
        }
    }

    fn handle_action_select_queue(&mut self, action: select_queue::Action) -> Task {
        type Action = select_queue::Action;

        match action {
            Action::None                      => Task::none(),
            Action::QueueSelected(queue_type) => {
                let (state, task) = SortQueue::new(queue_type, self.vault.clone());

                self.ui_mode = state.into();

                task.map(Message::SortQueue)
            }
        }
    }

    fn handle_action_sort_queue(&mut self, action: sort_queue::Action) -> Task {
        type Action = sort_queue::Action;

        match action {
            Action::None         => Task::none (),
            Action::Run(task)    => task.map   (Message::SortQueue),
            Action::NavigateBack => {
                self.ui_mode = SelectQueue::new().into();
                Task::none()
            },
        }
    }

    fn handle_key_event(&mut self, event: Event) -> Action {

        let Event::KeyPressed { key, .. } = event else {
            return Action::None;
        };


        match &mut self.ui_mode {
            UIMode::SelectQueue(x) => Action::SelectQueue(x.handle_key_event(&key)),
            UIMode::SortQueue  (x) => Action::SortQueue  (x.handle_key_event( key)),
        }
    }

    fn view(&self) -> Element<'_, Message> {

        match &self.ui_mode {
            UIMode::SelectQueue(x) => x.view().map(Message::SelectQueue),
            UIMode::SortQueue  (x) => x.view().map(Message::SortQueue),
        }
    }
}



async fn send_vault_cmd<T>(vault: &Sender<VaultCommand>, cmd: impl Cmd<T>) -> T {
    let (tx, rx) = oneshot::channel();

    vault
        .send(cmd.to_command(tx))
        .await
        .unwrap()
    ;

    rx.await.unwrap()
}


impl Display for QueueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueType::NeedsType   => write!(f, "Needs Type"),
            QueueType::NeedsAction => write!(f, "Needs Action"),
        }
    }
}


struct VaultSubscriber {
    sub: Sender<VaultCommand>,
}

// Subscribers are identified by their data's hash, and their function pointer,
// but (my assumption is) i only need one vault subscriber per app
//
// this may cause weirdness depending on what iced does with the subscribers on repeated app inits
impl Hash for VaultSubscriber {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}
