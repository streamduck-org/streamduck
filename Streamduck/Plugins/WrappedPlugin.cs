using System.Collections.Generic;
using System.Linq;
using Streamduck.Definitions.Devices;

namespace Streamduck.Plugins;

public sealed class WrappedPlugin {
	private readonly Plugin _instance;

	public WrappedPlugin(Plugin instance) {
		_instance = instance;
		Name = _instance.Name;
	}

	public string Name { get; }

	public IEnumerable<WrappedDriver> Drivers =>
		_instance.Drivers
			.Select(d => new WrappedDriver(this, d));

	public NamespacedName NamespaceName(string name) =>
		new() {
			PluginName = Name,
			Name = name
		};
}