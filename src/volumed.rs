use std::process::Command;
use std::sync::{Arc, Mutex};
use dbus::blocking::Connection;
use dbus_crossroads::{Crossroads, IfaceBuilder};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Builder, glib, IconSize, Image, Label, ProgressBar};
use gtk::glib::{clone, Receiver, Sender};

#[derive(Debug, Clone)]
enum VolumeType {
    INCR,
    DECR,
    MUTE
}

#[derive(Debug, Clone)]
struct VolumeMessage {
    r#type: VolumeType,
    value: i64
}

fn incr_volume(step: i64) {
    // command: pactl set-sink-volume @DEFAULT_SINK@ +{step}%
    let mut cmd = Command::new("pactl")
        .arg("set-sink-volume")
        .arg("@DEFAULT_SINK@")
        .arg(format!("+{}%", step))
        .spawn()
        .expect("failed to execute process");
    cmd.wait().expect("failed to wait on child");
}

fn decr_volume(step: i64) {
    // command: pactl set-sink-volume @DEFAULT_SINK@ -{step}%
    let mut cmd = Command::new("pactl")
        .arg("set-sink-volume")
        .arg("@DEFAULT_SINK@")
        .arg(format!("-{}%", step))
        .spawn()
        .expect("failed to execute process");
    cmd.wait().expect("failed to wait on child");
}

fn mute_volume() {
    // command: pactl set-sink-mute @DEFAULT_SINK@ toggle
    let mut cmd = Command::new("pactl")
        .arg("set-sink-mute")
        .arg("@DEFAULT_SINK@")
        .arg("toggle")
        .spawn()
        .expect("failed to execute process");
    cmd.wait().expect("failed to wait on child");
}

fn is_muted() -> bool {
    // command: pactl list sinks | grep Mute | awk '{print $2}'
    let output = Command::new("pactl")
        .arg("list")
        .arg("sinks")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut lines = output.lines();
    let mut muted = false;
    while let Some(line) = lines.next() {
        if line.contains("Mute") {
            muted = line.contains("yes");
            break;
        }
    }
    muted
}

fn get_volume() -> i64 {
    // command: pactl list sinks | grep Volume | awk '{print $5}' | sed 's/%//'
    let output = Command::new("pactl")
        .arg("list")
        .arg("sinks")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut lines = output.lines();
    let mut volume = 0;
    while let Some(line) = lines.next() {
        if line.contains("Volume") {
            volume = line.split_whitespace().nth(4).unwrap().replace("%", "").parse().unwrap();
            break;
        }
    }

    volume
}

fn icon_name_from_volume(volume: i64, muted: bool) -> &'static str {
    if muted {
        "audio-volume-muted"
    } else {
        match volume {
            0..=30 => "audio-volume-low",
            31..=50 => "audio-volume-medium",
            51..=100 => "audio-volume-high",
            _ => "audio-volume-muted"
        }
    }
}

fn main() {
    let app = Application::new(Some("me.diced.volumed"), Default::default());

    app.connect_activate(|app| {
        let builder = Builder::new();
        builder.add_from_string(include_str!("ui.glade")).expect("rip");

        let win: ApplicationWindow = builder.object("ui").expect("rip");
        win.set_application(Some(app));
        win.set_keep_above(true);

        let (tx, rx): (Sender<VolumeMessage>, Receiver<VolumeMessage>) =
          glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        std::thread::spawn(move || {
            let c = Connection::new_session().unwrap();
            c.request_name("me.diced.volumed.server", false, true, false)
              .unwrap();
            let mut cr = Crossroads::new();

            let iface_token = cr.register(
                "me.diced.volumed.server",
                |b: &mut IfaceBuilder<Sender<VolumeMessage>>| {
                    b.method(
                        "IncrVol",
                        ("step", ),
                        (),
                        move |_, thread_tx, (step, ): (i64, )| {
                            thread_tx.send(VolumeMessage {
                                value: step,
                                r#type: VolumeType::INCR
                            }).unwrap();
                            Ok(())
                        },
                    );

                    b.method(
                        "DecrVol",
                        ("step", ),
                        (),
                        move |_, thread_tx, (step, ): (i64, )| {
                            thread_tx.send(VolumeMessage {
                                value: step,
                                r#type: VolumeType::DECR
                            }).unwrap();
                            Ok(())
                        },
                    );

                    b.method(
                        "MuteVol",
                        (),
                        (),
                        move |_, thread_tx, (): ()| {
                            thread_tx.send(VolumeMessage {
                                value: 0,
                                r#type: VolumeType::MUTE
                            }).unwrap();
                            Ok(())
                        },
                    );
                },
            );

            cr.insert("/", &[iface_token], tx);
            cr.serve(&c).unwrap();
        });

        let volume_icon: Image = builder.object("volume_icon").expect("rip");
        let volume_text: Label = builder.object("volume_text").expect("rip");
        let volume_bar: ProgressBar = builder.object("volume_bar").expect("rip");

        let queue = Arc::new(Mutex::new(Vec::new()));

        rx.attach(None, clone!(@weak win => @default-return Continue(false), move |msg| {
            match msg.r#type {
                VolumeType::INCR => {
                    incr_volume(msg.value);
                },
                VolumeType::DECR => {
                    decr_volume(msg.value);
                },
                VolumeType::MUTE => {
                    mute_volume();
                }
            }

            let volume = get_volume();
            let muted = is_muted();

            volume_icon.set_from_icon_name(Some(icon_name_from_volume(volume, muted)), IconSize::Button);
            volume_text.set_text(&format!("{}%", volume));
            volume_bar.set_fraction(volume as f64 / 100.0);

            let mut q = queue.lock().unwrap();

            if q.is_empty() {
                win.show_all();
            }

            q.push(msg);
            let queue = queue.clone();

            // this timeout will remove the first message from the queue, then if the queue is empty, it will hide the window
            // this is to make sure the window doesnt hide it self if holding down the volume key, etc.
            glib::timeout_add_seconds_local(1, clone!(@weak win => @default-return Continue(false), move || {
                let mut q = queue.lock().unwrap();
                q.remove(0);
                if q.is_empty() {
                    win.hide();
                }
                Continue(false)
            }));

            Continue(true)
        }));
    });

    app.run();
}
