use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "echo")]
pub struct Echo {
    #[clap()]
    pub contents: String,
}

#[derive(Parser, Debug)]
#[clap(name = "theme")]
pub struct ThemeSong {
    pub slug: String,
    pub start: u32,
    pub duration: u32,
}

// #[derive(Subcommand, Debug)]
// pub enum ThemeCommands {
//     /// Adds files to myapp
//     Add {
//     },
//
//     Set {
//         songname: String,
//     },
// }
