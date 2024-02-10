// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.ObjectModel;
using Streamduck.Devices;
using Streamduck.Plugins;
using Streamduck.UI.ViewModels.DeviceEditor;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI;

public class DesignData {
	public static readonly DeviceListViewModel DeviceList = new(null!) {
		Devices = new ObservableCollection<DeviceEntryViewModel>(new[] {
			new DeviceEntryViewModel(new NamespacedDeviceIdentifier(
				new NamespacedName("Plugin", "Driver"),
				new DeviceIdentifier("EL2425151512", "Stream Deck Plus")
			), true, null!),
			new DeviceEntryViewModel(new NamespacedDeviceIdentifier(
				new NamespacedName("Plugin", "Driver"),
				new DeviceIdentifier("EL2425151513", "Stream Deck Minus")
			), false, null!)
		})
	};

	public static readonly DeviceEditorViewModel DeviceEditor = new(null!, new NamespacedDeviceIdentifier(
		new NamespacedName("Plugin", "Driver"),
		new DeviceIdentifier("EL2425151512", "Stream Deck Plus")
	));
}