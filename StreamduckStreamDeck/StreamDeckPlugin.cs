using System;
using System.Collections.Generic;
using ElgatoStreamDeck;
using Streamduck.Plugins;

namespace StreamduckStreamDeck;

public class StreamDeckPlugin : Plugin {
	private readonly DeviceManager _manager = DeviceManager.Get();

	public override string Name => "StreamDeckPlugin";

	public override IEnumerable<Driver> Drivers => new Driver[] {
		new StreamDeckDriver(_manager)
	};

	public override void Dispose() {
		_manager.Dispose();
		GC.SuppressFinalize(this);
	}
}