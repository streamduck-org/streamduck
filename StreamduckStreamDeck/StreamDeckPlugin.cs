// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Streamduck.Attributes;
using Streamduck.Plugins;

namespace StreamduckStreamDeck;

public class StreamDeckPlugin : Plugin, IDisposable {
	private readonly DeviceManager _manager = DeviceManager.Get();

	private readonly StreamDeckDriver driver;

	public StreamDeckPlugin() {
		driver = new StreamDeckDriver(_manager);
	}

	public override string Name => "Stream Deck Plugin";

	public override IEnumerable<Driver> Drivers => new Driver[] {
		driver
	};

	public void Dispose() {
		_manager.Dispose();
		GC.SuppressFinalize(this);
	}

	[PluginMethod]
	public void MyAction() { }

	public override Task OnPluginsLoaded(IPluginQuery pluginQuery) {
		Console.WriteLine($"Driver has {driver.Config.ScreenBrightness} for brightness");

		return Task.CompletedTask;
	}
}