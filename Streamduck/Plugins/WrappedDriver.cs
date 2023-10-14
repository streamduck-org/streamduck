using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using NLog;
using Streamduck.Devices;

namespace Streamduck.Plugins;

public class WrappedDriver {
	private static readonly Logger L = LogManager.GetCurrentClassLogger();

	public Driver Instance { get; }

	public WrappedDriver(WrappedPlugin plugin, Driver instance) {
		Instance = instance;
		Name = plugin.NamespaceName(Instance.Name);
	}

	public NamespacedName Name { get; }

	public async Task<IEnumerable<NamespacedDeviceIdentifier>> ListDevices() {
		return (await Instance.ListDevices())
			.Select(i => new NamespacedDeviceIdentifier(Name, i));
	}

	public async Task<Device> ConnectDevice(NamespacedDeviceIdentifier identifier) =>
		await Instance.ConnectDevice(identifier.DeviceIdentifier);
}