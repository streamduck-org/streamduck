// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using Avalonia;
using Avalonia.Controls;
using Avalonia.ReactiveUI;
using Streamduck.UI.ViewModels;

namespace Streamduck.UI;

public partial class MainWindow : ReactiveWindow<MainWindowViewModel> {
	public MainWindow() {
		InitializeComponent();
		this.AttachDevTools();
	}

	protected override void OnClosing(WindowClosingEventArgs e) {
		e.Cancel = true;
		Hide();
	}
}