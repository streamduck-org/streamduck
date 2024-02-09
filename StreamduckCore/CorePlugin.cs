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