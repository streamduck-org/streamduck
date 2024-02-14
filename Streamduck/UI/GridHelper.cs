// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using Avalonia;
using Avalonia.Controls;

namespace Streamduck.UI;

public static class GridHelper {
	public static readonly StyledProperty<int> ColumnCountProperty = AvaloniaProperty.Register<Grid, int>(
		name: "ColumnCount",
		defaultValue: -1);
	
	public static readonly StyledProperty<int> RowCountProperty = AvaloniaProperty.Register<Grid, int>(
		name: "RowCount",
		defaultValue: -1);
	
	static GridHelper()
	{
		ColumnCountProperty.Changed.Subscribe(e => ColumnCountChanged(
			obj: e.Sender,
			args: e));

		RowCountProperty.Changed.Subscribe(e => RowCountChanged(
			obj: e.Sender,
			args: e));
	}
	
	public static void ColumnCountChanged(AvaloniaObject obj, AvaloniaPropertyChangedEventArgs args) {
		if (obj is not Grid grid
		    || (int)(args.NewValue ?? 0) < 0) return;
		grid.ColumnDefinitions.Clear();

		for (var index = 0; index < (int)(args.NewValue ?? 0); index++)
		{
			var definition = new ColumnDefinition {
				Width = GridLength.Star
			};

			grid.ColumnDefinitions.Add(definition);
		}
	}

	public static int GetColumnCount(AvaloniaObject obj) => obj.GetValue(ColumnCountProperty);

	public static int GetRowCount(AvaloniaObject obj) => obj.GetValue(RowCountProperty);

	public static void RowCountChanged(AvaloniaObject obj, AvaloniaPropertyChangedEventArgs args) {
		if (obj is not Grid grid
		    || (int)(args.NewValue ?? 0) < 0) return;
		grid.RowDefinitions.Clear();

		for (var index = 0; index < (int)(args.NewValue ?? 0); index++)
		{
			var definition = new RowDefinition {
				Height = GridLength.Star
			};

			grid.RowDefinitions.Add(definition);
		}
	}

	public static void SetColumnCount(AvaloniaObject obj, int value)
	{
	    obj.SetValue(ColumnCountProperty, value);
	}

	public static void SetRowCount(AvaloniaObject obj, int value)
	{
	    obj.SetValue(RowCountProperty, value);
	}
}