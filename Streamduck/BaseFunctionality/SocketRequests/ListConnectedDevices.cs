// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests;

[AutoAdd]
public class ListConnectedDevices : SocketRequest {
	public override string Name => "List Connected Devices";

	public override Task Received(SocketRequester request) {
		request.SendBack(App.CurrentInstance!.ConnectedDeviceList.Keys);
		return Task.CompletedTask;
	}
}