// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using Avalonia;
using Avalonia.Controls;
using Streamduck.UI.ViewModels.DeviceEditor;

namespace Streamduck.UI.Views.DeviceEditor;

public partial class InputGridItem : UserControl {
	public InputGridItem() {
		InitializeComponent();
	}

	protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e) {
		base.OnAttachedToVisualTree(e);
		
		if (DataContext is not InputGridItemViewModel itemModel) return;
		
		Button.Click += (sender, args) => {
			itemModel.Parent.SelectedInput = itemModel.InputIndex;
		};
	}
}