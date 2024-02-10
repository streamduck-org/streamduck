// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Reactive;
using ReactiveUI;
using Streamduck.Devices;

namespace Streamduck.UI.ViewModels.DeviceEditor;

public class DeviceEditorViewModel(IScreen hostScreen, NamespacedDeviceIdentifier deviceName) : ViewModelBase,
	IRoutableViewModel {
	public ReactiveCommand<Unit, IRoutableViewModel?> GoBack => HostScreen.Router.NavigateBack;
	public string Identifier => deviceName.Identifier;
	public string Description => deviceName.Description;

	public string UrlPathSegment => "editor";
	public IScreen HostScreen { get; } = hostScreen;
}