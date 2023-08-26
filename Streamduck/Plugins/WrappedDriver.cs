using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using NLog;
using Streamduck.Definitions.Devices;

namespace Streamduck.Plugins;

public class WrappedDriver {
	private static readonly Logger L = LogManager.GetCurrentClassLogger();

	private readonly Driver _instance;

	public WrappedDriver(WrappedPlugin plugin, Driver instance) {
		_instance = instance;
		Name = plugin.NamespaceName(_instance.Name);
	}

	public NamespacedName Name { get; }

	public async Task<IEnumerable<NamespacedDeviceIdentifier>> ListDevices() {
		return (await _instance.ListDevices())
			.Select(i => new NamespacedDeviceIdentifier(Name, i));
	}
}