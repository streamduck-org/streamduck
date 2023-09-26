using ReactiveUI;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI.ViewModels;

public class MainWindowViewModel : ViewModelBase, IScreen {
	public MainWindowViewModel() {
		Router.NavigateAndReset.Execute(new DeviceListViewModel(this));
	}

	public RoutingState Router { get; } = new();
}