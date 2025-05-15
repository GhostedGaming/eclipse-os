use crate::shell::Shell;
use crate::text_editor::express_editor::{self};
use crate::vga_buffer::WRITER;
use crate::{print, println, vga_buffer};
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
// OnceCell ensures it's initialized only once
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
///
/// Must not block or allocate.

pub fn cursror() {
    vga_buffer::set_cursor_visibility(true);
    vga_buffer::set_cursor_style(vga_buffer::CursorStyle::Underline);
}

pub(crate) fn add_scancode(scancode: u8) {
    // Try to get the scancode queue
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        // Try to push the scancode to the queue
        if let Err(_) = queue.push(scancode) {
            // If the queue is full, print a warning
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            // Wake up the task waiting for keyboard input
            WAKER.wake();
        }
    } else {
        // If the queue is not initialized, print a warning
        println!("WARNING: scancode queue uninitialized");
    }
}

// Stream of scancodes from the keyboard
pub struct ScancodeStream {
    // Private field to prevent direct construction
    _private: (),
}

impl ScancodeStream {
    // Create a new scancode stream
    pub fn new() -> Self {
        // Initialize the scancode queue with a capacity of 100 scancodes
        // This will panic if called more than once
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

// Implement the Stream trait for ScancodeStream
impl Stream for ScancodeStream {
    type Item = u8;

    // Poll for the next scancode
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        // Get the scancode queue
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // Fast path: check if there's a scancode available immediately
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        // Register the waker to be notified when a new scancode arrives
        WAKER.register(&cx.waker());

        // Check again in case a scancode arrived after we checked but before we registered the waker
        match queue.pop() {
            Some(scancode) => {
                // If we got a scancode, unregister the waker and return the scancode
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => {
                // If there's still no scancode, return Pending to indicate we need to be polled again
                Poll::Pending
            }
        }
    }
}

// Async function to handle keyboard input and print keypresses
pub async fn print_keypresses() {
    // Create a new scancode stream
    let mut scancodes = ScancodeStream::new();

    // Create a new keyboard with US layout and ignore control characters
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    // Track shift key state
    let mut shift_pressed = false;
    let mut ctrl_pressed = false;

    // Process scancodes as they arrive
    while let Some(scancode) = scancodes.next().await {
        // Add the scancode to the keyboard and get a key event if available
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            // Update modifier key states based on key event
            match key_event.code {
                KeyCode::LShift | KeyCode::RShift => {
                    shift_pressed = key_event.state == KeyState::Down;
                }
                KeyCode::LControl | KeyCode::RControl => {
                    ctrl_pressed = key_event.state == KeyState::Down;
                }
                _ => {}
            }

            // Process the key event to get a decoded key
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    // Handle Unicode characters
                    DecodedKey::Unicode(character) => {
                        // Check for Ctrl+C combination
                        if ctrl_pressed && (character == 'c' || character == 'C') {
                            express_editor::exit_editor();
                            continue;
                        }

                        if express_editor::EDITOR_DATA.lock().active {
                            express_editor::process_editor_key(character);
                            continue; // Don't send to shell
                        }

                        match character {
                            // Handle backspace (0x08) or delete (0x7F)
                            '\u{0008}' | '\u{007F}' => {
                                // Pass to shell for processing
                                SHELL.lock().process_keypress('\u{8}');
                            }
                            // Handle tab (0x09)
                            '\u{0009}' => {
                                print!("    ");
                            }
                            // Handle escape (0x1B)
                            '\u{001B}' => {
                                // Currently just removes the escape character
                            }
                            // Handle all other printable characters
                            _ => {
                                let column_pos = WRITER.lock().column_position();

                                if column_pos > 78 {
                                    println!("\n");
                                } else {
                                    // Pass to shell for processing
                                    SHELL.lock().process_keypress(character);
                                }
                            }
                        }
                    }
                    // Handle raw key codes
                    DecodedKey::RawKey(key) => {
                        let editor_data = express_editor::EDITOR_DATA.lock();
                        if ctrl_pressed && key == KeyCode::C && editor_data.active {
                            express_editor::exit_editor();
                            continue;
                        }

                        match key {
                            // Handle backspace key
                            KeyCode::Backspace => {
                                SHELL.lock().process_keypress('\u{8}');
                            }
                            // Handle delete key
                            KeyCode::Delete => {
                                SHELL.lock().process_keypress('\u{8}');
                            }
                            // Handle tab key (insert 4 spaces)
                            KeyCode::Tab => {
                                print!("    ");
                            }
                            // Handle OEM7 key (backslash or pipe with shift)
                            KeyCode::Oem7 => {
                                if shift_pressed {
                                    SHELL.lock().process_keypress('|');
                                } else {
                                    SHELL.lock().process_keypress('\\');
                                }
                            }

                            // Modifier keys (no visible output)
                            KeyCode::LShift
                            | KeyCode::RShift
                            | KeyCode::LControl
                            | KeyCode::RControl
                            | KeyCode::LAlt
                            | KeyCode::RAltGr => {}

                            // Navigation keys (no visible output currently)
                            KeyCode::ArrowUp => {
                                
                            }
                            KeyCode::ArrowDown => {

                            }
                            KeyCode::ArrowLeft => {}
                            KeyCode::ArrowRight => {}
                            KeyCode::Escape => {}
                            KeyCode::Home => {}
                            KeyCode::PageUp => {}
                            KeyCode::PageDown => {}
                            KeyCode::CapsLock => {}

                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
