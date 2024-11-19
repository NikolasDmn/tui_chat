mod networking;
mod tui;
use std::io;
fn main() -> io::Result<()> {
    tui::start()
}
