// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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