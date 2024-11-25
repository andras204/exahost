use std::collections::HashMap;

type Link = ();

#[derive(Debug)]
pub struct NetModule {
    links: HashMap<i16, Link>,
}
