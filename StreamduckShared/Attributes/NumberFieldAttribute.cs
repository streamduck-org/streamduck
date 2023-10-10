using System;
using System.Numerics;

namespace Streamduck.Attributes; 

/**
 * Settings for a number field. Min and Max can be used to limit the value to the bounds.
 * Slider can be disabled if number field is preferred.
 * Limit enforcement can be enabled if inputting number out of bounds is unacceptable.
 * Default bounds are 0 to 1.
 */
[AttributeUsage(AttributeTargets.Property)]
public class NumberFieldAttribute<T> : Attribute where T : INumber<T> {
	public NumberFieldAttribute(bool slider = true, bool enforceLimit = false) : this(T.Zero, T.One, slider, enforceLimit) {}
	public NumberFieldAttribute(T? min, T? max, bool slider = true, bool enforceLimit = false) {
		Min = min ?? T.Zero;
		Max = max ?? T.One;
		Slider = slider;
		EnforceLimit = enforceLimit;
	}

	public T Min { get; }
	public T Max { get; }
	public bool Slider { get; }
	public bool EnforceLimit { get; }
}