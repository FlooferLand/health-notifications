#![windows_subsystem = "windows"]

mod notifications;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{TrayIcon, TrayIconBuilder, menu::Menu, Icon};
use clokwerk::{Scheduler, TimeUnits};
use tray_icon::menu::{CheckMenuItem, CheckMenuItemBuilder, MenuEvent};

fn main() {
	let (pause_button, tray_icon) = spawn_tray_icon();

    let event_loop = EventLoopBuilder::new().build();
    let menu_channel = MenuEvent::receiver();

    // Running the scheduler
    let (scheduler_send, scheduler_recv) = mpsc::channel();
    thread::spawn(move || {	    
        let mut scheduler = spawn_scheduler();
        let mut paused = false;
	    let mut user_busy = false;
        loop {
            paused = scheduler_recv.try_recv().unwrap_or(paused);
	        user_busy = check_fullscreen();
	        
            if !paused && !user_busy {
                scheduler.run_pending();
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Main event loop
    let mut last_paused = false;
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Getting the pause state
        let paused = pause_button.is_checked();

        // Updating state
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == pause_button.id() {
                let _ = scheduler_send.send(paused);
            }
        }

        // Updating the icon
        if paused != last_paused {
            tray_icon.set_icon(Some(get_icon_image(paused))).unwrap();
            last_paused = paused;
        }
    });
}

fn spawn_scheduler() -> Scheduler {
    let mut scheduler = Scheduler::new();

    // 20/20/20 reminder
    scheduler.every(20.minutes())
        .run(||
            notifications::send(
                "Look away and blink for 30 seconds!",
                "Take care of cho eyes!!"
            )
        );

	// Returning the scheduler
    scheduler
}

fn spawn_tray_icon() -> (CheckMenuItem, TrayIcon) {
    let pause_button = CheckMenuItemBuilder::new()
        .text("Pause")
        .enabled(true)
        .checked(false)
        .build();

	let menu = Menu::new();
    menu.append(&pause_button).expect("Could not spawn menu item!");

    let tray_icon = TrayIconBuilder::new()
		.with_menu(Box::new(menu))
		.with_icon(get_icon_image(false))
		.with_tooltip("Health Notifications :3")
		.with_title("Health Notifications :3")
		.build().unwrap();

    (pause_button, tray_icon)
}

fn get_icon_image(paused: bool) -> Icon {
	const WIDTH:  u32 = 16;
	const HEIGHT: u32 = 16;
	const TOTAL_SIZE: usize = ((WIDTH * HEIGHT) * 4) as usize;

	let mut rgba = Vec::with_capacity(TOTAL_SIZE);
	for _ in 0..TOTAL_SIZE / 4 {
        let mut col = match paused {
            true  => vec![255, 0, 0, 100],
            false => vec![0, 255, 0, 100],
        };
        rgba.append(&mut col);
	}

	Icon::from_rgba(rgba, WIDTH, HEIGHT).unwrap()
}

// TODO: Implement Unix fullscreen checks
#[cfg(not(windows))]
fn check_fullscreen() -> bool {
	false
}

#[cfg(windows)]
fn check_fullscreen() -> bool {
	use windows::Win32::Foundation::RECT;
	use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY};
	use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, GetForegroundWindow, GetShellWindow, GetWindowRect};
	unsafe {
		// Ignore window if it's the desktop or the taskbar or smth
		let hwnd = GetForegroundWindow();
		if hwnd.is_invalid() || hwnd == GetDesktopWindow() || hwnd == GetShellWindow() {
			return false;
		}
		
		// Get the window size
		let mut win_rect = RECT::default();
		if GetWindowRect(hwnd, &mut win_rect).is_err() {
			println!("Failed to get window size");
			return false;
		}
		
		// Get the monitor info for the window
		let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
		let mut minfo = MONITORINFO {
			cbSize: size_of::<MONITORINFO>() as u32,
			..Default::default()
		};
		if !GetMonitorInfoW(hmonitor, &mut minfo).as_bool() {
			println!("Failed to get monitor info for monitor");
			return false;
		}
		
		win_rect.left == minfo.rcMonitor.left &&
		win_rect.top == minfo.rcMonitor.top &&
		win_rect.right == minfo.rcMonitor.right &&
		win_rect.bottom == minfo.rcMonitor.bottom
	}
}
