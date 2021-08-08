use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
use evdev::uinput::VirtualDevice;
use evdev::AttributeSet;
use evdev::SwitchType;
use evdev::{uinput::VirtualDeviceBuilder, EventType, InputEvent};
use std::process;

struct State {
    device: VirtualDevice,
    state: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut switches = AttributeSet::<SwitchType>::new();
    switches.insert(SwitchType::SW_TABLET_MODE);

    let mut device = VirtualDeviceBuilder::new()?
        .name("Tablet mode switch")
        .with_switches(&switches)?
        .build()
        .unwrap();

    device.emit(&[InputEvent::new(
        EventType::SWITCH,
        SwitchType::SW_TABLET_MODE.0,
        0,
    )])?;

    // Use dbus
    let c = Connection::new_session()?;
    c.request_name("de.devpi.tabmodesw", false, true, false)?;
    let mut cr = Crossroads::new();
    let token = cr.register("de.devpi.tabmodesw", |b| {
        b.method("Enable", (), (), |_, state: &mut State, _: ()| {
            println!("Enable tab mode");
            state.state = true;
            state.device.emit(&[InputEvent::new(
                EventType::SWITCH,
                SwitchType::SW_TABLET_MODE.0,
                1,
            )]);
            Ok(())
        });
        b.method("Disable", (), (), |_, state: &mut State, _: ()| {
            println!("Disable tab mode");
            state.state = false;
            state.device.emit(&[InputEvent::new(
                EventType::SWITCH,
                SwitchType::SW_TABLET_MODE.0,
                0,
            )]);
            Ok(())
        });
        b.method("State", (), ("state",), |_, state: &mut State, _: ()| {
            Ok((state.state,))
        });
        b.method("Pid", (), ("pid",), |_, _: &mut State, _: ()| {
            Ok((process::id(),))
        });
    });

    cr.insert(
        "/",
        &[token],
        State {
            device,
            state: false,
        },
    );
    cr.serve(&c)?;
    unreachable!()
}
