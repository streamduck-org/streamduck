// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using System.Threading.Tasks;
using Streamduck.Devices;
using Streamduck.Interfaces;

namespace Streamduck.Plugins;

public abstract class Driver : INamed {
	public abstract string Name { get; }
	public abstract Task<IEnumerable<DeviceIdentifier>> ListDevices();
	public abstract Task<Device> ConnectDevice(DeviceIdentifier identifier);
}