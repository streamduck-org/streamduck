using System;
using System.Collections.ObjectModel;
using Streamduck.UI.Models.Devices;

namespace Streamduck.UI.ViewModels; 

public class DeviceListViewModel : ViewModelBase {
	public ObservableCollection<DeviceEntry> ConnectedDevices { get; set; } = new();
	public ObservableCollection<DeviceEntry> DiscoveredDevices { get; set; } = new();

	public bool IsEmpty => ConnectedDevices.Count <= 0 && DiscoveredDevices.Count <= 0;
}