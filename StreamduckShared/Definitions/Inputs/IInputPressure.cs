using System;

namespace Streamduck.Definitions.Inputs;

/**
 * Button press pressure, 3D touch, etc.
 */
public interface IInputPressure {
	double Pressure { get; }
	double MinPressure { get; }
	double MaxPressure { get; }
	event Action<double>? PressureChanged;
}