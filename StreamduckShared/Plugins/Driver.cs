using System.Collections.Generic;
using System.Threading.Tasks;
using Streamduck.Definitions.Devices;

namespace Streamduck.Plugins;

public abstract class Driver {
	public abstract string Name { get; }
	public abstract Task<IEnumerable<DeviceIdentifier>> ListDevices();
	public abstract ValueTask<DeviceMetadata?> DescribeDevice(DeviceIdentifier identifier);
}