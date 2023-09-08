using Streamduck.Definitions.Api;
using Streamduck.Definitions.Devices;

namespace Streamduck.Definitions;

public class CoreImpl : Core {
	public CoreImpl(Device associatedDevice) : base(associatedDevice) { }
}