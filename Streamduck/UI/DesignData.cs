using System.Collections.ObjectModel;
using Streamduck.Definitions.Devices;
using Streamduck.UI.ViewModels.DeviceEditor;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI; 

public class DesignData {
	public static readonly DeviceListViewModel DeviceList = new(null!) {
		Devices = new ObservableCollection<DeviceEntryViewModel>(new [] {
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
	
	public static readonly DeviceEditorViewModel DeviceEditor = new (null!, new NamespacedDeviceIdentifier(
		new NamespacedName("Plugin", "Driver"),
		new DeviceIdentifier("EL2425151512", "Stream Deck Plus")
	)) {
		
	};
}