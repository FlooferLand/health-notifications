use std::time::Duration;
use notify_rust::{Notification, Timeout};

pub fn send(title: &str, description: &str) {
	Notification::new()
		.summary(title)
		.body(description)
		.sound_name("Mail")
		.appname("health-notifications")
		.auto_icon()
		.timeout(Timeout::from(Duration::from_secs(10)))
		.show().unwrap();
}
