
use crate::{cli::{MenuOption, choose}, obsidian::open_in_obsidian, vault::{Index, md_file::{FmProperty, MdFile}}};

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

    macro_rules! opt {
        ($code:expr, $name:expr, $action:expr) => {
            MenuOption {
                code:  $code,
                name:  $name,
                value: $action,
            }
        };
    }

    let opts = [
        opt!("w",  "waiting",  MenuAction::WaitingFor),
        opt!("c",  "calendar", MenuAction::Calendar),
        opt!("p",  "project",  MenuAction::Project),
        opt!("i",  "info",     MenuAction::Info),
        opt!("t",  "todo",     MenuAction::Todo),
        opt!("tc", "todo",     MenuAction::TodoCompleted),
        opt!("ta", "todo",     MenuAction::TodoArchived),
        opt!("m",  "maybe",    MenuAction::MaybeSomeday),
        opt!("n",  "next",     MenuAction::Next),
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
            self.set_property(FmProperty::Action, action.to_owned());
        }

        if let Some(status) = status {
            self.set_property(FmProperty::Status, status.to_owned());
        }

        if let Some(type_) = type_ {
            self.set_property(FmProperty::Type,   type_.to_owned());
        }
    }

}
