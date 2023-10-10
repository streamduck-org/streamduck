using System;
using System.Collections.Generic;
using ElgatoStreamDeck;
using Streamduck.Attributes;
using Streamduck.Plugins;

namespace StreamduckStreamDeck;

public class StreamDeckPlugin : Plugin, IDisposable {
	private readonly DeviceManager _manager = DeviceManager.Get();

	public override string Name => "StreamDeckPlugin";

	public override IEnumerable<Driver> Drivers => new Driver[] {
		new StreamDeckDriver(_manager)
	};

	public void Dispose() {
		_manager.Dispose();
		GC.SuppressFinalize(this);
	}

	[PluginMethod]
	public void MyAction() {
		
	}
}