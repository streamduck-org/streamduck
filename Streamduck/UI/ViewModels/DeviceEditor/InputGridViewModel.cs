// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
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

	private IEnumerable<InputGridItemViewModel> ProcessInputs(IReadOnlyCollection<Input> inputs, out int columns, out int rows) {
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
			foreach (var (input, index) in inputs.Select((value, i) => (value, i))) {
				yield return new InputGridItemViewModel(
					this,
					input,
					index,
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

	private double _width;
	public double Width {
		get => _width;
		set {
			_width = value;
			this.RaisePropertyChanged();
		}
	}

	private double _height;
	public double Height {
		get => _height;
		set {
			_height = value;
			this.RaisePropertyChanged();
		} 
	}

	private int _selectedInput = -1;

	public int SelectedInput {
		get => _selectedInput;
		set {
			var lastStates = Items.Select(i => i.IsSelected).ToArray();
			
			_selectedInput = value;
			this.RaisePropertyChanged();

			foreach (var (inputItem, lastState) in Items.Zip(lastStates)) {
				if (inputItem.IsSelected != lastState) inputItem.RaiseSelectionChanged();
			}
		}
	}
}