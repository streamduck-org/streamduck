// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Devices;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests.Devices;

[AutoAdd]
public class ConnectDevice : SocketRequest<ConnectDevice.Request> {
	public class Request {
		public required NamespacedDeviceIdentifier Identifier { get; set; }
	}

	public override string Name => "Connect Device";

	public override async Task Received(SocketRequester request, Request data) =>
		request.SendBack(await App.CurrentInstance!.ConnectDevice(data.Identifier));
}