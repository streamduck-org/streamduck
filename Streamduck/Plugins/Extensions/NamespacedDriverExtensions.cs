using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Data;
using Streamduck.Devices;

namespace Streamduck.Plugins.Extensions; 

public static class NamespacedDriverExtensions {
	public static async Task<Device> ConnectDevice(this Namespaced<Driver> driver, NamespacedDeviceIdentifier name) => 
		await driver.Instance.ConnectDevice(name.DeviceIdentifier);
	
	public static async Task<IEnumerable<NamespacedDeviceIdentifier>> ListDevices(this Namespaced<Driver> driver) =>
		(await driver.Instance.ListDevices())
		.Select(d => new NamespacedDeviceIdentifier(driver.NamespacedName, d));
}