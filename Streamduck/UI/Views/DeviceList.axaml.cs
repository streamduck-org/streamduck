using System.Collections.ObjectModel;
using Avalonia.Controls;

namespace Streamduck.UI.Views; 

public partial class DeviceList : UserControl {
	public ObservableCollection<Device> ConnectedDevices { get; } = new();
	public ObservableCollection<Device> DiscoveredDevices { get; } = new();
	public DeviceList() {
		InitializeComponent();
	}

	public class Device {
		
	}
}