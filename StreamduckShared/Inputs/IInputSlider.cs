using System;

namespace Streamduck.Inputs;

public interface IInputSlider {
	double SliderValue { get; }
	double MinSliderValue { get; }
	double MaxSliderValue { get; }
	event Action<double>? SliderValueChanged;
}