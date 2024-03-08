// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using Avalonia;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using ReactiveUI;
using Streamduck.Inputs;

namespace Streamduck.UI.ViewModels.DeviceEditor;

public class InputGridItemViewModel(InputGridViewModel parentModel, Input input, int inputIndex, uint x, uint y, uint w, uint h) : ViewModelBase {

	public readonly InputGridViewModel Parent = parentModel;
	public readonly int InputIndex = inputIndex;
	public uint X => x;
	public uint Y => y;
	public uint W => w;
	public uint H => h;
	public IImage Icon => new Bitmap(AssetLoader.Open(new Uri(input.Icon switch {
		InputIcon.Button => "avares://Streamduck/UI/Assets/InputIcons/button.png",
		InputIcon.Toggle => "avares://Streamduck/UI/Assets/InputIcons/toggle.png",
		InputIcon.AnalogButton => "avares://Streamduck/UI/Assets/InputIcons/analog-button.png",
		InputIcon.Slider => "avares://Streamduck/UI/Assets/InputIcons/slider.png",
		InputIcon.Knob => "avares://Streamduck/UI/Assets/InputIcons/knob.png",
		InputIcon.Encoder => "avares://Streamduck/UI/Assets/InputIcons/encoder.png",
		InputIcon.TouchScreen => "avares://Streamduck/UI/Assets/InputIcons/touchscreen.png",
		InputIcon.Joystick => "avares://Streamduck/UI/Assets/InputIcons/joystick.png",
		InputIcon.Trackball => "avares://Streamduck/UI/Assets/InputIcons/trackball.png",
		InputIcon.Touchpad => "avares://Streamduck/UI/Assets/InputIcons/touchpad.png",
		InputIcon.Sensor => "avares://Streamduck/UI/Assets/InputIcons/sensor.png",
		_ => throw new ArgumentOutOfRangeException()
	})));

	public CornerRadius CornerRadius => new(input.Icon switch {
		InputIcon.Knob or InputIcon.Encoder or InputIcon.Joystick or InputIcon.Sensor => 200000,
		_ => 10
	});
	
	public HorizontalAlignment Horizontal => input.Icon switch {
		InputIcon.Knob or InputIcon.Encoder or InputIcon.Joystick or InputIcon.Sensor => HorizontalAlignment.Center,
		_ => HorizontalAlignment.Right
	};

	public bool IsSelected => Parent.SelectedInput == InputIndex;
	public IBrush? BorderBrush => IsSelected ? Brushes.CadetBlue : null;

	public void RaiseSelectionChanged() {
		this.RaisePropertyChanged(nameof(IsSelected));
		this.RaisePropertyChanged(nameof(BorderBrush));
	}
}