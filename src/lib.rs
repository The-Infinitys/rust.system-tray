//! This crate provides a cross-platform system tray icon functionality using Qt.
//! It allows you to create a system tray icon, add menu items to it, set its icon,
//! and handle events such as clicks and menu item selections.

mod bind;
mod error;

pub use error::SystemTrayError as Error;
use std::{
    ffi::{c_char, CString},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

/// A transparent wrapper around a raw `bind::QtAppHandle` pointer.
///
/// This struct is `Send` safe, allowing the `QtAppHandle` to be moved between threads.
#[repr(transparent)]
#[derive(Clone, Copy)]
struct SafeQtAppHandle(*mut bind::QtAppHandle);

unsafe impl Send for SafeQtAppHandle {}

impl SafeQtAppHandle {
    /// Creates a new `SafeQtAppHandle` from a raw `bind::QtAppHandle` pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided `ptr` is a valid pointer to a `QtAppHandle`
    /// and that its lifetime is managed correctly.
    pub unsafe fn new(ptr: *mut bind::QtAppHandle) -> Self {
        Self(ptr)
    }

    /// Returns the raw `bind::QtAppHandle` pointer.
    pub fn as_ptr(&self) -> *mut bind::QtAppHandle {
        self.0
    }
}

/// Represents the various events that can be received from the system tray.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Event {
    /// No event occurred.
    None,
    /// The system tray icon was clicked.
    TrayClicked,
    /// The system tray icon was double-clicked.
    TrayDoubleClicked,
    /// A menu item in the system tray was clicked, identified by its ID.
    MenuItemClicked(String),
}

/// Represents the system tray icon and its associated application.
///
/// This struct manages the underlying Qt application instance and its lifecycle.
#[derive(Clone)]
pub struct SystemTray {
    handle: Arc<Mutex<SafeQtAppHandle>>,
    instance: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Represents a menu item that can be added to the system tray context menu.
pub struct Menu {
    text: String,
    id: String,
}

impl Menu {
    /// Creates a new `Menu` item with the given `text` and unique `id`.
    ///
    /// The `id` is used to identify which menu item was clicked when an `Event::MenuItemClicked`
    /// is received.
    pub fn new(text: String, id: String) -> Self {
        Self { text, id }
    }
}

impl SystemTray {
    /// Creates a new `SystemTray` instance.
    ///
    /// This initializes the underlying Qt application.
    ///
    /// # Arguments
    ///
    /// * `organization` - The organization name for the application.
    /// * `app_id` - A unique application identifier.
    ///
    /// # Panics
    ///
    /// This method panics if the `organization` or `app_id` strings contain null bytes.
    pub fn new(organization: &str, app_id: &str) -> Self {
        let c_org = CString::new(organization).map_err(Error::Ffi).unwrap();
        let c_id = CString::new(app_id).map_err(Error::Ffi).unwrap();
        let handle = unsafe { bind::create_qt_app() };
        let safe_handle = unsafe { SafeQtAppHandle::new(handle) };
        unsafe {
            bind::set_organization_name(safe_handle.as_ptr(), c_org.as_ptr());
            bind::set_app_id(safe_handle.as_ptr(), c_id.as_ptr());
            bind::init_tray(safe_handle.as_ptr());
        }
        Self {
            handle: Arc::new(Mutex::new(safe_handle)),
            instance: Arc::new(Mutex::new(None)),
        }
    }

    /// Adds a menu item to the system tray's context menu.
    ///
    /// This method consumes `self` and returns a new `SystemTray` instance, allowing for
    /// method chaining.
    ///
    /// # Arguments
    ///
    /// * `menu` - The `Menu` item to add.
    ///
    /// # Panics
    ///
    /// This method panics if the `menu.text` or `menu.id` strings contain null bytes.
    pub fn menu(self, menu: Menu) -> Self {
        let c_text = CString::new(menu.text).map_err(Error::Ffi).unwrap();
        let c_id = CString::new(menu.id).map_err(Error::Ffi).unwrap();
        unsafe {
            bind::add_tray_menu_item(
                self.handle.lock().unwrap().as_ptr(),
                c_text.as_ptr(),
                c_id.as_ptr(),
            );
        }
        self
    }

    /// Sets the icon for the system tray.
    ///
    /// This method consumes `self` and returns a new `SystemTray` instance, allowing for
    /// method chaining.
    ///
    /// # Arguments
    ///
    /// * `icon_data` - A static slice of bytes representing the icon data.
    /// * `icon_format` - The format of the icon data (e.g., "png", "ico").
    ///
    /// # Panics
    ///
    /// This method panics if the `icon_format` string contains null bytes.
    pub fn icon(self, icon_data: &'static [u8], icon_format: &str) -> Self {
        let c_format = CString::new(icon_format).map_err(Error::Ffi).unwrap();
        unsafe {
            bind::set_app_icon_from_data(
                self.handle.lock().unwrap().as_ptr(),
                icon_data.as_ptr(),
                icon_data.len(),
                c_format.as_ptr(),
            );
        }
        self
    }

    /// Starts the Qt event loop in a new thread.
    ///
    /// This is a non-blocking operation. Events can be polled using `poll_event`.
    pub fn start(&self) {
        let handle = {
            let handle_guard = self.handle.lock().unwrap();
            *handle_guard
        };
        let join_handle = std::thread::spawn(move || {
            let mut argv: Vec<*mut c_char> = Vec::new(); // Currently unused in the bind, but required by Qt signature
            let result = unsafe { bind::run_qt_app(handle.as_ptr(), 0, argv.as_mut_ptr()) };
            if result != 0 {
                eprintln!("Qt application exited with code: {}", result);
            }
        });
        *self.instance.lock().unwrap() = Some(join_handle);
    }

    /// Requests the Qt application to quit and waits for the Qt event loop thread to finish.
    ///
    /// This method is blocking until the Qt thread has terminated.
    pub fn stop(&self) {
        {
            let handle = self.handle.lock().unwrap();
            unsafe {
                bind::request_quit_qt_app_safe(handle.as_ptr());
            }
        }
        if let Some(join_handle) = self.instance.lock().unwrap().take() {
            join_handle.join().unwrap_or_else(|e| {
                eprintln!("Failed to join Qt thread: {:?}", e);
            });
        }
    }

    /// Polls for a new event from the system tray.
    ///
    /// This method is non-blocking and returns an `Event` immediately.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Event` or a `SystemTrayError` if an unknown event type is received.
    pub fn poll_event(&self) -> Result<Event, Error> {
        let handle = self.handle.lock().unwrap();
        let event = unsafe { bind::poll_event(handle.as_ptr()) };

        match event.type_ {
            bind::AppEventType_None => Ok(Event::None),
            bind::AppEventType_TrayClicked => Ok(Event::TrayClicked),
            bind::AppEventType_TrayDoubleClicked => Ok(Event::TrayDoubleClicked),
            bind::AppEventType_MenuItemClicked => {
                // IMPORTANT: CString::from_raw takes ownership of the pointer.
                // It will call free() when `c_str` is dropped.
                // Therefore, we MUST NOT call bind::free_char_ptr here.
                let c_str = unsafe { CString::from_raw(event.menu_id_str as *mut c_char) };
                let rust_str = c_str.to_string_lossy().into_owned();
                Ok(Event::MenuItemClicked(rust_str))
            }
            _ => Err(Error::PollEventError(format!(
                "Unknown event type value: {}",
                event.type_
            ))),
        }
    }
}

impl Default for SystemTray {
    /// Creates a default `SystemTray` instance with "MyOrganization" and "MyApp" as identifiers.
    fn default() -> Self {
        Self::new("MyOrganization", "MyApp")
    }
}

impl Drop for SystemTray {
    /// Cleans up the Qt application resources when the `SystemTray` instance is dropped.
    ///
    /// This ensures that the Qt application is properly shut down and memory is freed.
    fn drop(&mut self) {
        self.stop();
        let handle = self.handle.lock().unwrap();
        if !handle.as_ptr().is_null() {
            unsafe {
                bind::cleanup_qt_app(handle.as_ptr());
            }
        }
    }
}