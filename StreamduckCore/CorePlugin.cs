// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.Plugins;
using Streamduck.Triggers;
using StreamduckCore.Triggers;

namespace StreamduckCore;

public class CorePlugin : Plugin {
	public override string Name => "Streamduck Core Plugin";

	public override IEnumerable<Trigger> Triggers => new[] {
		new ButtonDownTrigger()
	};
}