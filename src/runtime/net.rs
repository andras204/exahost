use std::{collections::HashMap, sync::Mutex};

type Link = ();

#[derive(Debug)]
pub struct NetModule {
    links: Mutex<HashMap<i16, Link>>,
}
