use evdev::AttributeSet;
use evdev::SwitchType;
use evdev::{uinput::VirtualDeviceBuilder, EventType, InputEvent};
use std::fs;
use std::io::ErrorKind;
use std::io::Read;
use std::os::unix::net::UnixListener;

fn main() -> std::io::Result<()> {
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

    let file_result = fs::remove_file("/tmp/tabmodesw.sock");
    if let Err(err) = file_result {
        if err.kind() != ErrorKind::NotFound {
            return Err(err);
        }
    }
    let listener = UnixListener::bind("/tmp/tabmodesw.sock")?;
    for conn in listener.incoming() {
        match conn {
            Ok(mut stream) => {
                let mut b = [0; 1];
                stream.read(&mut b)?;
                println!("Result {:?}", b);
                if b[0] == 50u8 {
                    break;
                }
                if b[0] == 48u8 {
                    //"0"
                    //tablet mode disable
                    println!("Disable tab mode");
                    device.emit(&[InputEvent::new(
                        EventType::SWITCH,
                        SwitchType::SW_TABLET_MODE.0,
                        0,
                    )])?;
                } else {
                    //tablet mode enable
                    println!("Enable tab mode");
                    device.emit(&[InputEvent::new(
                        EventType::SWITCH,
                        SwitchType::SW_TABLET_MODE.0,
                        1,
                    )])?;
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}
