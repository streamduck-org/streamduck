// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
public class NumberFieldAttribute<T>(T? min, T? max, bool slider = true, bool enforceLimit = false)
	: Attribute
	where T : INumber<T> {
	public NumberFieldAttribute(bool slider = true, bool enforceLimit = false) : this(
		T.Zero, T.One, slider,
		enforceLimit
	) { }

	public T Min { get; } = min ?? T.Zero;
	public T Max { get; } = max ?? T.One;
	public bool Slider { get; } = slider;
	public bool EnforceLimit { get; } = enforceLimit;
}