using Streamduck.Definitions.Devices;

namespace Streamduck.Plugins;

public abstract class Device {
	protected Device(DeviceIdentifier identifier) {
		Identifier = identifier;
	}

	public bool Busy { get; set; }
	public DeviceIdentifier Identifier { get; }
}