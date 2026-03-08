use lifecycle::{self, runnable::Runnable};
use tokio::sync::{broadcast, oneshot};

use std::{pin::Pin, time::Duration};

struct Runs {
	name: &'static str,
	timeout: Duration,
	complete_delay: Duration,
}

impl Default for Runs {
	fn default() -> Self {
		Self {
			name: "test",
			timeout: Duration::from_millis(10),
			complete_delay: Duration::from_millis(5),
		}
	}
}

impl Runnable for Runs {
	fn name(&self) -> &'static str {
		self.name
	}

	fn timeout(&self) -> Duration {
		self.timeout
	}

	fn run(
		self: Box<Self>,
		mut cancel_rx: broadcast::Receiver<()>,
	) -> Pin<Box<dyn Future<Output = ()> + Send>> {
		Box::pin(async move {
			let _ = cancel_rx.recv();
			tokio::time::sleep(self.complete_delay).await;
		})
	}
}

#[test]
fn test_build() {
	let r = Runs::default();

	let _l = lifecycle::Lifecycle::builder()
		.with_shutdown_timeout(Duration::from_millis(100))
		.with_runnable(Box::new(r))
		.build();
}

#[tokio::test]
async fn test_start_stop() {
	let r = Runs::default();

	let l = lifecycle::Lifecycle::builder()
		.with_runnable(Box::new(r))
		.build()
		.unwrap();

	let (cancel_tx, cancel_rx) = oneshot::channel();
	let (done_tx, done_rx) = oneshot::channel();

	tokio::spawn(async move {
		assert!(l.run(cancel_rx).await.is_ok(), "run unexpectedly errored");
		let _ = done_tx.send(());
	});

	let _ = cancel_tx.send(());

	assert!(
		tokio::time::timeout(Duration::from_millis(10), done_rx)
			.await
			.is_ok(),
		"stop did not complete within timeout"
	);
}

#[tokio::test]
async fn test_build_multiple_names() {
	let r0 = Runs {
		name: "test0",
		..Default::default()
	};
	let r1 = Runs {
		name: "test1",
		..Default::default()
	};

	assert!(
		lifecycle::Lifecycle::builder()
			.with_runnable(Box::new(r0))
			.with_runnable(Box::new(r1))
			.build()
			.is_ok()
	);
}

#[tokio::test]
async fn test_build_duplicate_names() {
	assert!(
		lifecycle::Lifecycle::builder()
			.with_runnable(Box::new(Runs::default()))
			.with_runnable(Box::new(Runs::default()))
			.build()
			.is_err()
	);
}

#[tokio::test]
async fn test_build_excessive_timeout() {
	let r = Runs {
		complete_delay: Duration::from_millis(20),
		..Default::default()
	};

	let l = lifecycle::Lifecycle::builder()
		.with_runnable(Box::new(r))
		.build()
		.unwrap();

	let (cancel_tx, cancel_rx) = oneshot::channel();
	let (done_tx, done_rx) = oneshot::channel();

	tokio::spawn(async move {
		assert!(l.run(cancel_rx).await.is_err(), "run unexpectedly ok");

		let _ = done_tx.send(());
	});

	let _ = cancel_tx.send(());

	let _ = tokio::time::timeout(Duration::from_millis(10), done_rx)
		.await
		.is_ok();
}
