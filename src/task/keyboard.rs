use crate::serial::info;
use crate::shell::Shell;
use crate::text_editor::express_editor::{self};
use crate::uefi_text_buffer::{backspace, print_char, print_message};
use alloc::string::ToString;
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, KeyState, Keyboard, ScancodeSet1, layouts};
use spin::Mutex;

// Static queue to store keyboard scancodes
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

// Atomic waker to notify the executor when new scancodes are available
static WAKER: AtomicWaker = AtomicWaker::new();

lazy_static! {
    static ref SHELL: Arc<Mutex<Shell>> = Arc::new(Mutex::new(Shell::new()));
}

pub fn init_shell() {
    SHELL.lock().start();
}

/// Called by the keyboard interrupt handler
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            print_message("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        print_message("WARNING: scancode queue uninitialized");
    }
}

// Stream of scancodes from the keyboard
pub struct ScancodeStream {
    _private: (),
}

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());

        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

// Async function to handle keyboard input and print keypresses
pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    // Track modifier key states
    let mut shift_pressed = false;
    let mut ctrl_pressed = false;

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            // Update modifier key states
            match key_event.code {
                KeyCode::LShift | KeyCode::RShift => {
                    shift_pressed = key_event.state == KeyState::Down;
                }
                KeyCode::LControl | KeyCode::RControl => {
                    ctrl_pressed = key_event.state == KeyState::Down;
                }
                _ => {}
            }

            // Only process key down events for most keys
            if key_event.state == KeyState::Down {
                // Handle special key combinations first
                if ctrl_pressed && key_event.code == KeyCode::C {
                    // Check if editor is active
                    let editor_active = express_editor::EDITOR_DATA.lock().active;
                    if editor_active {
                        express_editor::exit_editor();
                        continue;
                    }
                }

                // Handle arrow keys and special keys (regardless of editor state)
                match key_event.code {
                    KeyCode::ArrowLeft => {
                        let editor_data_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_data_active {
                            express_editor::move_cursor_left();
                        }
                        continue;
                    }
                    KeyCode::ArrowRight => {
                        let editor_data_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_data_active {
                            express_editor::move_cursor_right();
                        }
                        continue;
                    }
                    KeyCode::ArrowUp => {
                        let editor_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_active {
                            express_editor::move_cursor_up();
                        }
                        continue;
                    }
                    KeyCode::ArrowDown => {
                        let editor_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_active {
                            express_editor::move_cursor_down();
                        }
                        continue;
                    }
                    KeyCode::Home => {
                        let editor_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_active {
                            express_editor::move_to_line_start();
                        }
                        continue;
                    }
                    KeyCode::End => {
                        let editor_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_active {
                            express_editor::move_to_line_end();
                        }
                        continue;
                    }
                    KeyCode::PageUp => {
                        let editor_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_active {
                            for _ in 0..10 {
                                express_editor::move_cursor_up();
                            }
                        }
                        continue;
                    }
                    KeyCode::PageDown => {
                        let editor_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_active {
                            for _ in 0..10 {
                                express_editor::move_cursor_down();
                            }
                        }
                        continue;
                    }
                    _ => {}
                }
            }

            // Process the key event to get a decoded key
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    // Handle Unicode characters
                    DecodedKey::Unicode(character) => {
                        // Check for Ctrl+C combination first
                        if ctrl_pressed && (character == 'c' || character == 'C') {
                            let editor_active = express_editor::EDITOR_DATA.lock().active;
                            if editor_active {
                                express_editor::exit_editor();
                                continue;
                            }
                        }

                        // Check if editor is active
                        let editor_active = express_editor::EDITOR_DATA.lock().active;
                        if editor_active {
                            express_editor::process_editor_key(character);
                            continue; // Don't send to shell
                        }

                        // Shell mode handling
                        match character {
                            // Handle backspace (0x08) or delete (0x7F)
                            '\u{0008}' | '\u{007F}' => {
                                backspace();
                                SHELL.lock().process_keypress('\u{8}');
                            }
                            // Handle tab (0x09)
                            '\u{0009}' => {
                                print_message("    ");
                            }
                            // Handle escape (0x1B)
                            '\u{001B}' => {
                                // Currently just removes the escape character
                            }
                            // Handle all other printable characters
                            _ => {
                                print_char(character);
                                SHELL.lock().process_keypress(character);
                                info(character.to_string().as_str());
                            }
                        }
                    }

                    // Handle raw key codes
                    DecodedKey::RawKey(key) => {
                        match key {
                            KeyCode::Backspace => {
                                let editor_active = express_editor::EDITOR_DATA.lock().active;
                                if editor_active {
                                    express_editor::process_editor_key('\u{8}');
                                } else {
                                    backspace();
                                    SHELL.lock().process_keypress('\u{8}');
                                }
                            }
                            // Handle delete key
                            KeyCode::Delete => {
                                let editor_active = express_editor::EDITOR_DATA.lock().active;
                                if editor_active {
                                    express_editor::process_editor_key('\u{8}');
                                } else {
                                    backspace();
                                    SHELL.lock().process_keypress('\u{8}');
                                }
                            }
                            // Handle tab key (insert 4 spaces)
                            KeyCode::Tab => {
                                let editor_active = express_editor::EDITOR_DATA.lock().active;
                                if editor_active {
                                    // Insert 4 spaces in editor
                                    for _ in 0..4 {
                                        express_editor::process_editor_key(' ');
                                    }
                                } else {
                                    print_message("    ");
                                }
                            }
                            // Handle Enter/Return key
                            KeyCode::Return => {
                                let editor_active = express_editor::EDITOR_DATA.lock().active;
                                if editor_active {
                                    express_editor::process_editor_key('\n');
                                } else {
                                    print_char('\n');
                                    SHELL.lock().process_keypress('\n');
                                }
                            }
                            // Handle OEM7 key (backslash or pipe with shift)
                            KeyCode::Oem7 => {
                                let editor_active = express_editor::EDITOR_DATA.lock().active;
                                let char_to_insert = if shift_pressed { '|' } else { '\\' };

                                if editor_active {
                                    express_editor::process_editor_key(char_to_insert);
                                } else {
                                    print_char(char_to_insert);
                                    SHELL.lock().process_keypress(char_to_insert);
                                }
                            }

                            // Modifier keys (no visible output)
                            KeyCode::LShift
                            | KeyCode::RShift
                            | KeyCode::LControl
                            | KeyCode::RControl
                            | KeyCode::LAlt
                            | KeyCode::RAltGr
                            | KeyCode::CapsLock
                            | KeyCode::Escape => {
                                // These are handled elsewhere or ignored
                            }

                            // Arrow keys are handled above in the key_event.state == KeyState::Down block
                            KeyCode::ArrowUp
                            | KeyCode::ArrowDown
                            | KeyCode::ArrowLeft
                            | KeyCode::ArrowRight
                            | KeyCode::Home
                            | KeyCode::End
                            | KeyCode::PageUp
                            | KeyCode::PageDown => {
                                // Already handled above
                            }

                            _ => {
                                // Unknown key, ignore
                            }
                        }
                    }
                }
            }
        }
    }
}