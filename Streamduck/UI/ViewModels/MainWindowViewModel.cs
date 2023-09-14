namespace Streamduck.UI.ViewModels; 

public class MainWindowViewModel : ViewModelBase {
	public DeviceListViewModel DeviceList { get; } = new();
}