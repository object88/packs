use anyhow::Result;
use pcap::Device;

pub fn list() -> Result<()> {
	let list = match Device::list() {
		Ok(x) => x,
		Err(e) => {
			return Err(e.into());
		},
	};

	for d in list.into_iter() {
		println!(
			"{} ({}), addressses {:?}, flags: {:?}",
			d.name,
			d.desc.unwrap_or_default(),
			d.addresses,
			d.flags
		)
	}

	Ok(())
}
