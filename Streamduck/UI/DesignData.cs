using System.Collections.ObjectModel;
using Streamduck.Definitions.Devices;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI; 

public class DesignData {
	public static readonly DeviceListViewModel DeviceList = new() {
		Devices = new ObservableCollection<DeviceEntryViewModel>(new [] {
			new DeviceEntryViewModel(new NamespacedDeviceIdentifier(
				new NamespacedName("Plugin", "Driver"),
				new DeviceIdentifier("EL2425151512", "Stream Deck Plus")
			), true),
			new DeviceEntryViewModel(new NamespacedDeviceIdentifier(
				new NamespacedName("Plugin", "Driver"),
				new DeviceIdentifier("EL2425151513", "Stream Deck Minus")
			), false)
		})
	};
}