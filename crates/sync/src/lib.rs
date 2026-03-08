use tokio::sync::{mpsc, oneshot};

pub fn mpsc<T: Send + Sync + 'static>(
	buffer: usize,
	label: &str,
) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
	let (tx, rx) = hotpath::channel!(mpsc::channel(buffer), label = label);

	(tx, rx)
}

pub fn oneshot<T: Send + Sync + 'static>(
	_label: &str,
) -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
	let (tx, rx) = oneshot::channel();

	// Instrument them only when the feature is enabled
	#[cfg(feature = "hotpath")]
	let (tx, rx) = channels_console::channel!((tx, rx), label = label);

	(tx, rx)
}
