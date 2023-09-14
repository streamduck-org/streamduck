using System;
using System.Collections.ObjectModel;
using Streamduck.UI.Models.Devices;
using Streamduck.UI.ViewModels;

namespace Streamduck.UI; 

public class DesignData {
	public static readonly DeviceListViewModel DeviceList = new() {
		ConnectedDevices = new ObservableCollection<DeviceEntry>(new [] {
			new DeviceEntry("EL2425151512", "Stream Deck Plus", true),
			new DeviceEntry("EL3513563183", "Stream Deck Minus", false)
		}),
		// ConnectedDevices = new ObservableCollection<DeviceEntry>(Array.Empty<DeviceEntry>()),
		DiscoveredDevices = new ObservableCollection<DeviceEntry>(new [] {
			new DeviceEntry("EL2425151512", "Stream Deck Plus", true),
			new DeviceEntry("EL3513563183", "Stream Deck Minus", false),
			new DeviceEntry("EL2425151512", "Stream Deck Plus", true),
			new DeviceEntry("EL3513563183", "Stream Deck Minus", false),
			new DeviceEntry("EL2425151512", "Stream Deck Plus", true),
			new DeviceEntry("EL3513563183", "Stream Deck Minus", false),
			new DeviceEntry("EL2425151512", "Stream Deck Plus", true),
			new DeviceEntry("EL3513563183", "Stream Deck Minus", false),
			new DeviceEntry("EL2425151512", "Stream Deck Plus", true),
			new DeviceEntry("EL3513563183", "Stream Deck Minus", false),
		})
	};
}