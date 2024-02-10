// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using ReactiveUI;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI.ViewModels;

public class MainWindowViewModel : ViewModelBase, IScreen {
	public MainWindowViewModel() {
		Router.NavigateAndReset.Execute(new DeviceListViewModel(this));
	}

	public RoutingState Router { get; } = new();
}