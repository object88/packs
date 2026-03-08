use tokio::sync::broadcast::Receiver;

use std::{
	future::Future,
	hash::{Hash, Hasher},
	pin::Pin,
	time::Duration,
};

pub enum State {
	Initial,
	Starting,
	Running,
	Stopping,
	Complete,
}

pub enum Lifetime {
	Continuous,
	Oneshot,
}

pub trait Runnable: Send {
	// Identifier; must be unique
	fn name(&self) -> &'static str;

	fn timeout(&self) -> Duration;

	fn run(self: Box<Self>, cancel_rx: Receiver<()>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl Eq for Box<dyn Runnable> {}

// Implement Hash on the Box so that Runnables can be collected into a Hashset
impl Hash for Box<dyn Runnable> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name().hash(state);
	}
}

impl PartialEq for Box<dyn Runnable> {
	fn eq(&self, other: &Self) -> bool {
		self.name() == other.name()
	}
}
