// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;
using Streamduck.Configuration;
using Streamduck.Constants;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Plugins;
using Device = Streamduck.BaseFunctionality.SocketRequests.Device;

namespace Streamduck.BaseFunctionality;

public class CorePlugin : Plugin {
	public override string Name => "Core";

	public override async Task OnDeviceConnected(NamespacedDeviceIdentifier identifier, Core deviceCore) =>
		SendEventToSocket(EventNames.DeviceConnected, new Device {
			Identifier = identifier,
			Connected = true,
			Autoconnect = (await Config.Get()).AutoconnectDevices.Contains(identifier)
		});

	public override Task OnDeviceDisconnected(NamespacedDeviceIdentifier identifier) {
		SendEventToSocket(EventNames.DeviceDisconnected, identifier);
		return Task.CompletedTask;
	}

	public override async Task OnDeviceAppeared(NamespacedDeviceIdentifier identifier) =>
		SendEventToSocket(EventNames.DeviceAppeared, new Device {
			Identifier = identifier,
			Connected = false,
			Autoconnect = (await Config.Get()).AutoconnectDevices.Contains(identifier)
		});

	public override Task OnDeviceDisappeared(NamespacedDeviceIdentifier identifier) {
		SendEventToSocket(EventNames.DeviceDisappeared, identifier);
		return Task.CompletedTask;
	}
}