using System;
using System.Collections.Generic;
using System.Threading.Tasks;
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

	public override Task OnPluginsLoaded(IPluginQuery pluginQuery) {
		foreach (var action in pluginQuery.AllPluginActions()) {
			Console.WriteLine($"Action '{action.Name}' by '{action.PluginName}' was loaded");
		}

		return Task.CompletedTask;
	}
}