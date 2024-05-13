// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Configuration;
using Streamduck.Devices;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests.DeviceList;

[AutoAdd]
public class SetDeviceAutoconnect : SocketRequest<SetDeviceAutoconnect.Request> {
	public class Request {
		public NamespacedDeviceIdentifier Identifier { get; set; }
		public bool Autoconnect { get; set; }
	}

	public override string Name => "Set Device Autoconnect";

	public override async Task Received(SocketRequester request, Request data) {
		var config = await Config.Get();

		if (data.Autoconnect)
			config.AutoconnectDevices.Add(data.Identifier);
		else
			config.AutoconnectDevices.Remove(data.Identifier);

		await config.SaveConfig();
		await App.CurrentInstance!.RefreshDevices();

		request.SendBack(null);
	}
}