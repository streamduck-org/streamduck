using Streamduck.Definitions.Inputs;

namespace Streamduck.Definitions.Devices;

public abstract class Device {
	protected Device(DeviceIdentifier identifier) {
		Identifier = identifier;
		Alive = true;
	}
		
	public bool Alive { get; protected set; }
	public bool Busy { get; set; }
	public DeviceIdentifier Identifier { get; }

	public abstract Input[] Inputs { get; }
}