#pragma once

#include <stddef.h> // For size_t

#ifdef __cplusplus
extern "C" {
#endif

// Opaque pointer to the C++ implementation
typedef struct QtAppHandle QtAppHandle;

// Enum for event types that can be polled from Rust
// Changed back to C-compatible enum for easier FFI binding (e.g., with bindgen)
typedef enum {
    None,
    TrayClicked,
    TrayDoubleClicked,
    MenuItemClicked
} AppEventType;

// Struct to hold event data
typedef struct {
    AppEventType type_; // Renamed from 'type' to 'type_' to avoid C++ keyword collision
    const char* menu_id_str; // For MenuItemClicked events, now a string
} AppEvent;

/**
 * @brief Creates a new Qt application handle.
 */
QtAppHandle* create_qt_app();

/**
 * @brief Sets the application ID.
 * @param handle The application handle.
 * @param id The application ID string.
 */
void set_app_id(QtAppHandle* handle, const char* id);

/**
 * @brief Sets the organization name for QSettings.
 * This helps prevent "QSettings::value: Empty key passed" warnings.
 * @param handle The application handle.
 * @param name The organization name string.
 */
void set_organization_name(QtAppHandle* handle, const char* name); // Added this function

/**
 * @brief Sets the application icon from raw binary data.
 *
 * @param handle The application handle.
 * @param data Pointer to the raw icon data.
 * @param size The size of the data in bytes.
 * @param format The format of the icon data (e.g., "PNG", "JPG", "SVG").
 */
void set_app_icon_from_data(QtAppHandle* handle, const unsigned char* data, size_t size, const char* format);

/**
 * @brief Initializes the system tray icon with a menu.
 * @param handle The application handle.
 */
void init_tray(QtAppHandle* handle);

/**
 * @brief Runs the Qt application event loop.
 * This is a blocking call that starts the Qt event loop.
 * It should be called from the thread intended to be the Qt GUI thread.
 * @param handle The application handle.
 * @param argc The number of command-line arguments.
 * @param argv An array of command-line argument strings.
 */
int run_qt_app(QtAppHandle* handle, int argc, char* argv[]);

/**
 * @brief Polls for the next event from the Qt application.
 * @param handle The application handle.
 * @return An AppEvent struct containing the event type and any associated data.
 */
AppEvent poll_event(QtAppHandle* handle);

/**
 * @brief Requests the Qt application event loop to quit safely from any thread.
 * This function is thread-safe and will post a quit event to the Qt main thread,
 * resolving "Timers cannot be stopped from another thread" warnings.
 * @param handle The application handle.
 */
void request_quit_qt_app_safe(QtAppHandle* handle); // Added this function, replaces 'quit_qt_app'

/**
 * @brief Cleans up all resources associated with the handle.
 * @param handle The application handle.
 */
void cleanup_qt_app(QtAppHandle* handle);

/**
 * @brief Adds a menu item to the system tray icon's context menu.
 *
 * @param handle The application handle.
 * @param text The text to display for the menu item.
 * @param id A unique string ID for the menu item, used to identify clicks.
 */
void add_tray_menu_item(QtAppHandle* handle, const char* text, const char* id);

/**
 * @brief Frees a character pointer allocated by the C++ side.
 * This must be called by the Rust side after consuming a string like menu_id_str
 * to prevent memory leaks.
 * @param ptr The character pointer to free.
 */
void free_char_ptr(const char* ptr);

#ifdef __cplusplus
} // extern "C"
#endif