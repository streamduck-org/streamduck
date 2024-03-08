// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using Avalonia.Controls;
using Streamduck.UI.ViewModels.DeviceEditor;

namespace Streamduck.UI.Views.DeviceEditor;

public partial class InputGrid : UserControl {
	public InputGrid() {
		InitializeComponent();
	}

	protected override void OnSizeChanged(SizeChangedEventArgs e) {
		base.OnSizeChanged(e);

		if (DataContext is not InputGridViewModel model) return;

		var squareSize = Math.Min(Bounds.Height / model.Rows, Bounds.Width / model.Columns);

		var idealWidth = squareSize * model.Columns;
		var idealHeight = squareSize * model.Rows;

		model.Width = idealWidth;
		model.Height = idealHeight;
	}
}