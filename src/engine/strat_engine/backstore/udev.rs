// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! udev-related methods
use std::{collections::HashMap, fs, path::Path};

use libudev;

use crate::stratis::StratisResult;

/// Make an enumerator for enumerating block devices. Return an error if there
/// was any udev-related error.
pub fn block_enumerator(context: &libudev::Context) -> libudev::Result<libudev::Enumerator> {
    let mut enumerator = libudev::Enumerator::new(context)?;
    enumerator.match_subsystem("block")?;
    Ok(enumerator)
}

/// Takes a libudev device entry and returns the properties as a HashMap.
fn device_as_map(device: &libudev::Device) -> HashMap<String, String> {
    let rc: HashMap<_, _> = device
        .properties()
        .map(|i| {
            (
                String::from(i.name().to_string_lossy()),
                String::from(i.value().to_string_lossy()),
            )
        })
        .collect();
    rc
}

/// Common function used to retrieve the udev db entry for a block device as a HashMap when found
pub fn get_udev_block_device(
    dev_node_search: &Path,
) -> StratisResult<Option<HashMap<String, String>>> {
    let context = libudev::Context::new()?;
    let mut enumerator = block_enumerator(&context)?;

    // Get canonical form to ensure we do correct lookup in udev db
    let canonical = fs::canonicalize(dev_node_search)?;

    let result = enumerator
        .scan_devices()?
        .filter(|dev| dev.is_initialized())
        .find(|x| x.devnode().map_or(false, |d| canonical == d))
        .map(|dev| device_as_map(&dev));
    Ok(result)
}

/// Lookup the WWN from the udev db using the device node eg. /dev/sda
pub fn hw_lookup(dev_node_search: &Path) -> StratisResult<Option<String>> {
    let dev = get_udev_block_device(dev_node_search)?;
    Ok(dev.and_then(|dev| dev.get("ID_WWN").cloned()))
}
