using System.Collections.Generic;
using System.Linq;
using NLog;
using Streamduck.Definitions.Devices;

namespace Streamduck.Plugins;

public class WrappedDriver {
	private static readonly Logger L = LogManager.GetCurrentClassLogger();

	private readonly Driver _instance;

	public WrappedDriver(WrappedPlugin plugin, Driver instance) {
		_instance = instance;
		Name = plugin.NamespaceName(_instance.Name);

		L.Info("Created instance {0}", instance.GetHashCode());
	}

	public NamespacedName Name { get; }

	public IEnumerable<NamespacedDeviceIdentifier> ListDevices() {
		return _instance.ListDevices()
			.Select(i => new NamespacedDeviceIdentifier(Name, i));
	}
}