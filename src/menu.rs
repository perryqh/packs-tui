#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Home,
    Constants,
    Actions,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Constants => 1,
            MenuItem::Actions => 2,
        }
    }
}
