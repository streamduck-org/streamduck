using System;
using System.Linq;
using System.Reactive.Linq;
using Avalonia;
using DynamicData;
using ReactiveUI;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI.ViewModels; 

public class MainWindowViewModel : ViewModelBase, IScreen {
	public RoutingState Router { get; } = new();

	public MainWindowViewModel() {
		Router.NavigateAndReset.Execute(new DeviceListViewModel(this));
	}
}