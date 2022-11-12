use clap::Parser;
use std::time::Duration;
use dbus::blocking::Connection;
use dbus::channel::Sender;
use dbus::Message;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
  #[command(aliases = ["inc", "i", "+"], about = "Increase volume by [step]")]
  Increase {
    step: i64,
  },
  #[command(aliases = ["dec", "d", "-"], about = "Decrease volume by [step]")]
  Decrease {
    step: i64,
  },
  // slash is meant to represent the slash that goes through the mute icon lol
  #[command(name = "toggle-mute", aliases = ["mute", "unmute", "m", "/"], about = "Toggle mute")]
  ToggleMute,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Args = Args::parse();

  let conn = Connection::new_session()?;
  let proxy = conn.with_proxy("me.diced.volumed.server", "/", Duration::from_millis(5000));


  match args.command {
    Commands::Increase { step } => {
      let msg = Message::new_method_call("me.diced.volumed.server", "/", "me.diced.volumed.server", "IncrVol")?
        .append1(step);

      conn.send(msg).unwrap();
    }
    Commands::Decrease { step } => {
      let msg = Message::new_method_call("me.diced.volumed.server", "/", "me.diced.volumed.server", "DecrVol")?
        .append1(step);

      conn.send(msg).unwrap();
    }
    Commands::ToggleMute => {
      proxy.method_call("me.diced.volumed.server", "MuteVol", ())?;
    }
  }

  Ok(())
}