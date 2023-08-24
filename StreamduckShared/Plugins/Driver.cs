using System.Collections.Generic;
using Streamduck.Definitions.Devices;

namespace Streamduck.Plugins;

public abstract class Driver {
	public abstract string Name { get; }

	public abstract IEnumerable<DeviceIdentifier> ListDevices();
	public abstract DeviceMetadata? DescribeDevice(DeviceIdentifier identifier);
}