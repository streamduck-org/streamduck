// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using Avalonia.ReactiveUI;
using Streamduck.UI.ViewModels.DeviceEditor;

namespace Streamduck.UI.Views.DeviceEditor;

public partial class DeviceEditorView : ReactiveUserControl<DeviceEditorViewModel> {
	public DeviceEditorView() {
		InitializeComponent();
	}
}