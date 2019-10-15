use super::printer::{send, PSender, Printer};
use crate::{command::RunelCommand::Set, error::PrinterError, widget::RunelWidget};
use std::thread;
use xcb::{self, xkb};
use xkbcommon::xkb::{self as xxkb, x11, Context};

pub struct Keyboard;

impl Keyboard {
    pub fn new() -> Self {
        Self
    }

    fn main_loop(tx: &PSender) -> Result<(), PrinterError> {
        if let Ok((conn, _)) = xcb::Connection::connect(None) {
            {
                let cookie = xkb::use_extension(&conn, 1, 0);
                match cookie.get_reply() {
                    Ok(r) => {
                        if !r.supported() {
                            return Err(PrinterError::KbdError("xkb: not supported"));
                        }
                    }
                    Err(_) => return Err(PrinterError::KbdError("xkb: no reply")),
                }
            }
            {
                let map_parts = xcb::xkb::MAP_PART_MODIFIER_MAP;
                let events = xcb::xkb::EVENT_TYPE_STATE_NOTIFY;
                let cookie = xkb::select_events_checked(
                    &conn,
                    xkb::ID_USE_CORE_KBD as u16,
                    events as u16,
                    0,
                    events as u16,
                    map_parts as u16,
                    map_parts as u16,
                    None,
                );
                let _ = cookie.request_check();
            }
            let mut last_id = 0;
            if !x11::setup_xkb_extension(
                &conn,
                x11::MIN_MAJOR_XKB_VERSION,
                x11::MIN_MINOR_XKB_VERSION,
                x11::SetupXkbExtensionFlags::NoFlags,
                &mut 0,
                &mut 0,
                &mut 0,
                &mut 0,
            ) {
                return Err(PrinterError::KbdError("xkb: setup extension"));
            }
            let ctx = Context::new(xxkb::CONTEXT_NO_FLAGS);
            let dev_id = x11::get_core_keyboard_device_id(&conn);
            if dev_id == -1 {
                return Err(PrinterError::KbdError("xkb: get core kbd device"));
            }
            let keymap =
                x11::keymap_new_from_device(&ctx, &conn, dev_id, xxkb::KEYMAP_COMPILE_NO_FLAGS);
            if keymap.get_raw_ptr().is_null() {
                return Err(PrinterError::KbdError("xkb: keymap from device"));
            }

            send(
                tx,
                Ok(Set {
                    widget: RunelWidget::Keyboard,
                    value: keymap.layout_get_name(u32::from(last_id)).into(),
                }),
            );
            loop {
                let event = conn.wait_for_event();
                match event {
                    None => {
                        break;
                    }
                    Some(event) => {
                        let evt: &xkb::StateNotifyEvent = unsafe { xcb::cast_event(&event) };
                        let new_id = evt.group();
                        if last_id != new_id {
                            last_id = new_id;
                            send(
                                tx,
                                Ok(Set {
                                    widget: RunelWidget::Keyboard,
                                    value: keymap.layout_get_name(u32::from(last_id)).into(),
                                }),
                            );
                        }
                    }
                }
            }
        }
        Err(PrinterError::Unreachable("xkb".to_string()))
    }
}

impl Printer for Keyboard {
    fn spawn(&self, tx: PSender) {
        thread::Builder::new()
            .name("keyboard".into())
            .spawn(move || loop {
                if let Err(e) = Self::main_loop(&tx) {
                    send(&tx, Err(e));
                }
            })
            .unwrap();
    }
}
