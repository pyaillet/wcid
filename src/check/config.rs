#[derive(Clone, Debug)]
pub struct Config {
    pub display_group: bool,
    pub namespace: Option<String>,
    pub hide_forbidden: bool,
}
