using System.Collections.Generic;
using System.Linq;
using Streamduck.Devices;
using Streamduck.Plugins.Loaders;

namespace Streamduck.Plugins;

public sealed class WrappedPlugin {
	private readonly WrappedDriver[] _drivers;
	private readonly Plugin _instance;
	private readonly PluginLoadContext _originatedFrom;

	public WrappedPlugin(Plugin instance, PluginLoadContext originatedFrom) {
		_instance = instance;
		Name = _instance.Name;
		_originatedFrom = originatedFrom;
		_drivers = _instance.Drivers
			.Select(d => new WrappedDriver(this, d))
			.ToArray();
	}

	public string Name { get; }

	public IEnumerable<WrappedDriver> Drivers => _drivers;

	public NamespacedName NamespaceName(string name) =>
		new(Name, name);

	public bool BelongsTo(PluginAssembly assembly) => assembly.Context.Equals(_originatedFrom);
}