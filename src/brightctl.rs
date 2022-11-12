use clap::Parser;
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
  #[command(aliases = ["inc", "i", "+"], about = "Increase brightness by [step]")]
  Increase {
    step: i64,
  },
  #[command(aliases = ["dec", "d", "-"], about = "Decrease brightness by [step]")]
  Decrease {
    step: i64,
  },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Args = Args::parse();

  let conn = Connection::new_session()?;

  match args.command {
    Commands::Increase { step } => {
      let msg = Message::new_method_call("me.diced.brightd.server", "/", "me.diced.brightd.server", "IncrLight")?
        .append1(step);

      conn.send(msg).unwrap();
    }
    Commands::Decrease { step } => {
      let msg = Message::new_method_call("me.diced.brightd.server", "/", "me.diced.brightd.server", "DecrLight")?
        .append1(step);

      conn.send(msg).unwrap();
    }
  }

  Ok(())
}