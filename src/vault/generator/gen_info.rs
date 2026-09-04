use std::fs;

use rand::random_bool;
use yaml_serde::Mapping;

use crate::vault::generator::{GeneratorOpts, generate_title_info, generate_title_unique, get_file_name, lorem_ipsum};



pub fn generate_info(opts: &GeneratorOpts) -> Option<()>{

    if !opts.settings.gen_info {
        None?
    }

    let (title, body) = generate_info_details(&opts)?;
    let file          = get_file_name(&opts.path, &title);

    fs::write(file, &body).unwrap();

    Some(())
}

fn generate_info_details(opts: &GeneratorOpts) -> Option<(String, String)> {
    let is_named = random_bool(0.90);

    let (title, body) = if is_named {(
        generate_title_info(),
        lorem_ipsum        (),
    )}
    else {
        if !(opts.settings.gen_unnamed && opts.settings.gen_unsorted) {
            None?
        }

        let body = if random_bool(0.50) {
            lorem_ipsum()
        }
        else {
            ""
        };

        (
            generate_title_unique(),
            body,
        )
    };

    let fm = if random_bool(0.90) {
        "type: info"
    }
    else if opts.settings.gen_unsorted {
        ""
    }
    else {
        None?
    };

    let fm: Option<Mapping> = yaml_serde::from_str(&fm).ok();

    let fm = match fm {
        None     => String::new(),
        Some(fm) => format!("---\n{}\n---\n", yaml_serde::to_string(&fm).unwrap()),
    };

    Some((title, format!("{fm}{body}")))

}
