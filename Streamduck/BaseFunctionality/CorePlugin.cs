// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.BaseFunctionality.SocketRequests;
using Streamduck.BaseFunctionality.Triggers;
using Streamduck.Plugins;
using Streamduck.Socket;
using Streamduck.Triggers;

namespace Streamduck.BaseFunctionality;

public class CorePlugin : Plugin {
	public override string Name => "Core";

	public override IEnumerable<Trigger> Triggers => [
		new ButtonDownTrigger()
	];

	public override IEnumerable<SocketRequest> SocketRequests => [
		new ListConnectedDevices(),
		new ListDiscoveredDevices()
	];
}