use std::{collections::HashMap, io::{self, Write}};


pub fn get_usr_in(prompt: &str) -> String {
    let mut buffer = String::new();
    let     stdin  = io::stdin ();
    let mut stdout = io::stdout();

    print!("{prompt}\n> ");
    stdout.flush().unwrap();

    stdin.read_line(&mut buffer).unwrap();

    buffer.trim().to_owned()

}

pub fn choose<T>(title: &str, opts: &[MenuOption<T>]) -> T
where
    T: Copy
{
    choose_with(title, opts, None)
}

pub fn choose_with<T>(title: &str, opts: &[MenuOption<T>], default: Option<T>) -> T
where
    T: Copy
{

    let width = opts.iter().map(|o| o.code.len()).max().unwrap();

    println!("{title}");

    for o in opts {
        let w = " ".repeat(width - o.code.len());
        println!("  {}{w} - {}", o.code, o.name);
    }

    let opts: HashMap<_, T> = opts
        .into_iter()
        .map      (|o| (o.code, o.value))
        .collect  ()
    ;

    loop {
        let usrin = get_usr_in("").to_lowercase();

        if let (true, Some(default)) = (usrin.is_empty(), default) {
            return default
        }

        let Some(val) = opts.get(usrin.as_str()) else {
            println!("Unknown type");
            continue;
        };

        break *val;
    }
}

pub struct MenuOption<T>
where
    T: Copy
{
    pub code:  &'static str,
    pub name:  &'static str,
    pub value: T,
}
