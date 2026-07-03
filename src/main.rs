
mod backup;
mod cli;
mod sort_actions;
mod sort_type;
mod summary;
mod vault;



use vault::Index;


fn main() {

    let mut index = Index::build();

    index.delete_empty_unnamed_files();


    write_summary_page(&mut index);

    sort_type::main(&mut index);
}

fn write_summary_page(index: &mut Index) {
    index.backup();

    let summary = summary::get_summary(&index);

    let file = index
        .md_files
        .iter_mut()
        .find    (|x| x.file_name == "Copperminds Summary Page.md")
        .expect  ("Unable to find Copperminds Summary Page")
    ;

    file.md_text = summary;
    file.write_file();

}
// ---- print status
