using Avalonia.ReactiveUI;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI.Views.DeviceList;

public partial class DeviceListView : ReactiveUserControl<DeviceListViewModel> {
	public DeviceListView() {
		InitializeComponent();
	}
}