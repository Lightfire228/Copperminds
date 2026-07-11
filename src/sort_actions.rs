
use crate::{cli::{MenuOption, choose}, obsidian::open_in_obsidian, vault::{Index, md_file::{FmProperty, MdFile}, regex}};

pub fn main(index: &mut Index) {

    println!("Sorting by Action");

    let files: Vec<_> = index
        .iter_files_mut()
        .filter        (filter)
        .collect       ()
    ;

    println!("to be sorted: {}", files.len());


    for file in files {

        display_file(file);

        let action = get_action();

        if matches!(action, MenuAction::Calendar) {
            println!("!! Set it in Tasks or Calendar apps");
            continue;
        }

        if matches!(action, MenuAction::Next) {
            continue;
        }

        // re-load any changes made to the file while the cli was waiting for input
        // NOTE: this doesn't catch renames or deletes
        file.refresh();

        file.set_action(action);
        file.write_file();
    }
}

fn filter(f: &&mut MdFile) -> bool {
    // regex!(RE = r"^\d{4}-\d{2}-\d{2}");

    // f.is_unactioned() && !RE.is_match(&f.file_name)
    f.is_unactioned()
}

fn display_file(file: &MdFile) {
    println!("\n\n");
    println!("=== {} ===", file.file_name);
    // println!("{}",         file.raw_text)

    open_in_obsidian(file);
}


fn get_action() -> MenuAction {
    let opts = [
        MenuOption {
            code:  "w",
            name:  "waiting for",
            value: MenuAction::WaitingFor,
        },
        MenuOption {
            code:  "c",
            name:  "calendar",
            value: MenuAction::Calendar,
        },
        MenuOption {
            code:  "p",
            name:  "project",
            value: MenuAction::Project,
        },
        MenuOption {
            code:  "i",
            name:  "info",
            value: MenuAction::Info,
        },
        MenuOption {
            code:  "t",
            name:  "todo",
            value: MenuAction::Todo,
        },
        MenuOption {
            code:  "tc",
            name:  "todo completed",
            value: MenuAction::TodoCompleted,
        },
        MenuOption {
            code:  "ta",
            name:  "todo archived",
            value: MenuAction::TodoArchived,
        },
        MenuOption {
            code:  "m",
            name:  "maybe someday",
            value: MenuAction::MaybeSomeday,
        },
        MenuOption {
            code:  "n",
            name:  "next",
            value: MenuAction::Next,
        },
    ];

    choose("Type", &opts)
}

#[derive(Debug, Clone, Copy)]
enum MenuAction {
    WaitingFor,
    Calendar,
    Project,
    Info,
    Todo,
    TodoCompleted,
    TodoArchived,
    MaybeSomeday,
    Next,
}

impl MdFile {
    fn set_action(&mut self, menu_choice: MenuAction) {

        let action = match menu_choice {
            MenuAction::WaitingFor    => Some("waiting_for"),
            MenuAction::Calendar      => Some("calendar"),
            MenuAction::Project       => Some("project"),
            MenuAction::Todo          |
            MenuAction::TodoCompleted |
            MenuAction::TodoArchived  => Some("todo"),
            MenuAction::MaybeSomeday  => Some("maybe_someday"),

            MenuAction::Info          => None,
            MenuAction::Next          => None,
        };

        let status = match menu_choice {
            MenuAction::TodoCompleted => Some("completed"),
            MenuAction::TodoArchived  => Some("archived"),
            _ => None,
        };

        let type_ = match menu_choice {
            MenuAction::Info => Some("info"),

            _ => None
        };

        if let Some(action) = action {
            self.assign_property(FmProperty::Action, action.to_owned());
        }

        if let Some(status) = status {
            self.assign_property(FmProperty::Status, status.to_owned());
        }

        if let Some(type_) = type_ {
            self.assign_property(FmProperty::Type,   type_.to_owned());
        }
    }

}
