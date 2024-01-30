use askama::Template;

#[derive(Template)]
#[template(path = "root.html")]
pub struct RootTemplate<'a> {
    pub name: &'a str,
    pub title: &'a str,
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate<'a> {
    pub name: &'a str,
}
