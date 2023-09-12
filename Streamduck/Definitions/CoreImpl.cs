using NLog;
using Streamduck.Definitions.Api;
using Streamduck.Definitions.Devices;

namespace Streamduck.Definitions;

public class CoreImpl : Core {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();
	protected readonly NamespacedDeviceIdentifier _deviceIdentifier;
	
	public CoreImpl(Device device, NamespacedDeviceIdentifier deviceIdentifier) : base(device) {
		_deviceIdentifier = deviceIdentifier;
		device.Died += () => _l.Warn("Device {} died", _deviceIdentifier);
	}
}