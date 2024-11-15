use std::{collections::HashMap, sync::Mutex};

type File = ();

#[derive(Debug)]
pub struct FsModule {
    files: Mutex<HashMap<i16, File>>,
}
