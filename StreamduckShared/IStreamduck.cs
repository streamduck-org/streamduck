// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Images;
using Streamduck.Plugins;

namespace Streamduck;

/**
 * Collection of references to systems used around Streamduck project
 */
public interface IStreamduck {
	public IPluginQuery Plugins { get; }
	public IImageCollection Images { get; }
	public IReadOnlyCollection<NamespacedDeviceIdentifier> DiscoveredDevices { get; }
	public IReadOnlyDictionary<NamespacedDeviceIdentifier, Core> ConnectedDevices { get; }
}