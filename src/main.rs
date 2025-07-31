use std::{thread, time::Duration};
use system_tray::{Event, Menu, SystemTray};

fn main() {
    // Create a system tray instance
    let mut tray = SystemTray::new("TestApp", "com.example.testapp");

    // Add menu items
    tray = tray
        .menu(Menu::new("Open".to_string(), "open".to_string()))
        .menu(Menu::new("Exit".to_string(), "exit".to_string()));

    // Set a placeholder icon (minimal PNG for testing)
    static ICON: &[u8] = include_bytes!("../icon.svg");
    tray = tray.icon(ICON, "SVG");

    // Start the system tray event loop
    tray.start();

    // Poll for events in the main thread
    loop {
        match tray.poll_event() {
            Ok(event) => match event {
                Event::None => {}
                Event::TrayClicked => println!("Tray icon clicked"),
                Event::TrayDoubleClicked => println!("Tray icon double-clicked"),
                Event::MenuItemClicked(id) => {
                    println!("Menu item clicked: {}", id);
                    if id == "exit" {
                        tray.stop();
                        break;
                    } else if id == "open" {
                        println!("Open menu item selected");
                    }
                }
            },
            Err(e) => {
                eprintln!("Error polling event: {}", e);
                tray.stop();
                break;
            }
        }
        // Prevent busy-waiting
        thread::sleep(Duration::from_millis(100));
    }
}
