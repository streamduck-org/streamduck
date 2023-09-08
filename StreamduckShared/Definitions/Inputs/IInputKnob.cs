using System;

namespace Streamduck.Definitions.Inputs;

public interface IInputKnob {
	double KnobValue { get; }
	double MinKnobValue { get; }
	double MaxKnobValue { get; }
	event Action<double>? KnobValueChanged;
}