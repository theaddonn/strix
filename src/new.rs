use std::path::PathBuf;
use dialoguer::Select;
use crate::args::CliNewSubCommand;

pub async fn new(new: CliNewSubCommand) -> bool {
    let name = new.name;

    let select = Select::new()
        .with_prompt(format!("Select an Addon template for {name:?}"))
        .items(&["📄 Vanilla", "🗿 Regolith", "🛠️ Dash"])
        .report(true)
        .default(0);

    match select.interact() {
        Ok(0) => { new_vanilla(name, new.path) }
        Ok(1) => {}
        Ok(3) => {}
        Ok(other) => {}
        Err(err) => {}
    }

    false
}

fn new_vanilla(name: String, path: Option<PathBuf>) {

}
