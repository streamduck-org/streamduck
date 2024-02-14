// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Reactive;
using ReactiveUI;
using Streamduck.Cores;

namespace Streamduck.UI.ViewModels.DeviceEditor;

public class DeviceEditorViewModel(IScreen hostScreen, Core deviceCore) : ViewModelBase, IRoutableViewModel {
	public ReactiveCommand<Unit, IRoutableViewModel?> GoBack => HostScreen.Router.NavigateBack;
	public string Identifier => deviceCore.DeviceIdentifier.Identifier;
	public string Description => deviceCore.DeviceIdentifier.Description;
	public string UrlPathSegment => "editor";
	public IScreen HostScreen { get; } = hostScreen;
	public InputGridViewModel InputGrid { get; } = new(deviceCore.Inputs);
}