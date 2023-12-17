using System.Collections.Generic;
using System.Threading.Tasks;
using Streamduck.Devices;
using Streamduck.Interfaces;

namespace Streamduck.Plugins;

public abstract class Driver : INamed {
	public abstract string Name { get; }
	public abstract Task<IEnumerable<DeviceIdentifier>> ListDevices();
	public abstract Task<Device> ConnectDevice(DeviceIdentifier identifier);
}