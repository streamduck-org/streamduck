using System;
using System.Collections.Generic;
using Streamduck.Plugins;

namespace StreamduckStreamDeck;

public class StreamDeckPlugin : Plugin {
	public override string Name => "StreamDeckPlugin";

	public override IEnumerable<Driver> Drivers => Array.Empty<Driver>();
}