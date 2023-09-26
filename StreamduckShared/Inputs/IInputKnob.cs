using System;

namespace Streamduck.Inputs;

public interface IInputKnob {
	double KnobValue { get; }
	double MinKnobValue { get; }
	double MaxKnobValue { get; }
	event Action<double>? KnobValueChanged;
}