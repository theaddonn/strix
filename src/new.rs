use std::path::PathBuf;
use dialoguer::{MultiSelect, Select};
use crate::args::CliNewSubCommand;

pub async fn new(new: CliNewSubCommand) -> bool {
    let name = new.name;

    let select = Select::new()
        .with_prompt(format!("Select an Addon template for {name:?}"))
        .items(&["📄 Vanilla", "🗿 Regolith", "🛠️ Dash"])
        .report(true)
        .default(0);

    let ret = match select.interact() {
        Ok(0) => { new_vanilla(name, new.path) }
        Ok(1) => { unimplemented!() }
        Ok(3) => { unimplemented!() }
        Ok(other) => {
            true
        }
        Err(err) => {
            true
        }
    };

    false
}

fn new_vanilla(name: String, path: Option<PathBuf>) -> bool {
    let select = MultiSelect::new()
        .with_prompt(format!("Select an Addon template for {name:?}"))
        .items(&["📄 Behaviour Pack", "🗿 Resource Pack", "🛠️ World Template", "🛠️ Skin Pack"])
        .report(true);

    let selected = select.interact().unwrap_or_else(|err| {

    });
}
