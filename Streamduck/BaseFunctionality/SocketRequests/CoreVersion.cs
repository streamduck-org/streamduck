// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests;

[AutoAdd]
public class CoreVersion : SocketRequest {
	public const string VERSION = "0.1";
	public override string Name => "Socket Version";

	public override Task Received(SocketRequester request) {
		request.SendBack(VERSION);
		return Task.CompletedTask;
	}
}