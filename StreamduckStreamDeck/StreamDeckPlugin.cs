using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Streamduck.Attributes;
using Streamduck.Plugins;

namespace StreamduckStreamDeck;

public class StreamDeckPlugin : Plugin, IDisposable {
	private readonly DeviceManager _manager = DeviceManager.Get();

	public override string Name => "Stream Deck Plugin";

	private readonly StreamDeckDriver driver;
	
	public StreamDeckPlugin() {
		driver = new StreamDeckDriver(_manager);
	}

	public override IEnumerable<Driver> Drivers => new Driver[] {
		driver
	};

	public void Dispose() {
		_manager.Dispose();
		GC.SuppressFinalize(this);
	}

	[PluginMethod]
	public void MyAction() {
		
	}

	public override Task OnPluginsLoaded(IPluginQuery pluginQuery) {
		Console.WriteLine($"Driver has {driver.Config.ScreenBrightness} for brightness");

		return Task.CompletedTask;
	}
}