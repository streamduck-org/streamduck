// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using Streamduck.Devices;

namespace Streamduck.Configuration.Devices;

public class DeviceConfig {
	public NamespacedDeviceIdentifier Device { get; set; }
	public SerializedScreen[] ScreenStack { get; set; } = [];
}