
use crate::{cli::{MenuOption, choose}, obsidian::open_in_obsidian, vault::{Index, md_file::{FmAction, FmProperty, MdFile}}};


pub fn main(index: &mut Index) {

    println!("Sorting by Action");

    let files: Vec<_> = index
        .iter_files_mut()
        .filter        (|f| f.is_unactioned())
        .collect       ()
    ;

    println!("to be sorted: {}", files.len());


    for file in files {

        display_file(file);

        let action = get_action();

        if matches!(action, Action::Calendar) {
            println!("!! Set it in Tasks or Calendar apps");
            continue;
        }

        // re-load any changes made to the file while the cli was waiting for input
        // NOTE: this doesn't catch renames or deletes
        file.refresh();

        file.set_action(action);
        file.write_file();
    }
}

fn display_file(file: &MdFile) {
    println!("\n\n");
    println!("=== {} ===", file.file_name);
    // println!("{}",         file.raw_text)

    open_in_obsidian(file);
}


fn get_action() -> Action {
    let opts = [
        MenuOption {
            code:  "w",
            name:  "waiting for",
            value: Action::WaitingFor,
        },
        MenuOption {
            code:  "c",
            name:  "calendar",
            value: Action::Calendar,
        },
        MenuOption {
            code:  "t",
            name:  "todo",
            value: Action::Todo,
        },
        MenuOption {
            code:  "tc",
            name:  "todo completed",
            value: Action::TodoCompleted,
        },
        MenuOption {
            code:  "ta",
            name:  "todo archived",
            value: Action::TodoArchived,
        },
        MenuOption {
            code:  "m",
            name:  "maybe someday",
            value: Action::MaybeSomeday,
        },
    ];

    choose("Type", &opts)
}

#[derive(Debug, Clone, Copy)]
enum Action {
    WaitingFor,
    Calendar,
    Todo,
    TodoCompleted,
    TodoArchived,
    MaybeSomeday,
}

impl MdFile {
    fn set_action(&mut self, action: Action) {
        self.assign_property(FmProperty::Action, match action {
            Action::WaitingFor    => "waiting_for"  .to_owned(),
            Action::Todo          => "todo"         .to_owned(),
            Action::TodoCompleted => "todo"         .to_owned(),
            Action::TodoArchived  => "todo"         .to_owned(),
            Action::MaybeSomeday  => "maybe_someday".to_owned(),
            Action::Calendar      => "calendar"     .to_owned(),
        });

        // TODO: make this a set func on MdFile::set_status(Status::Completed)
        let status = match action {
            Action::TodoCompleted => Some("completed".to_owned()),
            Action::TodoArchived  => Some("archived" .to_owned()),

            _ => None,
        };

        if let Some(status) = status {
            self.assign_property(FmProperty::Status, status);
        }
    }

}
