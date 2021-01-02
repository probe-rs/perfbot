use askama::Template;

use super::handlers::Pr;

#[derive(Template)]
#[template(path = "commands/perf.html")]
pub struct PerfTemplate {
    pub prs: Vec<Pr>,
}

#[derive(Template)]
#[template(path = "commands/help.html")]
pub struct HelpTemplate {}
