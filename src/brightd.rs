use std::process::Command;
use std::sync::{Arc, Mutex};
use dbus::blocking::Connection;
use dbus_crossroads::{Crossroads, IfaceBuilder};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Builder, glib, IconSize, Image, Label, ProgressBar};
use gtk::glib::{clone, Receiver, Sender};

#[derive(Debug, Clone)]
enum BrightType {
    INCR,
    DECR,
}

#[derive(Debug, Clone)]
struct BrightMessage {
    r#type: BrightType,
    value: i64
}

fn curr_brightness() -> i64 {
    // brightnessctl l -m

    let output = Command::new("brightnessctl")
        .arg("l")
        .arg("-m")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let output = output.split(",").collect::<Vec<&str>>();
    let output = output[3].replace("%", "");

    output.parse::<i64>().unwrap()
}

fn incr_brightness(step: i64) {
    // brightnessctl set {step}%+

    let mut cmd = Command::new("brightnessctl")
        .arg("set")
        .arg(format!("{}%+", step))
        .spawn()
        .expect("failed to execute process");
    cmd.wait().expect("failed to wait on child");
}

fn decr_brightness(step: i64) {
    // brightnessctl set {step}%-

    let mut cmd = Command::new("brightnessctl")
        .arg("set")
        .arg(format!("{}%-", step))
        .spawn()
        .expect("failed to execute process");
    cmd.wait().expect("failed to wait on child");
}

fn icon_name_from_brightness(brightness: i64) -> &'static str {
    match brightness {
        0..=50 => "display-brightness-low-symbolic",
        51..=100 => "display-brightness-high-symbolic",
        _ => "display-brightness-low-symbolic"
    }
}

fn main() {
    let app = Application::new(Some("me.diced.brightd"), Default::default());

    app.connect_activate(|app| {
        let builder = Builder::new();
        builder.add_from_string(include_str!("ui.glade")).expect("rip");

        let win: ApplicationWindow = builder.object("ui").expect("rip");
        win.set_application(Some(app));
        win.set_keep_above(true);

        let (tx, rx): (Sender<BrightMessage>, Receiver<BrightMessage>) =
          glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        std::thread::spawn(move || {
            let c = Connection::new_session().unwrap();
            c.request_name("me.diced.brightd.server", false, true, false)
              .unwrap();
            let mut cr = Crossroads::new();

            let iface_token = cr.register(
                "me.diced.brightd.server",
                |b: &mut IfaceBuilder<Sender<BrightMessage>>| {
                    b.method(
                        "IncrLight",
                        ("step", ),
                        (),
                        move |_, thread_tx, (step, ): (i64, )| {
                            thread_tx.send(BrightMessage {
                                value: step,
                                r#type: BrightType::INCR,
                            }).unwrap();
                            Ok(())
                        },
                    );

                    b.method(
                        "DecrLight",
                        ("step", ),
                        (),
                        move |_, thread_tx, (step, ): (i64, )| {
                            thread_tx.send(BrightMessage {
                                value: step,
                                r#type: BrightType::DECR,
                            }).unwrap();
                            Ok(())
                        },
                    );
                },
            );

            cr.insert("/", &[iface_token], tx);
            cr.serve(&c).unwrap();
        });

        let brightness_icon: Image = builder.object("volume_icon").expect("rip");
        let brightness_text: Label = builder.object("volume_text").expect("rip");
        let brightness_bar: ProgressBar = builder.object("volume_bar").expect("rip");

        let queue = Arc::new(Mutex::new(Vec::new()));

        rx.attach(None, clone!(@weak win => @default-return Continue(false), move |msg| {
            match msg.r#type {
                BrightType::INCR => {
                    incr_brightness(msg.value);
                },
                BrightType::DECR => {
                    decr_brightness(msg.value);
                },
            }

            let brightness = curr_brightness();

            brightness_icon.set_from_icon_name(Some(icon_name_from_brightness(brightness)), IconSize::Button);
            brightness_text.set_text(&format!("{}%", brightness));
            brightness_bar.set_fraction(brightness as f64 / 100.0);

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
