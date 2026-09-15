use std::{fs, path::Path, sync::Mutex};

use file_id::FileId;
use yaml_serde::Mapping;


static COUNTER: Mutex<u64> = Mutex::new(0);


pub fn mapping_to_str(fm: Mapping) -> String {
    format!("---\n{}---\n", yaml_serde::to_string(&fm).unwrap())
}

pub fn id() -> FileId {
    let mut id = COUNTER.lock().unwrap();

    *id += 1;

    FileId::Inode {
        device_id:    *id -1,
        inode_number: *id -1,
    }
}


pub fn load_file(name: &str) -> String {
    let dir  = format!("{}/test_files/{name}", env!("CARGO_MANIFEST_DIR"));
    let path = Path::new(&dir);

    fs::read_to_string(path).unwrap()
}
