using Streamduck.Definitions.Devices;

namespace Streamduck.Definitions.Api;

public abstract class Core {
	protected Device _associatedDevice;

	protected Core(Device associatedDevice) {
		_associatedDevice = associatedDevice;
	}
}