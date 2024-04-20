// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using Streamduck.Devices;

namespace Streamduck.BaseFunctionality.SocketRequests;

public class Device {
	public NamespacedDeviceIdentifier Identifier { get; set; }
	public bool Connected { get; set; }
	public bool Autoconnect { get; set; }
}