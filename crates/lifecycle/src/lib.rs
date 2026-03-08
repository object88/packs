//! Lifecycle management
//!
//! Requirements:
//! 1) Use a builder to construct the lifecycle
//! 2)
//!
//!
//! Operates over a collection of runnables
//!
//! Builder->add_runnable->Lifecycle
//!
//! Questions:
//! 1) Should all runnables be established beforehand?  Yes
//! 2) Can runnables be restarted?
//!

use anyhow::Result;
use thiserror::Error;
use tokio::{
	sync::{broadcast, mpsc, oneshot},
	task::JoinSet,
	time::timeout,
};
use tracing::info;

use std::{collections::HashSet, time::Duration};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(20);

#[derive(Error, Debug)]
pub enum LifecycleBuildError {
	#[error("")]
	ExcessiveTimeout,

	#[error("")]
	DuplicateNames,
}

#[derive(Error, Debug)]
pub enum LifecycleError {
	#[error("")]
	IncompleteShutdown,
}

// pub mod lifecycle;
pub mod runnable;

pub struct Initial;
pub struct Running;
pub struct Complete;

pub struct Unset;

pub trait RunnablesMarker {}
impl RunnablesMarker for Unset {}
impl RunnablesMarker for Vec<Box<dyn runnable::Runnable>> {}

pub struct Builder<RM>
where
	RM: RunnablesMarker,
{
	runnables: RM,
	shutdown_timeout: Duration,
}

impl Builder<Unset> {
	pub fn with_runnable(
		self,
		runnable: Box<dyn runnable::Runnable>,
	) -> Builder<Vec<Box<dyn runnable::Runnable>>> {
		Builder {
			runnables: vec![runnable],
			shutdown_timeout: self.shutdown_timeout,
		}
	}
}

impl<RM> Builder<RM>
where
	RM: RunnablesMarker,
{
	pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
		self.shutdown_timeout = timeout;
		self
	}
}

impl<'a> Builder<Vec<Box<dyn runnable::Runnable>>> {
	pub fn with_runnable(mut self, runnable: Box<dyn runnable::Runnable>) -> Self {
		self.runnables.push(runnable);
		self
	}

	pub fn build(self) -> Result<Lifecycle> {
		// Check that all properties are valid.
		for r in self.runnables.iter() {
			if r.timeout() > self.shutdown_timeout {
				return Err(LifecycleBuildError::ExcessiveTimeout.into());
			}
		}

		// Ensure that all runners are uniquely named
		let vec_count = self.runnables.iter().count();
		let mut duplicate = false;
		let runnables: HashSet<Box<dyn runnable::Runnable + 'static>> = self
			.runnables
			.into_iter()
			.fold(HashSet::with_capacity(vec_count), |mut acc, x| {
				duplicate |= !acc.insert(x);
				acc
			});
		if duplicate {
			return Err(LifecycleBuildError::DuplicateNames.into());
		}

		// Ok
		Ok(Lifecycle {
			runnables,
			_shutdown_timeout: self.shutdown_timeout,
		})
	}
}

pub struct Lifecycle {
	_shutdown_timeout: Duration,

	runnables: HashSet<Box<dyn runnable::Runnable>>,
}

impl<'a> Lifecycle {
	pub fn builder() -> Builder<Unset> {
		Builder {
			runnables: Unset,
			shutdown_timeout: SHUTDOWN_TIMEOUT,
		}
	}

	pub async fn run(self, cancel_rx: oneshot::Receiver<()>) -> Result<()> {
		let (cancel_broadcast_tx, _) = broadcast::channel(1);
		let (done_tx, mut done_rx) = mpsc::unbounded_channel::<String>();
		let mut futs = JoinSet::new();

		let mut cancel_gates: Vec<oneshot::Sender<()>> = Vec::new();

		// Start the async tasks
		for r in self.runnables {
			let name = r.name().to_owned();
			let rx = cancel_broadcast_tx.subscribe();
			let tm = r.timeout();

			let done_tx = done_tx.clone();

			let (gate_tx, gate_rx) = oneshot::channel::<()>();
			cancel_gates.push(gate_tx);

			futs.spawn(async move {
				info!("starting async task");
				let fut = r.run(rx);

				// When this is received, the timeout should start.
				let _ = gate_rx.await;
				if timeout(tm, fut).await.is_ok() {
					let _ = done_tx.send(name);
				}
			});
		}

		// No need to keep this sender.
		drop(done_tx);

		tokio::select! {
			_ = cancel_rx => {
				// Reciever the cancel message; stop.
				// break;
			}
			_ = futs.join_next() => {
				// If any one future stops; break
			},
		}

		// Let the runners know they should stop
		let _ = cancel_broadcast_tx.send(());
		for gate in cancel_gates {
			let _ = gate.send(());
		}

		// Drain the futs JoinSet.  Each will join when either the runner has
		// completed, or the timeout has run out.
		while futs.join_next().await.is_some() {}

		// Collect the runners that have completed cleanly
		while let Ok(_name) = done_rx.try_recv() {
			// The runner has cleanly completed.
			// TODO: use this to collect the failures
		}

		Ok(())
	}
}
