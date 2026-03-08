use futures::future;
use tokio::sync::oneshot;
use tokio::task::JoinSet;

use crate::runnable;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Initial {}
pub struct Running {}
pub struct Complete {}

pub struct Lifecycle {
	runnables: Vec<Box<dyn runnable::Runnable>>,
}

impl Lifecycle {
	pub fn run(self, cancel_rx: oneshot::Receiver<()>) -> Runs {
		let (cancel_broadcast_tx, _) = broadcast::channel(1);
		let mut futs = JoinSet::new();

		// Start the async tasks
		for r in self.runnables {
			let rx = cancel_broadcast_tx.subscribe();
			let abort_handle = futs.spawn(async move {
				info!("starting async task");
				r.run(rx).await;
			});
			futs_abort_ids.insert(abort_handle.id());
		}
	}
}

struct Runs {
	set: JoinSet<()>,
}

impl Future for Runs {
	type Output;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let mut results = Vec::new();

		loop {
			match self.set.poll_join_next_with_id(cx) {
				Poll::Ready(Some(Ok((id, value)))) => todo!(),
				Poll::Ready(Some(Err(e))) => todo!(),
				Poll::Ready(None) => return Poll::Ready(results),
				Poll::Pending => return Poll::Pending,
			}
		}
	}
}
