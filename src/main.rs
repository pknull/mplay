mod config;
mod cover;
mod mpris_client;
mod ui;

use anyhow::Result;
use config::Config;
use ui::App;

fn main() -> Result<()> {
    // Load config
    let config = Config::load()?;

    // Create and run app
    let mut app = App::new(config);
    app.run()?;

    Ok(())
}
