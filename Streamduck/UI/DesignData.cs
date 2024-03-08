// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.ObjectModel;
using Streamduck.Devices;
using Streamduck.Inputs;
using Streamduck.Plugins;
using Streamduck.UI.Design;
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

	public static readonly DeviceEditorViewModel DeviceEditor = new(
		null!,
		new ShellCore(new ShellDevice(new DeviceIdentifier("EL2425151512", "Stream Deck Plus")))
	);

	public static readonly InputGridViewModel InputGrid = new([
		// new ShellInput(0, 0, 1, 1, InputIcon.Button),
		// new ShellInput(1, 0, 1, 1, InputIcon.Button),
		// new ShellInput(2, 0, 1, 1, InputIcon.Button),
		// new ShellInput(3, 0, 1, 1, InputIcon.Button),
		// new ShellInput(0, 1, 1, 1, InputIcon.Button),
		// new ShellInput(1, 1, 1, 1, InputIcon.Button),
		// new ShellInput(2, 1, 1, 1, InputIcon.Button),
		// new ShellInput(3, 1, 1, 1, InputIcon.Button),
		// new ShellInput(0, 2, 4, 1, InputIcon.TouchScreen),
		// new ShellInput(0, 3, 1, 1, InputIcon.Encoder),
		// new ShellInput(1, 3, 1, 1, InputIcon.Encoder),
		// new ShellInput(2, 3, 1, 1, InputIcon.Encoder),
		// new ShellInput(3, 3, 1, 1, InputIcon.Encoder),
		// new ShellInput(4, 0, 1, 4, InputIcon.TouchScreen)
		
		new ShellInput(0, 0, 10, 10, InputIcon.Button),
		new ShellInput(5, 10, 10, 10, InputIcon.Button),
		new ShellInput(10, 0, 10, 10, InputIcon.Button),
		new ShellInput(8, 20, 10, 10, InputIcon.Button),
		new ShellInput(0, -5, 10, 5, InputIcon.Button),
		new ShellInput(10, -5, 10, 5, InputIcon.Button),
		new ShellInput(20, -5, 10, 5, InputIcon.Button)
	]) {
		SelectedInput = 0
	};
}