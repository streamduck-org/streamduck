// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using Avalonia.Controls;
using Avalonia.Controls.Templates;
using Avalonia.Metadata;

namespace Streamduck.UI.Utils;

public class ControlAsDataTemplate : IDataTemplate {
	[DataType]
	public Type? DataType { get; set; }
	
	public Type? ControlType { get; set; }

	public bool Match(object? data) => DataType == null || DataType.IsInstanceOfType(data);

	public Control? Build(object? data) => Build(data, null);

	public Control? Build(object? data, Control? existing) {
		if (existing is not null) return existing;
		if (!(ControlType?.IsAssignableTo(typeof(Control)) ?? false)) return null;
		
		var control = (Control?) ControlType.GetConstructor([])?.Invoke([]);
		if (control is null) return null;

		control.DataContext = data;

		return control;
	}
}