// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Avalonia.Media;
using ReactiveUI;
using Streamduck.Inputs;

namespace Streamduck.UI.ViewModels.DeviceEditor;

public class InputGridViewModel : ViewModelBase {
	public InputGridViewModel(IReadOnlyCollection<Input> inputs) {
		Items = new ObservableCollection<InputGridItemViewModel>(ProcessInputs(inputs, out var columns, out var rows));
		Columns = columns;
		Rows = rows;
	}

	private static IEnumerable<InputGridItemViewModel> ProcessInputs(IReadOnlyCollection<Input> inputs, out int columns, out int rows) {
		int minX = int.MaxValue, minY = int.MaxValue, 
			maxX = int.MinValue, maxY = int.MinValue;
		
		foreach (var input in inputs) {
			minX = int.Min(minX, input.X);
			minY = int.Min(minY, input.Y);
			maxX = int.Max(maxX, (int)(input.X + input.W - 1));
			maxY = int.Max(maxY, (int)(input.Y + input.H - 1));
		}

		columns = maxX - minX + 1;
		rows = maxY - minY + 1;

		return Iterator();

		IEnumerable<InputGridItemViewModel> Iterator() {
			foreach (var input in inputs) {
				yield return new InputGridItemViewModel(
					input,
					(uint)(input.X - minX),
					(uint)(input.Y - minY),
					input.W,
					input.H
				);
			}
		}
	}
	
	public ObservableCollection<InputGridItemViewModel> Items { get; set; }
	public int Columns { get; }
	public int Rows { get; }
	public double Width { get; set; }
	public double Height { get; set; }

	public void NotifyWidthAndHeight() {
		this.RaisePropertyChanged(nameof(Width));
		this.RaisePropertyChanged(nameof(Height));
	}
}