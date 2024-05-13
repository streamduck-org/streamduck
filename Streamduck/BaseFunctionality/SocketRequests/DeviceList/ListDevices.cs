// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Configuration;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests.DeviceList;

[AutoAdd]
public class ListDevices : SocketRequest {
	public override string Name => "List Devices";

	public override async Task Received(SocketRequester request) {
		var config = await Config.Get();

		request.SendBack(Devices());
		return;

		IEnumerable<Device> Devices() {
			foreach (var connected in App.CurrentInstance!.ConnectedDeviceList.Keys)
				yield return new Device {
					Identifier = connected,
					Connected = true,
					Autoconnect = config!.AutoconnectDevices.Contains(connected)
				};

			foreach (var discovered in App.CurrentInstance.DiscoveredDevices)
				yield return new Device {
					Identifier = discovered,
					Connected = false,
					Autoconnect = config!.AutoconnectDevices.Contains(discovered)
				};
		}
	}
}